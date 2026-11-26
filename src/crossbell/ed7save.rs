use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

pub struct Profile {
	pub name: &'static str,
	pub dir: &'static str,
	pub falcom: &'static str,
	pub size: usize,
	pub chr: usize,
	pub equip: usize,
	pub crafts: usize,
	pub craft_n: usize,
	pub scrafts: usize,
	pub orb: usize,
	pub items: usize,
	pub mira: usize,
	pub sepith: usize,
	pub time: usize,
	pub party: usize,
	pub chr_names: usize,
	pub difficulty: usize,
	pub master_quartz: bool,
	pub playable: &'static [usize],
	pub max_level: u32,
	pub dp: usize,
	pub ranks: &'static [(&'static str, u32)],
	pub rank_rewards: [u32; 15],
	pub battle: usize,
	pub tokens: usize,
	pub female: &'static [u32],
}

pub const CHARS: usize = 12;
pub const RECORD: usize = 52;
pub const PARTY: usize = 8;
pub const PARTY_ACTIVE: usize = 4;
pub const ITEMS_CAP: usize = 1000;

pub static ZERO: Profile = Profile {
	name: "Trails from Zero",
	dir: "Zero",
	falcom: "Zero",
	size: 155_624,
	chr: 0x98,
	equip: 0x308,
	crafts: 0xB00,
	craft_n: 10,
	scrafts: 0xBF0,
	orb: 0xC68,
	items: 0xDB8,
	mira: 0x19BB8,
	sepith: 0x19BC8,
	time: 0x19C10,
	party: 0x1AC04,
	chr_names: 0x1AF7C,
	difficulty: 0x1EF91,
	master_quartz: false,
	playable: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
	max_level: 50,
	dp: 0x19BA8,
	battle: 0x1EF9C,
	tokens: 0x19BC0,
	ranks: &[
		("15th Class", 0),
		("14th Class", 16),
		("13th Class", 41),
		("12th Class", 56),
		("11th Class", 70),
		("10th Class", 89),
		("9th Class", 110),
		("8th Class", 130),
		("7th Class", 150),
		("6th Class", 197),
		("5th Class", 224),
		("4th Class", 269),
		("3rd Class", 294),
		("2nd Class", 324),
		("1st Class", 376),
	],
	rank_rewards: [0, 76, 172, 77, 145, 79, 153, 78, 146, 80, 81, 152, 1615, 1515, 176],
	female: &[1, 2, 5, 6, 8],
};

pub static AZURE: Profile = Profile {
	name: "Trails to Azure",
	dir: "Azure",
	falcom: "Ao",
	size: 156_732,
	chr: 0x94,
	equip: 0x304,
	crafts: 0xAFC,
	craft_n: 15,
	scrafts: 0xC64,
	orb: 0xCDC,
	items: 0xE2C,
	mira: 0x19C2C,
	sepith: 0x19C3C,
	time: 0x19C84,
	party: 0x1AFE0,
	chr_names: 0x1B358,
	difficulty: 0x1F36D,
	master_quartz: true,
	playable: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
	max_level: 150,
	dp: 0x19C1C,
	battle: 0x1F378,
	tokens: 0x19C34,
	ranks: &[
		("15th Class", 0),
		("14th Class", 15),
		("13th Class", 35),
		("12th Class", 55),
		("11th Class", 75),
		("10th Class", 95),
		("9th Class", 120),
		("8th Class", 145),
		("7th Class", 170),
		("6th Class", 200),
		("5th Class", 230),
		("4th Class", 260),
		("3rd Class", 290),
		("2nd Class", 330),
		("1st Class", 380),
	],
	rank_rewards: [0, 76, 186, 77, 228, 52, 176, 78, 173, 53, 81, 1626, 1532, 237, 54],
	female: &[1, 2, 5, 32, 8],
};

pub const BATTLE_STATS: [(&str, usize); 10] = [
	("Total Battles Fought", 0),
	("Victories", 4),
	("Character K.O.s Suffered", 2),
	("Times Escaped", 8),
	("Enemies Slain", 20),
	("S-Breaks Used", 12),
	("Support Crafts Used", 14),
	("Party Advantages Gained", 16),
	("Enemy Advantages Suffered", 18),
	("Battles Retried", 10),
];

pub const DIFFICULTY: [(u8, &str); 4] = [(3, "Easy"), (0, "Normal"), (1, "Hard"), (2, "Nightmare")];

