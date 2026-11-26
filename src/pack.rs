use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

pub const FILE: &str = "data.dat";
pub(crate) const MAGIC: &[u8; 8] = b"KZPAK2\0\0";
const MAGIC1: &[u8; 8] = b"KZPAK1\0\0";
pub(crate) const HEAD: usize = 8 + 8 + 4 + 8;
const KEY: [u8; 32] = *b"\x9e\x37\x79\xb9\x7f\x4a\x7c\x15\xf3\x9c\xc0\x60\x5c\xed\xc8\x34\x10\x82\x27\x6b\xf3\xa2\x72\x37\x4c\x8f\x1a\xd5\x63\x2b\x09\xe1";

pub(crate) fn sum(b: &[u8]) -> u64 {
	xxhash_rust::xxh64::xxh64(b, 0)
}

pub(crate) fn xor(data: &mut [u8], nonce: u64) {
	let mut buf = [0u8; 40];
	buf[..32].copy_from_slice(&KEY);
	for (k, chunk) in data.chunks_mut(8).enumerate() {
		buf[32..].copy_from_slice(&(k as u64).to_le_bytes());
		let ks = xxhash_rust::xxh64::xxh64(&buf, nonce).to_le_bytes();
		for (b, s) in chunk.iter_mut().zip(ks) {
			*b ^= s;
		}
	}
}

pub(crate) fn file_nonce(nonce: u64, sum: u64) -> u64 {
	nonce ^ sum.rotate_left(17)
}

struct Entry {
	nested: bool,
	at: u64,
	len: u32,
	sum: u64,
}

pub struct Pack {
	file: Arc<Mutex<File>>,
	data: u64,
	nonce: u64,
	toc: HashMap<String, Entry>,
	subs: Vec<(String, Pack)>,
}

fn take<'a>(b: &'a [u8], o: &mut usize, n: usize) -> Option<&'a [u8]> {
	let s = b.get(*o..o.checked_add(n)?)?;
	*o += n;
	Some(s)
}

fn read_at(f: &Mutex<File>, at: u64, n: usize) -> Option<Vec<u8>> {
	let mut f = f.lock().ok()?;
	f.seek(SeekFrom::Start(at)).ok()?;
	let mut v = Vec::new();
	(&mut *f).take(n as u64).read_to_end(&mut v).ok()?;
	(v.len() == n).then_some(v)
}

impl Pack {
	pub fn open(path: &Path) -> Option<Pack> {
		Pack::at(Arc::new(Mutex::new(File::open(path).ok()?)), 0, 0)
	}

	fn at(file: Arc<Mutex<File>>, base: u64, depth: u8) -> Option<Pack> {
		let h = read_at(&file, base, HEAD)?;
		if &h[..8] != MAGIC || depth > 1 {
			return None;
		}
		let nonce = u64::from_le_bytes(h[8..16].try_into().ok()?);
		let len = u32::from_le_bytes(h[16..20].try_into().ok()?) as usize;
		let mut t = read_at(&file, base + HEAD as u64, len)?;
		xor(&mut t, nonce);
		if sum(&t).to_le_bytes() != h[20..28] {
			return None;
		}
		let data = base + (HEAD + len) as u64;
		let size = file.lock().ok()?.metadata().ok()?.len();
		let mut o = 0;
		let count = u32::from_le_bytes(take(&t, &mut o, 4)?.try_into().ok()?);
		let (mut toc, mut subs) = (HashMap::new(), Vec::new());
		for _ in 0..count {
			let nested = take(&t, &mut o, 1)?[0] == 1;
			let nl = u16::from_le_bytes(take(&t, &mut o, 2)?.try_into().ok()?) as usize;
			let name = String::from_utf8(take(&t, &mut o, nl)?.to_vec()).ok()?;
			let at = u64::from_le_bytes(take(&t, &mut o, 8)?.try_into().ok()?);
			let len = u32::from_le_bytes(take(&t, &mut o, 4)?.try_into().ok()?);
			let sum = u64::from_le_bytes(take(&t, &mut o, 8)?.try_into().ok()?);
			if data.checked_add(at)?.checked_add(len as u64)? > size {
				return None;
			}
			if nested {
				subs.push((name.clone(), Pack::at(file.clone(), data.checked_add(at)?, depth + 1)?));
			}
			toc.insert(name, Entry { nested, at, len, sum });
		}
		Some(Pack { file, data, nonce, toc, subs })
	}

