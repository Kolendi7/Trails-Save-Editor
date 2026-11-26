fn crc_table() -> [u32; 256] {
	let mut t = [0u32; 256];
	for i in 0..256u32 {
		let mut c = i;
		for _ in 0..8 {
			c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
		}
		t[i as usize] = c;
	}
	t
}

pub fn checksum(d: &[u8]) -> u32 {
	let t = crc_table();
	let mut c = (d.len() - 0xC) as u32;
	for &b in &d[0xC..] {
		c = t[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
	}
	c
}

pub fn pack(d: &[u8]) -> Vec<u8> {
	let z = ruzstd::encoding::compress_to_vec(d, ruzstd::encoding::CompressionLevel::Fastest);
	let fhd = z[4];
	let single = fhd & 0x20 != 0;
	let fcs_len = match fhd >> 6 {
		0 => usize::from(single),
		1 => 2,
		2 => 4,
		_ => 8,
	};
	let did_len = [0usize, 1, 2, 4][(fhd & 3) as usize];
	let start = 5 + usize::from(!single) + did_len + fcs_len;
	let mut p = start;
	loop {
		let h = u32::from_le_bytes([z[p], z[p + 1], z[p + 2], 0]);
		let typ = (h >> 1) & 3;
		p += 3 + if typ == 1 { 1 } else { (h >> 3) as usize };
		if h & 1 != 0 {
			break;
		}
	}
	let mut out = vec![0x28, 0xB5, 0x2F, 0xFD, 0xA0];
	out.extend((d.len() as u32).to_le_bytes());
	out.extend(&z[start..p]);
	out
}

pub fn faces(b: &[u8]) -> std::collections::HashMap<u32, Vec<u8>> {
	let mut m = std::collections::HashMap::new();
	if b.len() < 12 || &b[..6] != b"KFACE1" {
		return m;
	}
	let u = |o: usize| b.get(o..o + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]));
	let mut o = 12;
	for _ in 0..u(8).unwrap_or(0) {
		let (Some(id), Some(len)) = (u(o), u(o + 4)) else { break };
		let Some(png) = b.get(o + 8..o + 8 + len as usize) else { break };
		m.insert(id, png.to_vec());
		o += 8 + len as usize;
	}
	m
}

