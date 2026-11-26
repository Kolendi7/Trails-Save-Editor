#![cfg_attr(windows, windows_subsystem = "windows")]

mod gui_common;
mod extra;

use eframe::egui::{self, Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use gui_common::*;
use kiseki_tool::calvard::save::{self, db1};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const TITLE: &str = concat!("Calvard Save Editor ", env!("VER_CALVARDSAVEEDITOR"), env!("VER_SUFFIX"));

fn title() -> &'static str {
	TITLE
}

#[path = "calvard_fish.rs"]
mod fish;

const GAMES: [(&str, &str, bool); 3] = [("db1", "Trails through Daybreak", true), ("db2", "Trails through Daybreak II", true), ("horizon", "Trails beyond the Horizon", true)];

fn profile(game: usize) -> &'static save::Profile {
	match game {
		1 => &save::DB2,
		2 => &save::HZ,
		_ => &save::DB1,
	}
}
fn falcom_dir(game: usize) -> &'static str {
	match game {
		1 => "Kuro2",
		2 => "Kai",
		_ => "Kuro",
	}
}

const SEPITH: [&str; 8] = ["Earth", "Water", "Fire", "Wind", "Time", "Space", "Mirage", "Sepith Mass"];

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

fn tip_table(ui: &mut egui::Ui, p: &Pal, title: &str, head: [&str; 2], rows: &[[String; 2]], cur: Option<usize>) {
	ui.label(egui::RichText::new(title).font(f_semi(13.5)).color(p.text));
	ui.add_space(4.0);
	egui::Grid::new(("tip", title)).num_columns(2).spacing([28.0, 3.0]).show(ui, |ui| {
		for h in head {
			ui.label(egui::RichText::new(h).font(f_body(12.0)).color(p.muted));
		}
		ui.end_row();
		for (i, row) in rows.iter().enumerate() {
			let col = if cur == Some(i) { p.accent } else { p.text };
			for (j, c) in row.iter().enumerate() {
				let t = egui::RichText::new(c).font(if j == 0 { f_semi(13.0) } else { f_body(13.0) }).color(col);
				if j == 0 {
					ui.label(t);
				} else {
					ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(t));
				}
			}
			ui.end_row();
		}
	});
}

const TEX_SMOOTH: egui::TextureOptions = egui::TextureOptions { magnification: egui::TextureFilter::Linear, minification: egui::TextureFilter::Linear, wrap_mode: egui::TextureWrapMode::ClampToEdge, mipmap_mode: None };

#[derive(Clone, Copy, PartialEq, Debug)]
enum LPos {
	Line(usize),
	Avail(usize),
}

fn lgc_chart(ui: &mut egui::Ui, p: &Pal, lv: [usize; 3], max: usize) {
	let (rect, _) = ui.allocate_exact_size(Vec2::new(300.0, 230.0), Sense::hover());
	let pt = ui.painter();
	let c = Pos2::new(rect.center().x - 10.0, rect.center().y + 6.0);
	let r = 86.0;
	let ang = [std::f32::consts::PI, -std::f32::consts::FRAC_PI_3, std::f32::consts::FRAC_PI_3];
	let at = |k: usize, f: f32| c + Vec2::angled(ang[k]) * r * f;
	let law = Color32::from_rgb(40, 130, 180);
	let chaos = Color32::from_rgb(190, 60, 140);
	pt.circle_stroke(c, r + 6.0, Stroke::new(2.0_f32, p.text));
	for lvl in 1..=max {
		let f = lvl as f32 / max as f32;
		let pts = [at(0, f), at(1, f), at(2, f), at(0, f)];
		for w in pts.windows(2) {
			pt.extend(egui::Shape::dashed_line(&[w[0], w[1]], Stroke::new(1.0_f32, p.div2), 4.0, 3.0));
		}
	}
	for k in 0..3 {
		pt.line_segment([c, at(k, 1.0)], Stroke::new(1.0_f32, p.div2));
	}
	let f = |k: usize| (lv[k] as f32 / max as f32).max(0.06);
	let tri = vec![at(0, f(0)), at(1, f(1)), at(2, f(2))];
	pt.add(egui::Shape::convex_polygon(tri.clone(), Color32::from_rgba_unmultiplied(90, 90, 170, 150), Stroke::new(1.5_f32, Color32::from_rgb(70, 70, 140))));
	for k in 0..3 {
		let q = at(k, 1.0);
		pt.add(egui::Shape::convex_polygon(vec![q + Vec2::new(0.0, -5.0), q + Vec2::new(5.0, 0.0), q + Vec2::new(0.0, 5.0), q + Vec2::new(-5.0, 0.0)], p.text, Stroke::NONE));
	}
	let label = |k: usize| if lv[k] >= max { format!("{max} MAX") } else { lv[k].to_string() };
	pt.text(at(0, 1.0) + Vec2::new(-14.0, -10.0), Align2::RIGHT_BOTTOM, "LAW", f_semi(13.0), law);
	pt.text(at(0, 1.0) + Vec2::new(-14.0, 6.0), Align2::RIGHT_TOP, label(0), f_semi(15.0), law);
	pt.text(at(1, 1.0) + Vec2::new(10.0, -8.0), Align2::LEFT_BOTTOM, format!("GRAY {}", label(1)), f_semi(13.0), p.text);
	pt.text(at(2, 1.0) + Vec2::new(10.0, 8.0), Align2::LEFT_TOP, format!("CHAOS {}", label(2)), f_semi(13.0), chaos);
}

#[allow(clippy::too_many_arguments)]
fn lgc_game(ui: &mut egui::Ui, bg: &egui::TextureHandle, ch: &egui::TextureHandle, lv: [usize; 3], max: usize, digits: ([f32; 4], &str), k: f32) {
	let (rect, _) = ui.allocate_exact_size(Vec2::new(576.0, 320.0) * k, Sense::hover());
	let pt = ui.painter_at(rect);
	let ts = ch.size_vec2();
	let uv = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(Pos2::new(x / ts.x, y / ts.y), Vec2::new(w / ts.x, h / ts.y));
	pt.image(bg.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
	let g = 0.7;
	let c = rect.min + Vec2::new(414.0, 134.0) * k;
	let at = |x: f32, y: f32| c + Vec2::new(x, y) * g * k;
	let sprite = |cx: f32, cy: f32, u: Rect, w: f32, h: f32, tint: Color32| pt.image(ch.id(), Rect::from_center_size(at(cx, cy), Vec2::new(w, h) * g * k), u, tint);
	sprite(-1.67, -3.33, uv(0.0, 318.0, 321.0, 319.0), 321.0, 319.0, Color32::WHITE);
	let o = Vec2::new(0.5, -0.5);
	let tip = [Vec2::new(-136.5, -6.5), Vec2::new(72.5, -113.5), Vec2::new(64.5, 118.5)];
	let tri: Vec<Pos2> = (0..3).map(|i| at(o.x, o.y) + (tip[i] - o) * (lv[i].min(max) as f32 / max as f32).max(0.04) * g * k).collect();
	pt.add(egui::Shape::convex_polygon(tri, Color32::from_rgba_unmultiplied(84, 86, 140, 235), Stroke::NONE));
	sprite(0.0, 0.0, uv(0.0, 0.0, 321.0, 319.0), 321.0, 319.0, Color32::WHITE);
	type Grp = ((f32, f32), f32, [f32; 3], (f32, f32), (f32, f32));
	let groups: [Grp; 3] = [
		((-227.0, -56.0), 9.0, [0.13, 0.4, 0.56], (27.5, 24.0), (12.0, 80.0)),
		((39.0, -165.0), 44.0, [0.33, 0.33, 0.33], (72.5, 9.0), (102.0, 40.0)),
		((8.0, 140.0), 79.0, [0.56, 0.26, 0.49], (89.5, -22.0), (119.0, 8.0)),
	];
	for (i, &((gx, gy), ty, col, (nx, ny), (mx, my))) in groups.iter().enumerate() {
		let tint = Color32::from_rgb((col[0] * 255.0) as u8, (col[1] * 255.0) as u8, (col[2] * 255.0) as u8);
		sprite(gx + 43.0, gy + 17.0, uv(426.0, ty, 86.0, 34.0), 86.0, 34.0, Color32::WHITE);
		let d = lv[i].min(max);
		let ([dx, dy, dw, dh], order) = digits;
		let gw = dw / order.chars().count() as f32;
		if let Some(slot) = order.chars().position(|c| c.to_digit(10) == Some(d as u32)) {
			pt.image(ch.id(), Rect::from_min_size(at(gx + nx, gy + ny), Vec2::new(gw, dh) * g * k), uv(dx + slot as f32 * gw, dy, gw, dh), tint);
		}
		if d >= max {
			pt.image(ch.id(), Rect::from_min_size(at(gx + mx, gy + my), Vec2::new(46.0, 24.0) * g * k), uv(513.0, 9.0, 46.0, 24.0), tint);
		}
	}
}

fn atlas_uv(t: &egui::TextureHandle, c: u32) -> Rect {
	let s = t.size();
	let (w, h) = (48.0 / s[0] as f32, 48.0 / s[1] as f32);
	Rect::from_min_size(Pos2::new((c % 21) as f32 * w, (c / 21) as f32 * h), Vec2::new(w, h))
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

const VAN_STAYS: &str = "Van is always in the party — he can't be removed";

fn lineup_swap(list: &mut [u32], have: &[u32], a: LPos, b: LPos) -> Option<&'static str> {
	match (a, b) {
		(LPos::Line(i), LPos::Line(j)) => list.swap(i, j),
		(LPos::Line(i), LPos::Avail(j)) | (LPos::Avail(j), LPos::Line(i)) => {
			if list[i] == 0 {
				return Some(VAN_STAYS);
			}
			list[i] = have[j];
		}
		(LPos::Avail(_), LPos::Avail(_)) => {}
	}
	None
}

fn lineup_move(list: &mut Vec<u32>, have: &[u32], a: LPos, zone: u8) -> Option<&'static str> {
	let c = match a {
		LPos::Line(i) => list[i],
		LPos::Avail(j) => have[j],
	};
	match (a, zone) {
		(LPos::Line(_), 2) if c == 0 => return Some(VAN_STAYS),
		(LPos::Line(i), 2) => {
			list.remove(i);
		}
		(LPos::Line(i), 1) => {
			let c = list.remove(i);
			list.push(c);
		}
		(LPos::Avail(_), 0) if list.len() >= db1::LINEUP_ACTIVE => {}
		(LPos::Avail(_), 0 | 1) if list.len() < db1::LINEUP_MAX => list.push(c),
		_ => {}
	}
	None
}