pub fn read(path: &Path) -> Result<Vec<u8>, String> {
	let raw = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
	if raw.len() < 8 || raw[..4] != [0x28, 0xB5, 0x2F, 0xFD] {
		return Err(format!("{}: not a Zero / Azure save (no zstd compression)", path.display()));
	}
	let mut dec = ruzstd::decoding::StreamingDecoder::new(&raw[..]).map_err(|e| format!("{}: {e}", path.display()))?;
	let mut d = Vec::new();
	dec.read_to_end(&mut d).map_err(|e| format!("{}: {e}", path.display()))?;
	if d.len() != ZERO.size && d.len() != AZURE.size {
		return Err(format!("{}: data size {} — not a Trails from Zero / Trails to Azure save", path.display(), d.len()));
	}
	Ok(d)
}

pub fn read_for(g: &Profile, path: &Path) -> Result<Vec<u8>, String> {
	let d = read(path)?;
	if d.len() != g.size {
		let other = if d.len() == ZERO.size { ZERO.name } else { AZURE.name };
		return Err(format!("{}: this save is from {other}", path.display()));
	}
	Ok(d)
}

pub fn sums(d: &[u8]) -> (u32, u32) {
	let n = (d.len() - 8) / 4;
	let s = d[..n * 4].chunks_exact(4).fold(0u32, |a, w| a.wrapping_add(u32::from_le_bytes([w[0], w[1], w[2], w[3]])));
	(s, 0u32.wrapping_sub(s.wrapping_add(n as u32)))
}

pub fn sums_ok(d: &[u8]) -> bool {
	let (a, b) = sums(d);
	u32_at(d, d.len() - 8) == a && u32_at(d, d.len() - 4) == b
}

pub fn fix_sums(d: &mut [u8]) {
	let (a, b) = sums(d);
	let n = d.len();
	set_u32(d, n - 8, a);
	set_u32(d, n - 4, b);
}