	pub fn get(&self, name: &str) -> Option<Vec<u8>> {
		match self.toc.get(name).filter(|e| !e.nested) {
			Some(e) => {
				let mut b = read_at(&self.file, self.data.checked_add(e.at)?, e.len as usize)?;
				xor(&mut b, file_nonce(self.nonce, e.sum));
				(sum(&b) == e.sum).then_some(b)
			}
			None => self.subs.iter().find_map(|s| s.1.get(name)),
		}
	}

	pub fn names(&self) -> Vec<String> {
		let mut v: Vec<String> = self.toc.iter().filter(|e| !e.1.nested).map(|e| e.0.clone()).chain(self.subs.iter().flat_map(|s| s.1.names())).collect();
		v.sort();
		v
	}

	pub fn nested(&self) -> Vec<(String, usize)> {
		self.subs.iter().map(|s| (s.0.clone(), s.1.names().len())).collect()
	}
}

pub fn check(path: &Path) -> Vec<String> {
	let Some(p) = Pack::open(path) else { return vec![format!("{}: cannot be opened", path.display())] };
	let mut bad = Vec::new();
	for n in p.names() {
		match p.get(&n) {
			None => bad.push(format!("{n}: damaged")),
			Some(b) => {
				let l = n.to_lowercase();
				if (l.ends_with(".png") || l.ends_with(".webp")) && image::load_from_memory(&b).is_err() {
					bad.push(format!("{n}: the image cannot be decoded"));
				}
				if l.ends_with(".tsv") && std::str::from_utf8(&b).is_err() {
					bad.push(format!("{n}: the table is not UTF-8"));
				}
			}
		}
	}
	bad
}

pub fn parse(raw: &[u8]) -> Option<HashMap<String, Vec<u8>>> {
	match raw.get(..8)? {
		m if m == MAGIC1 => parse1(raw),
		m if m == MAGIC => parse2(raw, 0),
		_ => None,
	}
}

fn parse1(raw: &[u8]) -> Option<HashMap<String, Vec<u8>>> {
	if raw.len() < 28 {
		return None;
	}
	let nonce = u64::from_le_bytes(raw[8..16].try_into().ok()?);
	let mut body = raw[16..].to_vec();
	xor(&mut body, nonce);
	let (body, s) = body.split_at(body.len() - 8);
	if sum(body).to_le_bytes() != s {
		return None;
	}
	let mut o = 0usize;
	let count = u32::from_le_bytes(take(body, &mut o, 4)?.try_into().ok()?);
	let mut out = HashMap::new();
	for _ in 0..count {
		let nl = u16::from_le_bytes(take(body, &mut o, 2)?.try_into().ok()?) as usize;
		let name = String::from_utf8(take(body, &mut o, nl)?.to_vec()).ok()?;
		let dl = u32::from_le_bytes(take(body, &mut o, 4)?.try_into().ok()?) as usize;
		out.insert(name, take(body, &mut o, dl)?.to_vec());
	}
	Some(out)
}

