use super::ed7save::{Profile, item_count, set_item_count, set_u16, u16_at};

const SIZE: usize = 116;
const PLACES: usize = 4;
const COUNTS: usize = 4;
const COUNT_N: usize = 50;
const BAIT: usize = 104;
const BAIT_FIRST: u32 = 350;
const GAINED: usize = 112;
const RECORD: usize = 114;

pub struct Medal {
	pub flag: usize,
	pub item: u32,
	pub at: (f32, f32),
}

const FLAGS: usize = 0x1B008;

pub static MEDALS: [Medal; 5] = [
	Medal { flag: 3590, item: 40, at: (432.0, 138.0) },
	Medal { flag: 3587, item: 43, at: (591.0, 213.0) },
	Medal { flag: 3589, item: 41, at: (591.0, 138.0) },
	Medal { flag: 3588, item: 42, at: (432.0, 213.0) },
	Medal { flag: 3209, item: 44, at: (511.5, 175.5) },
];

pub fn medals(g: &Profile) -> &'static [Medal] {
	if g.master_quartz { &MEDALS } else { &[] }
}

fn base(g: &Profile) -> usize {
	if g.master_quartz { 0x19CC0 } else { 0x19C4C }
}

pub struct Fish {
	pub idx: usize,
	pub item: u32,
	pub min: u32,
	pub max: u32,
	pub gains: [u32; 4],
	pub baits: Vec<u32>,
	pub places: Vec<(u8, String)>,
	pub desc: String,
}

impl Fish {
	fn gain_bits(&self) -> u32 {
		(0..4).filter(|&q| self.gains[q] != 0).fold(0, |a, q| a | 2 << q)
	}
	pub fn place(&self, n: u8) -> &str {
		self.places.iter().find(|x| x.0 == n).map_or("?", |x| &x.1)
	}
}

pub fn fishes(tsv: &str) -> Vec<Fish> {
	let nums = |s: &str| -> Vec<u32> { s.split(',').filter_map(|v| v.parse().ok()).collect() };
	tsv.lines()
		.filter(|l| !l.starts_with('#'))
		.filter_map(|l| {
			let c: Vec<&str> = l.split('\t').collect();
			let [idx, item, min, max, gains, baits, places, desc] = c[..] else { return None };
			Some(Fish {
				idx: idx.parse().ok()?,
				item: item.parse().ok()?,
				min: min.parse().ok()?,
				max: max.parse().ok()?,
				gains: nums(gains).try_into().ok()?,
				baits: nums(baits),
				places: places.split(';').filter_map(|p| p.split_once('=').and_then(|(n, t)| Some((n.parse().ok()?, t.to_string())))).collect(),
				desc: desc.to_string(),
			})
		})
		.collect()
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rec {
	pub record: u32,
	pub caught: u32,
	pub places: Vec<u8>,
	pub baits: Vec<u32>,
	pub gained: u32,
}

fn at(g: &Profile, f: &Fish) -> usize {
	base(g) + f.idx * SIZE
}

pub fn rec(g: &Profile, d: &[u8], f: &Fish) -> Rec {
	let o = at(g, f);
	Rec {
		record: u16_at(d, o + RECORD),
		caught: (0..COUNT_N).map(|k| u16_at(d, o + COUNTS + k * 2)).sum(),
		places: d[o..o + PLACES].iter().copied().filter(|&p| p != 0).collect(),
		baits: (0..64).filter(|k| d[o + BAIT + k / 8] >> (k % 8) & 1 == 1).map(|k| BAIT_FIRST + k as u32).collect(),
		gained: u16_at(d, o + GAINED),
	}
}

fn reveal(g: &Profile, d: &mut [u8], f: &Fish) {
	let o = at(g, f);
	let mut list = rec(g, d, f).places;
	for (n, _) in &f.places {
		if !list.contains(n) {
			list.push(*n);
		}
	}
	for k in 0..PLACES {
		d[o + k] = list.get(k).copied().unwrap_or(0);
	}
	for b in &f.baits {
		let k = (b - BAIT_FIRST) as usize;
		d[o + BAIT + k / 8] |= 1 << (k % 8);
	}
	let gained = u16_at(d, o + GAINED) | f.gain_bits();
	set_u16(d, o + GAINED, gained);
}

pub fn mark(g: &Profile, d: &mut [u8], f: &Fish, on: bool) {
	let o = at(g, f);
	if !on {
		d[o..o + SIZE].fill(0);
		return;
	}
	let r = rec(g, d, f);
	if r.record < f.min {
		set_u16(d, o + RECORD, f.min);
	}
	if r.caught == 0 {
		set_caught(g, d, f, 1);
	}
	reveal(g, d, f);
}

pub fn set_caught(g: &Profile, d: &mut [u8], f: &Fish, n: u32) {
	if n == 0 {
		return mark(g, d, f, false);
	}
	let o = at(g, f);
	let r = rec(g, d, f);
	let Some(first) = r.places.first().copied().or(f.places.first().map(|x| x.0)) else { return };
	if r.places.is_empty() {
		d[o] = first;
	}
	let cell = |k: usize| o + COUNTS + k * 2;
	if n >= r.caught {
		let v = u16_at(d, cell(first as usize)) + n - r.caught;
		set_u16(d, cell(first as usize), v);
	} else {
		let mut cut = r.caught - n;
		let order = (0..COUNT_N).rev().filter(|&k| k != first as usize).chain([first as usize]);
		for k in order {
			let v = u16_at(d, cell(k));
			let m = v.min(cut);
			set_u16(d, cell(k), v - m);
			cut -= m;
		}
	}
	if r.caught == 0 {
		if r.record < f.min {
			set_u16(d, o + RECORD, f.min);
		}
		reveal(g, d, f);
	}
}

pub fn set_record(g: &Profile, d: &mut [u8], f: &Fish, v: u32) {
	if rec(g, d, f).caught == 0 {
		mark(g, d, f, true);
	}
	set_u16(d, at(g, f) + RECORD, v.clamp(f.min, f.max));
}

pub fn counts(g: &Profile, d: &[u8], list: &[Fish]) -> (usize, u32) {
	list.iter().map(|f| rec(g, d, f).caught).fold((0, 0), |a, c| (a.0 + usize::from(c > 0), a.1 + c))
}

pub fn has_medal(d: &[u8], m: &Medal) -> bool {
	d[FLAGS + m.flag / 8] >> (m.flag % 8) & 1 == 1
}

pub fn set_medal(g: &Profile, d: &mut [u8], m: &Medal, on: bool) {
	let bit = 1u8 << (m.flag % 8);
	if on {
		d[FLAGS + m.flag / 8] |= bit;
		if item_count(g, d, m.item) == 0 {
			set_item_count(g, d, m.item, 1);
		}
	} else {
		d[FLAGS + m.flag / 8] &= !bit;
		set_item_count(g, d, m.item, 0);
	}
}