pub fn write(path: &Path, d: &mut [u8]) -> Result<(), String> {
	fix_sums(d);
	let out = crate::crossbell::save::pack(d);
	let tmp = path.with_extension("tmp");
	std::fs::write(&tmp, &out).map_err(|e| format!("{}: {e}", tmp.display()))?;
	let back = read(&tmp)?;
	if back != d {
		let _ = std::fs::remove_file(&tmp);
		return Err("write check failed: the file does not unpack to the same data".into());
	}
	std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn u32_at(d: &[u8], o: usize) -> u32 {
	u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
pub fn set_u32(d: &mut [u8], o: usize, v: u32) {
	d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
pub fn u16_at(d: &[u8], o: usize) -> u32 {
	u16::from_le_bytes([d[o], d[o + 1]]) as u32
}
pub fn set_u16(d: &mut [u8], o: usize, v: u32) {
	d[o..o + 2].copy_from_slice(&(v.min(0xFFFF) as u16).to_le_bytes());
}

pub fn chr_at(g: &Profile, k: usize) -> usize {
	g.chr + k * RECORD
}

pub mod field {
	pub const HP_MAX: usize = 0;
	pub const HP: usize = 4;
	pub const LEVEL: usize = 8;
	pub const EP_MAX: usize = 10;
	pub const EP: usize = 12;
	pub const CP: usize = 14;
	pub const EXP: usize = 16;
	pub const STATS: usize = 20;
	pub const CP_MAX: usize = 40;
}
pub const STATS: [&str; 8] = ["STR", "DEF", "ATS", "ADF", "DEX", "AGL", "MOV", "SPD"];

pub fn record_name(g: &Profile, d: &[u8], k: usize) -> u32 {
	u16_at(d, g.chr_names + k * 2)
}

pub fn equip_at(g: &Profile, k: usize, slot: usize) -> usize {
	g.equip + k * 10 + slot * 2
}

pub fn orb_at(g: &Profile, k: usize, slot: usize) -> usize {
	g.orb + k * 28 + slot * 4
}

pub fn crafts(g: &Profile, d: &[u8], k: usize) -> Vec<u32> {
	(0..g.craft_n).map(|i| u16_at(d, g.crafts + (k * g.craft_n + i) * 2)).filter(|&c| c != 0 && c != 0xFFFF).collect()
}
pub fn scrafts(g: &Profile, d: &[u8], k: usize) -> Vec<u32> {
	(0..5).map(|i| u16_at(d, g.scrafts + (k * 5 + i) * 2)).filter(|&c| c != 0 && c != 0xFFFF).collect()
}

pub fn party(g: &Profile, d: &[u8]) -> Vec<u32> {
	(0..PARTY).map(|i| u16_at(d, g.party + i * 2)).filter(|&c| c != 0xFF && c != 0xFFFF).collect()
}
pub fn set_party(g: &Profile, d: &mut [u8], list: &[u32]) {
	for i in 0..PARTY {
		set_u16(d, g.party + i * 2, list.get(i).copied().unwrap_or(0xFF));
	}
}

pub fn difficulty(g: &Profile, d: &[u8]) -> u8 {
	d[g.difficulty]
}
pub fn set_difficulty(g: &Profile, d: &mut [u8], v: u8) {
	d[g.difficulty] = v;
}

fn item_slots(g: &Profile, d: &[u8]) -> usize {
	(0..ITEMS_CAP).take_while(|&i| u16_at(d, g.items + i * 4) != 0).count()
}

pub fn item_count(g: &Profile, d: &[u8], id: u32) -> u32 {
	(0..item_slots(g, d)).find(|&i| u16_at(d, g.items + i * 4) == id).map_or(0, |i| u16_at(d, g.items + i * 4 + 2))
}

pub fn set_item_count(g: &Profile, d: &mut [u8], id: u32, n: u32) {
	if id == 0 {
		return;
	}
	let used = item_slots(g, d);
	let at = |i: usize| g.items + i * 4;
	match (0..used).find(|&i| u16_at(d, at(i)) == id) {
		Some(i) if n == 0 => {
			d.copy_within(at(i + 1)..at(used), at(i));
			set_u32(d, at(used - 1), 0);
		}
		Some(i) => set_u16(d, at(i) + 2, n),
		None if n > 0 && used < ITEMS_CAP => {
			set_u16(d, at(used), id);
			set_u16(d, at(used) + 2, n);
		}
		None => {}
	}
}

fn rd16(b: &[u8], o: usize) -> usize {
	b.get(o..o + 2).map_or(0, |x| u16::from_le_bytes([x[0], x[1]]) as usize)
}

fn cstr(b: &[u8], o: usize) -> String {
	let Some(s) = b.get(o..) else { return String::new() };
	let e = s.iter().position(|&c| c == 0).unwrap_or(s.len());
	cp932::decode_lossy(&s[..e])
}

fn grouped(b: &[u8]) -> Vec<usize> {
	let first = rd16(b, 0);
	if first < 2 {
		return Vec::new();
	}
	let starts: Vec<usize> = (0..first / 2).map(|g| rd16(b, g * 2)).collect();
	let Some(end) = starts.iter().map(|&a| rd16(b, a)).filter(|&r| r >= first).min() else { return Vec::new() };
	let mut recs = Vec::new();
	for (g, &s) in starts.iter().enumerate() {
		let e = starts.get(g + 1).copied().unwrap_or(end).min(end);
		recs.extend((s..e).step_by(2).map(|o| rd16(b, o)).filter(|&r| r >= end && r < b.len()));
	}
	recs
}

#[derive(Clone, Debug)]
pub struct Item {
	pub id: u32,
	pub name: String,
	pub desc: String,
	pub cell: u32,
	pub owner: u8,
	pub max: u32,
	pub stats: [i16; 4],
	pub sub: [i8; 4],
	pub rank: u8,
	pub price: u32,
	pub element: u8,
	pub values: [u8; 7],
}

pub const TABS: [(&str, &[u32]); 7] = [
	("Tools", &[14, 19, 22]),
	("Equip", &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]),
	("Quartz", &[18, 26]),
	("Event", &[16, 17]),
	("Ingred.", &[20]),
	("Books", &[21]),
	("Fishing", &[23, 24, 25]),
];

impl Item {
	pub fn service(&self) -> bool {
		self.cell == 18 && self.values.iter().all(|&v| v == 0)
	}
}

pub fn tab_of(it: &Item) -> Option<usize> {
	TABS.iter().position(|t| t.1.contains(&it.cell))
}

pub const CELL_SEPITH: u32 = 105;
pub const CELL_QUARTZ: u32 = 89;
pub const CELL_MASTER: u32 = 73;

pub fn items(item: &[&[u8]], text: &[&[u8]], quartz: &[u8], azure: bool) -> Vec<Item> {
	let mut names: HashMap<u32, (String, String)> = HashMap::new();
	for b in text {
		for r in grouped(b) {
			names.entry(rd16(b, r) as u32).or_insert_with(|| {
				let t = |o: usize| cstr(b, o).replace('㍉', "è").replace('㈱', "♥");
				(t(rd16(b, r + 4)), t(rd16(b, r + 6)))
			});
		}
	}
	let mut elem: HashMap<u32, u8> = HashMap::new();
	let mut vals: HashMap<u32, [u8; 7]> = HashMap::new();
	let (start, end) = if azure { (2, rd16(quartz, 0)) } else { (0, quartz.len()) };
	for o in (start..end.min(quartz.len())).step_by(28).take_while(|o| o + 28 <= end) {
		elem.insert(rd16(quartz, o) as u32 + 100, rd16(quartz, o + 2) as u8);
		let mut v = [0u8; 7];
		v.copy_from_slice(&quartz[o + 21..o + 28]);
		vals.insert(rd16(quartz, o) as u32 + 100, v);
	}
	let mut master: HashMap<u32, u8> = HashMap::new();
	if azure {
		for o in (end..quartz.len()).step_by(44).take_while(|o| o + 44 <= quartz.len()) {
			master.insert(rd16(quartz, o) as u32, rd16(quartz, o + 2) as u8);
		}
	}
	let mut out: Vec<Item> = Vec::new();
	let mut seen = std::collections::HashSet::new();
	for b in item {
		for r in grouped(b) {
			let Some(x) = b.get(r..r + 28) else { continue };
			let id = u16::from_le_bytes([x[0], x[1]]) as u32;
			if id == 0 || id >= 9999 || !seen.insert(id) {
				continue;
			}
			let Some((name, desc)) = names.get(&id).cloned() else { continue };
			let name = name.trim().to_string();
			if name.is_empty() || name.chars().all(|c| c == '-') {
				continue;
			}
			let cell = x[5] as u32;
			let element = if cell == 26 { master.get(&id).copied().unwrap_or(0) } else if cell == 18 { elem.get(&id).copied().unwrap_or(0) } else { 0 };
			let i16at = |o: usize| i16::from_le_bytes([x[o], x[o + 1]]);
			out.push(Item {
				id,
				name,
				desc,
				cell,
				owner: x[3],
				max: if x[4] == 0 { 99 } else { x[4] as u32 },
				stats: [i16at(12), i16at(14), i16at(16), i16at(18)],
				sub: [x[20] as i8, x[21] as i8, x[22] as i8, x[23] as i8],
				rank: x[9],
				price: u32::from_le_bytes([x[24], x[25], x[26], x[27]]),
				element,
				values: if cell == 18 { vals.get(&id).copied().unwrap_or_default() } else { [0; 7] },
			});
		}
	}
	out
}

impl Item {
	pub fn icon(&self) -> Option<u32> {
		match (self.cell, self.element) {
			(18, e @ 1..=7) => Some(CELL_QUARTZ + e as u32 - 1),
			(26, e @ 1..=7) => Some(CELL_MASTER + e as u32 - 1),
			(0 | 99, _) => None,
			(c, _) => Some(c),
		}
	}
	pub fn fits(&self, rec: usize, female: bool) -> bool {
		match self.owner {
			0 => true,
			12 => !female,
			13 => female,
			n => n as usize == rec + 1,
		}
	}
}

pub fn names(b: &[u8]) -> HashMap<u32, String> {
	let first = rd16(b, 2);
	let mut m = HashMap::new();
	let mut o = 0;
	while o + 20 <= first.min(b.len()) {
		let n = cstr(b, rd16(b, o + 2)).trim().to_string();
		if !n.is_empty() {
			m.insert(rd16(b, o) as u32, n);
		}
		o += 20;
	}
	m
}

pub const CALC: [(&str, usize, bool); 9] = [("HP", 0, true), ("STR", 20, false), ("DEF", 22, false), ("ATS", 24, false), ("ADF", 26, false), ("DEX", 28, false), ("AGL", 30, false), ("MOV", 32, false), ("SPD", 34, false)];
pub const RATES: [(&str, usize); 2] = [("ACC", 36), ("EVA", 38)];

pub fn rate(d: &[u8], o: usize) -> i16 {
	i16::from_le_bytes([d[o], d[o + 1]])
}

pub type Mod = (&'static str, i32, bool);

pub fn parse_mods(desc: &str) -> Vec<Mod> {
	let head = desc.split("\\n").next().unwrap_or("").trim_start_matches('[');
	let head = head.split(']').next().unwrap_or("");
	let mut out = Vec::new();
	for part in head.split('/') {
		let part = part.trim();
		let part = part.strip_prefix("Max ").unwrap_or(part);
		let Some(name) = CALC.iter().map(|c| c.0).chain(RATES.iter().map(|r| r.0)).chain(["EP"]).find(|n| part.starts_with(n)) else { continue };
		let rest = part[name.len()..].trim();
		let pct = rest.ends_with('％') || rest.ends_with('%');
		let num: String = rest.chars().take_while(|c| *c == '+' || *c == '-' || c.is_ascii_digit()).collect();
		if let Ok(v) = num.parse::<i32>() {
			out.push((name, v, pct));
		}
	}
	out
}

pub fn mods_table(items: &[Item]) -> HashMap<u32, Vec<Mod>> {
	items.iter().filter(|it| it.cell != 26).map(|it| (it.id, parse_mods(&it.desc))).filter(|(_, m)| !m.is_empty()).collect()
}

fn char_mods(g: &Profile, d: &[u8], k: usize, mods: &HashMap<u32, Vec<Mod>>) -> HashMap<&'static str, (i64, Vec<i64>)> {
	let mut m: HashMap<&'static str, (i64, Vec<i64>)> = HashMap::new();
	let ids = (0..5).map(|i| u16_at(d, equip_at(g, k, i))).chain((0..7).map(|s| u16_at(d, orb_at(g, k, s))));
	for id in ids {
		for &(n, v, pct) in mods.get(&id).map(|v| v.as_slice()).unwrap_or(&[]) {
			let e = m.entry(n).or_default();
			if pct && !RATES.iter().any(|r| r.0 == n) { e.1.push(v as i64) } else { e.0 += v as i64 }
		}
	}
	m
}

pub fn calc_stat(base: i64, flat: i64, pcts: &[i64]) -> i64 {
	base + flat + pcts.iter().map(|p| base * p / 100).sum::<i64>()
}

fn solve_base(total: i64, flat: i64, pcts: &[i64]) -> i64 {
	let mut best = (i64::MAX, 0);
	for b in 0..=total.max(0) + 1 {
		let diff = (calc_stat(b, flat, pcts) - total).abs();
		if diff < best.0 {
			best = (diff, b);
			if diff == 0 {
				break;
			}
		}
	}
	best.1
}

pub type SlotCosts = HashMap<usize, [[u32; 2]; 7]>;

pub fn slot_costs(b: &[u8]) -> SlotCosts {
	let mut offs = Vec::new();
	let mut o = 0;
	while o + 2 <= b.len() && rd16(b, o) != 0xFFFF {
		offs.push(rd16(b, o));
		o += 2;
	}
	offs.push(b.len());
	let mut m = HashMap::new();
	for k in 0..offs.len() - 1 {
		let rs = offs[k + 1].saturating_sub(offs[k]) / 7;
		if rs < 24 {
			continue;
		}
		m.insert(k, std::array::from_fn(|s| [rd16(b, offs[k] + s * rs + 6) as u32, rd16(b, offs[k] + s * rs + 22) as u32]));
	}
	m
}

fn ep_base(g: &Profile, d: &[u8], k: usize, slots: &SlotCosts) -> Option<i64> {
	let c = slots.get(&k)?;
	Some(50 + (0..7).map(|s| (0..(u16_at(d, orb_at(g, k, s) + 2) as usize).min(2)).map(|l| (c[s][l] / 4) as i64).sum::<i64>()).sum::<i64>())
}

pub fn recalc(g: &Profile, d: &mut [u8], prev: &[u8], k: usize, mods: &HashMap<u32, Vec<Mod>>, slots: &SlotCosts) {
	let orig = prev;
	if orig.len() != d.len() {
		return;
	}
	let o = chr_at(g, k);
	let (was, now) = (char_mods(g, orig, k, mods), char_mods(g, d, k, mods));
	let get = |m: &HashMap<&'static str, (i64, Vec<i64>)>, n: &str| m.get(n).cloned().unwrap_or_default();
	for (n, off, wide) in CALC {
		let rd = |d: &[u8]| if wide { u32_at(d, o + off) as i64 } else { u16_at(d, o + off) as i64 };
		let total = rd(orig);
		let (wf, wp) = get(&was, n);
		let (nf, np) = get(&now, n);
		let base = solve_base(total, wf, &wp);
		let rest = total - calc_stat(base, wf, &wp);
		let v = (calc_stat(base, nf, &np) + rest).max(0) as u32;
		if wide { set_u32(d, o + off, v) } else { set_u16(d, o + off, v) }
	}
	for (n, off) in RATES {
		let v = rate(orig, o + off) as i64 - get(&was, n).0 + get(&now, n).0;
		d[o + off..o + off + 2].copy_from_slice(&(v.clamp(i16::MIN as i64, i16::MAX as i64) as i16).to_le_bytes());
	}
	if let (Some(wb), Some(nb)) = (ep_base(g, orig, k, slots), ep_base(g, d, k, slots)) {
		let total = u16_at(orig, o + field::EP_MAX) as i64;
		let rest = total - calc_stat(wb, 0, &get(&was, "EP").1);
		let v = calc_stat(nb, 0, &get(&now, "EP").1) + rest;
		set_u16(d, o + field::EP_MAX, v.max(0) as u32);
	}
	let (hp_max_old, hp_old) = (u32_at(orig, o + field::HP_MAX), u32_at(orig, o + field::HP));
	let hp_max = u32_at(d, o + field::HP_MAX);
	if hp_old >= hp_max_old || u32_at(d, o + field::HP) > hp_max {
		set_u32(d, o + field::HP, hp_max);
	}
	let (ep_max_old, ep_old) = (u16_at(orig, o + field::EP_MAX), u16_at(orig, o + field::EP));
	let ep_max = u16_at(d, o + field::EP_MAX);
	if ep_old >= ep_max_old || u16_at(d, o + field::EP) > ep_max {
		set_u16(d, o + field::EP, ep_max);
	}
}

pub type OrbLayout = ([u8; 7], Vec<Vec<usize>>);

pub fn orb_layouts(b: &[u8]) -> HashMap<usize, OrbLayout> {
	let mut offs = Vec::new();
	let mut o = 0;
	while o + 2 <= b.len() && rd16(b, o) != 0xFFFF {
		offs.push(rd16(b, o));
		o += 2;
	}
	let mut m = HashMap::new();
	for (k, &r) in offs.iter().enumerate() {
		let Some(h) = b.get(r..r + 10) else { continue };
		let mut elems = [0u8; 7];
		elems.copy_from_slice(&h[..7]);
		let lines: Vec<Vec<usize>> = (0..h[9] as usize).filter_map(|i| b.get(r + 10 + i * 8..r + 18 + i * 8)).map(|l| l.iter().take_while(|&&s| s != 0xFF).map(|&s| s as usize).filter(|&s| s < 7).collect()).collect();
		m.insert(k, (elems, lines));
	}
	m
}

#[derive(Clone, Debug)]
pub struct Craft {
	pub name: String,
	pub cell: Option<u32>,
	pub cp: u32,
	pub desc: String,
}

pub fn crafts_table(b: &[u8]) -> HashMap<u32, Craft> {
	let first = rd16(b, 0);
	let offs: Vec<usize> = (0..first / 2).map(|i| rd16(b, i * 2)).collect();
	let mut m = HashMap::new();
	let Some(sz) = offs.get(1).zip(offs.first()).map(|(a, b)| a.saturating_sub(*b)).filter(|&s| (24..=64).contains(&s)) else { return m };
	for (i, &o) in offs.iter().enumerate() {
		if o + sz > b.len() {
			continue;
		}
		let name = cstr(b, rd16(b, o + sz - 4)).trim().to_string();
		if !name.is_empty() {
			let (cp, sn) = if sz >= 32 { (rd16(b, o + 16), rd16(b, o + 18)) } else { (rd16(b, o + 12), b[o + 8] as usize) };
			let desc = strip_codes(&cstr(b, rd16(b, o + sz - 2))).replace("\\n", "\n").trim().to_string();
			m.insert(i as u32, Craft { name, cell: craft_cell(rd16(b, o), cp, sn), cp: cp as u32, desc });
		}
	}
	m
}

pub struct Notes {
	pub log: usize,
	pub record: usize,
	pub bonds: usize,
	pub per_heart: u32,
	pub bond_chr: &'static [(&'static str, u32)],
	pub bond_rec: &'static [(u32, usize)],
}