fn parse2(raw: &[u8], depth: u8) -> Option<HashMap<String, Vec<u8>>> {
	if raw.len() < HEAD || &raw[..8] != MAGIC || depth > 1 {
		return None;
	}
	let nonce = u64::from_le_bytes(raw[8..16].try_into().ok()?);
	let len = u32::from_le_bytes(raw[16..20].try_into().ok()?) as usize;
	let mut t = raw.get(HEAD..HEAD.checked_add(len)?)?.to_vec();
	xor(&mut t, nonce);
	if sum(&t).to_le_bytes() != raw[20..28] {
		return None;
	}
	let data = &raw[HEAD + len..];
	let mut o = 0;
	let count = u32::from_le_bytes(take(&t, &mut o, 4)?.try_into().ok()?);
	let mut out = HashMap::new();
	for _ in 0..count {
		let nested = take(&t, &mut o, 1)?[0] == 1;
		let nl = u16::from_le_bytes(take(&t, &mut o, 2)?.try_into().ok()?) as usize;
		let name = String::from_utf8(take(&t, &mut o, nl)?.to_vec()).ok()?;
		let mut at = u64::from_le_bytes(take(&t, &mut o, 8)?.try_into().ok()?) as usize;
		let n = u32::from_le_bytes(take(&t, &mut o, 4)?.try_into().ok()?) as usize;
		let s = u64::from_le_bytes(take(&t, &mut o, 8)?.try_into().ok()?);
		let b = take(data, &mut at, n)?;
		if nested {
			out.extend(parse2(b, depth + 1)?);
		} else {
			let mut b = b.to_vec();
			xor(&mut b, file_nonce(nonce, s));
			if sum(&b) != s {
				return None;
			}
			out.insert(name, b);
		}
	}
	Some(out)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
	Ok,
	Missing,
	Broken,
	Old,
}

#[allow(dead_code)]
enum Src {
	Pack(Pack),
	Dir(std::path::PathBuf),
	None(State),
}

fn open(path: &Path) -> Result<Pack, State> {
	if let Some(p) = Pack::open(path) {
		return Ok(p);
	}
	let mut m = [0u8; 8];
	match File::open(path) {
		Err(_) => Err(State::Missing),
		Ok(mut f) => Err(if f.read_exact(&mut m).is_ok() && &m == MAGIC1 { State::Old } else { State::Broken }),
	}
}

static DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

pub fn use_dir(d: &Path) {
	let _ = DIR.set(d.to_path_buf());
}

fn dir_get(d: &Path, name: &str) -> Option<Vec<u8>> {
	["", "enemy.dat", "fish.dat"].iter().find_map(|s| std::fs::read(d.join(s).join(name)).ok())
}

fn src() -> &'static Src {
	static S: OnceLock<Src> = OnceLock::new();
	S.get_or_init(|| {
		#[cfg(debug_assertions)]
		if let Ok(d) = std::env::var("KISEKI_ASSETS") {
			return Src::Dir(d.into());
		}
		let path = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join(FILE))).unwrap_or_default();
		open(&path).map_or_else(Src::None, Src::Pack)
	})
}

pub fn state() -> State {
	match src() {
		Src::None(s) => *s,
		_ => State::Ok,
	}
}

pub fn get(name: &str) -> Option<Vec<u8>> {
	if let Some(d) = DIR.get() {
		return dir_get(d, name);
	}
	match src() {
		Src::Pack(p) => p.get(name),
		Src::Dir(d) => dir_get(d, name),
		Src::None(_) => None,
	}
}

pub fn text(name: &str) -> Option<&'static str> {
	static T: OnceLock<Mutex<HashMap<String, Option<&'static str>>>> = OnceLock::new();
	let mut m = T.get_or_init(Default::default).lock().ok()?;
	*m.entry(name.to_string()).or_insert_with(|| get(name).and_then(|b| String::from_utf8(b).ok()).map(|s| &*Box::leak(s.into_boxed_str())))
}

pub fn has_game(g: &str) -> bool {
	let pre = format!("{g}/");
	match src() {
		Src::Pack(p) => p.names().iter().any(|k| k.starts_with(&pre)),
		Src::Dir(d) => d.join(g).is_dir(),
		Src::None(_) => false,
	}
}

