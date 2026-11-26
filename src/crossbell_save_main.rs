#![cfg_attr(windows, windows_subsystem = "windows")]

mod gui_common;
mod extra;

use eframe::egui::{self, Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use gui_common::*;
use kiseki_tool::crossbell::ed7save::{self as save, Profile, field};
use kiseki_tool::crossbell::fish;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const TITLE: &str = concat!("Crossbell Save Editor ", env!("VER_CROSSBELLSAVEEDITOR"), env!("VER_SUFFIX"));

fn title() -> &'static str {
	TITLE
}

#[path = "crossbell_fish.rs"]
mod fish_tab;

const GAMES: [(&str, &str); 2] = [("zero", "Trails from Zero"), ("azure", "Trails to Azure")];

fn profile(game: usize) -> &'static Profile {
	if game == 1 { &save::AZURE } else { &save::ZERO }
}

const SEPITH: [&str; 7] = ["Earth", "Water", "Fire", "Wind", "Time", "Space", "Mirage"];

fn atlas_uv(t: &egui::TextureHandle, c: u32) -> Rect {
	let s = t.size();
	let (w, h) = (64.0 / s[0] as f32, 64.0 / s[1] as f32);
	Rect::from_min_size(Pos2::new((c % 16) as f32 * w, (c / 16) as f32 * h), Vec2::new(w, h))
}

struct MonCard<'a> {
	g: &'static save::Profile,
	m: &'a save::Mon,
	f: Option<[u8; 4]>,
	diff: u8,
	atlas: Option<&'a egui::TextureHandle>,
	icons: Option<&'a egui::TextureHandle>,
	notes: Option<&'a egui::TextureHandle>,
	items: &'a [save::Item],
	item_of: &'a HashMap<u32, usize>,
}

const ELEM_COL: [(&str, [u8; 3]); 7] = [("Earth", [214, 150, 40]), ("Water", [60, 150, 230]), ("Fire", [220, 60, 60]), ("Wind", [40, 190, 110]), ("Time", [120, 120, 132]), ("Space", [225, 195, 90]), ("Mirage", [200, 204, 214])];