pub static NOTES_ZERO: Notes = Notes { log: 0x1AF90, record: 0x1F078, bonds: 0x1AF59, per_heart: 5, bond_chr: &[("Elie", 13), ("Tio", 13), ("Randy", 13)], bond_rec: &[(46, 0), (47, 1), (48, 2)] };
pub static NOTES_AZURE: Notes = Notes {
	log: 0x1B36C,
	record: 0x1F454,
	bonds: 0x1B334,
	per_heart: 2,
	bond_chr: &[("KeA", 13), ("Elie", 10), ("Tio", 10), ("Randy", 9), ("Noel", 11), ("Wazy", 9), ("Rixia", 6), ("Dudley", 3), ("Ilya", 13), ("Cecile", 13), ("Fran", 13), ("Sully", 13)],
	bond_rec: &[(48, 1), (49, 2), (50, 3), (51, 4), (52, 5), (53, 6), (54, 7), (55, 0)],
};

pub fn notes(g: &Profile) -> &'static Notes {
	if g.dir == "Azure" { &NOTES_AZURE } else { &NOTES_ZERO }
}

fn mon_code(id: &str) -> u32 {
	0x3000_0000 | u32::from_str_radix(id, 16).unwrap_or(0)
}

pub fn journal(g: &Profile, d: &[u8]) -> Vec<(String, [u8; 4])> {
	let mut o = notes(g).log;
	let mut v = Vec::new();
	while o + 12 <= d.len() && d[o + 7] == 0x30 && v.len() < 1000 {
		v.push((format!("{:x}{:02x}{:02x}", d[o + 6] & 15, d[o + 5], d[o + 4]), [d[o + 8], d[o + 9], d[o + 10], d[o + 11]]));
		o += 8;
	}
	v
}