struct Slot {
	name: String,
	dir: PathBuf,
	title: String,
	sub: String,
	when: String,
	playtime: String,
	chapter: String,
}

fn read_slots(root: &Path) -> Vec<Slot> {
	let mut v: Vec<Slot> = std::fs::read_dir(root)
		.into_iter()
		.flatten()
		.flatten()
		.filter(|e| e.path().join("user.dat").is_file())
		.filter_map(|e| {
			let name = e.file_name().to_string_lossy().to_string();
			if !name.starts_with("save") {
				return None;
			}
			let j: serde_json::Value = std::fs::read_to_string(e.path().join("detail.json")).ok().and_then(|s| serde_json::from_str(s.trim_start_matches('\u{feff}')).ok()).unwrap_or_default();
			let s = |k: &str| j.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
			let t = &j["Time"];
			let n = |k: &str| t.get(k).and_then(|x| x.as_u64()).unwrap_or(0);
			let detail = s("detail");
			let playtime = detail
				.lines()
				.filter_map(|l| l.split_once(": "))
				.find(|(_, v)| {
					let v = v.trim();
					v.len() >= 7 && v.chars().all(|c| c.is_ascii_digit() || c == ':') && v.matches(':').count() == 2
				})
				.map(|(_, v)| {
					let v = v.trim();
					let (h, rest) = v.split_once(':').unwrap_or((v, ""));
					format!("{}:{rest}", h.parse::<u32>().unwrap_or(0))
				})
				.unwrap_or_default();
			let chapter = detail.lines().next().unwrap_or("").trim().to_string();
			Some(Slot {
				name,
				dir: e.path(),
				title: s("title"),
				sub: s("subtitle"),
				when: if n("year") > 0 { format!("{:02}.{:02}.{} {:02}:{:02}", n("day"), n("month"), n("year"), n("hour"), n("minute")) } else { String::new() },
				playtime,
				chapter,
			})
		})
		.collect();
	v.sort_by(|a, b| a.name.cmp(&b.name));
	v
}

fn default_dir(game: usize) -> String {
	std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join("Saved Games").join("Falcom").join(falcom_dir(game))).filter(|p| p.is_dir()).map(|p| p.display().to_string()).unwrap_or_default()
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

#[derive(Clone)]
enum Act {
	Overwrite,
	Close,
	Open(usize),
}

fn dir_signature(root: &Path) -> u64 {
	use std::hash::{Hash, Hasher};
	let mut h = std::collections::hash_map::DefaultHasher::new();
	let mut v: Vec<(String, u64)> = std::fs::read_dir(root)
		.into_iter()
		.flatten()
		.flatten()
		.filter_map(|e| {
			let m = std::fs::metadata(e.path().join("user.dat")).ok()?;
			let t = m.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
			Some((e.file_name().to_string_lossy().to_string(), t))
		})
		.collect();
	v.sort();
	v.hash(&mut h);
	h.finish()
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
	items: Vec<save::Item>,
	mons: Vec<save::Mon>,
	achs: Vec<save::Ach>,
	mon_group: usize,
	mon_sel: usize,
	enemies: HashMap<usize, save::Enemy>,
	btl: save::Battle,
	mon_diff: Option<u32>,
	ach_tab: usize,
	fish: Vec<save::Fish>,
	fish_sel: usize,
	names: HashMap<u32, (String, String)>,
	skills: HashMap<u32, save::Skill>,
	kinds: HashMap<u8, String>,
	dlcs: Vec<save::Dlc>,
	qlines: HashMap<(u32, usize), ([u8; 4], usize)>,
	collapsed: std::collections::HashSet<String>,
	models: HashMap<u32, u32>,
	art_tex: HashMap<String, Option<egui::TextureHandle>>,
	pics: Pics,
	hover_game: Option<usize>,
	game: Option<usize>,
	g: &'static save::Profile,
	owners: HashMap<usize, u32>,
	faces: HashMap<u32, Vec<u8>>,
	face_tex: HashMap<u32, Option<egui::TextureHandle>>,
	icons: Option<Option<egui::TextureHandle>>,
	status: String,
	dialog: Dialog<Act>,
	slot_pick: Option<SlotPick>,
	zoom: f32,
	about: bool,
	tips: bool,
	show_service: bool,
	allow_close: bool,
	scan_at: f64,
	scan_sig: u64,
	scan_dir: String,
	scroll_sel: bool,
	slot_q: String,
	shown_title: String,
}