impl MonCard<'_> {
	const NARROW: f32 = 900.0;

	fn narrow(w: f32) -> (f32, f32) {
		((w - 44.0 - 20.0) / (557.0 + 566.0), ((w - 44.0) / 660.0).clamp(0.45, 1.0))
	}

	fn size(avail: Vec2) -> Vec2 {
		if avail.x >= Self::NARROW {
			return avail.max(Vec2::new(0.0, 560.0));
		}
		let w = avail.x.max(520.0);
		let (q, z) = Self::narrow(w);
		Vec2::new(w, avail.y.max(14.0 + 467.0 * q + 16.0 + 730.0 * z))
	}

	fn show(&self, ui: &mut egui::Ui, p: &Pal, pick: &mut Option<u8>) -> bool {
		let m = self.m;
		let r = ui.max_rect();
		let pt = ui.painter().clone();
		let known_el = |k: usize| self.f.is_none_or(|f| f[1] >> k & 1 == 1);
		let known = |bit: u8| self.f.is_none_or(|f| f[2] & bit != 0);
		let known_item = |k: usize| self.f.is_none_or(|f| f[3] >> k & 1 == 1);
		let narrow = r.width() < Self::NARROW;
		let q = if narrow { Self::narrow(r.width()).0 } else { ((r.width() * 0.42) / 566.0).min((r.height() - 36.0) / 820.0).max(0.3) };
		let frame = Rect::from_min_size(Pos2::new(r.min.x + 22.0, r.min.y + 14.0 + 42.0 * q), Vec2::new(557.0, 425.0) * q);
		pt.rect(frame, 10, p.div2, Stroke::new(3.0_f32, p.div), StrokeKind::Inside);
		pt.rect_stroke(frame.shrink(7.0), 7, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
		let sheet = if narrow { Rect::from_min_size(Pos2::new(frame.max.x + 20.0, frame.min.y + 49.0 * q), Vec2::new(566.0, 327.0) * q) } else { Rect::from_min_size(Pos2::new(frame.min.x - 3.0 * q, frame.max.y + 26.0 * q), Vec2::new(566.0, 327.0) * q) };
		pt.rect_filled(sheet, 8, p.div2);
		if let Some(t) = self.atlas {
			let k = 2.0 * q;
			let cell = Rect::from_min_size(frame.min + Vec2::new(22.0 * q, 22.0 * q - 33.0 * k), Vec2::splat(256.0 * k));
			pt.image(t.id(), cell, UV_ALL, if self.f.is_some() { Color32::WHITE } else { Color32::from_gray(110) });
		}
		let txt = if known(8) { m.desc.clone() } else { "???".into() };
		let lines: Vec<&str> = txt.lines().collect();
		let fit = |size: f32| lines.iter().map(|l| pt.layout_no_wrap(l.to_string(), f_body(size), p.text).size().x).fold(0.0, f32::max);
		let size = 40.0 * q;
		let size = size * ((sheet.width() - 40.0 * q) / fit(size).max(1.0)).min(1.0);
		for (k, l) in lines.iter().enumerate() {
			pt.text(Pos2::new(sheet.min.x + 24.0 * q, sheet.min.y + (60.0 + k as f32 * 69.0) * q), Align2::LEFT_CENTER, *l, f_body(size), p.text);
		}
		let (rx, top) = if narrow { (frame.min.x, frame.max.y + 16.0) } else { (sheet.max.x + 28.0, r.min.y) };
		let rw = r.max.x - 20.0 - rx;
		let z = if narrow { Self::narrow(r.width()).1 } else { (rw / 660.0).min((r.height() - 12.0) / 730.0).clamp(0.45, 1.0) };
		let mut y = top + 33.0 * z;
		let fit_name = |s: f32| pt.layout_no_wrap(format!("◆ {} ◆", m.name), f_semi(s), p.text).size().x;
		let combo_w = 240.0;
		let ns = 25.0 * z;
		let ns = ns * ((rw - combo_w - 12.0) / fit_name(ns).max(1.0)).min(1.0);
		pt.text(Pos2::new(rx, y), Align2::LEFT_CENTER, format!("◆ {} ◆", m.name), f_semi(ns), p.text);
		let mut diff = pick.unwrap_or(self.diff);
		{
			let cr = Rect::from_min_max(Pos2::new(rx + rw - combo_w, y - 14.0), Pos2::new(rx + rw, y + 14.0));
			let mut c = ui.new_child(egui::UiBuilder::new().max_rect(cr).layout(egui::Layout::right_to_left(egui::Align::Center)));
			let name = |v: u8| save::DIFFICULTY.iter().find(|d| d.0 == v).map_or_else(|| format!("? ({v})"), |d| d.1.to_string());
			egui::ComboBox::from_id_salt("mon_diff").width(130.0).selected_text(name(diff)).show_ui(&mut c, |ui| {
				for (k, t) in save::DIFFICULTY {
					let t = if k == self.diff { format!("{t} — this save") } else { t.to_string() };
					ui.selectable_value(&mut diff, k, t);
				}
			});
			c.label(egui::RichText::new("Difficulty").font(f_body(13.0)).color(p.muted));
		}
		*pick = (diff != self.diff).then_some(diff);
		let pc = save::diff_pct(self.g, diff);
		let sc = |x: u32, k: usize| (x as u64 * pc[k] as u64 / 100).to_string();
		let hidden = |v: String| if known(0x40) { v } else { "????".into() };
		let part = |bit: u8, v: String| if known(bit) { v } else { "???".into() };
		let rows: [Vec<(&str, String)>; 2] = [
			vec![("Lv", part(2, m.level.to_string())), ("HP", part(1, sc(m.hp, 0))), ("EXP", part(4, m.exp.to_string()))],
			vec![("STR", hidden(sc(m.stats[0], 1))), ("DEF", hidden(sc(m.stats[1], 2))), ("ATS", hidden(sc(m.stats[2], 3))), ("ADF", hidden(sc(m.stats[3], 4))), ("SPD", hidden(sc(m.stats[4], 5)))],
		];
		y = top + 64.0 * z;
		for row in &rows {
			let gap = 10.0 * z;
			let bw = (rw - gap * (row.len() - 1) as f32) / row.len() as f32;
			for (k, (lab, v)) in row.iter().enumerate() {
				let bx = Rect::from_min_size(Pos2::new(rx + k as f32 * (bw + gap), y), Vec2::new(bw, 36.0 * z));
				pt.rect_filled(bx, 4, p.div2);
				pt.text(Pos2::new(bx.min.x + 12.0 * z, bx.center().y), Align2::LEFT_CENTER, *lab, f_body(16.0 * z), p.muted);
				pt.text(Pos2::new(bx.max.x - 12.0 * z, bx.center().y), Align2::RIGHT_CENTER, v, f_semi(17.0 * z), p.text);
			}
			y += 44.0 * z;
		}
		y += 16.0 * z;
		let head = |y: f32, x: f32, t: &str| pt.text(Pos2::new(x, y), Align2::LEFT_CENTER, t, f_body(16.0 * z), p.muted);
		head(y, rx, "[Elemental Efficacy]");
		let (n, step) = if m.seven { (7, 27.0 * z) } else { (4, 47.0 * z) };
		for (k, (name, c)) in ELEM_COL.iter().enumerate().take(n) {
			let yy = y + 28.0 * z + k as f32 * step;
			let v = m.elems[k];
			pt.text(Pos2::new(rx, yy), Align2::LEFT_CENTER, *name, f_body(16.0 * z), p.text);
			pt.text(Pos2::new(rx + 125.0 * z, yy), Align2::RIGHT_CENTER, if known_el(k) { v.to_string() } else { "???".into() }, f_semi(16.0 * z), p.text);
			let bar = Rect::from_min_max(Pos2::new(rx + 145.0 * z, yy - 5.0 * z), Pos2::new(rx + rw, yy + 5.0 * z));
			pt.rect_filled(bar, 5, p.div);
			if known_el(k) {
				let fill = Rect::from_min_size(bar.min, Vec2::new(bar.width() * (v.min(200) as f32 / 200.0), bar.height()));
				pt.rect_filled(fill, 5, Color32::from_rgb(c[0], c[1], c[2]));
			}
		}
		y += 226.0 * z;
		head(y, rx, "[Resistances]");
		let s = ((rw - 9.0 * 7.0 * z) / 10.0).min(50.0 * z);
		let g = 7.0 * z;
		if !known(0x10) {
			pt.text(Pos2::new(rx + 12.0, y + 34.0 * z), Align2::LEFT_CENTER, "???", f_semi(18.0 * z), p.text);
		} else if let Some(t) = self.notes {
			let ts = t.size();
			let cell = |row: usize, col: usize| Rect::from_min_size(Pos2::new((col as f32 * 128.0 + 9.0) / ts[0] as f32, (row as f32 * 128.0 + 9.0) / ts[1] as f32), Vec2::new(111.0 / ts[0] as f32, 111.0 / ts[1] as f32));
			let tint = |on: bool| if on { Color32::WHITE } else { Color32::from_white_alpha(46) };
			let y1 = y + 16.0 * z;
			for k in 0..10 {
				pt.image(t.id(), Rect::from_min_size(Pos2::new(rx + k as f32 * (s + g), y1), Vec2::splat(s)), cell(0, k), tint(m.res >> k & 1 == 1));
			}
			let y2 = y1 + s + 6.0 * z;
			pt.image(t.id(), Rect::from_min_size(Pos2::new(rx, y2), Vec2::splat(s)), cell(0, 10), tint(m.res >> 10 & 1 == 1));
			let order: [usize; 8] = if self.g.dir == "Azure" { [2, 3, 4, 5, 6, 7, 8, 9] } else { [2, 3, 4, 5, 9, 6, 8, 7] };
			for (k, &c) in order.iter().enumerate() {
				pt.image(t.id(), Rect::from_min_size(Pos2::new(rx + (k + 1) as f32 * (s + g), y2), Vec2::splat(s)), cell(1, c), tint(m.down >> k & 1 == 1));
			}
			pt.image(t.id(), Rect::from_min_size(Pos2::new(rx + 9.0 * (s + g), y2), Vec2::splat(s)), cell(1, 10), tint(m.at));
		}
		y += 16.0 * z + 2.0 * s + 6.0 * z + 28.0 * z;
		head(y, rx, "[Sepith Gained]");
		let ix = rx + rw * 0.57;
		if !known(0x20) {
			pt.text(Pos2::new(rx + 12.0, y + 34.0 * z), Align2::LEFT_CENTER, "???", f_semi(18.0 * z), p.text);
		} else if let Some(t) = self.icons {
			let step = ((ix - rx - 12.0) / 7.0).min(54.0 * z);
			let si = step.min(44.0 * z);
			let mut x = rx;
			for (k, &v) in m.sepith.iter().enumerate() {
				if v == 0 {
					continue;
				}
				pt.image(t.id(), Rect::from_min_size(Pos2::new(x, y + 14.0 * z), Vec2::splat(si)), atlas_uv(t, 56 + k as u32), Color32::WHITE);
				pt.text(Pos2::new(x + si / 2.0, y + 72.0 * z), Align2::CENTER_CENTER, v.to_string(), f_semi(17.0 * z), p.text);
				x += step;
			}
		}
		head(y, ix, "[Items Gained]");
		let mut yy = y + 36.0 * z;
		for (k, &(id, chance)) in m.items.iter().enumerate() {
			if id == 0 || !known_item(k) {
				continue;
			}
			let it = self.item_of.get(&id).map(|&i| &self.items[i]);
			if let (Some(t), Some(cell)) = (self.icons, it.and_then(|it| it.icon())) {
				pt.image(t.id(), Rect::from_center_size(Pos2::new(ix + 17.0 * z, yy), Vec2::splat(34.0 * z)), atlas_uv(t, cell), Color32::WHITE);
			}
			let name = it.map_or_else(|| format!("#{id}"), |it| it.name.clone());
			paint_text_clipped(ui, Rect::from_min_max(Pos2::new(ix + 44.0 * z, yy - 18.0 * z), Pos2::new(rx + rw - 46.0 * z, yy + 18.0 * z)), &name, f_body(16.0 * z), p.text, Align2::LEFT_CENTER);
			pt.text(Pos2::new(rx + rw, yy), Align2::RIGHT_CENTER, format!("{chance}%"), f_body(14.0 * z), p.muted);
			yy += 40.0 * z;
		}
		if !self.f.is_some_and(save::journal_full) {
			let msg = if self.f.is_some() { "Partial entry — some data is «???», as in game; the tick in the list logs it fully" } else { "Not in the log — hidden in game" };
			let by = r.max.y - 27.0;
			let mut b = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_max(Pos2::new(rx, by - 16.0), Pos2::new(rx + rw, by + 16.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
			b.add(egui::Label::new(egui::RichText::new(msg).font(f_body(12.5)).color(p.caution)).truncate());
		}
		false
	}
}

fn help_icon(ui: &mut egui::Ui, p: &Pal, tip: impl FnOnce(&mut egui::Ui)) {
	let (r, resp) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
	let col = if resp.hovered() { p.accent } else { p.muted };
	ui.painter().circle_stroke(r.center(), 7.5, Stroke::new(1.2_f32, col));
	ui.painter().text(r.center() + Vec2::new(0.0, 0.5), Align2::CENTER_CENTER, "?", f_semi(11.5), col);
	if resp.hovered() {
		egui::Area::new(resp.id.with("help"))
			.order(egui::Order::Tooltip)
			.fixed_pos(r.right_top() + Vec2::new(8.0, 0.0))
			.constrain(true)
			.interactable(false)
			.show(ui.ctx(), |ui| egui::Frame::popup(ui.style()).show(ui, tip));
	}
}

fn sepith_icon(ui: &mut egui::Ui, atlas: &Option<egui::TextureHandle>, k: usize, size: f32) {
	match atlas {
		Some(t) => {
			let r = atlas_uv(t, save::CELL_SEPITH + k as u32);
			let (w, h) = (r.width() / 64.0, r.height() / 64.0);
			let uv = Rect::from_min_size(r.min + Vec2::new(0.0, 12.0 * h), Vec2::new(37.0 * w, 40.0 * h));
			let (b, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
			ui.painter().image(t.id(), Rect::from_min_size(b.min, Vec2::new(size * 37.0 / 40.0, size)), uv, Color32::WHITE);
		}
		None => {
			ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
		}
	}
}

fn atlas_cell(ui: &mut egui::Ui, atlas: &Option<egui::TextureHandle>, cell: Option<u32>, size: f32) {
	match (atlas, cell) {
		(Some(t), Some(c)) => {
			ui.add(egui::Image::new(t).uv(atlas_uv(t, c)).fit_to_exact_size(Vec2::splat(size)));
		}
		_ => {
			ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
		}
	}
}

struct Slot {
	name: String,
	dir: PathBuf,
	title: String,
	sub: String,
	when: String,
	playtime: String,
	difficulty: String,
}

fn info_str(b: &[u8], o: usize) -> String {
	let Some(s) = b.get(o..) else { return String::new() };
	let e = s.iter().position(|&c| c == 0 || c == b'\n').unwrap_or(s.len());
	cp932::decode_lossy(&s[..e]).replace(['\u{301C}', '\u{FF5E}'], "~").trim().to_string()
}

fn read_slots(root: &Path) -> Vec<Slot> {
	let mut v: Vec<Slot> = std::fs::read_dir(root)
		.into_iter()
		.flatten()
		.flatten()
		.filter(|e| e.path().join("savedata.dat").is_file())
		.filter_map(|e| {
			let name = e.file_name().to_string_lossy().to_string();
			if !name.starts_with("data") {
				return None;
			}
			let info = std::fs::read(e.path().join("info.txt")).unwrap_or_default();
			let head = info_str(&info, 0);
			let (title, difficulty) = head.rsplit_once('\u{3000}').map(|(a, b)| (a.trim().to_string(), b.trim().to_string())).unwrap_or((head.clone(), String::new()));
			if title.is_empty() && difficulty.is_empty() || head.contains("System") || head.contains("\u{421}\u{438}\u{441}\u{442}\u{435}\u{43c}") {
				return None;
			}
			let line = info_str(&info, 0x80);
			let playtime = line.rsplit(' ').next().filter(|t| t.matches(':').count() == 2).unwrap_or("").to_string();
			let sub = line.split("  ").next().unwrap_or("").trim().to_string();
			Some(Slot { name, dir: e.path(), title, sub, when: info_str(&info, 0x480), playtime, difficulty })
		})
		.collect();
	v.sort_by(|a, b| a.name.cmp(&b.name));
	v
}

fn default_dir(game: usize) -> String {
	std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join("Saved Games").join("Falcom").join(profile(game).falcom)).filter(|p| p.is_dir()).map(|p| p.display().to_string()).unwrap_or_default()
}

fn stamp() -> String {
	let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
	let (days, rem) = (s.div_euclid(86400), s.rem_euclid(86400));
	let z = days + 719_468;
	let era = z.div_euclid(146_097);
	let doe = z - era * 146_097;
	let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
	let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
	let mp = (5 * doy + 2) / 153;
	let d = doy - (153 * mp + 2) / 5 + 1;
	let m = if mp < 10 { mp + 3 } else { mp - 9 };
	let y = yoe + era * 400 + i64::from(m <= 2);
	format!("{y:04}-{m:02}-{d:02}_{:02}-{:02}-{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
	std::fs::create_dir_all(to)?;
	for e in std::fs::read_dir(from)?.flatten() {
		if e.path().is_file() {
			std::fs::copy(e.path(), to.join(e.file_name()))?;
		}
	}
	Ok(())
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum LPos {
	Line(usize),
	Avail(usize),
}

fn lineup_swap(list: &mut [u32], have: &[u32], a: LPos, b: LPos) {
	match (a, b) {
		(LPos::Line(i), LPos::Line(j)) => list.swap(i, j),
		(LPos::Line(i), LPos::Avail(j)) | (LPos::Avail(j), LPos::Line(i)) => list[i] = have[j],
		(LPos::Avail(_), LPos::Avail(_)) => {}
	}
}

fn lineup_move(list: &mut Vec<u32>, have: &[u32], a: LPos, zone: u8) {
	match (a, zone) {
		(LPos::Line(i), 2) if list.len() > 1 => {
			list.remove(i);
		}
		(LPos::Line(i), 1) => {
			let c = list.remove(i);
			list.push(c);
		}
		(LPos::Avail(_), 0) if list.len() >= save::PARTY_ACTIVE => {}
		(LPos::Avail(j), 0 | 1) if list.len() < save::PARTY => list.push(have[j]),
		_ => {}
	}
}

#[derive(Clone)]
enum Act {
	Overwrite,
	Close,
	Open(usize),
}

struct App {
	pal: Pal,
	st: Settings,
	dir: String,
	slots: Vec<Slot>,
	sel: Option<usize>,
	data: Option<Vec<u8>>,
	orig: Vec<u8>,
	tab: usize,
	item_tab: usize,
	lsel: Option<LPos>,
	drag_src: Option<LPos>,
	chr_sel: usize,
	pick_game: Option<usize>,
	search: String,
	owned_only: bool,
	show_service: bool,
	collapsed: std::collections::HashSet<String>,
	game: Option<usize>,
	g: &'static Profile,
	items: Vec<save::Item>,
	mons: Vec<save::Mon>,
	records: Vec<save::Record>,
	fish: Vec<fish::Fish>,
	fish_sel: usize,
	item_of: HashMap<u32, usize>,
	names: HashMap<u32, String>,
	crafts: HashMap<u32, save::Craft>,
	orbs: HashMap<usize, save::OrbLayout>,
	mods: HashMap<u32, Vec<save::Mod>>,
	slot_costs: save::SlotCosts,
	faces: HashMap<u32, Vec<u8>>,
	face_tex: HashMap<u32, Option<egui::TextureHandle>>,
	art_tex: HashMap<String, Option<egui::TextureHandle>>,
	hover_game: Option<usize>,
	icons: Option<Option<egui::TextureHandle>>,
	rec_tex: Option<Option<egui::TextureHandle>>,
	rec_sel: usize,
	mon_sel: Option<usize>,
	mon_diff: Option<u8>,
	mon_open: std::collections::HashSet<String>,
	pics: Pics,
	note_tex: Option<Option<egui::TextureHandle>>,
	status: String,
	dialog: Dialog<Act>,
	slot_pick: Option<SlotPick>,
	zoom: f32,
	about: bool,
	tips: bool,
	allow_close: bool,
	scan_at: f64,
	scan_sig: u64,
	scan_dir: String,
	scroll_sel: bool,
	slot_q: String,
	shown_title: String,
}

fn slot_label(name: &str) -> String {
	match name.strip_prefix("data").and_then(|n| n.parse::<u32>().ok()) {
		Some(n @ 1..=199) => format!("{name} · slot {n}"),
		_ => name.to_string(),
	}
}

fn dir_signature(root: &Path) -> u64 {
	use std::hash::{Hash, Hasher};
	let mut h = std::collections::hash_map::DefaultHasher::new();
	let mut v: Vec<(String, u64)> = std::fs::read_dir(root)
		.into_iter()
		.flatten()
		.flatten()
		.filter_map(|e| {
			let m = std::fs::metadata(e.path().join("savedata.dat")).ok()?;
			let t = m.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
			Some((e.file_name().to_string_lossy().to_string(), t))
		})
		.collect();
	v.sort();
	v.hash(&mut h);
	h.finish()
}

impl App {
	fn new(cc: &eframe::CreationContext) -> App {
		setup_fonts(&cc.egui_ctx);
		number_cursor(&cc.egui_ctx);
		tick_green();
		egui_extras::install_image_loaders(&cc.egui_ctx);
		let st = Settings::load("Crossbell-SaveEditor.ini", "Crossbell-SaveEditor");
		let pal = Pal::get(st.get("dark") != Some("0"));
		apply_visuals(&cc.egui_ctx, &pal);
		let mut app = App {
			pal,
			dir: String::new(),
			st,
			slots: Vec::new(),
			sel: None,
			data: None,
			orig: Vec::new(),
			tab: 0,
			item_tab: 0,
			lsel: None,
			drag_src: None,
			chr_sel: 0,
			pick_game: None,
			search: String::new(),
			owned_only: true,
			show_service: false,
			collapsed: std::collections::HashSet::new(),
			game: None,
			g: &save::ZERO,
			items: Vec::new(),
			mons: Vec::new(),
			records: Vec::new(),
			fish: Vec::new(),
			fish_sel: 0,
			rec_tex: None,
			rec_sel: 0,
			mon_sel: None,
			mon_diff: None,
			mon_open: std::collections::HashSet::new(),
			pics: Pics::default(),
			note_tex: None,
			item_of: HashMap::new(),
			names: HashMap::new(),
			crafts: HashMap::new(),
			orbs: HashMap::new(),
			mods: HashMap::new(),
			slot_costs: HashMap::new(),
			faces: HashMap::new(),
			face_tex: HashMap::new(),
			art_tex: HashMap::new(),
			hover_game: None,
			icons: None,
			status: "Select a slot on the left. Before writing, the program backs the slot up to the «Backups» folder next to it.".into(),
			dialog: Dialog::None,
			slot_pick: None,
			zoom: 1.0,
			about: false,
			tips: false,
			allow_close: false,
			scan_at: 0.0,
			scan_sig: 0,
			scan_dir: String::new(),
			scroll_sel: false,
			slot_q: String::new(),
			shown_title: String::new(),
		};
		app.zoom = app.st.get("zoom").and_then(|v| v.parse().ok()).unwrap_or(1.0f32).clamp(0.7, 1.6);
		cc.egui_ctx.set_zoom_factor(app.zoom);
		app.tips = app.st.get("tips_seen") != Some("1");
		app.show_service = app.st.get("show_service") == Some("1");
		app.tab = match app.st.get("tab3").and_then(|v| v.parse::<usize>().ok()) {
			Some(v) => v.min(5),
			None => match app.st.get("tab").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0) {
				3 => 2,
				2 => 1,
				v => v.min(1),
			},
		};
		app.pick_game = app.st.get("last_game").and_then(|v| v.parse::<usize>().ok()).filter(|&i| i < GAMES.len());
		app
	}

	fn key(&self, k: &str) -> String {
		format!("{k}_{}", GAMES[self.game.unwrap_or(0)].0)
	}

	fn table(&self, f: &str) -> Vec<u8> {
		kiseki_tool::pack::get(&format!("{}/{f}", self.g.dir)).unwrap_or_default()
	}

	fn set_game(&mut self, i: usize) {
		self.save_settings();
		self.game = Some(i);
		self.st.set("last_game", &i.to_string());
		self.g = profile(i);
		let azure = self.g.master_quartz;
		self.items = save::items(&[&self.table("t_item._dt"), &self.table("t_item2._dt")], &[&self.table("t_ittxt._dt"), &self.table("t_ittxt2._dt")], &self.table("t_quartz._dt"), azure);
		self.item_of = self.items.iter().enumerate().map(|(k, it)| (it.id, k)).collect();
		self.mons = save::mons(std::str::from_utf8(&self.table("mons.tsv")).unwrap_or(""));
		self.records = save::records(&self.table("t_record._dt"));
		self.fish = fish::fishes(std::str::from_utf8(&self.table("fish.tsv")).unwrap_or(""));
		self.fish_sel = 0;
		self.names = save::names(&self.table("t_name._dt"));
		self.crafts = save::crafts_table(&self.table("t_magic._dt"));
		self.orbs = save::orb_layouts(&self.table("t_orb._dt"));
		self.mods = save::mods_table(&self.items);
		self.slot_costs = save::slot_costs(&self.table("t_sltget._dt"));
		self.faces = kiseki_tool::crossbell::save::faces(&self.table("faces.bin"));
		self.face_tex.clear();
		self.icons = None;
		self.rec_tex = None;
		self.rec_sel = 0;
		self.mon_sel = None;
		self.mon_diff = None;
		self.mon_open.clear();
		self.note_tex = None;
		self.data = None;
		self.sel = None;
		self.chr_sel = 0;
		self.lsel = None;
		self.drag_src = None;
		self.dir = self.st.get(&self.key("dir")).map(|s| s.to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| default_dir(i));
		self.item_tab = self.st.get(&self.key("item_tab")).and_then(|v| v.parse().ok()).unwrap_or(0).min(save::TABS.len() - 1);
		if self.items.is_empty() {
			self.status = format!("✖ No {} tables in data.dat next to the program — item names can't be shown", self.g.dir);
		}
		self.rescan();
		if let Some(k) = self.st.get(&self.key("slot")).and_then(|n| self.slots.iter().position(|s| s.name == n)) {
			self.open(k);
		}
	}

	fn save_settings(&mut self) {
		self.st.set("dark", if self.pal.dark { "1" } else { "0" });
		self.st.set("tab3", &self.tab.to_string());
		self.st.set("zoom", &format!("{:.2}", self.zoom));
		if self.game.is_some() {
			self.st.set(&self.key("dir"), &self.dir);
			self.st.set(&self.key("item_tab"), &self.item_tab.to_string());
			let slot = self.sel.and_then(|i| self.slots.get(i)).map(|s| s.name.clone()).unwrap_or_default();
			self.st.set(&self.key("slot"), &slot);
		}
		self.st.save();
	}

	fn vis_slots(&self) -> Vec<usize> {
		(0..self.slots.len()).filter(|&i| slot_matches(&self.slots[i].name, &self.slot_q)).collect()
	}

	fn rescan(&mut self) {
		let keep = self.sel.and_then(|i| self.slots.get(i)).map(|s| s.name.clone());
		self.slots = read_slots(Path::new(self.dir.trim()));
		self.sel = keep.and_then(|n| self.slots.iter().position(|s| s.name == n));
	}

	fn open(&mut self, i: usize) {
		self.mon_diff = None;
		self.sel = Some(i);
		self.scroll_sel = true;
		match save::read_for(self.g, &self.slots[i].dir.join("savedata.dat")) {
			Ok(d) => {
				self.status = if save::sums_ok(&d) {
					format!("Opened {}: {} — {}", self.slots[i].name, self.slots[i].title, self.slots[i].sub)
				} else {
					format!("Opened {}: the checksums don't match (the slot was edited by another tool) — they will be recalculated on write", self.slots[i].name)
				};
				self.orig = d.clone();
				self.data = Some(d);
			}
			Err(e) => {
				self.data = None;
				self.wrong_game(&e);
				self.status = format!("✖ {e}");
			}
		}
	}

	fn wrong_game(&mut self, e: &str) {
		if let (Some((_, other)), Some(i)) = (e.rsplit_once(": this save is from "), self.game) {
			self.dialog = Dialog::Info(format!("This is a save from {other}, but the selected game is {}.\n\nChoose the right game (“Select game”) or another save folder.", GAMES[i].1));
		}
	}

	fn open_file(&mut self, file: &Path) {
		let (Some(slot), Some(root)) = (file.parent(), file.parent().and_then(|p| p.parent())) else { return };
		self.dir = root.display().to_string();
		self.rescan();
		match self.slots.iter().position(|s| s.dir == slot) {
			Some(i) => self.open(i),
			None => match save::read_for(self.g, file) {
				Ok(d) => {
					self.slots.push(Slot { name: slot.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), dir: slot.to_path_buf(), title: "(opened file)".into(), sub: file.display().to_string(), when: String::new(), playtime: String::new(), difficulty: String::new() });
					self.sel = Some(self.slots.len() - 1);
					self.orig = d.clone();
					self.data = Some(d);
					self.status = format!("Opened {}", file.display());
				}
				Err(e) => {
					self.wrong_game(&e);
					self.status = format!("✖ {e}");
				}
			},
		}
		self.save_settings();
	}

	fn texture(ctx: &egui::Context, name: &str, png: &[u8]) -> Option<egui::TextureHandle> {
		image::load_from_memory(png).ok().map(|img| {
			let img = img.to_rgba8();
			ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw()), egui::TextureOptions::LINEAR)
		})
	}

	fn face(&mut self, ctx: &egui::Context, who: u32) -> Option<egui::TextureHandle> {
		if let Some(t) = self.face_tex.get(&who) {
			return t.clone();
		}
		let t = self.faces.get(&who).and_then(|png| image::load_from_memory(png).ok()).map(|img| {
			face_texture(ctx, format!("face{who}"), &img.to_rgba8())
		});
		self.face_tex.insert(who, t.clone());
		t
	}

	fn cover(&mut self, ctx: &egui::Context, key: &str) -> Option<egui::TextureHandle> {
		if let Some(t) = self.art_tex.get(key) {
			return t.clone();
		}
		let t = kiseki_tool::pack::get(&format!("covers/{key}.png")).and_then(|png| Self::texture(ctx, key, &png));
		self.art_tex.insert(key.to_string(), t.clone());
		t
	}

	fn cover_blur(&mut self, ctx: &egui::Context, key: &str) -> Option<egui::TextureHandle> {
		let name = format!("blur:{key}");
		if let Some(t) = self.art_tex.get(&name) {
			return t.clone();
		}
		let t = kiseki_tool::pack::get(&format!("covers/{key}.png")).and_then(|png| blurred_texture(ctx, &name, &png));
		self.art_tex.insert(name, t.clone());
		t
	}

	fn record_tex(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
		if self.rec_tex.is_none() {
			self.rec_tex = Some(Self::texture(ctx, "record", &self.table("record.png")));
		}
		self.rec_tex.clone().flatten()
	}

	fn note_icons(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
		if self.note_tex.is_none() {
			self.note_tex = Some(Self::texture(ctx, "note_icons", &self.table("note_icons.png")));
		}
		self.note_tex.clone().flatten()
	}

	fn icons(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
		if self.icons.is_none() {
			self.icons = Some(Self::texture(ctx, "icons", &self.table("c_icon1.png")));
		}
		self.icons.clone().flatten()
	}

	fn changed(&self) -> bool {
		self.data.as_ref().is_some_and(|d| *d != self.orig)
	}

	fn ask_open(&mut self, i: usize) {
		if self.sel == Some(i) && self.data.is_some() {
			return;
		}
		if self.changed() {
			self.dialog = Dialog::Ask("There are unsaved changes. Open another slot without saving?".into(), Act::Open(i));
		} else {
			self.open(i);
		}
	}

	fn overwrite(&mut self) {
		let (Some(i), Some(mut d)) = (self.sel, self.data.clone()) else { return };
		let slot = &self.slots[i];
		let backup = exe_dir().join("Backups").join(GAMES[self.game.unwrap_or(0)].1).join(format!("{}_{}", slot.name, stamp()));
		if let Err(e) = copy_dir(&slot.dir, &backup) {
			self.dialog = Dialog::Info(format!("Could not back up the slot — it was not changed.\n{e}"));
			return;
		}
		match save::write(&slot.dir.join("savedata.dat"), &mut d) {
			Ok(()) => {
				self.orig = d.clone();
				self.data = Some(d);
				self.status = format!("Written to {}. Backup of the previous slot: {}", slot.name, backup.display());
			}
			Err(e) => self.dialog = Dialog::Info(format!("Could not write {}:\n{e}\n\nThe slot was not changed (backup: {}).", slot.name, backup.display())),
		}
	}

	fn ask_slot(&mut self) {
		let mut used = std::collections::BTreeMap::new();
		for e in std::fs::read_dir(self.dir.trim()).into_iter().flatten().flatten() {
			let name = e.file_name().to_string_lossy().to_string();
			let Some(n) = name.strip_prefix("data").filter(|n| n.len() == 4).and_then(|n| n.parse::<u32>().ok()) else { continue };
			let s = self.slots.iter().find(|s| s.name == name);
			used.insert(n, UsedSlot { title: s.map(|s| s.title.clone()).unwrap_or_default(), when: s.map(|s| s.when.clone()).unwrap_or_default(), locked: s.is_none() });
		}
		self.slot_pick = Some(SlotPick::new((1, 199), used));
	}

	fn save_to(&mut self, num: u32) {
		let (Some(i), Some(mut d)) = (self.sel, self.data.clone()) else { return };
		let n = format!("data{num:04}");
		if self.slots[i].name == n {
			self.overwrite();
			return;
		}
		let to = PathBuf::from(self.dir.trim()).join(&n);
		let from = self.slots[i].dir.clone();
		let backup = to.exists().then(|| exe_dir().join("Backups").join(GAMES[self.game.unwrap_or(0)].1).join(format!("{n}_{}", stamp())));
		if let Some(b) = &backup
			&& let Err(e) = copy_dir(&to, b)
		{
			self.dialog = Dialog::Info(format!("Could not back up the slot — it was not changed.\n{e}"));
			return;
		}
		let res = (|| -> Result<(), String> {
			std::fs::create_dir_all(&to).map_err(|e| e.to_string())?;
			save::write(&to.join("savedata.dat"), &mut d)?;
			for f in ["info.txt", "icon0.png", "messagelog.dat"] {
				if from.join(f).is_file() {
					std::fs::copy(from.join(f), to.join(f)).map_err(|e| e.to_string())?;
				}
			}
			Ok(())
		})();
		match res {
			Ok(()) => {
				let name = self.slots[i].name.clone();
				self.rescan();
				if let Some(k) = self.slots.iter().position(|s| s.name == n) {
					self.open(k);
				}
				self.status = match &backup {
					Some(b) => format!("Written to slot {n} (in game it looks like {name}). Backup of the previous one: {}", b.display()),
					None => format!("Written to new slot {n} (in game it looks like {name})."),
				};
			}
			Err(e) => match &backup {
				Some(b) => {
					let back = copy_dir(b, &to).is_ok();
					self.dialog = Dialog::Info(if back { format!("Could not write {n}:\n{e}\n\nThe previous slot was restored (backup: {}).", b.display()) } else { format!("Could not write {n}:\n{e}\n\nThe previous slot is in the backup: {}", b.display()) });
				}
				None => {
					let _ = std::fs::remove_dir_all(&to);
					self.dialog = Dialog::Info(format!("Could not write {n}:\n{e}"));
				}
			},
		}
	}

	fn who_name(&self, who: u32) -> String {
		short_name(&self.names, who)
	}
	fn party_name_id(&self, d: &[u8], p: u32) -> u32 {
		if (p as usize) < save::CHARS { save::record_name(self.g, d, p as usize) } else { p }
	}

	fn picker(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let p = self.pal;
		let full = ui.max_rect();
		let covers: Vec<Option<egui::TextureHandle>> = GAMES.iter().map(|g| self.cover(ctx, g.0)).collect();
		const ACCENT: [Color32; 2] = [Color32::from_rgb(0xE0, 0x8A, 0x3C), Color32::from_rgb(0x4A, 0x8F, 0xE7)];
		const ARC: [&str; 2] = ["CROSSBELL ARC · I", "CROSSBELL ARC · II"];
		let mut open = None;
		let blurs: Vec<Option<egui::TextureHandle>> = GAMES.iter().map(|g| self.cover_blur(ctx, g.0)).collect();
		let (head, foot) = picker_bg(ui, &p, full, &blurs, self.hover_game);
		let mut hovered = None;
		ui.painter().text(Pos2::new(head.min.x + 24.0, head.center().y), Align2::LEFT_CENTER, title(), f_semi(15.0), p.text);
		{
			let mut r = ui.new_child(egui::UiBuilder::new().max_rect(head.shrink2(Vec2::new(24.0, 14.0))).layout(egui::Layout::right_to_left(egui::Align::Center)));
			r.spacing_mut().item_spacing.x = 8.0;
			if theme_button(&mut r, &p, 28.0).clicked() {
				self.pal = Pal::get(!self.pal.dark);
				apply_visuals(ctx, &self.pal);
				self.save_settings();
			}
			if small_button(&mut r, &p, "About", Some("info"), 28.0, true, p.muted).clicked() {
				self.about = true;
			}
		}
		let cw = (full.width() - 64.0).min(960.0);
		let x0 = full.center().x - cw / 2.0;
		let gap = 24.0;
		let cols = if cw >= 2.0 * 280.0 + gap { 2 } else { 1 };
		let card_w = (cw - gap * (cols - 1) as f32) / cols as f32;
		let rows = GAMES.len().div_ceil(cols);
		let fit_h = (foot.min.y - head.max.y - 84.0 - 52.0 - (rows - 1) as f32 * gap) / rows as f32 - 116.0;
		let art_h = (card_w * 430.0 / 920.0).min(fit_h.max(120.0));
		let card_h = art_h + 116.0;
		let block_h = 84.0 + rows as f32 * card_h + (rows - 1) as f32 * gap;
		let mut y = head.max.y + ((foot.min.y - head.max.y - block_h) / 2.0).max(40.0);
		ui.painter().text(Pos2::new(x0, y), Align2::LEFT_TOP, "Select a game", f_semi(26.0), p.text);
		y += 40.0;
		ui.painter().text(Pos2::new(x0, y), Align2::LEFT_TOP, "Then pick a save slot. The editor backs up the slot before writing.", f_body(14.0), p.muted);
		y += 44.0;
		let area = Rect::from_min_max(Pos2::new(full.min.x, y), Pos2::new(full.max.x, foot.min.y - 12.0));
		let mut c = ui.new_child(egui::UiBuilder::new().max_rect(area));
		egui::ScrollArea::vertical().id_salt("picker").auto_shrink([false, false]).show(&mut c, |ui| {
			let top = ui.cursor().min.y;
			ui.allocate_space(Vec2::new(area.width(), rows as f32 * (card_h + gap)));
			for (i, g) in GAMES.iter().enumerate() {
				let r = Rect::from_min_size(Pos2::new(x0 + (i % cols) as f32 * (card_w + gap), top + (i / cols) as f32 * (card_h + gap)), Vec2::new(card_w, card_h));
				let resp = ui.interact(r, ui.id().with(("game", i)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
				let on = self.pick_game == Some(i);
				picker_card(ui, &p, r, art_h, &PickCard { cover: covers[i].as_ref(), arc: ARC[i], title: g.1, color: ACCENT[i], selected: on, ready: true }, resp.hovered());
				if resp.hovered() {
					hovered = Some(i);
				}
				if resp.double_clicked() {
					open = Some(i);
				} else if resp.clicked() {
					self.pick_game = if on { None } else { Some(i) };
				}
			}
		}).middle_scroll(&c);
		if hovered != self.hover_game {
			self.hover_game = hovered;
			ctx.request_repaint();
		}
		match self.pick_game {
			Some(i) => {
				ui.painter().text(Pos2::new(x0, foot.center().y - 9.0), Align2::LEFT_CENTER, "Selected", f_body(11.5), p.muted);
				ui.painter().text(Pos2::new(x0, foot.center().y + 9.0), Align2::LEFT_CENTER, GAMES[i].1, f_semi(15.0), p.text);
			}
			None => {
				ui.painter().text(Pos2::new(x0, foot.center().y), Align2::LEFT_CENTER, "Back up your save files before editing. This is an unofficial fan tool.", f_body(13.0), p.muted);
			}
		}
		let br = Rect::from_min_max(Pos2::new(x0, foot.min.y + 14.0), Pos2::new(x0 + cw, foot.max.y - 14.0));
		let mut b = ui.new_child(egui::UiBuilder::new().max_rect(br).layout(egui::Layout::right_to_left(egui::Align::Center)));
		b.spacing_mut().item_spacing.x = 10.0;
		let sel = self.pick_game;
		if primary_button_icon(&mut b, &p, "Continue", Some("arrow-right"), Vec2::new(150.0, 40.0), 9, f_semi(15.0), sel.is_some()).clicked() {
			open = sel;
		}
		if small_button(&mut b, &p, "Browse save folder…", Some("folder-open"), 40.0, sel.is_some(), p.text).on_hover_text("Choose the save folder of the selected game").clicked()
			&& let Some(i) = sel
				&& let Some(dir) = pick_folder_with_files(&default_dir(i), "Save folder (or a file)") {
					self.set_game(i);
					let root = if dir.is_file() { dir.ancestors().nth(2).unwrap_or(&dir) } else { &dir };
					self.dir = root.display().to_string();
					self.data = None;
					self.sel = None;
					self.rescan();
					self.save_settings();
					self.shown_title.clear();
				}
		if let Some(i) = open {
			self.pick_game = Some(i);
			self.set_game(i);
			self.shown_title.clear();
		}
	}

	fn windows(&mut self, ctx: &egui::Context) {
		let p = self.pal;
		if self.about {
			let backups = exe_dir().join("Backups");
			let mut open = true;
			egui::Window::new("About").open(&mut open).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
				ui.label(egui::RichText::new(title()).font(f_semi(16.0)).color(p.text));
				ui.label(egui::RichText::new("Save editor for Trails from Zero and Trails to Azure (PC).").font(f_body(13.0)).color(p.muted));
				ui.label(egui::RichText::new("Edits: mira and tokens, difficulty, rank, party, items, Combat Notebook, System Record.").font(f_body(13.0)).color(p.muted));
				ui.add_space(8.0);
				ui.label(egui::RichText::new("Slot backups:").font(f_body(12.5)).color(p.muted));
				ui.label(egui::RichText::new(backups.display().to_string()).font(f_mono(12.0)).color(p.text));
				ui.add_space(8.0);
				ui.horizontal(|ui| {
					if small_button(ui, &p, "Open backups folder", Some("folder-open"), 28.0, true, p.text).clicked() {
						let _ = std::fs::create_dir_all(&backups);
						open_folder(&backups.display().to_string());
					}
					if small_button(ui, &p, "Program folder", Some("folder-open"), 28.0, true, p.text).clicked() {
						open_folder(&exe_dir().display().to_string());
					}
				});
				ui.add_space(10.0);
				let mut svc = self.show_service;
				if checkbox(ui, &p, &mut svc, "Show internal items", 16.0, f_body(13.0)).on_hover_text("The game's test quartz (“100％ Poison”, “Phys Reflect”…). Hidden by default.").changed() {
					self.show_service = svc;
					self.st.set("show_service", if svc { "1" } else { "0" });
				}
				ui.add_space(6.0);
				ui.label(egui::RichText::new("Unofficial fan tool. Keep backups of your saves.").font(f_body(12.0)).color(p.muted));
			});
			self.about = open;
		}
		if self.tips && self.game.is_some() {
			if self.st.get("tips_seen") != Some("1") {
				self.st.set("tips_seen", "1");
				self.st.save();
			}
			let mut close = false;
			egui::Window::new("Tips").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
				ui.set_max_width(560.0);
				for t in [
					"Save slots are on the left: search by number, Up / Down arrows — next slot. «Browse…» — the save folder or a file; you can also drop a savedata.dat file onto the window.",
					"«General» — mira, tokens, difficulty, DP and detective rank, sepith, combat notebook, play time.",
					"«Party» — drag a card onto another one, or click two cards, to swap them; click a card and then an empty «+» slot to move it. Into «Can be added» — remove from the party.",
					"«Items» — the amount of any item; untick «Owned only» to add a new one. Click a name for its description, a section title to fold it.",
					"«Monsters» — Combat Notebook: a tick — fully logged, a dash — partial entry. Difficulty on the card only changes the numbers shown.",
					"«System Record» — click an icon: obtained / not; hover for its name and description.",
					"«Fishing» — Fishing Note: a tick — caught with everything revealed; pale text — where and with what to catch it. Azure, TOP: click a medal slot to get or remove it.",
					"«Save to new slot» — pick the number; a used slot gets overwritten. The previous save is always copied to «Backups» next to the program (About opens the folder) first.",
					"Ctrl+S — save to this slot, Ctrl + mouse wheel — zoom. Middle-click in a list — scroll by moving the mouse (click again or Esc to stop).",
					"Numbers: drag left/right, or click and type. «?» next to a rank — thresholds and rewards.",
					"The game's test quartz is hidden — you can show it in «About».",
				] {
					ui.label(egui::RichText::new(format!("•  {t}")).font(f_body(13.0)).color(p.text));
					ui.add_space(3.0);
				}
				ui.add_space(6.0);
				if primary_button_icon(ui, &p, "Got it", Some("check"), Vec2::new(120.0, 30.0), 8, f_semi(14.0), true).clicked() {
					close = true;
				}
			});
			if close {
				self.tips = false;
				self.st.set("tips_seen", "1");
				self.st.save();
			}
		}
	}

	fn global_input(&mut self, ctx: &egui::Context) {
		let z = ctx.input(|i| i.zoom_delta());
		if z != 1.0 {
			self.zoom = (self.zoom * z).clamp(0.7, 1.6);
			ctx.set_zoom_factor(self.zoom);
			self.save_settings();
		}
		if let Some(path) = ctx.input(|i| i.raw.dropped_files.iter().find_map(|f| f.path.clone())) {
			let game = save::read(&path).ok().map(|d| usize::from(d.len() == save::AZURE.size));
			match game {
				_ if self.changed() => self.dialog = Dialog::Info("There are unsaved changes — save them first or reopen the slot.".into()),
				Some(gi) => {
					if self.game != Some(gi) {
						self.pick_game = Some(gi);
						self.set_game(gi);
						self.shown_title.clear();
					}
					self.open_file(&path);
				}
				None => self.dialog = Dialog::Info(format!("{} is not a Zero / Azure save.", path.display())),
			}
		}
		if ctx.input(|i| i.viewport().close_requested()) && self.changed() && !self.allow_close {
			ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
			self.dialog = Dialog::Ask("There are unsaved changes. Close without saving?".into(), Act::Close);
		}
		let now = ctx.input(|i| i.time);
		if self.game.is_some() && now - self.scan_at > 2.0 {
			self.scan_at = now;
			let sig = dir_signature(Path::new(self.dir.trim()));
			if self.scan_dir != self.dir.trim() {
				self.scan_dir = self.dir.trim().to_string();
				self.scan_sig = 0;
			}
			if self.scan_sig != 0 && sig != self.scan_sig {
				self.rescan();
				self.status = match (self.sel.and_then(|i| self.slots.get(i)), self.data.is_some()) {
					(Some(s), _) => format!("Slot list updated · open: {}", s.name),
					(None, true) => "Slot list updated · the open slot is no longer in the folder (save to a new slot)".into(),
					(None, false) => "Slot list updated — the save folder changed".into(),
				};
				self.scroll_sel = true;
			}
			self.scan_sig = sig;
		}
		ctx.request_repaint_after(std::time::Duration::from_secs(2));
		if self.game.is_none() || !matches!(self.dialog, Dialog::None) || self.slot_pick.is_some() {
			return;
		}
		let typing = ctx.memory(|m| m.focused().is_some());
		let (ctrl_s, up, down) = ctx.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::S), i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::ArrowDown)));
		if ctrl_s && self.changed() {
			let name = self.sel.map(|i| self.slots[i].name.clone()).unwrap_or_default();
			self.dialog = Dialog::Ask(format!("Write changes to {name}?\n\nThe previous slot is first copied to the «Backups» folder next to the program."), Act::Overwrite);
		}
		if typing {
			return;
		}
		let vis = self.vis_slots();
		if (up || down) && !vis.is_empty() {
			let k = match self.sel.and_then(|s| vis.iter().position(|&v| v == s)) {
				Some(k) if up => k.saturating_sub(1),
				Some(k) => (k + 1).min(vis.len() - 1),
				None => 0,
			};
			self.ask_open(vis[k]);
		}
	}

	fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let p = self.pal;
		let full = ui.max_rect();
		let x0 = full.min.x + 20.0;
		let w = full.width() - 40.0;
		let mut y = full.min.y + 16.0;
		ui.painter().galley(Pos2::new(x0, y), ui.fonts(|f| f.layout_job(caps_job("Save folder", p.muted))), p.muted);
		{
			let mut r = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(Pos2::new(x0, y + 21.0), Vec2::new(w, 36.0))));
			let hint = format!("C:\\Users\\…\\Saved Games\\Falcom\\{}", self.g.falcom);
			let (ch, cl, _) = path_field(&mut r, &p, "dir", &mut self.dir, 36.0, &hint, "Browse…", false, true);
			let br = Rect::from_min_size(Pos2::new(x0 + w - 360.0, y - 4.0), Vec2::new(360.0, 22.0));
			let mut b = ui.new_child(egui::UiBuilder::new().max_rect(br).layout(egui::Layout::right_to_left(egui::Align::Center)));
			if small_button(&mut b, &p, "Select game", Some("switch"), 22.0, true, p.muted).on_hover_text("Back to game selection").clicked() {
				self.save_settings();
				self.game = None;
				self.status.clear();
				self.shown_title.clear();
			}
			let one = if cl { pick_folder_with_files(&self.dir, "Save folder (or a file)") } else { None };
			if one.is_some() && self.changed() {
				self.dialog = Dialog::Info("There are unsaved changes — save them first or reopen the slot.".into());
			} else if let Some(f) = one.as_ref().map(|d| if d.is_dir() { d.join("savedata.dat") } else { d.clone() }).filter(|f| f.is_file()) {
				self.open_file(&f);
			} else if ch || one.is_some() {
				if let Some(d) = one {
					self.dir = d.display().to_string();
				}
				self.data = None;
				self.sel = None;
				self.rescan();
				self.save_settings();
			}
		}
		y += 21.0 + 36.0 + 8.0;
		{
			let mut r = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(Pos2::new(x0, y), Vec2::new(w, 28.0))).layout(egui::Layout::right_to_left(egui::Align::Center)));
			r.spacing_mut().item_spacing.x = 8.0;
			if theme_button(&mut r, &p, 26.0).clicked() {
				self.pal = Pal::get(!self.pal.dark);
				apply_visuals(ctx, &self.pal);
				self.save_settings();
			}
			if small_button(&mut r, &p, "About", Some("info"), 26.0, true, p.muted).clicked() {
				self.about = true;
			}
			if small_button(&mut r, &p, "Tips", Some("help"), 26.0, true, p.muted).clicked() {
				self.tips = true;
			}
		}
		y += 36.0;
		let bottom = full.max.y - 56.0;
		let lw = 300.0;
		let list = Rect::from_min_max(Pos2::new(x0, y), Pos2::new(x0 + lw, bottom));
		ui.painter().rect(list, 10, Color32::TRANSPARENT, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
		let mut open = None;
		{
			let mut c = ui.new_child(egui::UiBuilder::new().max_rect(list.shrink(6.0)));
			let total = self.slots.len();
			let found = self.vis_slots().len();
			if slot_search(&mut c, &p, "slot_q", &mut self.slot_q, found, total) {
				self.scroll_sel = true;
			}
			let vis = self.vis_slots();
			let mut area = egui::ScrollArea::vertical().id_salt(("slots", self.g.dir)).auto_shrink([false, false]);
			if std::mem::take(&mut self.scroll_sel)
				&& let Some(k) = self.sel.and_then(|s| vis.iter().position(|&v| v == s)) {
					let row = 82.0 + c.spacing().item_spacing.y;
					let view = c.available_height();
					let cur = egui::scroll_area::State::load(c.ctx(), c.make_persistent_id(egui::Id::new(("slots", self.g.dir)))).map_or(0.0, |s| s.offset.y);
					let top = k as f32 * row;
					if top < cur || top + 82.0 > cur + view {
						area = area.vertical_scroll_offset(if top < cur { top } else { top + 82.0 - view });
					}
				}
			area.show_rows(&mut c, 82.0, vis.len(), |ui, range| {
				for i in range.map(|k| vis[k]) {
					let s = &self.slots[i];
					let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 82.0), Sense::click());
					if self.sel == Some(i) {
						ui.painter().rect_filled(r, 7, p.sel);
					} else if resp.hovered() {
						ui.painter().rect_filled(r, 7, p.div2.gamma_multiply(0.6));
					}
					ui.painter().text(Pos2::new(r.min.x + 10.0, r.min.y + 13.0), Align2::LEFT_CENTER, slot_label(&s.name), f_mono(12.0), p.muted);
					ui.painter().text(Pos2::new(r.max.x - 10.0, r.min.y + 11.0), Align2::RIGHT_CENTER, &s.when, f_body(11.5), p.muted);
					let w = r.width() - 20.0;
					paint_text_clipped(ui, Rect::from_min_size(Pos2::new(r.min.x + 10.0, r.min.y + 24.0), Vec2::new(w, 18.0)), &s.title, f_semi(13.5), p.text, Align2::LEFT_CENTER);
					paint_text_clipped(ui, Rect::from_min_size(Pos2::new(r.min.x + 10.0, r.min.y + 43.0), Vec2::new(w, 17.0)), &s.sub, f_body(12.5), p.text, Align2::LEFT_CENTER);
					let mut pt = if s.playtime.is_empty() { String::new() } else { format!("Play time {}", s.playtime) };
					if !s.difficulty.is_empty() {
						pt += &format!("  ·  {}", s.difficulty);
					}
					paint_text_clipped(ui, Rect::from_min_size(Pos2::new(r.min.x + 10.0, r.min.y + 62.0), Vec2::new(w, 16.0)), &pt, f_body(11.5), p.muted, Align2::LEFT_CENTER);
					if resp.clicked() {
						open = Some(i);
					}
				}
			}).middle_scroll(&c);
			if self.slots.is_empty() {
				ui.painter().text(list.center(), Align2::CENTER_CENTER, "No saves found", f_body(13.0), p.muted);
			} else if vis.is_empty() {
				ui.painter().text(list.center(), Align2::CENTER_CENTER, "No slot with this number", f_body(13.0), p.muted);
			}
		}
		if let Some(i) = open {
			self.ask_open(i);
		}
		let right = Rect::from_min_max(Pos2::new(x0 + lw + 16.0, y), Pos2::new(x0 + w, bottom));
		if self.data.is_none() {
			ui.painter().rect(right, 10, Color32::TRANSPARENT, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
			ui.painter().text(right.center(), Align2::CENTER_CENTER, "Select a slot on the left", f_body(14.0), p.muted);
		} else {
			let mut r = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(right.min, Vec2::new(right.width(), 36.0))));
			let tabs = [(0, "General"), (1, "Party"), (2, "Items"), (3, "Monsters"), (5, "Fishing"), (4, "System Record")];
			let mut k = tabs.iter().position(|t| t.0 == self.tab).unwrap_or(0);
			segmented(&mut r, &p, "tab", &tabs.map(|t| t.1), &mut k, None, 36.0, 9, f_body(14.0), &[]);
			self.tab = tabs[k].0;
			let body = Rect::from_min_max(Pos2::new(right.min.x, right.min.y + 48.0), right.max);
			ui.painter().rect(body, 10, Color32::TRANSPARENT, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
			let mut c = ui.new_child(egui::UiBuilder::new().max_rect(body.shrink2(Vec2::new(16.0, 12.0))));
			match self.tab {
				0 => self.general(&mut c, ctx),
				1 => self.lineup(&mut c, ctx),
				2 => self.inventory(&mut c, ctx),
				3 => self.monsters(&mut c, ctx),
				5 => self.fishing(&mut c, ctx),
				_ => self.records_tab(&mut c, ctx),
			}
		}
		let br = Rect::from_min_max(Pos2::new(x0, bottom + 12.0), Pos2::new(x0 + w, full.max.y - 10.0));
		let mut r = ui.new_child(egui::UiBuilder::new().max_rect(br).layout(egui::Layout::right_to_left(egui::Align::Center)));
		r.spacing_mut().item_spacing.x = 10.0;
		let can = self.changed();
		if small_button(&mut r, &p, "Save to new slot", Some("file-plus"), 34.0, self.data.is_some(), p.text).on_hover_text("Writes the edited save to another slot (you choose the number); the open one stays as it was").clicked() {
			self.ask_slot();
		}
		if primary_button_icon(&mut r, &p, "Save to this slot", Some("save"), Vec2::new(220.0, 34.0), 9, f_semi(15.0), can).clicked() {
			let name = self.sel.map(|i| self.slots[i].name.clone()).unwrap_or_default();
			self.dialog = Dialog::Ask(format!("Write changes to {name}?\n\nThe previous slot is first copied to the «Backups» folder next to the program."), Act::Overwrite);
		}
		if can && small_button(&mut r, &p, "Discard changes", Some("undo"), 34.0, true, p.muted).clicked() {
			self.data = Some(self.orig.clone());
		}
		let sw = r.available_width();
		paint_text_clipped(&r, Rect::from_min_size(Pos2::new(br.min.x, br.center().y - 9.0), Vec2::new(sw - 10.0, 18.0)), &self.status, f_body(12.5), if self.status.starts_with('✖') { p.warn } else { p.muted }, Align2::LEFT_CENTER);
	}

	fn general(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let p = self.pal;
		let atlas = self.icons(ctx);
		let (chapter, place) = self.sel.and_then(|i| self.slots.get(i)).map(|s| (s.title.clone(), s.sub.clone())).unwrap_or_default();
		let g = self.g;
		let cell = |c: u32| atlas.as_ref().map(|t| (t.id(), atlas_uv(t, c)));
		let rank_rows: Vec<RankRow> = g.ranks.iter().zip(g.rank_rewards).map(|(&(r, t), id)| RankRow { name: r.into(), pts: t, rewards: if id == 0 { Vec::new() } else { vec![self.item_of.get(&id).map(|&k| &self.items[k]).map_or((None, format!("#{id}")), |it| (it.icon().and_then(cell), it.name.clone()))] } }).collect();
		let d = self.data.as_mut().unwrap();
		const LW: f32 = 118.0;
		const RW: f32 = 196.0;
		const FW: f32 = 96.0;
		const GAP: f32 = 10.0;
		const LEFT_W: f32 = 2.0 * (LW + GAP + FW) + GAP;
		const RIGHT_W: f32 = 2.0 * (RW + GAP + FW) + GAP;
		const COL_GAP: f32 = 48.0;
		let head = |ui: &mut egui::Ui, t: &str| {
			ui.add_space(14.0);
			ui.label(egui::RichText::new(t).font(f_semi(15.5)).color(p.text));
			ui.add_space(4.0);
		};
		let label = |ui: &mut egui::Ui, w: f32, t: &str| {
			ui.allocate_ui_with_layout(Vec2::new(w, 26.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
				ui.set_min_width(w);
				ui.label(egui::RichText::new(t).font(f_body(13.0)).color(p.muted));
			});
		};
		let num = |ui: &mut egui::Ui, v: &mut u32, max: u32, _speed: f64| ui.add_sized([FW, 26.0], egui::DragValue::new(v).clamp_existing_to_range(false).range(0..=max).speed(0.0)).changed();
		let wide = ui.available_width() >= LEFT_W + COL_GAP + RIGHT_W;
		let row_layout = if wide { egui::Layout::left_to_right(egui::Align::Min) } else { egui::Layout::top_down(egui::Align::Min) };
		egui::ScrollArea::vertical().id_salt("general").auto_shrink([false, false]).show(ui, |ui| {
			ui.spacing_mut().item_spacing = Vec2::new(GAP, 6.0);
			ui.add_space(4.0);
			ui.add(egui::Label::new(egui::RichText::new(if chapter.is_empty() { "—" } else { chapter.as_str() }).font(f_semi(26.0)).color(p.text)).wrap());
			if !place.is_empty() {
				ui.label(egui::RichText::new(&place).font(f_semi(14.0)).color(p.muted));
			}
			ui.horizontal_top(|ui| {
				ui.vertical(|ui| {
					head(ui, "Currency");
					ui.horizontal(|ui| {
						label(ui, LW, "Mira");
						let mut mira = save::u32_at(d, g.mira);
						if num(ui, &mut mira, 9_999_999, 100.0) {
							save::set_u32(d, g.mira, mira);
						}
					});
					ui.horizontal(|ui| {
						label(ui, LW, "Tokens");
						let mut v = save::u32_at(d, g.tokens);
						if num(ui, &mut v, 9_999_999, 10.0) {
							save::set_u32(d, g.tokens, v);
						}
					});
				});
				ui.add_space(24.0);
				ui.vertical(|ui| {
					head(ui, "Difficulty");
					let mut v = save::difficulty(g, d);
					let v0 = v;
					ui.horizontal(|ui| {
						label(ui, 76.0, "Difficulty");
						let cur = save::DIFFICULTY.iter().find(|x| x.0 == v).map_or_else(|| format!("? ({v})"), |x| x.1.to_string());
						egui::ComboBox::from_id_salt("diff").width(FW - 8.0).selected_text(cur).show_ui(ui, |ui| {
							for (k, t) in save::DIFFICULTY {
								ui.selectable_value(&mut v, k, t);
							}
						});
					});
					if v != v0 {
						save::set_difficulty(g, d, v);
					}
				});
			});
			ui.with_layout(row_layout, |ui| {
				ui.vertical(|ui| {
					ui.set_width(LEFT_W);
					head(ui, "Detective Notebook");
					ui.horizontal(|ui| {
						label(ui, LW, "DP");
						let mut dp = save::u32_at(d, g.dp);
						let top = g.ranks.last().map_or(99_999, |r| r.1).max(dp);
						if num(ui, &mut dp, top, 1.0) {
							save::set_u32(d, g.dp, dp);
						}
						if !g.ranks.is_empty() {
							let cur = g.ranks.iter().rposition(|&(_, t)| dp >= t);
							help_icon(ui, &p, |ui| {
								rank_table(ui, &p, "Detective ranks", "DP", &rank_rows, cur);
							});
						}
					});
					if !g.ranks.is_empty() {
						let dp = save::u32_at(d, g.dp);
						let cur = g.ranks.iter().rposition(|&(_, t)| dp >= t);
						ui.horizontal(|ui| {
							label(ui, LW, "Detective Rank");
							ui.allocate_ui_with_layout(Vec2::new(FW, 26.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
								ui.set_min_width(FW);
								ui.label(egui::RichText::new(cur.map_or("—", |i| g.ranks[i].0)).font(f_semi(14.0)).color(p.text));
							});
							if let Some(next) = g.ranks.get(cur.map_or(0, |i| i + 1)) {
								ui.label(egui::RichText::new(format!("{1} more to {0}", next.0, next.1 - dp)).font(f_body(12.5)).color(p.muted));
							}
						});
					}
					head(ui, "Sepith");
					egui::Grid::new("sepith").num_columns(4).spacing([GAP, 6.0]).show(ui, |ui| {
						for (k, nm) in SEPITH.iter().enumerate() {
							ui.allocate_ui_with_layout(Vec2::new(LW, 26.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
								ui.set_min_width(LW);
								sepith_icon(ui, &atlas, k, 24.0);
								ui.label(egui::RichText::new(*nm).font(f_body(13.0)).color(p.muted));
							});
							let o = g.sepith + k * 4;
							let mut v = save::u32_at(d, o);
							if num(ui, &mut v, 99_999, 10.0) {
								save::set_u32(d, o, v);
							}
							if k % 2 == 1 {
								ui.end_row();
							}
						}
					});
				});
				if wide {
					ui.add_space(COL_GAP - GAP);
				}
				ui.vertical(|ui| {
					ui.set_width(RIGHT_W);
					head(ui, "Combat Notebook");
					egui::Grid::new("battle").num_columns(4).spacing([GAP, 6.0]).show(ui, |ui| {
						for (i, (nm, off)) in save::BATTLE_STATS.iter().enumerate() {
							label(ui, RW, nm);
							let at = g.battle + off;
							let mut v = save::u16_at(d, at);
							if num(ui, &mut v, 65_535, 1.0) {
								save::set_u16(d, at, v);
							}
							if i % 2 == 1 {
								ui.end_row();
							}
						}
					});
				});
			});
			ui.with_layout(row_layout, |ui| {
				ui.vertical(|ui| {
					ui.set_width(LEFT_W);
					head(ui, "Play time");
					let t = save::u32_at(d, g.time);
					let (mut h, mut m, mut s) = (t / 3600, t % 3600 / 60, t % 60);
					ui.horizontal(|ui| {
						label(ui, LW, "Playtime");
						let w3 = (FW * 2.0 + LW + GAP * 2.0 - GAP * 2.0) / 3.0;
						let mut ch = false;
						ch |= ui.add_sized([w3, 26.0], egui::DragValue::new(&mut h).clamp_existing_to_range(false).range(0..=9_999).suffix(" h").speed(0.0)).changed();
						ch |= ui.add_sized([w3, 26.0], egui::DragValue::new(&mut m).clamp_existing_to_range(false).range(0..=59).suffix(" min").speed(0.0)).changed();
						ch |= ui.add_sized([w3, 26.0], egui::DragValue::new(&mut s).clamp_existing_to_range(false).range(0..=59).suffix(" s").speed(0.0)).changed();
						if ch {
							save::set_u32(d, g.time, h * 3600 + m * 60 + s);
						}
					});
				});
			});
			ui.add_space(10.0);
			ui.label(egui::RichText::new("The game updates the time and difficulty in the slot description on the next save.").font(f_body(12.0)).color(p.muted));
		}).middle_scroll(ui);
	}

	fn lineup(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let g = self.g;
		let p = self.pal;
		let (list, have, names) = {
			let d = self.data.as_ref().unwrap();
			let list = save::party(g, d);
			let have: Vec<u32> = g.playable.iter().map(|&k| k as u32).filter(|c| !list.contains(c)).collect();
			let names: HashMap<u32, u32> = list.iter().chain(have.iter()).map(|&c| (c, self.party_name_id(d, c))).collect();
			(list, have, names)
		};
		let unjoined: Vec<u32> = {
			let d = self.data.as_ref().unwrap();
			g.playable.iter().map(|&k| k as u32).filter(|&c| save::u16_at(d, save::chr_at(g, c as usize) + field::LEVEL) == 0).collect()
		};
		let at = |pos: LPos| match pos {
			LPos::Line(i) => list.get(i).copied(),
			LPos::Avail(j) => have.get(j).copied(),
		};
		if self.lsel.is_some_and(|s| at(s).is_none()) {
			self.lsel = None;
		}
		let faces: HashMap<u32, Option<egui::TextureHandle>> = names.values().map(|&n| (n, self.face(ctx, n))).collect();
		let dragging = egui::DragAndDrop::has_any_payload(ctx);
		let mut act: Option<(LPos, Result<LPos, u8>)> = None;
		let mut zone_act: Option<(LPos, Result<LPos, u8>)> = None;
		let slot_click = std::cell::Cell::new(None::<u8>);
		let armed = self.lsel.is_some() || dragging;
		let mut drag_from: Option<LPos> = None;
		let zone = |ui: &mut egui::Ui, title: &str, right: &str, add: &mut dyn FnMut(&mut egui::Ui)| -> egui::Response {
			let r = egui::Frame::new().fill(p.surface).corner_radius(10).inner_margin(egui::Margin::symmetric(14, 10)).show(ui, |ui| {
				ui.set_width(ui.available_width());
				ui.horizontal(|ui| {
					ui.label(egui::RichText::new(title).font(f_semi(13.5)).color(p.text));
					ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(egui::RichText::new(right).font(f_body(12.0)).color(p.muted)));
				});
				ui.add_space(4.0);
				ui.horizontal_wrapped(|ui| {
					ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
					add(ui)
				});
			});
			ui.add_space(8.0);
			ui.interact(r.response.rect, ui.id().with(("zone", title)), Sense::hover())
		};
		const BIG: CardSize = CardSize { size: Vec2::new(160.0, 150.0), face: 84.0 };
		const SMALL: CardSize = CardSize { size: Vec2::new(140.0, 124.0), face: 64.0 };
		let mut card_at = |this: &Self, ui: &mut egui::Ui, pos: LPos, c: u32, big: bool, tag: &str| {
			let cs = if big { BIG } else { SMALL };
			let nid = names.get(&c).copied().unwrap_or(c);
			let name = this.who_name(nid);
			let selected = this.lsel == Some(pos);
			let hovered_drop = dragging && ui.rect_contains_pointer(Rect::from_min_size(ui.cursor().min, cs.size));
			let r = lineup_card(ui, &p, cs, faces.get(&nid).and_then(|t| t.as_ref()), &name, tag, selected, hovered_drop);
			if r.drag_started() || r.dragged() {
				drag_from = Some(pos);
			}
			if r.dnd_release_payload::<String>().is_some()
				&& let Some(from) = this.drag_src {
					act = Some((from, Ok(pos)));
				}
			if r.clicked() {
				act = Some((pos, Err(255)));
			}
		};
		ui.label(egui::RichText::new("Drag a card, or click two cards to swap them. Drag a card into “Can be added” to remove it from the party.").font(f_body(12.5)).color(p.muted));
		ui.add_space(6.0);
		egui::ScrollArea::vertical().id_salt("lineup").auto_shrink([false, false]).show(ui, |ui| {
			let tag = |c: u32, zone: u8| {
				if c as usize >= save::CHARS {
					"guest"
				} else {
					["in party", "reserve", "not in party"][zone as usize]
				}
			};
			let z0 = zone(ui, "In party", "4 slots", &mut |ui| {
				for i in 0..save::PARTY_ACTIVE {
					match list.get(i) {
						Some(&c) => card_at(self, ui, LPos::Line(i), c, true, tag(c, 0)),
						None => {
							let (rect, resp) = ui.allocate_exact_size(BIG.size, Sense::click());
							if resp.clicked() {
								slot_click.set(Some(0));
							}
							empty_slot(ui, &p, rect, false, armed);
						}
					}
				}
			});
			let drop_slot = |ui: &mut egui::Ui, hint: &str, zone: u8| {
				let (rect, _) = ui.allocate_exact_size(SMALL.size, Sense::hover());
				empty_slot(ui, &p, rect, true, armed);
				if ui.interact(rect, ui.id().with(("slot_tip", hint)), Sense::click()).on_hover_text(hint).clicked() {
					slot_click.set(Some(zone));
				}
			};
			let reserve = list.len().saturating_sub(save::PARTY_ACTIVE);
			let z1 = zone(ui, "Reserve", &format!("{reserve} of 4"), &mut |ui| {
				for (i, &c) in list.iter().enumerate().skip(save::PARTY_ACTIVE) {
					card_at(self, ui, LPos::Line(i), c, false, tag(c, 1));
				}
				if reserve < 4 {
					drop_slot(ui, "drag here, or select a card and click here", 1);
				}
			});
			let z2 = zone(ui, "Can be added", "main cast and playable guests", &mut |ui| {
				for (j, &c) in have.iter().enumerate() {
					card_at(self, ui, LPos::Avail(j), c, false, tag(c, 2));
				}
				drop_slot(ui, "remove from party", 2);
			});
			for (k, z) in [z0, z1, z2].iter().enumerate() {
				if z.dnd_release_payload::<String>().is_some()
					&& let Some(from) = self.drag_src {
						zone_act = Some((from, Err(k as u8)));
					}
			}
			ui.add_space(4.0);
			ui.label(egui::RichText::new("Guests (KeA, Renne…) are added and removed by the story.").font(f_body(12.0)).color(p.muted));
			ui.label(egui::RichText::new("⚠ The game loads models and animations per story chapter: a character who isn't in the party at this point may show up with someone else's model or none. Test on a new slot.").font(f_body(12.0)).color(p.warn));
		}).middle_scroll(ui);
		if act.is_none() {
			act = zone_act;
		}
		if let (None, Some(z), Some(s)) = (act, slot_click.get(), self.lsel) {
			act = Some((s, Err(z)));
		}
		if let Some(f) = drag_from {
			self.drag_src = Some(f);
		}
		if !dragging && drag_from.is_none() && act.is_none() {
			self.drag_src = None;
		}
		if let (Some(from), Some(pos)) = (self.drag_src, ctx.pointer_interact_pos())
			&& dragging
				&& let Some(c) = at(from) {
					let nid = names.get(&c).copied().unwrap_or(c);
					let sub = if c as usize >= save::CHARS {
						"guest"
					} else {
						match from {
							LPos::Line(i) if i < save::PARTY_ACTIVE => "in party",
							LPos::Line(_) => "reserve",
							LPos::Avail(_) => "not in party",
						}
					};
					lineup_ghost(ctx, &p, SMALL, pos, faces.get(&nid).and_then(|t| t.as_ref()), &self.who_name(nid), sub);
				}
		let Some((a, b)) = act else { return };
		let mut new = list.clone();
		match b {
			Err(255) => match self.lsel.take() {
				None => self.lsel = Some(a),
				Some(s) if s == a => {}
				Some(s) => lineup_swap(&mut new, &have, s, a),
			},
			Ok(t) => lineup_swap(&mut new, &have, a, t),
			Err(z) => lineup_move(&mut new, &have, a, z),
		}
		if b != Err(255) {
			self.lsel = None;
			self.drag_src = None;
		}
		if new != list {
			let added: Vec<u32> = new.iter().copied().filter(|c| !list.contains(c) && unjoined.contains(c)).collect();
			let names: Vec<String> = added.iter().map(|&c| self.who_name(names.get(&c).copied().unwrap_or(c))).collect();
			let d = self.data.as_mut().unwrap();
			save::set_party(g, d, &new);
			let lv = save::u16_at(d, save::chr_at(g, 0) + field::LEVEL).max(1);
			for (c, name) in added.iter().zip(names) {
				save::set_u16(d, save::chr_at(g, *c as usize) + field::LEVEL, lv);
				self.status = format!("{name} hasn't joined yet: set to level {lv}, same as Lloyd; no weapon or crafts — check in game on a new slot");
			}
		}
	}

	fn monsters(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let g = self.g;
		let p = self.pal;
		if self.mons.is_empty() {
			ui.label(egui::RichText::new("No monster list for this game in data.dat next to the program.").font(f_body(13.0)).color(p.muted));
			return;
		}
		let icons = self.icons(ctx);
		let notes_tex = self.note_icons(ctx);
		let atlas = self.mon_sel.and_then(|i| self.pics.pack(ctx, &format!("{}/mons/{i}.webp", g.dir), egui::TextureOptions::LINEAR, plain));
		let d = self.data.as_ref().unwrap();
		let journal: HashMap<String, [u8; 4]> = save::journal(g, d).into_iter().collect();
		let full = ui.available_rect_before_wrap();
		let lw = 300.0;
		let left = Rect::from_min_size(full.min, Vec2::new(lw, full.height()));
		let right = Rect::from_min_max(Pos2::new(full.min.x + lw + 14.0, full.min.y), full.max);
		let mut toggle: Option<usize> = None;
		let mut select: Option<Option<usize>> = None;
		let mut fold: Option<String> = None;
		{
			ui.painter().rect_filled(left, 8, p.surface);
			let mut c = ui.new_child(egui::UiBuilder::new().max_rect(left.shrink(8.0)));
			egui::ScrollArea::vertical().id_salt(("mons", g.dir)).auto_shrink([false, false]).show(&mut c, |ui| {
				let w = ui.available_width();
				let row = |ui: &mut egui::Ui, h: f32| ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
				let (r, resp) = row(ui, 36.0);
				ui.painter().rect_filled(r, 5, if self.mon_sel.is_none() { p.sel } else if resp.hovered() { p.div2 } else { p.surface });
					row_border(ui.painter(), &p, r);
				ui.painter().text(Pos2::new(r.min.x + 12.0, r.center().y), Align2::LEFT_CENTER, "◆ Record", f_semi(14.5), p.text);
				if g.dir == "Azure" {
					ui.painter().text(Pos2::new(r.max.x - 10.0, r.center().y), Align2::RIGHT_CENTER, format!("{}/{}", journal.len(), self.mons.len()), f_body(12.5), p.muted);
				}
				if resp.clicked() {
					select = Some(None);
				}
				ui.add_space(4.0);
				for (area, list) in save::mon_areas(&self.mons) {
					let done = list.iter().filter(|&&j| journal.contains_key(&self.mons[j].id)).count();
					let known = done > 0;
					let open = self.mon_open.contains(&area);
					let (r, resp) = row(ui, 34.0);
					ui.painter().rect_filled(r, 5, if resp.hovered() { p.div2 } else { p.surface });
					row_border(ui.painter(), &p, r);
					let tc = if known { p.text } else { p.muted };
					ui.painter().text(Pos2::new(r.min.x + 10.0, r.center().y), Align2::LEFT_CENTER, if open { "▾" } else { "▸" }, f_body(13.0), tc);
					paint_icon(ui, "folder", Rect::from_center_size(Pos2::new(r.min.x + 32.0, r.center().y), Vec2::splat(15.0)), if known { p.gold } else { p.muted });
					paint_text_clipped(ui, Rect::from_min_max(Pos2::new(r.min.x + 46.0, r.min.y), Pos2::new(r.max.x - 52.0, r.max.y)), if known { &area } else { "????" }, f_semi(13.5), tc, Align2::LEFT_CENTER);
					ui.painter().text(Pos2::new(r.max.x - 10.0, r.center().y), Align2::RIGHT_CENTER, format!("{done}/{}", list.len()), f_body(12.0), if done == list.len() { p.ok_strong } else { p.muted });
					if resp.clicked() {
						fold = Some(area.clone());
					}
					if open {
						for &j in &list {
							let m = &self.mons[j];
							let st = journal.get(&m.id).map(|&f| if save::journal_full(f) { 2 } else { 1 }).unwrap_or(0);
							let (full, resp) = row(ui, 36.0);
							let r = Rect::from_min_max(Pos2::new(full.min.x + 40.0, full.min.y), full.max);
							if self.mon_sel == Some(j) || resp.hovered() {
								ui.painter().rect_filled(r, 5, if self.mon_sel == Some(j) { p.sel } else { p.div2 });
							}
							row_border(ui.painter(), &p, r);
							let bx = Rect::from_center_size(Pos2::new(full.min.x + 20.0, r.center().y), Vec2::splat(20.0));
							let bresp = ui.interact(bx.expand(4.0), ui.id().with(("mbox", j)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
							paint_tick(ui, bx, st, bresp.hovered());
							let bresp = bresp.on_hover_text(match st {
								2 => "Fully logged — click to remove from the log",
								1 => "Partial entry — click to reveal everything",
								_ => "Not in the log — click to log fully",
							});
							if bresp.clicked() {
								toggle = Some(j);
							}
							paint_text_clipped(ui, Rect::from_min_max(Pos2::new(r.min.x + 12.0, r.min.y), Pos2::new(r.max.x - 6.0, r.max.y)), &m.name, f_body(14.0), if st > 0 { p.text } else { p.muted }, Align2::LEFT_CENTER);
							if resp.clicked() {
								select = Some(Some(j));
							}
						}
						ui.add_space(4.0);
					}
				}
			}).middle_scroll(&c);
		}
		let mut rc = ui.new_child(egui::UiBuilder::new().max_rect(right));
		match self.mon_sel.filter(|&i| i < self.mons.len()) {
			None => self.mons_record(&mut rc, journal.len()),
			Some(i) => {
				let m = &self.mons[i];
				let f = journal.get(&m.id).copied();
				let diff = save::difficulty(g, self.data.as_ref().unwrap());
				let card = MonCard { g, m, f, diff, atlas: atlas.as_ref(), icons: icons.as_ref(), notes: notes_tex.as_ref(), items: &self.items, item_of: &self.item_of };
				let pick = &mut self.mon_diff;
				rc.painter().rect(right, 10, p.surface, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
				let inner = right.shrink(3.0);
				let mut rc = rc.new_child(egui::UiBuilder::new().max_rect(inner));
				rc.set_clip_rect(inner);
				egui::ScrollArea::both().id_salt(("mcard", g.dir)).auto_shrink([false, false]).show(&mut rc, |ui| {
					let (cr, _) = ui.allocate_exact_size(MonCard::size(ui.available_size()), Sense::hover());
					let mut c = ui.new_child(egui::UiBuilder::new().max_rect(cr));
					if card.show(&mut c, &p, pick) {
						toggle = Some(i);
					}
				}).middle_scroll(&rc);
			}
		}
		if let Some(a) = fold
			&& !self.mon_open.remove(&a) {
				self.mon_open.insert(a);
			}
		if let Some(s) = select {
			self.mon_sel = s;
		}
		if let Some(j) = toggle {
			let m = &self.mons[j];
			let d = self.data.as_mut().unwrap();
			let f = save::journal(g, d).into_iter().find(|(id, _)| *id == m.id).map(|(_, f)| f);
			let on = !f.is_some_and(save::journal_full);
			if on && f.is_none() && save::journal(g, d).len() >= save::journal_cap(g) {
				self.status = "The log is full".into();
			} else {
				save::set_journal(g, d, m, on);
				self.status = if on { format!("“{}” — fully logged", m.name) } else { format!("“{}” — removed from the log", m.name) };
			}
		}
	}

	fn mons_record(&mut self, ui: &mut egui::Ui, logged: usize) {
		let g = self.g;
		let p = self.pal;
		let total = self.mons.len();
		let d = self.data.as_mut().unwrap();
		let r = ui.max_rect();
		ui.painter().rect(r, 10, p.surface, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
		ui.painter().text(Pos2::new(r.min.x + 36.0, r.min.y + 40.0), Align2::LEFT_CENTER, "◆ Record ◆", f_semi(26.0), p.text);
		if g.dir == "Azure" {
			ui.painter().text(Pos2::new(r.max.x - 36.0, r.min.y + 40.0), Align2::RIGHT_CENTER, format!("Information Recorded {logged}/{total}"), f_body(20.0), p.text);
		}
		ui.painter().line_segment([Pos2::new(r.min.x + 36.0, r.min.y + 72.0), Pos2::new(r.max.x - 36.0, r.min.y + 72.0)], Stroke::new(1.0_f32, p.div));
		let (gap, cw) = (24.0, (r.width() - 72.0 - 24.0) / 2.0);
		let step = ((r.height() - 108.0 - 16.0) / 5.0).min(92.0);
		for (k, (nm, off)) in save::BATTLE_STATS.iter().enumerate() {
			let min = Pos2::new(r.min.x + 36.0 + (k / 5) as f32 * (cw + gap), r.min.y + 100.0 + (k % 5) as f32 * step);
			let b = Rect::from_min_size(min, Vec2::new(cw, step - 20.0));
			ui.painter().rect_filled(b, 8, p.div2);
			let field = Rect::from_center_size(Pos2::new(b.max.x - 84.0, b.center().y), Vec2::new(132.0, (b.height() - 20.0).clamp(26.0, 44.0)));
			paint_text_clipped(ui, Rect::from_min_max(Pos2::new(b.min.x + 22.0, b.min.y), Pos2::new(field.min.x - 8.0, b.max.y)), nm, f_body(if cw < 420.0 { 17.0 } else { 22.0 }), p.text, Align2::LEFT_CENTER);
			let at = g.battle + off;
			let mut v = save::u16_at(d, at);
			let mut fc = ui.new_child(egui::UiBuilder::new().max_rect(field));
			fc.style_mut().override_font_id = Some(f_semi(20.0));
			if fc.add_sized(field.size(), egui::DragValue::new(&mut v).clamp_existing_to_range(false).range(0..=65_535).speed(0.0)).changed() {
				save::set_u16(d, at, v);
			}
		}
	}

	fn records_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let g = self.g;
		let p = self.pal;
		let n = save::notes(g);
		let icons = self.icons(ctx);
		let rtex = self.record_tex(ctx);
		let d = self.data.as_ref().unwrap();
		if self.records.is_empty() {
			ui.label(egui::RichText::new("No t_record table in data.dat next to the program.").font(f_body(13.0)).color(p.muted));
			return;
		}
		let mut order: Vec<usize> = (0..self.records.len()).collect();
		order.sort_by_key(|&i| self.records[i].rid);
		let sel = self.rec_sel.min(self.records.len() - 1);
		let mut toggle: Option<usize> = None;
		let mut new_sel: Option<usize> = None;
		let mut set_bond: Option<(usize, u32)> = None;
		let full = ui.available_rect_before_wrap();
		let top = Rect::from_min_size(full.min, Vec2::new(full.width(), 56.0));
		let foot = Rect::from_min_max(Pos2::new(full.min.x, full.max.y - 104.0), full.max);
		let grid = Rect::from_min_max(Pos2::new(full.min.x, top.max.y + 12.0), Pos2::new(full.max.x, foot.min.y - 12.0));
		let pt = ui.painter();
		let tw = full.width() * 0.68;
		let tbox = Rect::from_min_size(top.min, Vec2::new(tw, top.height()));
		let cbox = Rect::from_min_max(Pos2::new(top.min.x + tw + 16.0, top.min.y), top.max);
		for r in [tbox, cbox] {
			pt.rect(r, 10, p.surface, Stroke::new(2.0_f32, p.div), StrokeKind::Inside);
		}
		let hov_id = egui::Id::new(("rec_hover", g.dir));
		let hov: Option<usize> = ui.data(|x| x.get_temp(hov_id));
		let r0 = &self.records[hov.filter(|&i| i < self.records.len()).unwrap_or(sel)];
		let mut hov_now: Option<usize> = None;
		pt.text(tbox.center(), Align2::CENTER_CENTER, &r0.name, f_body(24.0), p.text);
		let done = self.records.iter().filter(|r| save::record_done(g, d, r.pos)).count();
		let pts: u32 = self.records.iter().filter(|r| save::record_done(g, d, r.pos)).map(|r| r.pts).sum();
		pt.text(cbox.center(), Align2::CENTER_CENTER, format!("{done}/{}  {pts}pt", self.records.len()), f_body(24.0), p.text);
		pt.rect(grid, 10, p.surface, Stroke::new(2.0_f32, p.div), StrokeKind::Inside);
		let mut c = ui.new_child(egui::UiBuilder::new().max_rect(grid.shrink(14.0)));
		egui::ScrollArea::vertical().id_salt(("rec", g.dir)).auto_shrink([false, false]).show(&mut c, |ui| {
			let w = ui.available_width();
			let step = w / 8.0;
			let size = (step - 18.0).min(120.0);
			for row in order.chunks(8) {
				ui.horizontal(|ui| {
					ui.spacing_mut().item_spacing.x = 0.0;
					for &i in row {
						let r = &self.records[i];
						let (cell, resp) = ui.allocate_exact_size(Vec2::new(step, size + 18.0), Sense::click());
						let ir = Rect::from_center_size(cell.center(), Vec2::splat(size));
						let on = save::record_done(g, d, r.pos);
						if let Some(t) = &rtex {
							let k = if on { r.pos as u32 } else { 63 };
							let uv = Rect::from_min_size(Pos2::new((k % 8) as f32 / 8.0, (k / 8) as f32 / 8.0), Vec2::splat(1.0 / 8.0));
							ui.painter().image(t.id(), ir, uv, if resp.hovered() { Color32::WHITE } else { Color32::from_gray(232) });
						}
						if i == hov.unwrap_or(sel) {
							ui.painter().rect_stroke(ir.expand(5.0), 4, Stroke::new(4.0_f32, p.accent), StrokeKind::Outside);
						}
						if resp.hovered() {
							hov_now = Some(i);
						}
						if resp.clicked() {
							new_sel = Some(i);
							toggle = Some(i);
						}
					}
				});
			}
		}).middle_scroll(&c);
		if hov_now != hov {
			ui.data_mut(|x| match hov_now {
				Some(i) => x.insert_temp(hov_id, i),
				None => x.remove::<usize>(hov_id),
			});
			ctx.request_repaint();
		}
		let pt = ui.painter();
		pt.rect(foot, 10, p.surface, Stroke::new(2.0_f32, p.div), StrokeKind::Inside);
		let mut desc_w = foot.width() - 44.0;
		pt.text(Pos2::new(foot.min.x + 22.0, foot.min.y + 72.0), Align2::LEFT_CENTER, format!("{}pt", r0.pts), f_body(22.0), p.text);
		if let Some(&(_, k)) = n.bond_rec.iter().find(|b| b.0 == r0.rid) {
			let hearts = n.bond_chr.get(k).map_or(0, |c| c.1);
			let v = save::bond(g, d, k);
			let (whole, half) = ((v / n.per_heart).min(hearts), !v.is_multiple_of(n.per_heart) && v / n.per_heart < hearts);
			let hs = 34.0;
			let x0 = foot.max.x - 120.0 - hearts as f32 * hs;
			let note = format!("{} pts — hearts for watched scenes are added by the game", v);
			let nx = x0.min(foot.max.x - 20.0 - text_w(ui, &note, f_body(13.0)));
			ui.painter().text(Pos2::new(nx, foot.min.y + 28.0), Align2::LEFT_CENTER, note, f_body(13.0), p.muted);
			desc_w = nx - foot.min.x - 22.0 - 20.0;
			for h in 0..hearts {
				let r = Rect::from_min_size(Pos2::new(x0 + h as f32 * hs, foot.min.y + 52.0), Vec2::splat(hs - 2.0));
				let cell = if h < whole { 216 } else if h == whole && half { 200 } else { 199 };
				if let Some(t) = &icons {
					ui.painter().image(t.id(), r, atlas_uv(t, cell), Color32::WHITE);
				}
				let resp = ui.interact(r, ui.id().with(("heart", h)), Sense::click()).on_hover_text(format!("{} hearts", h + 1));
				if resp.clicked() {
					set_bond = Some((k, (h + 1) * n.per_heart));
				}
			}
			let br = Rect::from_min_size(Pos2::new(foot.max.x - 100.0, foot.min.y + 52.0), Vec2::new(80.0, 32.0));
			if ui.put(br, egui::Button::new(egui::RichText::new("max").font(f_body(15.0)))).clicked() {
				set_bond = Some((k, save::bond_max(g, k)));
			}
		}
		let full_w = text_w(ui, &r0.desc, f_body(22.0));
		ui.painter().text(Pos2::new(foot.min.x + 22.0, foot.min.y + 30.0), Align2::LEFT_CENTER, &r0.desc, f_body(22.0 * (desc_w / full_w.max(1.0)).min(1.0)), p.text);
		if let Some(i) = new_sel {
			self.rec_sel = i;
		}
		let d = self.data.as_mut().unwrap();
		if let Some(i) = toggle {
			let r = &self.records[i];
			let on = !save::record_done(g, d, r.pos);
			save::set_record_done(g, d, r.pos, on);
			self.status = if on { format!("“{}” — obtained", r.name) } else { format!("“{}” — not obtained again", r.name) };
		}
		if let Some((k, v)) = set_bond {
			save::set_bond(g, d, k, v);
			self.status = format!("Bonds: {} pts — check in game on a new slot", v);
		}
	}

	fn inventory(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let g = self.g;
		let p = self.pal;
		let atlas = self.icons(ctx);
		let tabs: Vec<&str> = save::TABS.iter().map(|t| t.0).collect();
		let mut r = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(ui.max_rect().min, Vec2::new(ui.max_rect().width(), 32.0))));
		segmented(&mut r, &p, "itab", &tabs, &mut self.item_tab, None, 32.0, 8, f_body(12.0), &[]);
		ui.add_space(42.0);
		ui.horizontal(|ui| {
			ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search by name").desired_width(260.0));
			extra::watch(ui.ctx(), &mut self.search);
			ui.add_space(12.0);
			checkbox(ui, &p, &mut self.owned_only, "Owned only", 16.0, f_body(13.0));
		});
		ui.add_space(6.0);
		let q = self.search.trim().to_lowercase();
		let tab = self.item_tab;
		let d = self.data.as_mut().unwrap();
		let show_service = self.show_service;
		let shown = |it: &save::Item, d: &[u8]| save::tab_of(it) == Some(tab) && (show_service || !it.service()) && (q.is_empty() || it.name.to_lowercase().contains(&q)) && (!self.owned_only || save::item_count(g, d, it.id) > 0);
		let mut groups: Vec<(String, String, Option<u32>, Vec<&save::Item>)> = Vec::new();
		let mut list: Vec<&save::Item> = self.items.iter().filter(|it| shown(it, d)).collect();
		list.sort_by_key(|it| (if it.cell == 22 { 19 } else { it.cell }, it.element, it.id));
		for it in list {
			let (key, title, icon) = match it.cell {
				1..=10 => {
					let rec = (it.owner as usize).saturating_sub(1);
					let who = if it.owner == 0 { "Anyone".to_string() } else { short_name(&self.names, save::record_name(g, d, rec)) };
					(format!("{}:w{}", g.dir, it.owner), who, it.icon())
				}
				18 | 26 => {
					let e = it.element.clamp(1, 7) as usize;
					let title = if it.cell == 26 { "Master Quartz".to_string() } else { SEPITH[e - 1].to_string() };
					(format!("{}:q{}:{e}", g.dir, it.cell), title, it.icon())
				}
				c => {
					let title = match c {
						11 => "Armor",
						12 => "Footwear",
						13 => "Accessories",
						14 => "Recovery",
						19 | 22 => "Food",
						23 => "Bait",
						24 => "Fish",
						25 => "Rods",
						_ => "",
					};
					(format!("{}:k{title}", g.dir), title.to_string(), if title.is_empty() { None } else { Some(c) })
				}
			};
			match groups.iter_mut().find(|gr| gr.0 == key) {
				Some(gr) => gr.3.push(it),
				None => groups.push((key, title, icon, vec![it])),
			}
		}
		let mut set_all = None;
		ui.horizontal(|ui| {
			ui.label(egui::RichText::new("Set value:").font(f_body(13.0)).color(p.muted));
			for n in [0u32, 1, 99] {
				if small_button(ui, &p, &format!("All “{n}”"), None, 26.0, !groups.is_empty(), p.text).clicked() {
					set_all = Some(n);
				}
			}
		});
		if let Some(n) = set_all {
			for gr in &groups {
				for it in &gr.3 {
					save::set_item_count(g, d, it.id, n.min(it.max));
				}
			}
		}
		ui.add_space(6.0);
		let cols = ((ui.available_width() / 170.0).floor() as usize).clamp(1, 6);
		let cell_w = ui.available_width() / cols as f32;
		let item = |ui: &mut egui::Ui, d: &mut Vec<u8>, it: &save::Item| {
			let (rect, _) = ui.allocate_exact_size(Vec2::new(cell_w - 8.0, 30.0), Sense::hover());
			let fill = if p.dark { p.surface } else { Color32::from_black_alpha(14) };
			ui.painter().rect_filled(rect, 6, fill);
			let field = Rect::from_min_max(Pos2::new(rect.max.x - 46.0, rect.min.y), rect.max).shrink(1.0);
			let mut n = save::item_count(g, d, it.id);
			let mut fc = ui.new_child(egui::UiBuilder::new().max_rect(field));
			{
				let v = &mut fc.style_mut().visuals.widgets;
				for w in [&mut v.inactive, &mut v.hovered, &mut v.active, &mut v.open] {
					w.bg_stroke = Stroke::NONE;
					w.corner_radius = egui::CornerRadius { nw: 0, sw: 0, ne: 5, se: 5 };
					w.expansion = 0.0;
				}
			}
			if fc.add_sized(field.size(), egui::DragValue::new(&mut n).clamp_existing_to_range(false).range(0..=it.max).speed(0.0)).changed() {
				save::set_item_count(g, d, it.id, n);
			}
			ui.painter().line_segment([Pos2::new(field.min.x, rect.min.y), Pos2::new(field.min.x, rect.max.y)], Stroke::new(1.0_f32, p.div));
			ui.painter().rect_stroke(rect, 6, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
			let inner = rect.shrink2(Vec2::new(6.0, 0.0));
			let icon_r = Rect::from_min_size(Pos2::new(inner.min.x, inner.center().y - 12.0), Vec2::splat(24.0));
			if let (Some(t), Some(cell)) = (&atlas, it.icon()) {
				egui::Image::new(t).uv(atlas_uv(t, cell)).paint_at(ui, icon_r);
			}
			let name_r = Rect::from_min_max(Pos2::new(icon_r.max.x + 8.0, inner.min.y), Pos2::new(field.min.x - 8.0, inner.max.y));
			let mut nc = ui.new_child(egui::UiBuilder::new().max_rect(name_r).layout(egui::Layout::left_to_right(egui::Align::Center)));
			let r = nc.add(egui::Label::new(egui::RichText::new(&it.name).font(f_num(13.5)).color(p.text)).truncate().sense(Sense::click()));
			click_tip(&nc, &r, ("item", it.id), |ui| item_tip(ui, &p, it, &atlas));
		};
		let headers = groups.len() > 1;
		egui::ScrollArea::vertical().id_salt(("items", self.item_tab, g.dir)).auto_shrink([false, false]).show(ui, |ui| {
			for (key, title, gicon, its) in &groups {
				let open = !self.collapsed.contains(key);
				if headers {
					ui.add_space(4.0);
					ui.horizontal(|ui| {
						if gicon.is_some() {
							atlas_cell(ui, &atlas, *gicon, 22.0);
						}
						let r = section_head(ui, &p, open, title, its.len());
						if r.clicked() {
							if open {
								self.collapsed.insert(key.clone());
							} else {
								self.collapsed.remove(key);
							}
						}
					});
				}
				if open {
					let gap = ui.spacing().item_spacing;
					ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);
					for row in its.chunks(cols) {
						ui.horizontal(|ui| {
							for it in row {
								item(ui, d, it);
							}
						});
					}
					ui.spacing_mut().item_spacing = gap;
				}
			}
		}).middle_scroll(ui);
		if groups.is_empty() {
			ui.label(egui::RichText::new(if self.owned_only { "Nothing in this section (uncheck “Owned only” to add items)" } else { "Nothing found" }).font(f_body(13.0)).color(p.muted));
		}
	}
}