pub fn logged_mons(g: &Profile, d: &[u8]) -> Vec<String> {
	journal(g, d).into_iter().map(|(id, _)| id).collect()
}

pub fn journal_cap(g: &Profile) -> usize {
	if g.dir == "Azure" { 305 } else { 238 }
}

pub fn journal_full(f: [u8; 4]) -> bool {
	let need = if f[2] & 0x80 != 0 { 0x7F } else { 0x0F };
	f[1] & need == need && f[2] & 0x7F == 0x7F && (f[3] & 0x10 == 0 || f[3] & 1 != 0) && (f[3] & 0x20 == 0 || f[3] & 2 != 0)
}

pub fn set_journal(g: &Profile, d: &mut [u8], m: &Mon, on: bool) {
	let start = notes(g).log + 4;
	let list = journal(g, d);
	let k = list.iter().position(|(x, _)| *x == m.id);
	let (el, card) = if m.seven { (0x7F, 0xFF) } else { (0x0F, 0x7F) };
	match (k, on) {
		(Some(k), true) => d[start + 8 * k + 5..start + 8 * k + 8].copy_from_slice(&[list[k].1[1] | el, card, 0xFF]),
		(None, true) if list.len() < journal_cap(g) => {
			let o = start + 8 * list.len();
			d[o..o + 4].copy_from_slice(&mon_code(&m.id).to_le_bytes());
			d[o + 4..o + 8].copy_from_slice(&[0, el, card, 0xFF]);
		}
		(Some(k), false) => {
			let end = start + 8 * list.len();
			d.copy_within(start + 8 * (k + 1)..end, start + 8 * k);
			d[end - 8..end].fill(0);
		}
		_ => {}
	}
}