impl App {
	fn new(cc: &eframe::CreationContext) -> App {
		setup_fonts(&cc.egui_ctx);
		number_cursor(&cc.egui_ctx);
		tick_green();
		egui_extras::install_image_loaders(&cc.egui_ctx);
		let st = Settings::load("Calvard-SaveEditor.ini", "Calvard-SaveEditor");
		let pal = Pal::get(st.get("dark") != Some("0"));
		apply_visuals(&cc.egui_ctx, &pal);
		let dir = st.get("dir").map(|s| s.to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| default_dir(0));
		let collapsed = st.get("ach_folded").unwrap_or("").split(',').filter(|k| !k.is_empty()).map(str::to_string).collect();
		let mut app = App {
			pal,
			dir,
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
			owners: HashMap::new(),
			faces: HashMap::new(),
			face_tex: HashMap::new(),
			icons: None,
			items: Vec::new(),
			mons: Vec::new(),
			achs: Vec::new(),
			mon_group: 0,
			mon_sel: 0,
			enemies: HashMap::new(),
			btl: save::Battle::default(),
			mon_diff: None,
			ach_tab: 0,
			fish: Vec::new(),
			fish_sel: 0,
			names: HashMap::new(),
			skills: HashMap::new(),
			models: HashMap::new(),
			kinds: HashMap::new(),
			dlcs: Vec::new(),
			qlines: HashMap::new(),
			collapsed,
			art_tex: HashMap::new(),
			pics: Pics::default(),
			hover_game: None,
			game: None,
			g: &save::DB1,
			status: "Select a slot on the left. Before writing, the program backs the slot up to the «Backups» folder next to it.".into(),
			dialog: Dialog::None,
			slot_pick: None,
			zoom: 1.0,
			about: false,
			tips: false,
			show_service: false,
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
		match self.game {
			Some(1) => format!("{k}2"),
			Some(2) => format!("{k}3"),
			_ => k.to_string(),
		}
	}

	fn set_game(&mut self, i: usize) {
		self.save_settings();
		self.game = Some(i);
		self.st.set("last_game", &i.to_string());
		let g = profile(i);
		self.g = g;
		self.items = save::items(save::table(g.items_tsv));
		self.mons = save::mons(save::table(save::notes(g).mons_tsv));
		let key = save::notes(g).mons_tsv.trim_end_matches("_mons.tsv");
		self.enemies = save::enemies(save::table(&format!("{key}_enemy.tsv")));
		self.btl = save::battle(save::table(&format!("{key}_btl.tsv")));
		self.mon_diff = None;
		self.achs = save::achs(save::table(save::notes(g).ach_tsv));
		self.fish = save::fishing(g).map(|f| save::fishes(save::table(f.tsv))).unwrap_or_default();
		self.fish_sel = 0;
		self.names = save::names(save::table(g.names_tsv));
		for n in self.names.values_mut() {
			let short = n.0.strip_suffix(" the Immovable").or_else(|| n.0.strip_prefix("Security Chief ")).or_else(|| n.0.strip_prefix("Dominion ")).map(str::to_string);
			if let Some(s) = short {
				n.0 = s;
			}
		}
		self.skills = save::skills(save::table(g.skills_tsv));
		self.models = save::models(save::table(g.names_tsv));
		self.kinds = save::kinds(save::table(g.kinds_tsv));
		self.dlcs = save::dlcs(save::table(g.dlc_tsv));
		self.qlines = save::qlines(save::table(g.qline_tsv));
		self.owners = self.items.iter().map(|it| (it.id, it.owner)).collect();
		let dir = g.faces.rsplit_once('/').map_or("", |x| x.0);
		self.faces = match (kiseki_tool::pack::get(&format!("{dir}/portraits.png")), kiseki_tool::pack::get(&format!("{dir}/portraits.tsv"))) {
			(Some(a), Some(t)) => save::faces_atlas(&a, &String::from_utf8_lossy(&t), 128),
			_ => kiseki_tool::pack::get(g.faces).map(|b| save::faces(&b)).unwrap_or_default(),
		};
		self.face_tex.clear();
		self.icons = None;
		self.art_tex.retain(|k, _| k.starts_with("cover:"));
		self.data = None;
		self.sel = None;
		self.chr_sel = 0;
		self.lsel = None;
		self.drag_src = None;
		self.dir = self.st.get(&self.key("dir")).map(|s| s.to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| default_dir(i));
		self.item_tab = self.st.get(&self.key("item_tab")).and_then(|v| v.parse().ok()).unwrap_or(0).min(g.tabs.len());
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
		self.sel = Some(i);
		self.scroll_sel = true;
		match save::read_for(self.g, &self.slots[i].dir.join("user.dat")) {
			Ok(d) => {
				self.orig = d.clone();
				self.data = Some(d);
				self.status = format!("Opened {}: {} — {}", self.slots[i].name, self.slots[i].title, self.slots[i].sub);
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
			None => {
				match save::read_for(self.g, file) {
					Ok(d) => {
						self.slots.push(Slot { name: slot.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), dir: slot.to_path_buf(), title: "(opened file)".into(), sub: file.display().to_string(), when: String::new(), playtime: String::new(), chapter: String::new() });
						self.sel = Some(self.slots.len() - 1);
						self.orig = d.clone();
						self.data = Some(d);
						self.status = format!("Opened {}", file.display());
					}
					Err(e) => {
						self.wrong_game(&e);
						self.status = format!("✖ {e}");
					}
				}
			}
		}
		self.save_settings();
	}

	fn face(&mut self, ctx: &egui::Context, chr: u32) -> Option<egui::TextureHandle> {
		if let Some(t) = self.face_tex.get(&chr) {
			return t.clone();
		}
		let t = self.faces.get(&chr).and_then(|png| image::load_from_memory(png).ok()).map(|img| {
			face_texture(ctx, format!("face{chr}"), &img.to_rgba8())
		});
		self.face_tex.insert(chr, t.clone());
		t
	}

	fn art(&mut self, ctx: &egui::Context, name: &str) -> Option<egui::TextureHandle> {
		if let Some(t) = self.art_tex.get(name) {
			return t.clone();
		}
		let path = if let Some(c) = name.strip_prefix("cover:") {
			format!("covers/{c}.png")
		} else if let Some(f) = name.strip_prefix("ui:") {
			format!("{}/ui/{f}", self.g.dir)
		} else {
			format!("{}/art/{name}.png", self.g.dir)
		};
		if let Some(c) = name.strip_prefix("blur:") {
			let t = kiseki_tool::pack::get(&format!("covers/{c}.png")).and_then(|png| blurred_texture(ctx, name, &png));
			self.art_tex.insert(name.to_string(), t.clone());
			return t;
		}
		let t = kiseki_tool::pack::get(&path).and_then(|png| image::load_from_memory(&png).ok()).map(|img| {
			let img = img.to_rgba8();
			let size = [img.width() as usize, img.height() as usize];
			if name.starts_with("ui:fish") {
				return ctx.load_texture(name, premultiplied(&img), FACE_TEX);
			}
			ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), TEX_SMOOTH)
		});
		self.art_tex.insert(name.to_string(), t.clone());
		t
	}

	fn icons(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
		self.icons
			.get_or_insert_with(|| {
				kiseki_tool::pack::get(self.g.icons).and_then(|png| image::load_from_memory(&png).ok()).map(|img| {
					let img = img.to_rgba8();
					ctx.load_texture("icons", premultiplied(&img), TEX_SMOOTH)
				})
			})
			.clone()
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
		let backup = exe_dir().join("Backups").join(format!("{}_{}", slot.name, stamp()));
		if let Err(e) = copy_dir(&slot.dir, &backup) {
			self.dialog = Dialog::Info(format!("Could not back up the slot — it was not changed.\n{e}"));
			return;
		}
		match save::write(&slot.dir.join("user.dat"), &mut d) {
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
			let Some(n) = name.strip_prefix("save").filter(|n| n.len() == 3).and_then(|n| n.parse::<u32>().ok()) else { continue };
			let s = self.slots.iter().find(|s| s.name == name);
			used.insert(n, UsedSlot { title: s.map(|s| if s.chapter.is_empty() { s.title.clone() } else { s.chapter.clone() }).unwrap_or_default(), when: s.map(|s| s.when.clone()).unwrap_or_default(), locked: s.is_none() });
		}
		self.slot_pick = Some(SlotPick::new((0, 999), used));
	}

	fn save_to(&mut self, num: u32) {
		let (Some(i), Some(mut d)) = (self.sel, self.data.clone()) else { return };
		let n = format!("save{num:03}");
		if self.slots[i].name == n {
			self.overwrite();
			return;
		}
		let to = PathBuf::from(self.dir.trim()).join(&n);
		let from = self.slots[i].dir.clone();
		let backup = to.exists().then(|| exe_dir().join("Backups").join(format!("{n}_{}", stamp())));
		if let Some(b) = &backup
			&& let Err(e) = copy_dir(&to, b)
		{
			self.dialog = Dialog::Info(format!("Could not back up the slot — it was not changed.\n{e}"));
			return;
		}
		let res = (|| -> Result<(), String> {
			std::fs::create_dir_all(&to).map_err(|e| e.to_string())?;
			save::write(&to.join("user.dat"), &mut d)?;
			for f in ["detail.json", "icon0.png"] {
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

	fn picker(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let p = self.pal;
		let full = ui.max_rect();
		let covers: Vec<Option<egui::TextureHandle>> = GAMES.iter().map(|g| self.art(ctx, &format!("cover:{}", g.0))).collect();
		const ACCENT: [Color32; 3] = [Color32::from_rgb(0x4A, 0x8F, 0xE7), Color32::from_rgb(0xE5, 0x54, 0x3A), Color32::from_rgb(0x5C, 0xC3, 0xE0)];
		const ARC: [&str; 3] = ["CALVARD ARC · I", "CALVARD ARC · II", "CALVARD ARC · III"];
		let mut open = None;
		let blurs: Vec<Option<egui::TextureHandle>> = GAMES.iter().map(|g| self.art(ctx, &format!("blur:{}", g.0))).collect();
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
		let cw = (full.width() - 64.0).min(1320.0);
		let x0 = full.center().x - cw / 2.0;
		let gap = 24.0;
		let cols = (((cw + gap) / (280.0 + gap)).floor() as usize).clamp(1, GAMES.len());
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
		let fx0 = x0;
		match self.pick_game {
			Some(i) => {
				ui.painter().text(Pos2::new(fx0, foot.center().y - 9.0), Align2::LEFT_CENTER, "Selected", f_body(11.5), p.muted);
				ui.painter().text(Pos2::new(fx0, foot.center().y + 9.0), Align2::LEFT_CENTER, GAMES[i].1, f_semi(15.0), p.text);
			}
			None => {
				ui.painter().text(Pos2::new(fx0, foot.center().y), Align2::LEFT_CENTER, "Back up your save files before editing. This is an unofficial fan tool.", f_body(13.0), p.muted);
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
				ui.label(egui::RichText::new("Save editor for Trails through Daybreak, Daybreak II and beyond the Horizon (PC).").font(f_body(13.0)).color(p.muted));
				ui.label(egui::RichText::new("Edits: mira and tokens, difficulty, rank, L.G.C., party, items and DLC, enemy log, fishing, achievements.").font(f_body(13.0)).color(p.muted));
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
				if checkbox(ui, &p, &mut svc, "Show internal items (◆)", 16.0, f_body(13.0))
					.on_hover_text("Game entries marked with ◆: spare book slots, debug «Main Verification Slot» weapons, guest gear per chapter. Hidden by default.")
					.changed()
				{
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
					"Save slots are on the left: search by number, Up / Down arrows — next slot. «Browse…» — the save folder or a file; you can also drop a user.dat file onto the window.",
					"«General» — mira, tokens, difficulty, Total SP and rank, L.G.C., sepith, play time.",
					"«Party» — drag a card onto another one, or click two cards, to swap them; click a card and then an empty «+» slot to move it. Into «Can be added» — remove from the party.",
					"«Items» — the amount of any item; untick «Owned only» to add a new one. Click a name for its description, a section title to fold it.",
					"«Monsters» — the tick logs an enemy or removes it; log achievements follow.",
					"«Fishing» (Daybreak II, Horizon) — the tick: caught, rank A; the first row shows gear and all fish. The game derives rank A from the record.",
					"«Achievements» — click the box: complete / uncheck; right-click a completed one — reward claimed. Click a section title to fold it.",
					"«Save to new slot» — pick the number; a used slot gets overwritten. The previous save is always copied to «Backups» next to the program (About opens the folder) first.",
					"Ctrl+S — save to this slot, Ctrl + mouse wheel — zoom. Middle-click in a list — scroll by moving the mouse (click again or Esc to stop).",
					"Numbers: drag left/right, or click and type. «?» next to a rank — thresholds and rewards.",
					"Internal game items (◆) are hidden — you can show them in «About».",
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
			let game = save::read(&path).ok().and_then(|d| (0..GAMES.len()).find(|&i| profile(i).size == d.len()));
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
				None => self.dialog = Dialog::Info(format!("{} is not a Calvard save.", path.display())),
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
			self.scroll_sel = true;
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
			let hint = format!("C:\\Users\\…\\Saved Games\\Falcom\\{}", falcom_dir(self.game.unwrap_or(0)));
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
			} else if let Some(f) = one.as_ref().map(|d| if d.is_dir() { d.join("user.dat") } else { d.clone() }).filter(|f| f.is_file()) {
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
					ui.painter().text(Pos2::new(r.min.x + 10.0, r.min.y + 13.0), Align2::LEFT_CENTER, &s.name, f_mono(12.0), p.muted);
					ui.painter().text(Pos2::new(r.max.x - 10.0, r.min.y + 11.0), Align2::RIGHT_CENTER, &s.when, f_body(11.5), p.muted);
					let w = r.width() - 20.0;
					paint_text_clipped(ui, Rect::from_min_size(Pos2::new(r.min.x + 10.0, r.min.y + 24.0), Vec2::new(w, 18.0)), &s.title, f_semi(13.5), p.text, Align2::LEFT_CENTER);
					paint_text_clipped(ui, Rect::from_min_size(Pos2::new(r.min.x + 10.0, r.min.y + 43.0), Vec2::new(w, 17.0)), &s.sub, f_body(12.5), p.text, Align2::LEFT_CENTER);
					let pt = if s.playtime.is_empty() { String::new() } else { format!("Play time {}", s.playtime) };
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
			let tabs: &[(usize, &'static str)] = &[(0, "General"), (1, "Party"), (2, "Items"), (3, "Monsters"), (5, "Fishing"), (4, "Achievements")];
			let tabs: Vec<_> = tabs.iter().filter(|t| t.0 != 5 || self.has_fishing()).collect();
			let mut k = tabs.iter().position(|t| t.0 == self.tab).unwrap_or(0);
			segmented(&mut r, &p, "tab", &tabs.iter().map(|t| t.1).collect::<Vec<_>>(), &mut k, None, 36.0, 9, f_body(14.0), &[]);
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
				_ => self.achievements(&mut c, ctx),
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
		let (chapter, place) = self.sel.and_then(|i| self.slots.get(i)).map(|s| (s.chapter.clone(), s.sub.clone())).unwrap_or_default();
		let lgc_tex = self.art(ctx, "ui:lgc_bg.png").zip(self.art(ctx, "ui:lgc_chart.png"));
		let g = self.g;
		let cell = |c: u32| atlas.as_ref().map(|t| (t.id(), atlas_uv(t, c)));
		let rank_rows: Vec<RankRow> = g.ranks.iter().zip(g.rank_rewards).map(|(&(r, t), rew)| RankRow { name: r.into(), pts: t, rewards: rew.iter().filter(|&&i| i > 0).map(|&i| self.items.iter().find(|it| it.id == i).map_or((None, format!("#{i}")), |it| (it.icon.and_then(cell), it.en.clone()))).collect() }).collect();
		let d = self.data.as_mut().unwrap();
		const LW: f32 = 118.0;
		const FW: f32 = 96.0;
		let head = |ui: &mut egui::Ui, t: &str| {
			ui.add_space(14.0);
			ui.label(egui::RichText::new(t).font(f_semi(15.5)).color(p.text));
			ui.add_space(4.0);
		};
		let name = |ui: &mut egui::Ui, t: &str| {
			ui.allocate_ui_with_layout(Vec2::new(LW, 26.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
				ui.set_min_width(LW);
				ui.label(egui::RichText::new(t).font(f_body(13.0)).color(p.muted));
			});
		};
		egui::ScrollArea::vertical().id_salt("general").auto_shrink([false, false]).show(ui, |ui| {
		ui.add_space(4.0);
		ui.add(egui::Label::new(egui::RichText::new(if chapter.is_empty() { "—" } else { chapter.as_str() }).font(f_semi(26.0)).color(p.text)).wrap());
		if !place.is_empty() {
			ui.label(egui::RichText::new(&place).font(f_semi(14.0)).color(p.muted));
		}
		ui.add_space(2.0);
		let wide = ui.available_width() >= 1140.0;
		ui.with_layout(if wide { egui::Layout::left_to_right(egui::Align::Min) } else { egui::Layout::top_down(egui::Align::Min) }, |ui| {
			ui.spacing_mut().item_spacing.y = 6.0;
			ui.vertical(|ui| {
				ui.horizontal_top(|ui| {
					ui.vertical(|ui| {
						head(ui, "Currency");
						ui.horizontal(|ui| {
							name(ui, "Mira");
							let mut mira = save::u32_at(d, g.mira);
							if ui.add_sized([FW, 26.0], egui::DragValue::new(&mut mira).clamp_existing_to_range(false).range(0..=9_999_999).speed(0.0)).changed() {
								save::set_u32(d, g.mira, mira);
							}
						});
						if let Some(o) = g.s_token {
							ui.horizontal(|ui| {
								name(ui, g.token_name);
								let mut v = save::u32_at(d, o);
								if ui.add_sized([FW, 26.0], egui::DragValue::new(&mut v).clamp_existing_to_range(false).range(0..=9_999_999).speed(0.0)).changed() {
									save::set_u32(d, o, v);
								}
							});
						}
						if let Some(o) = g.casino {
							ui.horizontal(|ui| {
								name(ui, "Tokens");
								let mut v = save::u32_at(d, o);
								if ui.add_sized([FW, 26.0], egui::DragValue::new(&mut v).clamp_existing_to_range(false).range(0..=9_999_999).speed(0.0)).changed() {
									save::set_u32(d, o, v);
								}
							});
						}
					});
					ui.add_space(24.0);
					ui.vertical(|ui| {
						head(ui, "Difficulty");
						let (mut cmd, mut fld) = save::difficulty(g, d);
						let (c0, f0) = (cmd, fld);
						const CMD: [&str; 5] = ["Very Easy", "Easy", "Normal", "Hard", "Nightmare"];
						const FLD: [&str; 3] = ["Easy", "Normal", "Hard"];
						ui.horizontal(|ui| {
							name(ui, "Command Battle");
							egui::ComboBox::from_id_salt("diff_cmd").width(FW + 40.0).selected_text(CMD.get(cmd as usize).copied().unwrap_or("?")).show_ui(ui, |ui| {
								for (i, t) in CMD.iter().enumerate() {
									ui.selectable_value(&mut cmd, i as u32, *t);
								}
							});
						});
						ui.horizontal(|ui| {
							name(ui, "Field Battle");
							egui::ComboBox::from_id_salt("diff_fld").width(FW + 40.0).selected_text(FLD.get((fld as usize).wrapping_sub(1)).copied().unwrap_or("?")).show_ui(ui, |ui| {
								for (i, t) in FLD.iter().enumerate() {
									ui.selectable_value(&mut fld, i as u32 + 1, *t);
								}
							});
						});
						if (cmd, fld) != (c0, f0) {
							save::set_difficulty(g, d, cmd, fld);
						}
					});
				});
				head(ui, "Arkride Solutions");
				ui.horizontal(|ui| {
					name(ui, "Total SP");
					let mut sp = save::total_sp(g, d);
					let can = save::any_quest_done(g, d);
					let top = g.ranks.last().map_or(9_999, |r| r.1).max(sp);
					let r = ui.add_enabled_ui(can, |ui| ui.add_sized([FW, 26.0], egui::DragValue::new(&mut sp).clamp_existing_to_range(false).range(0..=top).speed(0.0))).inner;
					if r.changed() {
						save::set_total_sp(g, d, sp);
					}
					r.on_hover_text(if g.sp_extra.is_some() { "The game counts SP from completed quests plus an extra amount; the difference goes into that extra amount" } else { "The game counts SP from completed quests; the difference goes into those quests' bonuses" });
					let sp = save::total_sp(g, d);
					help_icon(ui, &p, |ui| {
						let cur = g.ranks.iter().rposition(|&(_, t)| sp >= t);
						rank_table(ui, &p, "Arkride Solutions ranks", "SP", &rank_rows, cur);
					});
				});
				ui.horizontal(|ui| {
					name(ui, "Office Rank");
					let sp = save::total_sp(g, d);
					ui.label(egui::RichText::new(save::office_rank(g, sp)).font(f_semi(14.0)).color(p.text));
					if let Some(&(nm, t)) = g.ranks.iter().find(|r| r.1 > sp) {
						ui.add_space(8.0);
						ui.label(egui::RichText::new(format!("{1} more to {0}", nm, t - sp)).font(f_body(12.5)).color(p.muted));
					}
				});
				head(ui, "Sepith");
				egui::Grid::new("sepith").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
					for (k, nm) in SEPITH.iter().enumerate() {
						ui.horizontal(|ui| {
							ui.set_width(LW);
							atlas_cell(ui, &atlas, Some(save::cells::SEPITH + k as u32), 24.0);
							ui.label(egui::RichText::new(*nm).font(f_body(13.0)).color(p.muted));
						});
						let o = g.sepith + k * 4;
						let mut v = save::u32_at(d, o);
						if ui.add_sized([FW, 26.0], egui::DragValue::new(&mut v).clamp_existing_to_range(false).range(0..=99_999).speed(0.0)).changed() {
							save::set_u32(d, o, v);
						}
						if k % 2 == 1 {
							ui.end_row();
						}
					}
				});
				head(ui, "Play time");
				let t = save::playtime(g, d);
				let (mut h, mut m, mut s) = ((t / 3600.0) as u32, (t % 3600.0 / 60.0) as u32, (t % 60.0) as u32);
				ui.horizontal(|ui| {
					name(ui, "Playtime");
					let mut ch = false;
					ch |= ui.add_sized([64.0, 26.0], egui::DragValue::new(&mut h).clamp_existing_to_range(false).range(0..=9_999).suffix(" h").speed(0.0)).changed();
					ch |= ui.add_sized([64.0, 26.0], egui::DragValue::new(&mut m).clamp_existing_to_range(false).range(0..=59).suffix(" min").speed(0.0)).changed();
					ch |= ui.add_sized([64.0, 26.0], egui::DragValue::new(&mut s).clamp_existing_to_range(false).range(0..=59).suffix(" s").speed(0.0)).changed();
					if ch {
						save::set_playtime(g, d, (h * 3600 + m * 60 + s) as f64 + t.fract());
					}
				});
				ui.label(egui::RichText::new("The game updates the time in the slot description on the next save.").font(f_body(12.0)).color(p.muted));
			});
			ui.add_space(if wide { 96.0 } else { 24.0 });
			ui.vertical(|ui| {
				let lv: [usize; 3] = std::array::from_fn(|k| save::lgc_level(g, k, save::lgc(g, d, k)).0);
				let game_look = lgc_tex.is_some();
				if !game_look {
					head(ui, "L.G.C. Alignment");
				}
				let side = !game_look && ui.available_width() >= 660.0;
				ui.with_layout(if side { egui::Layout::left_to_right(egui::Align::Min) } else { egui::Layout::top_down(egui::Align::Min) }, |ui| {
					match &lgc_tex {
						Some((bg, ch)) => {
							ui.add_space(10.0);
							let room = if wide { ui.clip_rect().max.y - ui.cursor().min.y - 130.0 } else { f32::INFINITY };
							let k = (ui.available_width() / 576.0).min(room / 320.0).clamp(0.45, 1.0);
							lgc_game(ui, bg, ch, lv, save::lgc_max(g), g.lgc_digits, k);
						}
						None => lgc_chart(ui, &p, lv, save::lgc_max(g)),
					}
					ui.add_space(8.0);
					ui.vertical(|ui| {
						ui.add_space(if side { 40.0 } else { 4.0 });
						egui::Grid::new("lgc").num_columns(3).spacing([12.0, 10.0]).show(ui, |ui| {
							for (k, nm) in ["Law", "Gray", "Chaos"].iter().enumerate() {
								let mut v = save::lgc(g, d, k);
								let (lv, _) = save::lgc_level(g, k, v);
								ui.allocate_ui_with_layout(Vec2::new(118.0, 26.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
									ui.set_min_width(118.0);
									ui.label(egui::RichText::new(format!("{nm} LV: {lv}")).font(f_semi(13.5)).color(p.text));
									help_icon(ui, &p, |ui| {
										let s = g.lgc_steps[k];
										let rows: Vec<[String; 2]> = s.iter().enumerate().map(|(i, x)| [format!("LV {}", g.lgc_first + i), x.to_string()]).collect();
										tip_table(ui, &p, &format!("{nm} gauge EXP"), ["Level", "EXP"], &rows, lv.checked_sub(g.lgc_first));
									});
								});
								let top = g.lgc_steps[k].last().copied().unwrap_or(99_999_999).max(v);
								if ui.add_sized([FW, 26.0], egui::DragValue::new(&mut v).clamp_existing_to_range(false).range(0..=top).speed(0.0)).changed() {
									save::set_lgc(g, d, k, v);
									save::give_mare_form(g, d, k);
								}
								let (_, next) = save::lgc_level(g, k, v);
								let txt = match next {
									Some(n) => format!("{n} more to LV {}", lv + 1),
									None => "max".to_string(),
								};
								ui.label(egui::RichText::new(txt).font(f_body(12.5)).color(p.muted));
								ui.end_row();
							}
						});
						ui.add_space(6.0);
						ui.label(egui::RichText::new("The field holds the gauge EXP.").font(f_body(12.0)).color(p.muted));
					});
				});
			});
		});
		}).middle_scroll(ui);
	}
	fn chr_name(&self, c: u32) -> String {
		match c {
			62 => "Yin (Rixia)".into(),
			_ => self.names.get(&c).map(|n| n.0.clone()).unwrap_or_else(|| format!("#{c}")),
		}
	}

	fn lineup(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let g = self.g;
		let p = self.pal;
		let (list, have) = {
			let d = self.data.as_ref().unwrap();
			let list = save::lineup(g, d);
			let have: Vec<u32> = g.addable.iter().map(|a| a.0).filter(|c| !list.contains(c)).collect();
			(list, have)
		};
		let unjoined: Vec<u32> = {
			let d = self.data.as_ref().unwrap();
			g.addable.iter().filter(|&&(_, k)| save::u32_at(d, save::chr_at(g, k)) == 0).map(|a| a.0).collect()
		};
		let at = |pos: LPos| match pos {
			LPos::Line(i) => list.get(i).copied(),
			LPos::Avail(j) => have.get(j).copied(),
		};
		if self.lsel.is_some_and(|s| at(s).is_none()) {
			self.lsel = None;
		}
		let faces: HashMap<u32, Option<egui::TextureHandle>> = list.iter().chain(have.iter()).map(|&c| (c, self.face(ctx, c))).collect();
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
		const BIG: CardSize = CardSize { size: Vec2::new(150.0, 128.0), face: 60.0 };
		const SMALL: CardSize = CardSize { size: Vec2::new(140.0, 100.0), face: 44.0 };
		let mut card_at = |this: &Self, ui: &mut egui::Ui, pos: LPos, c: u32, big: bool, tag: &str| {
			let cs = if big { BIG } else { SMALL };
			let name = this.chr_name(c);
			let selected = this.lsel == Some(pos);
			let hovered_drop = dragging && ui.rect_contains_pointer(Rect::from_min_size(ui.cursor().min, cs.size));
			let r = lineup_card(ui, &p, cs, faces.get(&c).and_then(|t| t.as_ref()), &name, tag, selected, hovered_drop);
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
				if c == 0 {
					"always in party"
				} else if unjoined.contains(&c) {
					"hasn't joined"
				} else if g.npc_guests.contains(&c) {
					"guest, AI-controlled"
				} else {
					["in party", "reserve", "not in party"][zone as usize]
				}
			};
			let z0 = zone(ui, "In party", "4 slots", &mut |ui| {
				for i in 0..db1::LINEUP_ACTIVE {
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
			let reserve = list.len().saturating_sub(db1::LINEUP_ACTIVE);
			let z1 = zone(ui, "Reserve", &format!("{reserve}"), &mut |ui| {
				for (i, &c) in list.iter().enumerate().skip(db1::LINEUP_ACTIVE) {
					card_at(self, ui, LPos::Line(i), c, false, tag(c, 1));
				}
				drop_slot(ui, "drag here, or select a card and click here", 1);
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
					let sub = match from {
						LPos::Line(i) if i < db1::LINEUP_ACTIVE => "in party",
						LPos::Line(_) => "reserve",
						LPos::Avail(_) => "not in party",
					};
					lineup_ghost(ctx, &p, SMALL, pos, faces.get(&c).and_then(|t| t.as_ref()), &self.chr_name(c), sub);
				}
		let Some((a, b)) = act else { return };
		let mut new = list.clone();
		let mut warn = None;
		match b {
			Err(255) => match self.lsel.take() {
				None => self.lsel = Some(a),
				Some(s) if s == a => {}
				Some(s) => warn = lineup_swap(&mut new, &have, s, a),
			},
			Ok(t) => warn = lineup_swap(&mut new, &have, a, t),
			Err(z) => warn = lineup_move(&mut new, &have, a, z),
		}
		if let Some(w) = warn {
			self.status = format!("✖ {}", if w == VAN_STAYS { "Van is always in the party — he can't be removed" } else { w });
		}
		if b != Err(255) {
			self.lsel = None;
			self.drag_src = None;
		}
		if new != list {
			let added: Vec<(usize, String)> = new
				.iter()
				.filter(|c| !list.contains(c) && unjoined.contains(c))
				.filter_map(|&c| g.addable.iter().find(|a| a.0 == c).map(|a| (a.1, self.chr_name(c))))
				.collect();
			let d = self.data.as_mut().unwrap();
			save::set_lineup(g, d, &new);
			let lv = save::u32_at(d, save::chr_at(g, 0)).max(1);
			for (k, name) in added {
				save::set_u32(d, save::chr_at(g, k), lv);
				self.status = format!("{name} hasn't joined yet: set to level {lv}, same as Van; no weapon or crafts — check in game on a new slot");
			}
		}
	}

	fn monsters(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let g = self.g;
		let p = self.pal;
		let atlas = self.icons(ctx);
		let d = self.data.as_ref().unwrap();
		let mut groups: Vec<(u32, String)> = Vec::new();
		for m in &self.mons {
			if !groups.iter().any(|x| x.0 == m.group) {
				groups.push((m.group, m.group_name.clone()));
			}
		}
		if groups.is_empty() {
			return;
		}
		self.mon_group = self.mon_group.min(groups.len() - 1);
		let mut set: Vec<(usize, bool)> = Vec::new();
		let labels: Vec<String> = groups
			.iter()
			.map(|(gid, n)| {
				let all = self.mons.iter().filter(|m| m.group == *gid).count();
				let on = self.mons.iter().filter(|m| m.group == *gid && save::mon_logged(g, d, m.id)).count();
				format!("{}  {on}/{all}", if n.is_empty() { format!("#{gid}") } else { n.clone() })
			})
			.collect();
		let lr: Vec<&str> = labels.iter().map(String::as_str).collect();
		segmented_rows(ui, &p, "mgroups", &lr, &mut self.mon_group, 32.0, 8, f_body(13.0));
		ui.add_space(8.0);
		let total = self.mons.iter().filter(|m| !m.uncounted).count();
		let done = save::mons_logged_count(g, d, &self.mons) as usize;
		let gid = groups[self.mon_group].0;
		let title = format!("Logged {done} of {total} (EX not counted)");
		let need = text_w(ui, &title, f_semi(14.0)) + 40.0 + ["Clear all", "Log all", "Whole area"].iter().map(|t| small_button_w(ui, t)).sum::<f32>() + 300.0;
		let one = need <= ui.available_width();
		if !one {
			ui.label(egui::RichText::new(&title).font(f_semi(14.0)).color(p.text));
		}
		ui.horizontal(|ui| {
			if one {
				ui.label(egui::RichText::new(&title).font(f_semi(14.0)).color(p.text));
			}
			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				if small_button(ui, &p, "Clear all", None, 28.0, true, p.muted).clicked() {
					set.extend((0..self.mons.len()).map(|i| (i, false)));
				}
				if small_button(ui, &p, "Log all", None, 28.0, true, p.text).clicked() {
					set.extend((0..self.mons.len()).map(|i| (i, true)));
				}
				if small_button(ui, &p, "Whole area", None, 28.0, true, p.text).clicked() {
					set.extend((0..self.mons.len()).filter(|&i| self.mons[i].group == gid).map(|i| (i, true)));
				}
				let own = save::difficulty(g, d).0.min(4);
				let names: Vec<String> = ["Very Easy", "Easy", "Normal", "Hard", "Nightmare"].iter().enumerate().map(|(k, n)| if k as u32 == own { format!("{n} {}", "(in save)") } else { n.to_string() }).collect();
				let mut k = self.mon_diff.unwrap_or(own) as usize;
				ui.add_space(4.0);
				if combo(ui, &p, "mdiff", &names, &mut k, 210.0, 28.0, 6, f_body(13.0)) {
					self.mon_diff = Some(k as u32);
				}
				ui.label(egui::RichText::new("Difficulty").font(f_body(13.0)).color(p.muted));
			});
		});
		ui.add_space(6.0);
		let idx: Vec<usize> = (0..self.mons.len()).filter(|&i| self.mons[i].group == gid).collect();
		if !idx.contains(&self.mon_sel) {
			self.mon_sel = idx.first().copied().unwrap_or(0);
		}
		let h = ui.available_height();
		let lw = (ui.available_width() * 0.3).clamp(250.0, 340.0);
		let diff = self.mon_diff.unwrap_or_else(|| save::difficulty(g, d).0.min(4));
		let pics = &self.pics;
		let pic = |name: &str| if name.is_empty() { None } else { pics.pack(ctx, &format!("{}/ui/mons/{name}.webp", g.dir), FACE_TEX, premultiplied) };
		let levels = |m: &save::Mon, e: &save::Enemy| match save::mon_level(g, d, m.id) {
			Some(l) => {
				let b = if l > 0 && save::mon_logged(g, d, m.id) { l as u32 } else { e.level };
				(b, b + 2)
			}
			None => (e.level, e.level),
		};
		let enemies = &self.enemies;
		let mut sel = self.mon_sel;
		ui.horizontal_top(|ui| {
			ui.allocate_ui_with_layout(Vec2::new(lw, h), egui::Layout::top_down(egui::Align::Min), |ui| {
				egui::Frame::new().fill(p.surface).corner_radius(8).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
					ui.set_min_size(Vec2::new(lw - 16.0, h - 16.0));
					const ROW: f32 = 46.0;
					egui::ScrollArea::vertical().id_salt(("mlist", gid)).auto_shrink([false, false]).show_rows(ui, ROW, idx.len(), |ui, range| {
						ui.spacing_mut().item_spacing.y = 0.0;
						for &i in &idx[range] {
							let m = &self.mons[i];
							let e = enemies.get(&m.id);
							ui.horizontal(|ui| {
								ui.set_height(ROW);
								let on = save::mon_logged(g, d, m.id);
								let tip = if on { "Logged — click to remove from the log" } else { "Not in the log — click to log" };
								if tick(ui, if on { 2 } else { 0 }).on_hover_text(tip).clicked() {
									set.push((i, !on));
								}
								let (r, resp) = list_row(ui, &p, ROW - 4.0, sel == i);
								let x = r.left() + 4.0;
								let ir = Rect::from_min_size(Pos2::new(x, r.center().y - 19.0), Vec2::splat(38.0));
								match e.and_then(|e| pic(&e.pic)) {
									Some(t) => {
										ui.painter().image(t.id(), ir, UV_ALL, Color32::WHITE);
									}
									None => {
										ui.painter().rect_filled(ir, 4, p.div);
									}
								}
								let lv = e.map_or(String::new(), |e| levels(m, e).0.to_string());
								ui.painter().text(Pos2::new(r.right() - 8.0, r.center().y), Align2::RIGHT_CENTER, &lv, f_num(13.0), p.muted);
								let tr_ = Rect::from_min_max(Pos2::new(x + 46.0, r.top()), Pos2::new(r.right() - 34.0, r.bottom()));
								paint_text_clipped(ui, tr_, &m.name, f_body(15.0), if on { p.text } else { p.muted }, Align2::LEFT_CENTER);
								if resp.clicked() {
									sel = i;
								}
							});
						}
					})
					.middle_scroll(ui);
				});
			});
			ui.vertical(|ui| {
				egui::Frame::new().fill(p.surface).corner_radius(8).show(ui, |ui| {
					ui.set_min_size(Vec2::new(ui.available_width(), h));
					egui::ScrollArea::vertical().id_salt("mcard").auto_shrink([false, false]).max_height(h).show(ui, |ui| {
						let Some(m) = self.mons.get(sel) else { return };
						let on = save::mon_logged(g, d, m.id);
						let e = enemies.get(&m.id);
						let (base, x) = e.map_or((0, 0), |e| levels(m, e));
						let hr = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 52.0));
						let top = egui::CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 };
						ui.painter().rect_filled(hr, top, p.sel);
						if e.is_some_and(|e| e.boss) {
							let pink = Color32::from_rgb(170, 60, 110);
							let mut mesh = egui::Mesh::default();
							let l = hr.left() + hr.width() * 0.55;
							for (pt, c) in [(Pos2::new(l, hr.top()), Color32::TRANSPARENT), (Pos2::new(hr.right() - 8.0, hr.top()), pink), (Pos2::new(hr.right() - 8.0, hr.bottom()), pink), (Pos2::new(l, hr.bottom()), Color32::TRANSPARENT)] {
								mesh.colored_vertex(pt, c);
							}
							mesh.add_triangle(0, 1, 2);
							mesh.add_triangle(0, 2, 3);
							ui.painter().add(mesh);
							ui.painter().rect_filled(Rect::from_min_max(Pos2::new(hr.right() - 8.0, hr.top()), hr.max), egui::CornerRadius { nw: 0, ne: 8, sw: 0, se: 0 }, pink);
						}
						ui.scope_builder(egui::UiBuilder::new().max_rect(hr.shrink2(Vec2::new(12.0, 0.0))).layout(egui::Layout::left_to_right(egui::Align::Center)), |ui| {
							ui.set_min_size(hr.shrink2(Vec2::new(12.0, 0.0)).size());
							let tip = if on { "Logged — click to remove from the log" } else { "Not in the log — click to log" };
							if tick(ui, if on { 2 } else { 0 }).on_hover_text(tip).clicked() {
								set.push((sel, !on));
							}
							ui.add_space(6.0);
							ui.label(egui::RichText::new(&m.name).font(f_semi(24.0)).color(p.text));
							ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
								ui.label(egui::RichText::new(if base > 0 { base.to_string() } else { "—".into() }).font(f_num(30.0)).color(p.text));
								ui.label(egui::RichText::new("Base Lv.").font(f_body(13.0)).color(p.muted));
							});
						});
						let Some(e) = e else {
							ui.add_space(12.0);
							ui.label(egui::RichText::new("  No enemy data in the game tables").font(f_body(14.0)).color(p.muted));
							return;
						};
						let n = save::enemy_nums(&self.btl, e, diff, x);
						egui::Frame::new().inner_margin(egui::Margin::same(16)).show(ui, |ui| {
							let narrow = ui.available_width() < 560.0;
							let side = if narrow { 200.0 } else { 250.0 };
							let left = |ui: &mut egui::Ui| {
								ui.set_width(side);
								match pic(&e.pic) {
									Some(t) => {
										ui.add(egui::Image::new((t.id(), Vec2::splat(side))).fit_to_exact_size(Vec2::splat(side)));
									}
									None => {
										let (r, _) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
										ui.painter().rect_filled(r, 8, p.div);
									}
								}
								ui.add_space(8.0);
								let line = |ui: &mut egui::Ui, k: &str, v: u32, c: Color32, big: bool| {
									ui.horizontal(|ui| {
										ui.label(egui::RichText::new(k).font(if big { f_semi(15.0) } else { f_body(14.0) }).color(c));
										ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
											ui.label(egui::RichText::new(v.to_string()).font(f_num(if big { 20.0 } else { 15.0 })).color(p.text));
										});
									});
								};
								line(ui, "HP", n.hp, p.gold, true);
								line(ui, "EXP", e.exp, Color32::from_rgb(200, 100, 200), true);
								ui.separator();
								for (k, v) in [("STR", n.str), ("DEF", n.def), ("ATS", n.ats), ("ADF", n.adf), ("SPD", n.spd)] {
									line(ui, k, v, p.muted, false);
								}
							};
							let right = |ui: &mut egui::Ui| {
								let cap = |ui: &mut egui::Ui, t: &str| {
									ui.add_space(6.0);
									ui.label(egui::RichText::new(t).font(f_semi(13.0)).color(p.muted));
									ui.add_space(2.0);
								};
								const EL: [(&str, [u8; 3]); 7] = [("Ea", [201, 138, 46]), ("Wa", [58, 127, 208]), ("F", [208, 74, 58]), ("Wi", [58, 163, 90]), ("T", [90, 90, 100]), ("S", [201, 169, 46]), ("M", [150, 150, 170])];
								cap(ui, "Weaknesses");
								for (&(t, [r, g_, b]), &v) in EL.iter().zip(&e.elems).take(if e.tsm { 7 } else { 4 }) {
									let c = Color32::from_rgb(r, g_, b);
									ui.horizontal(|ui| {
										let (cr, _) = ui.allocate_exact_size(Vec2::new(30.0, 20.0), Sense::hover());
										ui.painter().rect_filled(cr, 3, c);
										ui.painter().text(cr.center(), Align2::CENTER_CENTER, t, f_semi(12.0), Color32::WHITE);
										let bw = (ui.available_width() - 56.0).max(40.0);
										let (br, _) = ui.allocate_exact_size(Vec2::new(bw, 20.0), Sense::hover());
										let bar = Rect::from_center_size(br.center(), Vec2::new(bw, 8.0));
										ui.painter().rect_filled(bar, 4, p.div);
										ui.painter().rect_filled(Rect::from_min_size(bar.min, Vec2::new(bw * (v.clamp(0, 200) as f32 / 200.0), 8.0)), 4, c);
										let vc = if v > 100 { p.ok_strong } else if v < 100 { p.warn } else { p.text };
										ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
											ui.label(egui::RichText::new(v.to_string()).font(f_num(16.0)).color(vc));
										});
									});
								}
								cap(ui, "Sepith Gained");
								const SN: [(&str, [u8; 3]); 8] = [("Earth", [201, 138, 46]), ("Water", [58, 127, 208]), ("Fire", [208, 74, 58]), ("Wind", [58, 163, 90]), ("Time", [70, 70, 80]), ("Space", [201, 169, 46]), ("Mirage", [150, 150, 170]), ("Mass", [215, 215, 220])];
								egui::Grid::new("msep").num_columns(2).spacing([28.0, 4.0]).min_col_width((ui.available_width() - 28.0) / 2.0).show(ui, |ui| {
									for row in 0..4 {
										for k in [row, row + 4] {
											let (t, [r, g_, b]) = SN[k];
											ui.horizontal(|ui| {
												let (cr, _) = ui.allocate_exact_size(Vec2::new(10.0, 16.0), Sense::hover());
												ui.painter().rect(cr, 2, Color32::from_rgb(r, g_, b), Stroke::new(1.0_f32, p.div2), StrokeKind::Inside);
												ui.label(egui::RichText::new(t).font(f_body(15.0)).color(p.text));
												ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
													ui.label(egui::RichText::new(e.sepith[k].to_string()).font(f_num(15.0)).color(p.text));
												});
											});
										}
										ui.end_row();
									}
								});
								cap(ui, "Items Gained");
								if e.items.is_empty() {
									ui.label(egui::RichText::new("—").font(f_body(14.0)).color(p.muted));
								}
								for &(id, rate) in &e.items {
									ui.horizontal(|ui| {
										match self.items.iter().find(|it| it.id == id) {
											Some(it) => {
												atlas_cell(ui, &atlas, it.icon, 24.0);
												ui.label(egui::RichText::new(&it.en).font(f_body(16.0)).color(p.text));
											}
											None => {
												ui.label(egui::RichText::new(format!("#{id}")).font(f_body(16.0)).color(p.text));
											}
										}
										ui.label(egui::RichText::new(format!("{rate}%")).font(f_body(13.0)).color(p.muted));
									});
								}
								ui.add_space(14.0);
								let area = if m.ex { format!("{} · EX", m.group_name) } else { m.group_name.clone() };
								let txt = if on { "in the log — recorded" } else { "in the log — not recorded" };
								ui.label(egui::RichText::new(format!("{area} · {txt}")).font(f_body(13.0)).color(p.muted));
							};
							if narrow {
								left(ui);
								ui.add_space(10.0);
								right(ui);
							} else {
								ui.horizontal_top(|ui| {
									ui.vertical(left);
									ui.add_space(28.0);
									ui.vertical(right);
								});
							}
						});
					})
					.middle_scroll(ui);
				});
			});
		});
		self.mon_sel = sel;
		if set.is_empty() {
			return;
		}
		let d = self.data.as_mut().unwrap();
		let logged_before = save::mons_logged_count(g, d, &self.mons);
		for (i, on) in set {
			let lv = save::area_level(g, d, &self.mons, self.mons[i].group);
			save::set_mon_logged(g, d, &self.mons[i], on, lv);
		}
		let before: Vec<u8> = self.achs.iter().map(|a| save::ach_state(g, d, a.id)).collect();
		save::sync_mons_achs(g, d, &self.mons, &self.achs, logged_before);
		let got = self.achs.iter().zip(&before).filter(|(a, b)| **b == 0 && save::ach_state(g, d, a.id) == 1).count();
		let n = save::mons_logged_count(g, d, &self.mons);
		self.status = if got > 0 { format!("Enemies logged: {n}; achievements completed: {got} — claim the rewards in game") } else { format!("Enemies logged: {n}") };
	}

	fn achievements(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let g = self.g;
		let p = self.pal;
		let atlas = self.icons(ctx);
		let d = self.data.as_ref().unwrap();
		let mons_type = save::notes(g).mons_type;
		let linked = |kind: u32| kind == mons_type || save::fish_kind(g, kind);
		let state = |id: usize| match save::ach_state(g, d, id) {
			0 => 0usize,
			3 => 2,
			_ => 1,
		};
		let cell = |c: u32| atlas.as_ref().map(|t| (t.id(), atlas_uv(t, c)));
		let shown = |a: &save::Ach| !a.cat_name.is_empty() && a.cat_name != "None";
		let counts: Vec<usize> = (0..3).map(|k| self.achs.iter().filter(|a| shown(a) && state(a.id) == k).count()).collect();
		let labels = [format!("In Progress  {}", counts[0]), format!("Completed  {}", counts[1]), format!("Claimed  {}", counts[2])];
		let lr: Vec<&str> = labels.iter().map(String::as_str).collect();
		tabs_with_hint(ui, &p, "atabs", &lr, &mut self.ach_tab, "Box: complete / uncheck (right-click a completed one — reward claimed). Slider — condition counter. Rewards for “completed” are claimed in game.");
		ui.add_space(6.0);
		let tab = self.ach_tab.min(2);
		let mut cats: Vec<(u32, String)> = Vec::new();
		for a in self.achs.iter().filter(|a| shown(a) && state(a.id) == tab) {
			if !cats.iter().any(|c| c.0 == a.cat) {
				cats.push((a.cat, a.cat_name.clone()));
			}
		}
		cats.sort_by_key(|c| c.0);
		let mut new_state: Option<(usize, u8)> = None;
		let mut new_counter: Option<(u32, u32)> = None;
		let mut fold: Option<String> = None;
		let list_h = ui.available_height().max(100.0);
		egui::ScrollArea::vertical().id_salt(("achs", tab)).max_height(list_h).auto_shrink([false, false]).show(ui, |ui| {
			if cats.is_empty() {
				ui.label(egui::RichText::new("Nothing here.").font(f_body(13.0)).color(p.muted));
			}
			for (cid, cname) in &cats {
				let key = format!("ach:{}:{cid}", g.dir);
				let open = !self.collapsed.contains(&key);
				let idx: Vec<usize> = (0..self.achs.len()).filter(|&i| self.achs[i].cat == *cid && shown(&self.achs[i]) && state(self.achs[i].id) == tab).collect();
				if ach_head(ui, &p, open, cname, idx.len()) {
					fold = Some(key);
				}
				ui.add_space(6.0);
				if !open {
					continue;
				}
				for i in idx {
					let a = &self.achs[i];
					let st = state(a.id);
					let rewards: Vec<(Option<Cell>, String, u32)> = a
						.rewards
						.iter()
						.map(|&(id, n)| {
							self.items.iter().find(|it| it.id == id).map_or((None, format!("#{id}"), n), |it| {
								let c = it.icon.or(((310..=317).contains(&id)).then(|| save::cells::SEPITH + (id - 310) as u32));
								(c.and_then(cell), it.en.clone(), n)
							})
						})
						.collect();
					let tip = match st {
						0 if save::fish_kind(g, a.kind) => "completed on the Fishing tab",
						0 if a.kind == mons_type => "completed by logging enemies on the Monsters tab",
						0 => "mark completed (claim the reward in game)",
						1 => "uncheck (right click — reward claimed)",
						_ => "back to Completed",
					};
					let act = ach_row(ui, &p, &AchRow { id: egui::Id::new(("ach", a.id, tab)), done: st > 0, name: &a.name, rewards: &rewards, counter: save::ach_counter(g, d, a.kind).map(|c| (c, a.goal)), slider: !linked(a.kind) && st == 0, tip, check: None });
					if act.clicked && !(st == 0 && linked(a.kind)) {
						new_state = Some((i, [1u8, 0, 1][st]));
					}
					if act.secondary && st == 1 {
						new_state = Some((i, 3));
					}
					if let Some(v) = act.counter {
						new_counter = Some((a.kind, v));
					}
					ui.add_space(5.0);
				}
				ui.add_space(8.0);
			}
		}).middle_scroll(ui);
		if let Some(key) = fold {
			if !self.collapsed.remove(&key) {
				self.collapsed.insert(key);
			}
			let mut v: Vec<&str> = self.collapsed.iter().filter(|k| k.starts_with("ach:")).map(String::as_str).collect();
			v.sort_unstable();
			let v = v.join(",");
			self.st.set("ach_folded", &v);
			self.save_settings();
		}
		let d = self.data.as_mut().unwrap();
		if let Some((kind, c)) = new_counter {
			save::set_ach_counter(g, d, kind, c);
			let mut got = 0;
			for a in self.achs.iter().filter(|a| a.kind == kind && c >= a.goal) {
				if save::ach_state(g, d, a.id) == 0 {
					save::set_ach_state(g, d, a.id, 1);
					got += 1;
				}
			}
			self.status = if got > 0 { format!("Condition counter: {c}; achievements completed: {got} — claim the rewards in game") } else { format!("Condition counter: {c}") };
		}
		if let Some((i, v)) = new_state {
			let a = &self.achs[i];
			save::set_ach_state(g, d, a.id, v);
			if v != 0 && let Some(c) = save::ach_counter(g, d, a.kind) && c < a.goal {
				save::set_ach_counter(g, d, a.kind, a.goal);
			}
			self.status = format!("“{}”: {}", a.name, ["not completed", "completed — claim the reward in game", "", "claimed"][v as usize]);
		}
	}

	fn inventory(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let g = self.g;
		let p = self.pal;
		let atlas = self.icons(ctx);
		let names: Vec<&str> = g.tabs.iter().copied().chain(["DLC"]).collect();
		let need: f32 = names.iter().map(|t| ui.fonts(|f| f.layout_no_wrap(t.to_string(), f_body(12.0), p.text).size().x) + 28.0).sum();
		let rows: Vec<&[&str]> = if need <= ui.available_width() { vec![&names[..]] } else { let (a, b) = names.split_at(names.len().div_ceil(2)); vec![a, b] };
		let mut start = 0;
		for (k, row) in rows.iter().enumerate() {
			let top = ui.max_rect().min + Vec2::new(0.0, k as f32 * 38.0);
			let mut r = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(top, Vec2::new(ui.max_rect().width(), 32.0))));
			let mut s = self.item_tab.checked_sub(start).filter(|&s| s < row.len()).unwrap_or(usize::MAX);
			if segmented(&mut r, &p, &format!("itab{k}"), row, &mut s, None, 32.0, 8, f_body(12.0), &[]) {
				self.item_tab = start + s;
			}
			start += row.len();
		}
		ui.add_space(42.0 + (rows.len() - 1) as f32 * 38.0);
		ui.horizontal(|ui| {
			ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search by name").desired_width(260.0));
			extra::watch(ui.ctx(), &mut self.search);
			ui.add_space(12.0);
			checkbox(ui, &p, &mut self.owned_only, "Owned only", 16.0, f_body(13.0));
		});
		ui.add_space(6.0);
		let dlc_tab = self.item_tab == g.tabs.len();
		let q = self.search.trim().to_lowercase();
		let d = self.data.as_mut().unwrap();
		if dlc_tab {
			let item_of: HashMap<usize, &save::Item> = self.items.iter().map(|it| (it.id, it)).collect();
			let contents = |ui: &mut egui::Ui, pack: &save::Dlc| {
				if pack.items.is_empty() {
					ui.label(&pack.en);
				}
				let total = pack.items.len();
				let cols = if total > 48 { 3 } else if total > 14 { 2 } else { 1 };
				let rows = total.div_ceil(cols);
				egui::Grid::new(("dlc_tip", pack.id)).num_columns(cols).spacing([18.0, 2.0]).show(ui, |ui| {
					for r in 0..rows {
						for c in 0..cols {
							let Some(&(id, n)) = pack.items.get(c * rows + r) else { continue };
							let it = item_of.get(&id);
							let icon = it.and_then(|it| it.icon).or(match id {
								310..=317 => Some(save::cells::SEPITH + (id - 310) as u32),
								_ if it.is_some_and(|it| it.cat == 12) => Some(49),
								_ => None,
							});
							ui.horizontal(|ui| {
								match icon {
									Some(c) => atlas_cell(ui, &atlas, Some(c), 20.0),
									None => {
										ui.add_space(20.0);
									}
								}
								ui.label(egui::RichText::new(format!("{} ×{n}", it.map_or("?", |it| it.en.as_str()))).font(f_body(13.5)));
							});
						}
						ui.end_row();
					}
				});
			};
			let cols = ((ui.available_width() / 300.0).floor() as usize).max(1);
			let cell_w = ui.available_width() / cols as f32;
			let packs: Vec<u32> = self
				.dlcs
				.iter()
				.filter(|pk| q.is_empty() || pk.en.to_lowercase().contains(&q) || pk.ru.to_lowercase().contains(&q))
				.filter(|pk| !self.owned_only || save::dlc_opened(g, d, pk.id))
				.map(|pk| pk.id)
				.collect();
			ui.horizontal(|ui| {
				ui.label(egui::RichText::new("Set value:").font(f_body(13.0)).color(p.muted));
				for n in [0u32, 1] {
					if small_button(ui, &p, &format!("All “{n}”"), None, 26.0, !packs.is_empty(), p.text).clicked() {
						for &id in &packs {
							save::set_dlc_opened(g, d, id, n == 1);
						}
					}
				}
			});
			ui.add_space(6.0);
			egui::ScrollArea::vertical().id_salt(("dlc", g.dir)).auto_shrink([false, false]).show(ui, |ui| {
				for row in packs.chunks(cols) {
					ui.horizontal(|ui| {
						for &id in row {
							let Some(pack) = self.dlcs.iter().find(|pk| pk.id == id) else { continue };
							let (rect, _) = ui.allocate_exact_size(Vec2::new(cell_w - 8.0, 30.0), Sense::hover());
							ui.painter().rect_filled(rect, 6, if p.dark { p.surface } else { Color32::from_black_alpha(14) });
							{
								let ir = Rect::from_min_size(Pos2::new(rect.min.x + 6.0, rect.center().y - 12.0), Vec2::splat(24.0));
								if let Some(t) = &atlas {
									ui.painter().image(t.id(), ir, atlas_uv(t, 6), Color32::WHITE);
								}
								let field = Rect::from_min_max(Pos2::new(rect.max.x - 46.0, rect.min.y), rect.max).shrink(1.0);
								let nr = Rect::from_min_max(Pos2::new(ir.max.x + 8.0, rect.min.y), Pos2::new(field.min.x - 8.0, rect.max.y));
								let mut nc = ui.new_child(egui::UiBuilder::new().max_rect(nr).layout(egui::Layout::left_to_right(egui::Align::Center)));
								let r = nc.add(egui::Label::new(egui::RichText::new(&pack.en).font(f_body(13.5)).color(p.text)).truncate().sense(Sense::click()));
								click_tip(&nc, &r, ("pack", id), |ui| contents(ui, pack));
								let mut n = u32::from(save::dlc_opened(g, d, id));
								let mut fc = ui.new_child(egui::UiBuilder::new().max_rect(field));
								{
									let v = &mut fc.style_mut().visuals.widgets;
									for w in [&mut v.inactive, &mut v.hovered, &mut v.active, &mut v.open] {
										w.bg_stroke = Stroke::NONE;
										w.corner_radius = egui::CornerRadius { nw: 0, sw: 0, ne: 5, se: 5 };
										w.expansion = 0.0;
									}
								}
								if fc.add_sized(field.size(), egui::DragValue::new(&mut n).clamp_existing_to_range(false).range(0..=1).speed(0.0)).on_hover_text("1 — set received (box opened), 0 — not received (if the DLC is installed in Steam, the game will give the box)").changed() {
									save::set_dlc_opened(g, d, id, n == 1);
								}
								ui.painter().line_segment([Pos2::new(field.min.x, rect.min.y), Pos2::new(field.min.x, rect.max.y)], Stroke::new(1.0_f32, p.div));
								ui.painter().rect_stroke(rect, 6, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
							}
						}
					});
				}
				ui.add_space(10.0);
				ui.label(egui::RichText::new("Note: without the downloadable content (DLC), character skins and items will not appear in the game. They can be unlocked separately in the «Key» tab.").font(f_body(13.0)).color(p.muted));
			}).middle_scroll(ui);
			return;
		}
		let shown = |it: &save::Item, d: &[u8]| it.id <= g.item_max_id && (q.is_empty() || it.en.to_lowercase().contains(&q) || it.ru.to_lowercase().contains(&q)) && (!self.owned_only || save::item_count(g, d, it.id) > 0);
		let icon_of = |it: &save::Item| it.icon.or(if it.cat == 12 { Some(49) } else { None });
		let tab = self.item_tab;
		let weapons = g.tabs[tab] == "Weapons";
		let rank = |kind: u8| match self.kinds.get(&kind).map(|s| s.as_str()) {
			Some("Recipes") => -2,
			Some("Event Items") if g.dir == "Daybreak-1" => -1,
			_ => kind as i32,
		};
		type Group<'a> = (String, String, Option<u32>, Vec<&'a save::Item>, i32);
		let mut groups: Vec<Group> = Vec::new();
		let mut list: Vec<&save::Item> = self.items.iter().filter(|it| save::item_tab(g, it) == Some(tab) && shown(it, d) && (self.show_service || !it.service())).collect();
		if weapons {
			list.sort_by_key(|it| it.id);
		}
		for it in list {
			let (key, title, icon, place) = if weapons && it.cat == 12 {
				let who = self.names.get(&it.owner).map(|n| n.0.clone()).unwrap_or_else(|| format!("#{}", it.owner));
				(format!("{}:w{}", g.dir, it.owner), who, icon_of(it), 0)
			} else if weapons {
				(format!("{}:sub", g.dir), "Sub Parts".to_string(), Some(21), 1)
			} else {
				let title = self.kinds.get(&it.kind).cloned().unwrap_or_else(|| format!("Kind {}", it.kind));
				(format!("{}:{}", g.dir, it.kind), title, None, rank(it.kind))
			};
			match groups.iter_mut().find(|gr| gr.0 == key) {
				Some(gr) => gr.3.push(it),
				None => groups.push((key, title, icon, vec![it], place)),
			}
		}
		groups.sort_by_key(|gr| gr.4);
		let mut set_all = None;
		ui.horizontal(|ui| {
			ui.label(egui::RichText::new("Set value:").font(f_body(13.0)).color(p.muted));
			let vals: &[u32] = if g.tabs[tab] == "Key" { &[0, 1] } else { &[0, 1, 99] };
			for &n in vals {
				if small_button(ui, &p, &format!("All “{n}”"), None, 26.0, !groups.is_empty(), p.text).clicked() {
					set_all = Some(n);
				}
			}
		});
		if let Some(n) = set_all {
			for gr in &groups {
				for it in &gr.3 {
					save::set_item_count(g, d, it.id, n.min(save::item_limit(g, it)));
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
			if fc.add_sized(field.size(), egui::DragValue::new(&mut n).clamp_existing_to_range(false).range(0..=save::item_limit(g, it)).speed(0.0)).changed() {
				save::set_item_count(g, d, it.id, n);
			}
			ui.painter().line_segment([Pos2::new(field.min.x, rect.min.y), Pos2::new(field.min.x, rect.max.y)], Stroke::new(1.0_f32, p.div));
			ui.painter().rect_stroke(rect, 6, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
			let inner = rect.shrink2(Vec2::new(6.0, 0.0));
			let icon_r = Rect::from_min_size(Pos2::new(inner.min.x, inner.center().y - 12.0), Vec2::splat(24.0));
			if let (Some(t), Some(cell)) = (&atlas, icon_of(it)) {
				egui::Image::new(t).uv(atlas_uv(t, cell)).paint_at(ui, icon_r);
			}
			let name_r = Rect::from_min_max(Pos2::new(icon_r.max.x + 8.0, inner.min.y), Pos2::new(field.min.x - 8.0, inner.max.y));
			let mut nc = ui.new_child(egui::UiBuilder::new().max_rect(name_r).layout(egui::Layout::left_to_right(egui::Align::Center)));
			let r = nc.add(egui::Label::new(egui::RichText::new(&it.en).font(f_num(13.5)).color(p.text)).truncate().sense(Sense::click()));
			click_tip(&nc, &r, ("item", it.id), |ui| item_tip(ui, &p, it, &atlas));
		};
		let headers = groups.len() > 1;
		egui::ScrollArea::vertical().id_salt(("items", self.item_tab, g.dir)).auto_shrink([false, false]).show(ui, |ui| {
			for (key, title, gicon, its, _) in &groups {
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

fn stats_line(stats: &[i32; 16]) -> String {
	save::ITEM_STATS.iter().zip(stats).filter(|(_, v)| **v != 0).map(|((n, _), v)| format!("{n} {}{}", if *v > 0 { "+" } else { "−" }, v.abs())).collect::<Vec<_>>().join(", ")
}

fn item_tip(ui: &mut egui::Ui, p: &Pal, it: &save::Item, atlas: &Option<egui::TextureHandle>) {
	ui.set_max_width(380.0);
	ui.label(egui::RichText::new(&it.en).font(f_semi(13.5)).color(p.text));
	for line in it.desc_en.trim().lines() {
		ui.horizontal_wrapped(|ui| {
			ui.spacing_mut().item_spacing.x = 0.0;
			let mut rest = line;
			while !rest.is_empty() {
				let (text, icon, tail) = match rest.find("<I").and_then(|a| rest[a..].find('>').map(|b| (a, a + b))) {
					Some((a, b)) => (&rest[..a], rest[a + 2..b].parse::<u32>().ok(), &rest[b + 1..]),
					None => (rest, None, ""),
				};
				if !text.is_empty() {
					ui.label(egui::RichText::new(text).font(f_body(12.5)).color(p.text));
				}
				if icon.is_some() {
					atlas_cell(ui, atlas, icon, 16.0);
				}
				rest = tail;
			}
		});
	}
	let st = stats_line(&it.stats);
	let flat: String = it.desc_en.chars().filter(|c| !c.is_whitespace()).collect();
	let in_desc = st.split(", ").all(|s| flat.contains(&s.replace(' ', "")));
	if !st.is_empty() && !in_desc {
		ui.label(egui::RichText::new(st).font(f_semi(12.5)).color(p.text));
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
	match image::load_from_memory(include_bytes!("icons/calvard_save.png")) {
		Ok(im) => {
			let im = im.to_rgba8();
			egui::IconData { width: im.width(), height: im.height(), rgba: im.into_raw() }
		}
		Err(_) => window_icon(),
	}
}

fn main() {
	kiseki_tool::crash::install("Calvard-SaveEditor", env!("VER_CALVARDSAVEEDITOR"));
	let opts = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_title(TITLE).with_inner_size([1360.0, 780.0]).with_min_inner_size([960.0, 560.0]).with_icon(editor_icon()),
		centered: true,
		..Default::default()
	};
	let _ = eframe::run_native(TITLE, opts, Box::new(|cc| Ok(Box::new(App::new(cc)))));
}