fn short_name(names: &HashMap<u32, String>, who: u32) -> String {
	match names.get(&who).map(|s| s.as_str()) {
		Some("Sergeant Major Seeker") => "Noel".into(),
		Some(n) => n.strip_prefix("Detective ").or_else(|| n.strip_prefix("Divine Wolf ")).unwrap_or(n).to_string(),
		None => format!("#{who}"),
	}
}

fn item_tip(ui: &mut egui::Ui, p: &Pal, it: &save::Item, atlas: &Option<egui::TextureHandle>) {
	ui.set_max_width(360.0);
	ui.label(egui::RichText::new(&it.name).font(f_semi(13.5)).color(p.text));
	let desc = it.desc.replace("\\n", "\n");
	if !desc.trim().is_empty() {
		ui.label(egui::RichText::new(desc.trim()).font(f_body(12.5)).color(p.text));
	}
	if matches!(it.cell, 1..=13 | 18 | 26) {
		let parts: Vec<String> = save::STATS.iter().zip(it.stats.iter().map(|&v| v as i32).chain(it.sub.iter().map(|&v| v as i32))).filter(|(_, v)| *v != 0).map(|(n, v)| format!("{n} {v:+}")).collect();
		let flat: String = desc.chars().filter(|c| !c.is_whitespace()).collect();
		if !parts.is_empty() && !parts.iter().all(|s| flat.contains(&s.replace(' ', ""))) {
			ui.label(egui::RichText::new(parts.join("  ")).font(f_semi(12.5)).color(p.text));
		}
	}
	if it.values.iter().any(|&v| v > 0) {
		ui.horizontal(|ui| {
			ui.label(egui::RichText::new("Element Value").font(f_body(12.5)).color(p.muted));
			for (k, &v) in it.values.iter().enumerate().filter(|(_, v)| **v > 0) {
				ui.add_space(4.0);
				sepith_icon(ui, atlas, k, 16.0);
				ui.label(egui::RichText::new(format!("×{v}")).font(f_semi(12.5)).color(p.text));
			}
		});
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
		let p = self.pal;
		let slot = self.sel.and_then(|i| self.slots.get(i)).filter(|_| self.data.is_some()).map_or(String::new(), |s| format!(" — {}", s.name));
		let t = match self.game {
			Some(i) => format!("{} — {}{slot}", title(), GAMES[i].1),
			None => title().into(),
		};
		if t != self.shown_title {
			ctx.send_viewport_cmd(egui::ViewportCommand::Title(t.clone()));
			self.shown_title = t;
		}
		self.global_input(ctx);
		data_warning(ctx, &p);
		egui::CentralPanel::default().frame(egui::Frame::new().fill(p.bg)).show(ctx, |ui| if self.game.is_some() { self.ui(ui, ctx) } else { self.picker(ui, ctx) });
		self.windows(ctx);
		if let Some(s) = self.slot_pick.as_mut()
			&& let Some(r) = slot_pick_window(ctx, &p, s, "The previous save is first copied to the «Backups» folder next to the program.")
		{
			self.slot_pick = None;
			if let Some(n) = r {
				self.save_to(n);
			}
		}
		match show_dialog(ctx, &p, "Save Editor", &mut self.dialog) {
			Some(Act::Overwrite) => self.overwrite(),
			Some(Act::Close) => {
				self.allow_close = true;
				ctx.send_viewport_cmd(egui::ViewportCommand::Close);
			}
			Some(Act::Open(i)) => self.open(i),
			None => {}
		}
	}
	fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
		self.save_settings();
	}
}

fn editor_icon() -> egui::IconData {
	match image::load_from_memory(include_bytes!("icons/crossbell_save.png")) {
		Ok(im) => {
			let im = im.to_rgba8();
			egui::IconData { width: im.width(), height: im.height(), rgba: im.into_raw() }
		}
		Err(_) => window_icon(),
	}
}

fn main() {
	kiseki_tool::crash::install("Crossbell-SaveEditor", env!("VER_CROSSBELLSAVEEDITOR"));
	let opts = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_title(TITLE).with_inner_size([1360.0, 780.0]).with_min_inner_size([960.0, 560.0]).with_icon(editor_icon()),
		centered: true,
		..Default::default()
	};
	let _ = eframe::run_native(TITLE, opts, Box::new(|cc| Ok(Box::new(App::new(cc)))));
}