pub fn diff_pct(g: &Profile, diff: u8) -> [u32; 6] {
	let azure = g.dir == "Azure";
	match diff {
		3 => [70; 6],
		1 if azure => [117, 118, 118, 120, 120, 120],
		1 => [110, 137, 114, 137, 114, 120],
		2 if azure => [131, 131, 131, 131, 131, 145],
		2 => [122, 183, 116, 183, 116, 145],
		_ => [100; 6],
	}
}

#[derive(Default)]
pub struct Mon {
	pub id: String,
	pub area: String,
	pub level: u32,
	pub name: String,
	pub hp: u32,
	pub stats: [u32; 5],
	pub exp: u32,
	pub elems: [u32; 7],
	pub seven: bool,
	pub sepith: [u32; 7],
	pub items: [(u32, u32); 2],
	pub res: u16,
	pub down: u16,
	pub at: bool,
	pub desc: String,
}

pub fn mon_areas(m: &[Mon]) -> Vec<(String, Vec<usize>)> {
	let mut v: Vec<(String, Vec<usize>)> = Vec::new();
	for (i, x) in m.iter().enumerate() {
		match v.iter_mut().find(|a| a.0 == x.area) {
			Some(a) => a.1.push(i),
			None => v.push((x.area.clone(), vec![i])),
		}
	}
	v
}

pub fn mons(tsv: &str) -> Vec<Mon> {
	let arr = |s: &str| -> Vec<u32> { s.split(',').map(|x| x.trim().parse().unwrap_or(0)).collect() };
	tsv.lines()
		.filter(|l| !l.starts_with('#') && !l.trim().is_empty())
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			let n = |i: usize| f.get(i).and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(0);
			let el = arr(f.get(11).unwrap_or(&""));
			let sp = arr(f.get(13).unwrap_or(&""));
			Some(Mon {
				id: f.first()?.to_string(),
				area: f.get(1)?.to_string(),
				level: n(2),
				name: f.get(3).unwrap_or(&"").to_string(),
				hp: n(4),
				stats: [n(5), n(6), n(7), n(8), n(9)],
				exp: n(10),
				elems: std::array::from_fn(|k| el.get(k).copied().unwrap_or(100)),
				seven: n(12) == 1,
				sepith: std::array::from_fn(|k| sp.get(k).copied().unwrap_or(0)),
				items: [(n(14), n(15)), (n(16), n(17))],
				res: n(18) as u16,
				down: n(19) as u16,
				at: n(20) == 1,
				desc: f.get(21).unwrap_or(&"").replace("\\n", "\n"),
			})
		})
		.collect()
}

pub struct Record {
	pub pos: usize,
	pub rid: u32,
	pub pts: u32,
	pub name: String,
	pub desc: String,
}

pub fn records(b: &[u8]) -> Vec<Record> {
	let s = |o: usize| b.get(o..).and_then(|t| t.iter().position(|&c| c == 0).map(|e| strip_codes(&String::from_utf8_lossy(&t[..e])))).unwrap_or_default();
	let mut first = usize::MAX;
	let mut k = 0;
	while (k + 1) * 12 <= b.len().min(first) {
		first = first.min(u16::from_le_bytes([b[k * 12 + 8], b[k * 12 + 9]]) as usize);
		k += 1;
	}
	(0..k)
		.filter_map(|k| {
			let o = k * 12;
			(b[o] != 0).then(|| Record { pos: k, rid: u32::from_le_bytes([b[o + 4], b[o + 5], b[o + 6], b[o + 7]]), pts: b[o + 1] as u32, name: s(u16::from_le_bytes([b[o + 8], b[o + 9]]) as usize), desc: s(u16::from_le_bytes([b[o + 10], b[o + 11]]) as usize).replace('\n', " ") })
		})
		.collect()
}

pub fn record_done(g: &Profile, d: &[u8], pos: usize) -> bool {
	d[notes(g).record + pos / 8] & (1 << (pos % 8)) != 0
}

pub fn set_record_done(g: &Profile, d: &mut [u8], pos: usize, on: bool) {
	let o = notes(g).record + pos / 8;
	if on { d[o] |= 1 << (pos % 8) } else { d[o] &= !(1 << (pos % 8)) }
}

pub fn bond(g: &Profile, d: &[u8], k: usize) -> u32 {
	d[notes(g).bonds + k] as u32
}

const GIRLS: [(usize, u32); 4] = [(8, 7), (9, 6), (10, 6), (11, 12)];

pub fn set_bond(g: &Profile, d: &mut [u8], k: usize, v: u32) {
	let at = notes(g).bonds;
	if g.dir == "Azure" && k == 0 {
		let (was, full) = (bond(g, d, 0), bond_max(g, 0));
		for (i, top) in GIRLS {
			let part = (top * v.min(full)).div_ceil(full);
			let cur = d[at + i] as u32;
			d[at + i] = if v >= was { cur.max(part) } else { cur.min(part) } as u8;
		}
	}
	d[at + k] = v.min(255) as u8;
}

pub fn bond_max(g: &Profile, k: usize) -> u32 {
	let n = notes(g);
	n.bond_chr.get(k).map_or(0, |c| c.1 * n.per_heart)
}

pub fn strip_codes(s: &str) -> String {
	let c: Vec<char> = s.chars().collect();
	let mut out = String::new();
	let mut i = 0;
	while i < c.len() {
		if c[i] == '#' {
			let mut j = i + 1;
			while j < c.len() && c[j].is_ascii_digit() {
				j += 1;
			}
			if j > i + 1 && j < c.len() && c[j].is_ascii_alphabetic() {
				i = j + 1;
				continue;
			}
		}
		out.push(c[i]);
		i += 1;
	}
	out
}

fn craft_cell(kind: usize, cp: usize, s_num: usize) -> Option<u32> {
	match kind {
		7 => Some(44),
		8 if s_num >= 2 => Some(45),
		8 => Some(42),
		4 if cp == 0 => Some(47),
		4 => Some(46),
		_ => None,
	}
}

