#![allow(dead_code)]

use eframe::egui::{self, Align2, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn blurred_texture(ctx: &egui::Context, name: &str, png: &[u8]) -> Option<egui::TextureHandle> {
	let img = image::load_from_memory(png).ok()?.to_rgba8();
	let w = 240u32.min(img.width());
	let h = (img.height() * w / img.width().max(1)).max(1);
	let small = image::imageops::resize(&img, w, h, image::imageops::FilterType::Triangle);
	let blur = image::imageops::blur(&small, 2.5);
	Some(ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied([blur.width() as usize, blur.height() as usize], blur.as_raw()), egui::TextureOptions::LINEAR))
}

pub fn picker_backdrops(ui: &Ui, rect: Rect, layers: &[(&egui::TextureHandle, f32)], bg: Color32) {
	let mut top = 0.0f32;
	let mut order: Vec<&(&egui::TextureHandle, f32)> = layers.iter().filter(|l| l.1 > 0.0).collect();
	order.sort_by(|a, b| a.1.total_cmp(&b.1));
	for (tex, t) in order {
		picker_image(ui, rect, tex, *t);
		top = top.max(*t);
	}
	if top > 0.0 {
		let light = bg.r() as u32 + bg.g() as u32 + bg.b() as u32 > 3 * 128;
		let veil = if light { 40.0 } else { 160.0 };
		ui.painter().rect_filled(rect, 0, Color32::from_rgba_unmultiplied(bg.r(), bg.g(), bg.b(), (top * veil) as u8));
	}
}

pub fn picker_bg(ui: &Ui, p: &Pal, full: Rect, covers: &[Option<egui::TextureHandle>], hovered: Option<usize>) -> (Rect, Rect) {
	let fades: Vec<f32> = (0..covers.len()).map(|i| ui.ctx().animate_bool_with_time(egui::Id::new(("picker_bg", i)), hovered == Some(i) && covers[i].is_some(), 0.4)).collect();
	let layers: Vec<(&egui::TextureHandle, f32)> = covers.iter().zip(&fades).filter_map(|(t, f)| t.as_ref().map(|t| (t, *f))).collect();
	let fade = fades.iter().copied().fold(0.0f32, f32::max);
	let head = Rect::from_min_size(full.min, Vec2::new(full.width(), 56.0));
	let foot = Rect::from_min_max(Pos2::new(full.min.x, full.max.y - 72.0), full.max);
	let tint = |c: Color32, a: f32| Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a as u8);
	if fade > 0.0 {
		picker_backdrops(ui, full, &layers, p.bg);
		ui.painter().rect_filled(head, 0, tint(p.bg, fade * if p.dark { 200.0 } else { 110.0 }));
	}
	ui.painter().rect_filled(foot, 0, tint(p.surface, if p.dark { 255.0 } else { 255.0 - fade * 100.0 }));
	for y in [head.max.y, foot.min.y] {
		ui.painter().line_segment([Pos2::new(full.min.x, y), Pos2::new(full.max.x, y)], Stroke::new(1.0_f32, p.div));
	}
	(head, foot)
}

pub struct PickCard<'a> {
	pub cover: Option<&'a egui::TextureHandle>,
	pub arc: &'a str,
	pub title: &'a str,
	pub color: Color32,
	pub selected: bool,
	pub ready: bool,
}

pub fn picker_card(ui: &Ui, p: &Pal, r: Rect, art_h: f32, c: &PickCard, hovered: bool) {
	let hovered = hovered && c.ready;
	let pt = ui.painter();
	pt.rect_filled(r, 12, if hovered { p.div2 } else { p.surface });
	let ar = Rect::from_min_size(r.min, Vec2::new(r.width(), art_h));
	let top = egui::CornerRadius { nw: 12, ne: 12, sw: 0, se: 0 };
	match c.cover {
		Some(t) => {
			let ts = t.size_vec2();
			let (want, have) = (ar.width() / art_h, ts.x / ts.y.max(1.0));
			let uv = if have < want {
				let h = have / want;
				let y0 = ((1.0 - h) * 0.6).clamp(0.0, 1.0 - h);
				Rect::from_min_max(Pos2::new(0.0, y0), Pos2::new(1.0, y0 + h))
			} else {
				let w = want / have;
				Rect::from_min_max(Pos2::new((1.0 - w) / 2.0, 0.0), Pos2::new((1.0 + w) / 2.0, 1.0))
			};
			egui::Image::new(t).uv(uv).corner_radius(top).paint_at(ui, ar);
		}
		None => {
			pt.rect_filled(ar, top, p.log);
			if !c.ready {
				pt.text(ar.center(), Align2::CENTER_CENTER, "Not released yet", f_semi(16.0), p.muted);
			}
		}
	}
	let tx = r.min.x + 20.0;
	pt.text(Pos2::new(tx, ar.max.y + 18.0), Align2::LEFT_TOP, c.arc, f_semi(11.0), if !c.ready { p.muted } else if p.dark { c.color } else { over(p.text, c.color, 0.4) });
	pt.text(Pos2::new(tx, ar.max.y + 36.0), Align2::LEFT_TOP, c.title, f_semi(19.0), if c.ready { p.text } else { p.muted });
	let pill = if !c.ready {
		"Not available yet"
	} else if c.selected {
		"Selected"
	} else {
		"Open saves"
	};
	let pw = ui.fonts(|f| f.layout_no_wrap(pill.to_string(), f_body(12.5), p.text).size().x) + 28.0;
	let pr = Rect::from_min_size(Pos2::new(tx, r.max.y - 38.0), Vec2::new(pw, 26.0));
	let on = c.selected && c.ready;
	pt.rect(pr, 13, if on { p.sel } else { p.bg }, Stroke::new(1.0_f32, if on { c.color } else { p.div }), StrokeKind::Inside);
	pt.text(pr.center(), Align2::CENTER_CENTER, pill, f_body(12.5), if c.ready { p.text } else { p.muted });
	let stroke = if on || hovered { Stroke::new(2.0_f32, c.color) } else { Stroke::new(1.0_f32, p.div) };
	pt.rect_stroke(r, 12, stroke, StrokeKind::Inside);
}

fn picker_image(ui: &Ui, rect: Rect, tex: &egui::TextureHandle, t: f32) {
	let ts = tex.size_vec2();
	let (want, have) = (rect.width() / rect.height().max(1.0), ts.x / ts.y.max(1.0));
	let uv = if have < want {
		let h = have / want;
		Rect::from_min_max(Pos2::new(0.0, (1.0 - h) / 2.0), Pos2::new(1.0, (1.0 + h) / 2.0))
	} else {
		let w = want / have;
		Rect::from_min_max(Pos2::new((1.0 - w) / 2.0, 0.0), Pos2::new((1.0 + w) / 2.0, 1.0))
	};
	let a = (t * 255.0) as u8;
	ui.painter().image(tex.id(), rect, uv, Color32::from_rgba_unmultiplied(255, 255, 255, a));
}

pub struct Settings {
	map: BTreeMap<String, String>,
	path: PathBuf,
	title: &'static str,
}

impl Settings {
	pub fn load(file: &str, title: &'static str) -> Settings {
		let path = exe_dir().join(file);
		let mut map = BTreeMap::new();
		if let Ok(s) = std::fs::read_to_string(&path) {
			for l in s.trim_start_matches('\u{feff}').lines() {
				if let Some((k, v)) = l.split_once('=') {
					map.insert(k.trim().to_string(), v.trim().to_string());
				}
			}
		}
		Settings { map, path, title }
	}
	pub fn get(&self, k: &str) -> Option<&str> {
		self.map.get(k).map(|s| s.as_str())
	}
	pub fn get_or<'a>(&'a self, k: &str, d: &'a str) -> &'a str {
		self.get(k).unwrap_or(d)
	}
	pub fn set(&mut self, k: &str, v: &str) {
		self.map.insert(k.to_string(), v.to_string());
	}
	pub fn save(&self) {
		let mut s = format!("; settings {}\r\n", self.title);
		for (k, v) in &self.map {
			s.push_str(&format!("{k}={v}\r\n"));
		}
		let _ = std::fs::write(&self.path, s);
	}
}

pub fn exe_dir() -> PathBuf {
	std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default()
}

#[derive(Clone, Copy)]
pub struct Pal {
	pub dark: bool,
	pub bg: Color32,
	pub surface: Color32,
	pub log: Color32,
	pub text: Color32,
	pub muted: Color32,
	pub div: Color32,
	pub div2: Color32,
	pub accent: Color32,
	pub accent_strong: Color32,
	pub on_accent: Color32,
	pub sel: Color32,
	pub hit: Color32,
	pub dict: Color32,
	pub ok: Color32,
	pub ok_strong: Color32,
	pub warn: Color32,
	pub caution: Color32,
	pub gold: Color32,
}

fn hex(v: u32) -> Color32 {
	Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

fn over(fg: Color32, bg: Color32, a: f32) -> Color32 {
	let m = |f: u8, b: u8| (f as f32 * a + b as f32 * (1.0 - a)).round() as u8;
	Color32::from_rgb(m(fg.r(), bg.r()), m(fg.g(), bg.g()), m(fg.b(), bg.b()))
}

impl Pal {
	pub fn light() -> Pal {
		let bg = hex(0xf2f2f3);
		let text = hex(0x1d1f20);
		Pal {
			dark: false,
			bg,
			surface: hex(0xe9e9ea),
			log: hex(0xebebec),
			text,
			muted: over(text, bg, 0.6),
			div: over(text, bg, 0.16),
			div2: over(text, bg, 0.08),
			accent: hex(0x5980a6),
			accent_strong: hex(0x416180),
			on_accent: hex(0xf2f2f3),
			sel: hex(0xd6ebff),
			hit: hex(0xb5d9fd),
			dict: hex(0xf9d68c),
			ok: hex(0xb3e4bd),
			ok_strong: hex(0x3f8a55),
			warn: hex(0xc0502e),
			caution: hex(0xb0600c),
			gold: hex(0x946800),
		}
	}
	pub fn graphite() -> Pal {
		let bg = hex(0x2a2c2f);
		let text = hex(0xd6d8da);
		Pal {
			dark: true,
			bg,
			surface: hex(0x323538),
			log: hex(0x26282b),
			text,
			muted: over(text, bg, 0.6),
			div: over(text, bg, 0.14),
			div2: over(text, bg, 0.07),
			accent: hex(0x8fb0d0),
			accent_strong: hex(0xaac6e2),
			on_accent: hex(0x1f2124),
			sel: hex(0x3a4a5c),
			hit: hex(0x3f5870),
			dict: hex(0xa07e45),
			ok: hex(0x2f4134),
			ok_strong: hex(0x8fd3a0),
			warn: hex(0xe8916f),
			caution: hex(0xe69646),
			gold: hex(0xe6be5a),
		}
	}
	pub fn get(dark: bool) -> Pal {
		if dark { Pal::graphite() } else { Pal::light() }
	}
}

pub fn apply_visuals(ctx: &egui::Context, p: &Pal) {
	let mut v = if p.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
	v.override_text_color = Some(p.text);
	v.panel_fill = p.bg;
	v.window_fill = p.bg;
	v.extreme_bg_color = p.surface;
	v.faint_bg_color = p.div2;
	v.window_stroke = Stroke::new(1.0_f32, p.div);
	v.window_corner_radius = CornerRadius::same(10);
	v.menu_corner_radius = CornerRadius::same(10);
	v.selection.bg_fill = p.hit;
	v.selection.stroke = Stroke::new(1.0_f32, p.text);
	v.hyperlink_color = p.accent;
	v.text_cursor.stroke = Stroke::new(1.5_f32, p.text);
	for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
		w.corner_radius = CornerRadius::same(7);
		w.fg_stroke = Stroke::new(1.0_f32, p.text);
	}
	v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.div);
	v.widgets.noninteractive.bg_fill = p.bg;
	v.widgets.inactive.bg_fill = p.surface;
	v.widgets.inactive.weak_bg_fill = p.surface;
	v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.div);
	v.widgets.hovered.bg_fill = p.div2;
	v.widgets.hovered.weak_bg_fill = p.sel;
	v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.div);
	v.widgets.active.bg_fill = p.sel;
	v.widgets.active.weak_bg_fill = p.sel;
	v.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
	v.widgets.open.bg_fill = p.surface;
	v.widgets.open.weak_bg_fill = p.surface;
	v.widgets.open.bg_stroke = Stroke::new(1.0_f32, p.accent);
	v.popup_shadow = egui::epaint::Shadow { offset: [0, 12], blur: 32, spread: 0, color: Color32::from_black_alpha(if p.dark { 90 } else { 56 }) };
	v.window_shadow = v.popup_shadow;
	ctx.set_visuals(v);
	ctx.style_mut(|s| {
		s.spacing.item_spacing = Vec2::new(8.0, 6.0);
		s.spacing.button_padding = Vec2::new(10.0, 4.0);
		s.spacing.interact_size.y = 26.0;
		s.interaction.selectable_labels = false;
		s.spacing.scroll = egui::style::ScrollStyle::solid();
		s.text_styles.insert(egui::TextStyle::Body, f_body(14.0));
		s.text_styles.insert(egui::TextStyle::Button, FontId::new(13.0, FontFamily::Name("num".into())));
		s.text_styles.insert(egui::TextStyle::Small, f_body(11.5));
		s.text_styles.insert(egui::TextStyle::Monospace, f_mono(12.5));
		s.text_styles.insert(egui::TextStyle::Heading, f_semi(19.0));
	});
}

pub fn setup_fonts(ctx: &egui::Context) {
	let read = |names: &[&str]| -> Option<Vec<u8>> {
		let win = std::env::var("WINDIR").map(|w| PathBuf::from(w).join("Fonts")).unwrap_or_else(|_| PathBuf::from("C:\\Windows\\Fonts"));
		for n in names {
			let cands = [win.join(n), PathBuf::from(n)];
			for c in cands {
				if let Ok(b) = std::fs::read(&c) {
					return Some(b);
				}
			}
		}
		None
	};
	let mut fd = egui::FontDefinitions::default();
	let mut add = |key: &str, bytes: Option<Vec<u8>>| -> bool {
		match bytes {
			Some(b) => {
				let mut d = egui::FontData::from_owned(b);
				if key == "cjk" {
					d.tweak.y_offset_factor = 0.21;
				}
				if key == "num" {
					d.tweak.y_offset_factor = NUM_Y;
				}
				fd.font_data.insert(key.to_string(), std::sync::Arc::new(d));
				true
			}
			None => false,
		}
	};
	let lin = |f: &str| format!("/usr/share/fonts/truetype/{f}");
	let zlin = |f: &str| format!("Z:\\usr\\share\\fonts\\truetype\\{}", f.replace('/', "\\"));
	let body = add("body", read(&["segoeui.ttf", &lin("liberation/LiberationSans-Regular.ttf"), &zlin("liberation/LiberationSans-Regular.ttf"), &lin("dejavu/DejaVuSans.ttf")]));
	let semi = add("semi", read(&["seguisb.ttf", "segoeuib.ttf", &lin("liberation/LiberationSans-Bold.ttf"), &zlin("liberation/LiberationSans-Bold.ttf"), &lin("dejavu/DejaVuSans-Bold.ttf")]));
	let num = add("num", read(&["segoeui.ttf", &lin("liberation/LiberationSans-Regular.ttf"), &zlin("liberation/LiberationSans-Regular.ttf"), &lin("dejavu/DejaVuSans.ttf")]));
	let mono = add("mono", read(&["consola.ttf", &lin("dejavu/DejaVuSansMono.ttf"), &zlin("dejavu/DejaVuSansMono.ttf")]));
	let sym = add("sym", read(&["seguisym.ttf", &lin("dejavu/DejaVuSans.ttf"), &zlin("dejavu/DejaVuSans.ttf")]));
	let cjk = add("cjk", read(&["YuGothR.ttc", "meiryo.ttc", "msgothic.ttc", "/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf", r"Z:\usr\share\fonts\opentype\ipafont-gothic\ipag.ttf"]));
	let zh = add("zh", read(&["msyh.ttc", "simsun.ttc", "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", r"Z:\usr\share\fonts\opentype\noto\NotoSansCJK-Regular.ttc"]));
	let ko = add("ko", read(&["malgun.ttf", "gulim.ttc", "/usr/share/fonts/truetype/nanum/NanumGothic.ttf", r"Z:\usr\share\fonts\truetype\nanum\NanumGothic.ttf"]));
	let prop = fd.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
	let monos = fd.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
	let mut body_fam = Vec::new();
	if body {
		body_fam.push("body".to_string());
	}
	if sym {
		body_fam.push("sym".to_string());
	}
	if cjk {
		body_fam.push("cjk".to_string());
	}
	if zh {
		body_fam.push("zh".to_string());
	}
	if ko {
		body_fam.push("ko".to_string());
	}
	body_fam.extend(prop.iter().cloned());
	let mut semi_fam = Vec::new();
	if semi {
		semi_fam.push("semi".to_string());
	}
	semi_fam.extend(body_fam.iter().cloned());
	let mut mono_fam = Vec::new();
	if mono {
		mono_fam.push("mono".to_string());
	}
	mono_fam.extend(monos.iter().cloned());
	mono_fam.extend(body_fam.iter().cloned());
	fd.families.insert(FontFamily::Proportional, body_fam);
	fd.families.insert(FontFamily::Name("semi".into()), semi_fam);
	fd.families.insert(FontFamily::Monospace, mono_fam);
	let mut num_fam = Vec::new();
	if num {
		num_fam.push("num".to_string());
	}
	num_fam.extend(fd.families.get(&FontFamily::Proportional).cloned().unwrap_or_default());
	fd.families.insert(FontFamily::Name("num".into()), num_fam);
	ctx.set_fonts(fd);
}

pub fn number_cursor(ctx: &egui::Context) {
	ctx.on_end_pass("number_cursor", std::sync::Arc::new(|ctx: &egui::Context| {
		ctx.output_mut(|o| {
			if o.cursor_icon == egui::CursorIcon::ResizeHorizontal {
				o.cursor_icon = egui::CursorIcon::PointingHand;
			}
		});
	}));
}

pub const NUM_Y: f32 = -0.10;

pub fn f_body(size: f32) -> FontId {
	FontId::new(size, FontFamily::Proportional)
}
pub fn f_num(size: f32) -> FontId {
	FontId::new(size, FontFamily::Name("num".into()))
}
pub fn f_semi(size: f32) -> FontId {
	FontId::new(size, FontFamily::Name("semi".into()))
}
pub fn f_mono(size: f32) -> FontId {
	FontId::new(size, FontFamily::Monospace)
}

pub fn icon(name: &str) -> egui::ImageSource<'static> {
	let (uri, bytes): (&'static str, &'static [u8]) = match name {
		"archive" => ("bytes://archive.svg", include_bytes!("icons/archive.svg")),
		"package" => ("bytes://package.svg", include_bytes!("icons/package.svg")),
		"folder" => ("bytes://folder.svg", include_bytes!("icons/folder.svg")),
		"moon" => ("bytes://moon.svg", include_bytes!("icons/moon.svg")),
		"sun" => ("bytes://sun.svg", include_bytes!("icons/sun.svg")),
		"search" => ("bytes://search.svg", include_bytes!("icons/search.svg")),
		"folder-open" => ("bytes://folder-open.svg", include_bytes!("icons/folder-open.svg")),
		"download" => ("bytes://download.svg", include_bytes!("icons/download.svg")),
		"save" => ("bytes://save.svg", include_bytes!("icons/save.svg")),
		"file-plus" => ("bytes://file-plus.svg", include_bytes!("icons/file-plus.svg")),
		"file" => ("bytes://file.svg", include_bytes!("icons/file.svg")),
		"sheet" => ("bytes://sheet.svg", include_bytes!("icons/sheet.svg")),
		"undo" => ("bytes://undo.svg", include_bytes!("icons/undo.svg")),
		"stop" => ("bytes://stop.svg", include_bytes!("icons/stop.svg")),
		"pause" => ("bytes://pause.svg", include_bytes!("icons/pause.svg")),
		"play" => ("bytes://play.svg", include_bytes!("icons/play.svg")),
		"x" => ("bytes://x.svg", include_bytes!("icons/x.svg")),
		"refresh" => ("bytes://refresh.svg", include_bytes!("icons/refresh.svg")),
		"info" => ("bytes://info.svg", include_bytes!("icons/info.svg")),
		"help" => ("bytes://help.svg", include_bytes!("icons/help.svg")),
		"languages" => ("bytes://languages.svg", include_bytes!("icons/languages.svg")),
		"arrow-left" => ("bytes://arrow-left.svg", include_bytes!("icons/arrow-left.svg")),
		"arrow-right" => ("bytes://arrow-right.svg", include_bytes!("icons/arrow-right.svg")),
		"switch" => ("bytes://switch.svg", include_bytes!("icons/switch.svg")),
		"heart" => ("bytes://heart.svg", include_bytes!("icons/heart.svg")),
		"list-plus" => ("bytes://list-plus.svg", include_bytes!("icons/list-plus.svg")),
		"chevron-down" => ("bytes://chevron-down.svg", include_bytes!("icons/chevron-down.svg")),
		"chevron-up" => ("bytes://chevron-up.svg", include_bytes!("icons/chevron-up.svg")),
		"eye-off" => ("bytes://eye-off.svg", include_bytes!("icons/eye-off.svg")),
		"external" => ("bytes://external.svg", include_bytes!("icons/external.svg")),
		"rotate-ccw" => ("bytes://rotate-ccw.svg", include_bytes!("icons/rotate-ccw.svg")),
		"eraser" => ("bytes://eraser.svg", include_bytes!("icons/eraser.svg")),
		"filter" => ("bytes://filter.svg", include_bytes!("icons/filter.svg")),
		"trash" => ("bytes://trash.svg", include_bytes!("icons/trash.svg")),
		"gamepad" => ("bytes://gamepad.svg", include_bytes!("icons/gamepad.svg")),
		"book" => ("bytes://book.svg", include_bytes!("icons/book.svg")),
		"columns" => ("bytes://columns.svg", include_bytes!("icons/columns.svg")),
		"wrench" => ("bytes://wrench.svg", include_bytes!("icons/wrench.svg")),
		_ => ("bytes://check.svg", include_bytes!("icons/check.svg")),
	};
	egui::ImageSource::Bytes { uri: uri.into(), bytes: egui::load::Bytes::Static(bytes) }
}

pub fn paint_icon(ui: &Ui, name: &str, rect: Rect, color: Color32) {
	let ppp = ui.ctx().pixels_per_point();
	let side = (rect.width().min(rect.height()) * ppp).round().max(1.0);
	let c = rect.center() * ppp;
	let min = Pos2::new((c.x - side / 2.0).round(), (c.y - side / 2.0).round());
	let rect = Rect::from_min_size(Pos2::new(min.x / ppp, min.y / ppp), Vec2::splat(side / ppp));
	let tex = icon_texture(ui.ctx(), name, side as u32);
	ui.painter().image(tex.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), color);
}

fn icon_texture(ctx: &egui::Context, name: &str, px: u32) -> egui::TextureHandle {
	let id = egui::Id::new(("icon_tex", name, px));
	if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
		return t;
	}
	let bytes: &[u8] = match icon(name) {
		egui::ImageSource::Bytes { bytes: egui::load::Bytes::Static(b), .. } => b,
		_ => &[],
	};
	let img = egui_extras::image::load_svg_bytes_with_size(bytes, Some(egui::load::SizeHint::Size(px, px))).unwrap_or_else(|_| egui::ColorImage::new([1, 1], Color32::TRANSPARENT));
	let t = ctx.load_texture(format!("icon:{name}:{px}"), img, egui::TextureOptions::LINEAR);
	ctx.data_mut(|d| d.insert_temp(id, t.clone()));
	t
}

pub fn caps_job(text: &str, color: Color32) -> egui::text::LayoutJob {
	let mut job = egui::text::LayoutJob::default();
	job.append(&text.to_uppercase(), 0.0, egui::TextFormat { font_id: f_body(11.0), color, extra_letter_spacing: 0.9, ..Default::default() });
	job
}

pub fn paint_text_clipped(ui: &Ui, rect: Rect, text: &str, font: FontId, color: Color32, align: Align2) {
	let mut job = egui::text::LayoutJob::single_section(text.to_string(), egui::TextFormat { font_id: font, color, ..Default::default() });
	job.wrap = egui::text::TextWrapping { max_width: rect.width().max(1.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
	let g = ui.fonts(|f| f.layout_job(job));
	let pos = align.align_size_within_rect(g.size(), rect).min;
	ui.painter().with_clip_rect(rect).galley(pos, g, color);
}

pub fn path_input(ui: &mut Ui, p: &Pal, id: &str, value: &mut String, size: Vec2, hint: &str, enabled: bool) -> bool {
	let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
	ui.painter().rect(rect, 8, p.surface, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
	let trect = rect.shrink2(Vec2::new(12.0, 0.0));
	let mut child = ui.new_child(egui::UiBuilder::new().max_rect(trect).layout(egui::Layout::left_to_right(egui::Align::Center)));
	let te = egui::TextEdit::singleline(value)
		.id(ui.id().with(id))
		.frame(false)
		.font(f_body(13.0))
		.hint_text(egui::RichText::new(hint).color(p.muted))
		.text_color(p.text)
		.desired_width(trect.width())
		.interactive(enabled)
		.margin(egui::Margin::ZERO);
	child.add(te).changed()
}

pub struct UsedSlot {
	pub title: String,
	pub when: String,
	pub locked: bool,
}

pub struct SlotPick {
	pub num: u32,
	first: Option<u32>,
	range: (u32, u32),
	used: BTreeMap<u32, UsedSlot>,
	scroll: bool,
}

impl SlotPick {
	pub fn new(range: (u32, u32), used: BTreeMap<u32, UsedSlot>) -> SlotPick {
		let first = (range.0..=range.1).find(|n| !used.contains_key(n));
		SlotPick { num: first.unwrap_or(range.0), first, range, used, scroll: true }
	}

	pub fn taken(&self, n: u32) -> bool {
		self.used.contains_key(&n)
	}

	fn locked(&self, n: u32) -> bool {
		self.used.get(&n).is_some_and(|u| u.locked)
	}
}

pub fn slot_pick_window(ctx: &egui::Context, p: &Pal, s: &mut SlotPick, backup_note: &str) -> Option<Option<u32>> {
	let mut out = None;
	let (warn, green) = (p.caution, p.ok_strong);
	let screen = ctx.screen_rect();
	egui::Area::new(egui::Id::new("slot_dim")).order(egui::Order::Middle).fixed_pos(screen.min).show(ctx, |ui| {
		let (r, _) = ui.allocate_exact_size(screen.size(), Sense::click());
		ui.painter().rect_filled(r, 0, Color32::from_black_alpha(if p.dark { 120 } else { 70 }));
	});
	egui::Area::new(egui::Id::new("slot_box")).order(egui::Order::Foreground).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
		egui::Frame::new().fill(p.bg).stroke(Stroke::new(1.0_f32, p.div)).corner_radius(12).inner_margin(egui::Margin::same(20)).show(ui, |ui| {
			let w = (screen.width() - 120.0).clamp(420.0, 660.0);
			ui.set_width(w);
			ui.label(egui::RichText::new("Save to a slot").font(f_semi(18.0)).color(p.text));
			ui.add_space(8.0);
			ui.horizontal(|ui| {
				ui.label(egui::RichText::new("Slot number").font(f_body(14.0)).color(p.muted));
				ui.add_space(6.0);
				if ui.add_sized([80.0, 28.0], egui::DragValue::new(&mut s.num).range(s.range.0..=s.range.1).speed(0.2)).changed() {
					s.scroll = true;
				}
				ui.add_space(8.0);
				let (t, c) = if s.locked(s.num) {
					("used by the game itself — choose another", warn)
				} else if s.taken(s.num) {
					("in use — will be overwritten", warn)
				} else if s.first == Some(s.num) {
					("free — the first free one", green)
				} else {
					("free", green)
				};
				ui.label(egui::RichText::new(t).font(f_body(14.0)).color(c));
			});
			ui.add_space(8.0);
			const ROW: f32 = 34.0;
			let total = (s.range.1 - s.range.0 + 1) as usize;
			let view = (screen.height() - 300.0).clamp(ROW * 4.0, ROW * 9.0);
			let mut area = egui::ScrollArea::vertical().id_salt("slot_pick").max_height(view).auto_shrink([false, false]);
			if std::mem::take(&mut s.scroll) {
				area = area.vertical_scroll_offset(((s.num - s.range.0) as f32 * ROW - view / 2.0 + ROW / 2.0).max(0.0));
			}
			ui.spacing_mut().item_spacing.y = 0.0;
			area.show_rows(ui, ROW, total, |ui, rows| {
				for k in rows {
					let n = s.range.0 + k as u32;
					let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width() - 8.0, ROW), Sense::click());
					let on = n == s.num;
					let b = r.shrink2(Vec2::new(0.0, 1.0));
					if on {
						ui.painter().rect(b, 7, p.sel, Stroke::new(1.0_f32, p.accent), StrokeKind::Inside);
					} else if resp.hovered() {
						ui.painter().rect_filled(b, 7, p.div2);
					}
					let u = s.used.get(&n);
					let pt = ui.painter();
					pt.text(Pos2::new(r.min.x + 14.0, r.center().y), Align2::LEFT_CENTER, n.to_string(), f_semi(14.0), if u.is_none() || on { p.text } else { p.muted });
					let (t, c) = match u {
						None => ("free", green),
						Some(u) if u.locked => ("game data", p.muted),
						Some(u) => (u.title.as_str(), if on { p.text } else { p.muted }),
					};
					let when = u.map_or("", |u| u.when.as_str());
					paint_text_clipped(ui, Rect::from_min_max(Pos2::new(r.min.x + 66.0, r.min.y), Pos2::new(r.max.x - 140.0, r.max.y)), t, f_body(14.0), c, Align2::LEFT_CENTER);
					ui.painter().text(Pos2::new(r.max.x - 10.0, r.center().y), Align2::RIGHT_CENTER, when, f_body(12.0), p.muted);
					if resp.clicked() {
						s.num = n;
					}
				}
			}).middle_scroll(ui);
			ui.add_space(10.0);
			let over = s.taken(s.num) && !s.locked(s.num);
			ui.label(egui::RichText::new(if over { backup_note } else { " " }).font(f_body(13.0)).color(warn));
			ui.add_space(10.0);
			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				let ok = !s.locked(s.num);
				let mut q = *p;
				if over {
					(q.accent, q.accent_strong) = (warn, warn);
				}
				let t = if over { trn("", "Overwrite", s.num) } else { trn("", "Save to", s.num) };
				if primary_button_icon(ui, &q, &t, Some("save"), Vec2::new(190.0, 34.0), 8, f_semi(15.0), ok).clicked() {
					out = Some(Some(s.num));
				}
				ui.add_space(8.0);
				if small_button(ui, p, "Cancel", Some("x"), 34.0, true, p.text).clicked() {
					out = Some(None);
				}
			});
		});
	});
	if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
		out = Some(None);
	}
	out
}

fn trn(_ru: &'static str, en: &'static str, n: u32) -> String {
	format!("{} {n}", en)
}

pub type Cell = (egui::TextureId, Rect);

pub struct RankRow {
	pub name: String,
	pub pts: u32,
	pub rewards: Vec<(Option<Cell>, String)>,
}

pub fn rank_table(ui: &mut Ui, p: &Pal, title: &str, unit: &str, rows: &[RankRow], cur: Option<usize>) {
	const LH: f32 = 25.0;
	const ICON: f32 = 22.0;
	let width = |t: &str, f: FontId| ui.fonts(|x| x.layout_no_wrap(t.to_string(), f, p.text).size().x);
	let c1 = rows.iter().map(|r| width(&r.name, f_semi(13.0))).fold(60.0, f32::max) + 28.0;
	let c2 = 64.0;
	let c4 = rows.iter().flat_map(|r| &r.rewards).map(|x| width(&x.1, f_body(13.0))).fold(120.0, f32::max) + 16.0;
	let lines: usize = rows.iter().map(|r| r.rewards.len().max(1)).sum();
	let w = c1 + c2 + ICON + 18.0 + c4;
	ui.set_min_width(w);
	let (r, _) = ui.allocate_exact_size(Vec2::new(w, 30.0 + 24.0 + lines as f32 * LH + 4.0), Sense::hover());
	let pt = ui.painter();
	let xs = [r.min.x + c1 / 2.0, r.min.x + c1 + c2 / 2.0, r.min.x + c1 + c2 + 8.0 + ICON, r.min.x + c1 + c2 + ICON + 18.0];
	pt.text(Pos2::new(r.center().x, r.min.y + 14.0), Align2::CENTER_CENTER, title, f_semi(15.0), p.text);
	let mut y = r.min.y + 30.0;
	let line = |y: f32| pt.line_segment([Pos2::new(r.min.x, y), Pos2::new(r.max.x, y)], Stroke::new(1.0_f32, p.div));
	line(y);
	for (x, a, t) in [(xs[0], Align2::CENTER_CENTER, "Rank"), (xs[1], Align2::CENTER_CENTER, unit), (xs[3], Align2::LEFT_CENTER, "Reward")] {
		pt.text(Pos2::new(x, y + 12.0), a, t, f_body(12.0), p.muted);
	}
	y += 24.0;
	line(y);
	for (k, row) in rows.iter().enumerate() {
		let h = row.rewards.len().max(1) as f32 * LH;
		let on = cur == Some(k);
		if on {
			pt.rect_filled(Rect::from_min_size(Pos2::new(r.min.x, y + 1.0), Vec2::new(w, h - 1.0)), 5, p.sel);
		}
		let col = if on { p.accent } else { p.text };
		pt.text(Pos2::new(xs[0], y + h / 2.0), Align2::CENTER_CENTER, &row.name, f_semi(13.0), col);
		pt.text(Pos2::new(xs[1], y + h / 2.0), Align2::CENTER_CENTER, row.pts.to_string(), f_body(13.0), col);
		let rc = if cur.is_some_and(|c| k <= c) { p.text } else { p.text.gamma_multiply(0.75) };
		if row.rewards.is_empty() {
			pt.text(Pos2::new(xs[3], y + h / 2.0), Align2::LEFT_CENTER, "—", f_body(13.0), p.muted);
		}
		for (j, (icon, name)) in row.rewards.iter().enumerate() {
			let cy = y + (j as f32 + 0.5) * LH;
			if let Some((t, uv)) = icon {
				pt.image(*t, Rect::from_min_size(Pos2::new(xs[2] - ICON, cy - ICON / 2.0), Vec2::splat(ICON)), *uv, Color32::WHITE);
			}
			pt.text(Pos2::new(xs[3], cy), Align2::LEFT_CENTER, name, f_body(13.0), rc);
		}
		y += h;
		line(y);
	}
}

const REWARD_W: f32 = 380.0;

pub fn ach_head(ui: &mut Ui, p: &Pal, open: bool, title: &str, count: usize) -> bool {
	let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::click());
	let pt = ui.painter();
	pt.rect_filled(r, 14, if resp.hovered() { p.sel } else { p.div2 });
	let mut c = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_max(Pos2::new(r.min.x + 18.0, r.min.y + 6.0), r.max)));
	let hit = section_head(&mut c, p, open, title, count).clicked();
	if open {
		let rew = REWARD_W.min((r.width() - 28.0) * 0.4);
		ui.painter().text(Pos2::new(r.max.x - 14.0 - rew / 2.0, r.center().y), Align2::CENTER_CENTER, "Reward", f_semi(15.0), p.text);
	}
	resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() || hit
}

pub struct AchRow<'a> {
	pub id: egui::Id,
	pub done: bool,
	pub name: &'a str,
	pub rewards: &'a [(Option<Cell>, String, u32)],
	pub counter: Option<(u32, u32)>,
	pub slider: bool,
	pub tip: &'a str,
	pub check: Option<(Cell, Cell)>,
}

#[derive(Default)]
pub struct AchAct {
	pub clicked: bool,
	pub secondary: bool,
	pub counter: Option<u32>,
}

pub fn ach_row(ui: &mut Ui, p: &Pal, a: &AchRow) -> AchAct {
	let mut act = AchAct::default();
	let h = 62.0 + a.rewards.len().saturating_sub(1) as f32 * 34.0;
	let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::hover());
	if !ui.is_rect_visible(r) {
		return act;
	}
	let cy = r.center().y;
	ui.painter().rect_filled(r, 5, p.surface);
	row_border(ui.painter(), p, r);
	let br = Rect::from_center_size(Pos2::new(r.min.x + 28.0, cy), Vec2::splat(28.0));
	let resp = ui.interact(br, a.id.with("box"), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
	match a.check {
		Some((off, on)) => {
			let (t, uv) = if a.done { on } else { off };
			ui.painter().image(t, br, uv, if resp.hovered() { Color32::WHITE } else { Color32::from_gray(220) });
		}
		None => paint_tick(ui, br.shrink(3.0), if a.done { 2 } else { 0 }, resp.hovered()),
	}
	let resp = resp.on_hover_text(a.tip);
	act.clicked = resp.clicked();
	act.secondary = resp.secondary_clicked();
	let x = r.min.x + 52.0;
	let rew = REWARD_W.min((r.width() - 28.0) * 0.4);
	let name_y = if a.counter.is_some() { cy - 12.0 } else { cy };
	paint_text_clipped(ui, Rect::from_min_max(Pos2::new(x, name_y - 14.0), Pos2::new(r.max.x - 14.0 - rew - 10.0, name_y + 14.0)), a.name, f_semi(17.0), p.text, Align2::LEFT_CENTER);
	if let Some((c, goal)) = a.counter {
		let bw = ((r.width() - 28.0) * 0.45).max(200.0).min(r.width() - 28.0 - rew - 160.0);
		let zone = Rect::from_min_size(Pos2::new(x, cy + 4.0), Vec2::new(bw, 22.0));
		let track = Rect::from_center_size(zone.center(), Vec2::new(bw, 4.0));
		let frac = if goal > 0 { (c as f32 / goal as f32).min(1.0) } else { 1.0 };
		let pt = ui.painter();
		pt.rect_filled(track, 2, p.div);
		pt.rect_filled(Rect::from_min_size(track.min, Vec2::new(track.width() * frac, track.height())), 2, p.accent);
		pt.text(Pos2::new(zone.max.x + 16.0, zone.center().y), Align2::LEFT_CENTER, format!("{c} / {goal}"), f_body(14.0), p.muted);
		if a.slider {
			let resp = ui.interact(zone, a.id.with("bar"), Sense::click_and_drag());
			ui.painter().circle_filled(Pos2::new(track.min.x + track.width() * frac, track.center().y), if resp.hovered() || resp.dragged() { 7.0 } else { 5.5 }, p.text);
			if (resp.dragged() || resp.clicked())
				&& let Some(pos) = resp.interact_pointer_pos()
			{
				let v = (((pos.x - track.min.x) / track.width()).clamp(0.0, 1.0) * goal as f32).round() as u32;
				act.counter = (v != c.min(goal)).then_some(v);
			}
			resp.on_hover_text("drag: condition counter; at the goal the achievement is completed");
		}
	}
	let pt = ui.painter();
	let lay: Vec<_> = a.rewards.iter().map(|(_, n, k)| (pt.layout_no_wrap(n.clone(), f_body(16.0), p.text), pt.layout_no_wrap(format!("×{k}"), f_semi(16.0), p.text))).collect();
	let total = lay.iter().map(|(n, k)| 36.0 + n.size().x + 14.0 + k.size().x).fold(0.0, f32::max);
	let x0 = r.max.x - 14.0 - rew / 2.0 - total / 2.0;
	for (j, ((n, k), (icon, ..))) in lay.into_iter().zip(a.rewards).enumerate() {
		let yy = cy + (j as f32 - (a.rewards.len() as f32 - 1.0) / 2.0) * 34.0;
		if let Some((t, uv)) = icon {
			pt.image(*t, Rect::from_min_size(Pos2::new(x0, yy - 14.0), Vec2::splat(28.0)), *uv, Color32::WHITE);
		}
		let nw = n.size().x;
		pt.galley(Pos2::new(x0 + 36.0, yy - n.size().y / 2.0), n, p.text);
		pt.galley(Pos2::new(x0 + 36.0 + nw + 14.0, yy - k.size().y / 2.0), k, p.text);
	}
	act
}

pub fn tick(ui: &mut Ui, state: u8) -> Response {
	let (r, resp) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
	paint_tick(ui, r.shrink(3.0), state, resp.hovered());
	resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, state > 0, ""));
	resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub struct Canvas<'a> {
	pub pt: egui::Painter,
	pub tex: &'a egui::TextureHandle,
	pub o: Pos2,
	pub base: Pos2,
	pub k: f32,
}

impl Canvas<'_> {
	pub fn at(&self, x: f32, y: f32) -> Pos2 {
		self.o + (Pos2::new(x, y) - self.base) * self.k
	}
	pub fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
		Rect::from_min_max(self.at(x0, y0), self.at(x1, y1))
	}
	pub fn uv(&self, s: [f32; 4]) -> Rect {
		let z = self.tex.size_vec2();
		Rect::from_min_max(Pos2::new(s[0] / z.x, s[1] / z.y), Pos2::new(s[2] / z.x, s[3] / z.y))
	}
	pub fn img(&self, s: [f32; 4], r: Rect) {
		self.pt.image(self.tex.id(), r, self.uv(s), Color32::WHITE);
	}
	pub fn piece(&self, s: [f32; 4], x: f32, y: f32) {
		self.img(s, self.rect(x, y, x + s[2] - s[0], y + s[3] - s[1]));
	}
	pub fn font(&self, size: f32, semi: bool) -> egui::FontId {
		if semi { f_semi(size * self.k) } else { f_body(size * self.k) }
	}
	#[allow(clippy::too_many_arguments)]
	pub fn text(&self, x: f32, y: f32, a: Align2, t: &str, size: f32, col: Color32, semi: bool) {
		self.pt.text(self.at(x, y), a, t, self.font(size, semi), col);
	}
	#[allow(clippy::too_many_arguments)]
	pub fn fit(&self, x: f32, y: f32, a: Align2, t: &str, size: f32, w: f32, col: Color32, semi: bool) {
		let full = self.pt.layout_no_wrap(t.to_string(), self.font(size, semi), col).size().x;
		self.text(x, y, a, t, size * (w * self.k / full.max(1.0)).min(1.0), col, semi);
	}
	pub fn check(&self, r: Rect, on: bool) {
		tick_box(&self.pt, r, if on { 2 } else { 0 }, Some(hex(0xececec)), tick_color(true));
	}
	pub fn check_paper(&self, r: Rect, on: bool, drawn: bool) {
		tick_box(&self.pt, r, if on { 2 } else { 0 }, (!drawn).then(|| hex(0x676767)), tick_color(false));
	}
}

pub fn paint_tick(ui: &Ui, b: Rect, state: u8, hovered: bool) {
	tick_mark(ui.painter(), b, state);
	if hovered && state == 0 {
		ui.painter().rect_filled(b.shrink(b.height() * 0.22), 2, tick_color(ui.visuals().dark_mode).gamma_multiply(0.35));
	}
}

static TICK_GREEN: AtomicBool = AtomicBool::new(false);

pub fn tick_green() {
	TICK_GREEN.store(true, Ordering::Relaxed);
}

fn tick_color(dark: bool) -> Color32 {
	hex(match (TICK_GREEN.load(Ordering::Relaxed), dark) {
		(true, true) => 0x4be651,
		(true, false) => 0x0fa836,
		(false, true) => 0x60a5eb,
		(false, false) => 0x2f72b8,
	})
}

#[derive(Default)]
pub struct Pics(std::cell::RefCell<std::collections::HashMap<String, (Option<egui::TextureHandle>, u64)>>);

impl Pics {
	const KEEP: usize = 48;

	pub fn get(&self, ctx: &egui::Context, name: &str, opt: egui::TextureOptions, conv: fn(&image::RgbaImage) -> egui::ColorImage, load: impl FnOnce() -> Option<Vec<u8>>) -> Option<egui::TextureHandle> {
		let now = ctx.cumulative_pass_nr();
		let mut map = self.0.borrow_mut();
		if let Some(p) = map.get_mut(name) {
			p.1 = now;
			return p.0.clone();
		}
		let t = load().and_then(|b| image::load_from_memory(&b).ok()).map(|i| ctx.load_texture(name, conv(&i.to_rgba8()), opt));
		while map.len() >= Self::KEEP
			&& let Some(old) = map.iter().filter(|p| p.1.1 < now).min_by_key(|p| p.1.1).map(|p| p.0.clone())
		{
			map.remove(&old);
		}
		map.insert(name.to_string(), (t.clone(), now));
		t
	}

	pub fn pack(&self, ctx: &egui::Context, name: &str, opt: egui::TextureOptions, conv: fn(&image::RgbaImage) -> egui::ColorImage) -> Option<egui::TextureHandle> {
		self.get(ctx, name, opt, conv, || kiseki_tool::pack::get(name))
	}
}

pub const UV_ALL: Rect = Rect { min: Pos2::ZERO, max: Pos2::new(1.0, 1.0) };

pub fn plain(img: &image::RgbaImage) -> egui::ColorImage {
	egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw())
}

pub fn premultiplied(img: &image::RgbaImage) -> egui::ColorImage {
	let mul = |v: u8, a: u8| (v as u16 * a as u16 / 255) as u8;
	let pixels = img.pixels().map(|q| Color32::from_rgba_premultiplied(mul(q[0], q[3]), mul(q[1], q[3]), mul(q[2], q[3]), q[3])).collect();
	egui::ColorImage { size: [img.width() as usize, img.height() as usize], pixels }
}

pub fn data_warning(ctx: &egui::Context, p: &Pal) {
	if !cfg!(test) {
		data_bar(ctx, p, kiseki_tool::pack::state());
	}
}

pub fn data_bar(ctx: &egui::Context, p: &Pal, state: kiseki_tool::pack::State) {
	use kiseki_tool::pack::State;
	let t = match state {
		State::Ok => return,
		State::Missing => "No data.dat — unpack the whole archive: without it there are no pictures or names",
		State::Broken => "data.dat is damaged — unpack the archive again",
		State::Old => "data.dat is from an older version — unpack the whole archive",
	};
	egui::TopBottomPanel::top("data_warning").frame(egui::Frame::new().fill(p.warn).inner_margin(egui::Margin::symmetric(16, 7))).show_separator_line(false).show(ctx, |ui| {
		ui.label(egui::RichText::new(t).font(f_semi(14.0)).color(if p.dark { hex(0x1f2124) } else { Color32::WHITE }));
	});
}

pub fn tick_mark(pt: &egui::Painter, b: Rect, state: u8) {
	let dark = pt.ctx().style().visuals.dark_mode;
	tick_box(pt, b, state, Some(hex(if dark { 0xececec } else { 0x7c8084 })), tick_color(dark));
}

pub fn tick_box(pt: &egui::Painter, b: Rect, state: u8, edge: Option<Color32>, color: Color32) {
	let k = pt.ctx().pixels_per_point();
	let b = Rect::from_min_max((b.min * k).round() / k, (b.max * k).round() / k);
	let z = b.height();
	if let Some(edge) = edge {
		pt.rect_stroke(b, (z * 0.2).round() as u8, Stroke::new(if z < 16.0 { 1.0_f32 } else { 2.0 }, edge), StrokeKind::Inside);
	}
	let at = |x: f32, y: f32| b.min + Vec2::new(x * z, y * z);
	let line = Stroke::new((z * 0.13).max(1.5), color);
	match state {
		2 => {
			pt.add(egui::Shape::line(vec![at(0.24, 0.52), at(0.42, 0.71), at(0.77, 0.29)], line));
		}
		1 => {
			pt.line_segment([at(0.27, 0.5), at(0.73, 0.5)], line);
		}
		_ => {}
	}
}

pub fn row_border(pt: &egui::Painter, p: &Pal, r: Rect) {
	pt.rect_stroke(r, 5, Stroke::new(1.5_f32, p.div), StrokeKind::Inside);
}

pub fn list_row(ui: &mut Ui, p: &Pal, h: f32, sel: bool) -> (Rect, Response) {
	let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::click());
	if sel || resp.hovered() {
		ui.painter().rect_filled(r, 5, if sel { p.sel } else { p.div2 });
	}
	row_border(ui.painter(), p, r);
	(r, resp.on_hover_cursor(egui::CursorIcon::PointingHand))
}

pub trait MiddleScroll {
	fn middle_scroll(self, ui: &Ui) -> Self;
}

impl<R> MiddleScroll for egui::scroll_area::ScrollAreaOutput<R> {
	fn middle_scroll(mut self, ui: &Ui) -> Self {
		let key = egui::Id::new("middle_scroll");
		let me = self.id;
		let (down, up, stop, pos, dt) = ui.input(|i| {
			let b = |b| i.pointer.button_pressed(b);
			(b(egui::PointerButton::Middle), i.pointer.button_released(egui::PointerButton::Middle), b(egui::PointerButton::Primary) || b(egui::PointerButton::Secondary) || i.key_pressed(egui::Key::Escape), i.pointer.latest_pos(), i.stable_dt.min(0.1))
		});
		let mut st: Option<(egui::Id, Pos2, bool)> = ui.data(|d| d.get_temp(key));
		match st {
			Some((who, ..)) if stop || down && who == me => {
				ui.data_mut(|d| d.remove::<(egui::Id, Pos2, bool)>(key));
				return self;
			}
			_ if down && ui.rect_contains_pointer(self.inner_rect) => st = pos.map(|p| (me, p, false)),
			_ => {}
		}
		let Some((who, from, mut moved)) = st else { return self };
		let (true, Some(pos)) = (who == me, pos) else { return self };
		let dy = pos.y - from.y;
		moved |= dy.abs() > 12.0;
		if up && moved {
			ui.data_mut(|d| d.remove::<(egui::Id, Pos2, bool)>(key));
			return self;
		}
		ui.data_mut(|d| d.insert_temp(key, (me, from, moved)));
		if dy.abs() > 12.0 {
			let end = (self.content_size.y - self.inner_rect.height()).max(0.0);
			self.state.offset.y = (self.state.offset.y + (dy - 12.0 * dy.signum()) * 8.0 * dt).clamp(0.0, end);
			self.state.store(ui.ctx(), me);
		}
		let pt = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, key));
		pt.circle(from, 13.0, Color32::from_black_alpha(170), Stroke::new(1.5_f32, Color32::from_gray(210)));
		for s in [-1.0_f32, 1.0] {
			pt.add(egui::Shape::convex_polygon(vec![from + Vec2::new(-4.0, 4.0 * s), from + Vec2::new(4.0, 4.0 * s), from + Vec2::new(0.0, 9.0 * s)], Color32::from_gray(230), Stroke::NONE));
		}
		ui.ctx().set_cursor_icon(egui::CursorIcon::AllScroll);
		ui.ctx().request_repaint();
		self
	}
}

pub fn section_head(ui: &mut Ui, p: &Pal, open: bool, title: &str, count: usize) -> Response {
	let lay = |t: String, f: FontId, c: Color32| ui.fonts(|x| x.layout_no_wrap(t, f, c));
	let name = lay(format!("{} {title}", if open { "▾" } else { "▸" }), f_semi(14.0), p.text);
	let num = lay(count.to_string(), f_body(13.0), p.muted);
	let cap = lay("H".into(), f_semi(14.0), p.text).mesh_bounds.height();
	let base = |g: &egui::Galley| g.rows.first().and_then(|r| r.glyphs.last()).map_or(0.0, |g| g.pos.y);
	let (nw, uw) = (name.size().x, num.size().x);
	let (r, resp) = ui.allocate_exact_size(Vec2::new(nw + 8.0 + uw, 22.0), Sense::click());
	let y = r.center().y + cap / 2.0;
	let (yn, yu) = (y - base(&name), y - base(&num));
	ui.painter().galley(Pos2::new(r.min.x, yn), name, p.text);
	ui.painter().galley(Pos2::new(r.min.x + nw + 8.0, yu), num, p.muted);
	resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[derive(Clone, Copy)]
pub struct CardSize {
	pub size: Vec2,
	pub face: f32,
}

const FACE: f32 = 84.0;

pub const FACE_TEX: egui::TextureOptions = egui::TextureOptions { magnification: egui::TextureFilter::Linear, minification: egui::TextureFilter::Linear, wrap_mode: egui::TextureWrapMode::ClampToEdge, mipmap_mode: Some(egui::TextureFilter::Linear) };

pub fn face_texture(ctx: &egui::Context, name: String, img: &image::RgbaImage) -> egui::TextureHandle {
	let px = (FACE * ctx.pixels_per_point()).round().max(16.0) as u32;
	let img = if img.width() > px || img.height() > px {
		let k = px as f32 / img.width().max(img.height()) as f32;
		image::imageops::resize(img, ((img.width() as f32 * k).round() as u32).max(1), ((img.height() as f32 * k).round() as u32).max(1), image::imageops::FilterType::Lanczos3)
	} else {
		img.clone()
	};
	let opts = egui::TextureOptions { magnification: egui::TextureFilter::Linear, minification: egui::TextureFilter::Linear, wrap_mode: egui::TextureWrapMode::ClampToEdge, mipmap_mode: None };
	ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw()), opts)
}

fn paint_card(pt: &egui::Painter, p: &Pal, r: Rect, cs: CardSize, tex: Option<&egui::TextureHandle>, name: &str, tag: &str, a: f32) {
	let fr = Rect::from_center_size(Pos2::new(r.center().x, r.min.y + 8.0 + cs.face / 2.0), Vec2::splat(cs.face));
	if let Some(t) = tex {
		pt.image(t.id(), fr, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE.gamma_multiply(a));
	}
	let ny = fr.max.y + 12.0;
	let clip = |y: f32, h: f32| Rect::from_center_size(Pos2::new(r.center().x, y), Vec2::new(cs.size.x - 10.0, h));
	pt.with_clip_rect(clip(ny, 18.0)).text(Pos2::new(r.center().x, ny), Align2::CENTER_CENTER, name, f_body(13.0), p.text.gamma_multiply(a));
	let col = if tag.starts_with('⚠') { p.warn } else { p.muted };
	pt.with_clip_rect(clip(ny + 16.0, 16.0)).text(Pos2::new(r.center().x, ny + 16.0), Align2::CENTER_CENTER, tag, f_body(11.0), col.gamma_multiply(a));
}

#[allow(clippy::too_many_arguments)]
pub fn lineup_card(ui: &mut Ui, p: &Pal, cs: CardSize, tex: Option<&egui::TextureHandle>, name: &str, tag: &str, selected: bool, target: bool) -> Response {
	let (r, resp) = ui.allocate_exact_size(cs.size, Sense::click_and_drag());
	let pt = ui.painter();
	pt.rect_filled(r, 8, if resp.hovered() || target { p.div2 } else { p.bg });
	paint_card(pt, p, r, cs, tex, name, tag, 1.0);
	let stroke = if selected || target { Stroke::new(2.0_f32, p.accent) } else { Stroke::new(1.0_f32, p.div) };
	pt.rect_stroke(r, 8, stroke, StrokeKind::Inside);
	if resp.drag_started() {
		egui::DragAndDrop::set_payload(ui.ctx(), name.to_string());
	}
	resp.on_hover_cursor(egui::CursorIcon::Grab)
}

pub fn lineup_ghost(ctx: &egui::Context, p: &Pal, cs: CardSize, pos: Pos2, tex: Option<&egui::TextureHandle>, name: &str, tag: &str) {
	let pt = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("lineup_drag")));
	let a = 0.85;
	let r = Rect::from_center_size(pos, cs.size);
	pt.rect(r, 8, p.bg.gamma_multiply(a), Stroke::new(2.0_f32, p.accent.gamma_multiply(a)), StrokeKind::Inside);
	paint_card(&pt, p, r, cs, tex, name, tag, a);
}

pub fn empty_slot(ui: &Ui, p: &Pal, rect: Rect, dashed: bool, armed: bool) {
	let over = ui.rect_contains_pointer(rect);
	let pt = ui.painter();
	if over {
		pt.rect_filled(rect, 8, p.div2);
	}
	let col = if over || armed { p.accent } else { p.muted };
	let w = if over { 2.0_f32 } else { 1.0 };
	if dashed {
		let r = rect.shrink(1.0);
		let corners = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
		for c in corners.windows(2) {
			pt.extend(egui::Shape::dashed_line(&[c[0], c[1]], Stroke::new(w, col), 6.0, 4.0));
		}
	} else {
		pt.rect_stroke(rect, 8, Stroke::new(w, if over || armed { col } else { p.muted.gamma_multiply(0.6) }), StrokeKind::Inside);
	}
	pt.text(rect.center(), Align2::CENTER_CENTER, "+", f_semi(26.0), col);
}

pub fn slot_matches(name: &str, q: &str) -> bool {
	let qd: String = q.chars().filter(char::is_ascii_digit).collect();
	qd.is_empty() || name.chars().filter(char::is_ascii_digit).collect::<String>().contains(&qd)
}

pub fn slot_search(ui: &mut Ui, p: &Pal, id: &str, q: &mut String, found: usize, total: usize) -> bool {
	let w = ui.available_width();
	let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 30.0), Sense::hover());
	let te_id = ui.id().with(id);
	let focused = ui.memory(|m| m.has_focus(te_id));
	ui.painter().rect(rect, 7, p.surface, Stroke::new(1.0_f32, if focused { p.accent } else { p.div }), StrokeKind::Inside);
	paint_icon(ui, "search", Rect::from_center_size(Pos2::new(rect.min.x + 16.0, rect.center().y), Vec2::splat(14.0)), p.muted);
	let mut changed = false;
	let xr = Rect::from_center_size(Pos2::new(rect.max.x - 16.0, rect.center().y), Vec2::splat(20.0));
	if !q.is_empty() {
		let xresp = ui.interact(xr, te_id.with("x"), Sense::click()).on_hover_text("Clear (Esc)");
		paint_icon(ui, "x", xr.shrink(3.0), if xresp.hovered() { p.text } else { p.muted });
		if xresp.clicked() {
			q.clear();
			changed = true;
		}
	}
	let trect = Rect::from_min_max(Pos2::new(rect.min.x + 30.0, rect.min.y), Pos2::new(xr.min.x - 4.0, rect.max.y));
	let mut child = ui.new_child(egui::UiBuilder::new().max_rect(trect).layout(egui::Layout::left_to_right(egui::Align::Center)));
	let resp = child.add(
		egui::TextEdit::singleline(q)
			.id(te_id)
			.frame(false)
			.font(f_body(13.0))
			.hint_text(egui::RichText::new("Slot number").color(p.muted))
			.text_color(p.text)
			.char_limit(6)
			.desired_width(trect.width())
			.margin(egui::Margin::ZERO),
	);
	if resp.changed() {
		q.retain(|c| c.is_ascii_digit());
		changed = true;
	}
	if focused && ui.input(|i| i.key_pressed(egui::Key::Escape)) && !q.is_empty() {
		q.clear();
		changed = true;
	}
	let (info, _) = ui.allocate_exact_size(Vec2::new(w, if q.is_empty() { 2.0 } else { 22.0 }), Sense::hover());
	if !q.is_empty() {
		ui.painter().text(Pos2::new(info.min.x + 6.0, info.center().y), Align2::LEFT_CENTER, format!("found {found} of {total}"), f_body(11.5), p.muted);
	}
	changed
}

#[allow(clippy::too_many_arguments)]
pub fn search_box(ui: &mut Ui, p: &Pal, id: &str, q: &mut String, hint: &str, w: f32, h: f32, fill: Color32) -> bool {
	let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
	let te_id = ui.id().with(id);
	let focused = ui.memory(|m| m.has_focus(te_id));
	ui.painter().rect(rect, 7, fill, Stroke::new(1.0_f32, if focused { p.accent } else { p.div }), StrokeKind::Inside);
	paint_icon(ui, "search", Rect::from_center_size(Pos2::new(rect.min.x + 15.0, rect.center().y), Vec2::splat(14.0)), p.muted);
	let mut changed = false;
	let xr = Rect::from_center_size(Pos2::new(rect.max.x - 14.0, rect.center().y), Vec2::splat(18.0));
	if !q.is_empty() {
		let resp = ui.interact(xr, te_id.with("x"), Sense::click()).on_hover_text("Clear (Esc)");
		paint_icon(ui, "x", xr.shrink(3.0), if resp.hovered() { p.text } else { p.muted });
		if resp.clicked() || focused && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
			q.clear();
			changed = true;
		}
	}
	let tr_ = Rect::from_min_max(Pos2::new(rect.min.x + 28.0, rect.min.y), Pos2::new(xr.min.x - 4.0, rect.max.y));
	let mut c = ui.new_child(egui::UiBuilder::new().max_rect(tr_).layout(egui::Layout::left_to_right(egui::Align::Center)));
	changed |= c.add(egui::TextEdit::singleline(q).id(te_id).frame(false).font(f_body(13.0)).hint_text(egui::RichText::new(hint).color(p.muted)).text_color(p.text).desired_width(tr_.width()).margin(egui::Margin::ZERO)).changed();
	changed
}

#[allow(clippy::too_many_arguments)]
pub fn path_field(ui: &mut Ui, p: &Pal, id: &str, value: &mut String, height: f32, hint: &str, button: &str, highlight: bool, enabled: bool) -> (bool, bool, Rect) {
	let w = ui.available_width();
	let (rect, _) = ui.allocate_exact_size(Vec2::new(w, height), Sense::hover());
	let r = 8u8;
	let stroke = if highlight { p.accent } else { p.div };
	ui.painter().rect(rect, r, p.surface, Stroke::new(1.0_f32, stroke), StrokeKind::Inside);
	let font = f_body(13.0);
	let tw = ui.fonts(|f| f.layout_no_wrap(button.to_string(), font.clone(), p.text).size().x);
	let bw = tw + 26.0 + 20.0;
	let brect = Rect::from_min_max(Pos2::new(rect.max.x - bw, rect.min.y), rect.max);
	let bresp = ui.interact(brect, ui.id().with(id).with("btn"), if enabled { Sense::click() } else { Sense::hover() });
	if bresp.hovered() && enabled {
		ui.painter().rect_filled(brect.shrink(1.0), CornerRadius { nw: 0, sw: 0, ne: r - 1, se: r - 1 }, p.div2);
	}
	ui.painter().line_segment([Pos2::new(brect.min.x, rect.min.y + 1.0), Pos2::new(brect.min.x, rect.max.y - 1.0)], Stroke::new(1.0_f32, p.div));
	let bc = if enabled { p.text } else { p.muted };
	let bx = brect.min.x + 13.0;
	paint_icon(ui, "folder-open", Rect::from_center_size(Pos2::new(bx + 7.0, brect.center().y), Vec2::splat(14.0)), bc);
	ui.painter().text(Pos2::new(bx + 20.0, brect.center().y), Align2::LEFT_CENTER, button, font.clone(), bc);
	let trect = Rect::from_min_max(Pos2::new(rect.min.x + 12.0, rect.min.y), Pos2::new(brect.min.x - 6.0, rect.max.y));
	let mut child = ui.new_child(egui::UiBuilder::new().max_rect(trect).layout(egui::Layout::left_to_right(egui::Align::Center)));
	let te = egui::TextEdit::singleline(value)
		.id(ui.id().with(id))
		.frame(false)
		.font(font)
		.hint_text(egui::RichText::new(hint).color(p.muted))
		.text_color(p.text)
		.desired_width(trect.width())
		.interactive(enabled)
		.margin(egui::Margin::ZERO);
	let resp = child.add(te);
	(resp.changed(), bresp.clicked() && enabled, brect)
}

pub fn text_w(ui: &Ui, t: &str, font: FontId) -> f32 {
	ui.fonts(|f| f.layout_no_wrap(t.to_string(), font, Color32::WHITE).size().x)
}

pub fn small_button_w(ui: &Ui, t: &str) -> f32 {
	text_w(ui, t, f_body(13.0)) + 24.0
}

#[allow(clippy::too_many_arguments)]
pub fn segmented_rows(ui: &mut Ui, p: &Pal, id: &str, items: &[&str], sel: &mut usize, height: f32, radius: u8, font: FontId) -> bool {
	let w: Vec<f32> = items.iter().map(|t| text_w(ui, t, font.clone()) + 28.0).collect();
	let avail = ui.available_width();
	let split = |limit: f32| {
		let (mut rows, mut x, mut from) = (Vec::new(), 0.0, 0);
		for (i, v) in w.iter().enumerate() {
			if i > from && x + v > limit {
				rows.push(from..i);
				(x, from) = (0.0, i);
			}
			x += v;
		}
		rows.push(from..w.len());
		rows
	};
	let n = split(avail).len();
	let mut limit = w.iter().sum::<f32>() / n as f32;
	while limit < avail && split(limit).len() > n {
		limit += 8.0;
	}
	let mut changed = false;
	for (k, r) in split(limit.min(avail)).into_iter().enumerate() {
		let mut s = sel.checked_sub(r.start).filter(|&s| s < r.len()).unwrap_or(usize::MAX);
		if segmented(ui, p, &format!("{id}{k}"), &items[r.clone()], &mut s, None, height, radius, font.clone(), &[]) {
			*sel = r.start + s;
			changed = true;
		}
	}
	changed
}

pub fn tabs_with_hint(ui: &mut Ui, p: &Pal, id: &str, items: &[&str], sel: &mut usize, hint: &str) {
	let tabs: f32 = items.iter().map(|t| text_w(ui, t, f_body(13.0)) + 28.0).sum();
	let wide = tabs + 30.0 + text_w(ui, hint, f_body(12.0)) <= ui.available_width();
	let text = egui::RichText::new(hint).font(f_body(12.0)).color(p.muted);
	ui.horizontal(|ui| {
		segmented(ui, p, id, items, sel, None, 32.0, 8, f_body(13.0), &[]);
		if wide {
			ui.add_space(12.0);
			ui.label(text.clone());
		}
	});
	if !wide {
		ui.add(egui::Label::new(text).wrap());
	}
}

#[allow(clippy::too_many_arguments)]
pub fn segmented(ui: &mut Ui, p: &Pal, id: &str, items: &[&str], sel: &mut usize, width: Option<f32>, height: f32, radius: u8, font: FontId, tips: &[&str]) -> bool {
	let pad = 14.0;
	let widths: Vec<f32> = items.iter().map(|t| ui.fonts(|f| f.layout_no_wrap(t.to_string(), font.clone(), p.text).size().x) + pad * 2.0).collect();
	let total = width.unwrap_or_else(|| widths.iter().sum());
	let equal = width.is_some();
	let (rect, _) = ui.allocate_exact_size(Vec2::new(total, height), Sense::hover());
	ui.painter().rect(rect, radius, Color32::TRANSPARENT, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
	let mut x = rect.min.x;
	let mut changed = false;
	let n = items.len();
	for (i, t) in items.iter().enumerate() {
		let w = if equal { total / n as f32 } else { widths[i] };
		let r = Rect::from_min_size(Pos2::new(x, rect.min.y), Vec2::new(w, height));
		let mut resp = ui.interact(r, ui.id().with(id).with(i), Sense::click());
		if let Some(tip) = tips.get(i) {
			resp = resp.on_hover_text(*tip);
		}
		let cr = CornerRadius {
			nw: if i == 0 { radius } else { 0 },
			sw: if i == 0 { radius } else { 0 },
			ne: if i == n - 1 { radius } else { 0 },
			se: if i == n - 1 { radius } else { 0 },
		};
		if *sel == i {
			ui.painter().rect_filled(r, cr, p.accent);
		} else if resp.hovered() {
			ui.painter().rect_filled(r.shrink(1.0), cr, p.div2);
		}
		if i > 0 && *sel != i && *sel != i - 1 {
			ui.painter().line_segment([Pos2::new(x, rect.min.y + 1.0), Pos2::new(x, rect.max.y - 1.0)], Stroke::new(1.0_f32, p.div));
		}
		ui.painter().text(r.center(), Align2::CENTER_CENTER, *t, font.clone(), if *sel == i { p.on_accent } else { p.text });
		if resp.clicked() && *sel != i {
			*sel = i;
			changed = true;
		}
		x += w;
	}
	changed
}

pub fn click_tip(ui: &Ui, resp: &Response, id: impl std::hash::Hash, add: impl FnOnce(&mut Ui)) {
	let pid = ui.id().with(("click_tip", id));
	let resp = resp.clone().on_hover_cursor(egui::CursorIcon::PointingHand);
	if resp.clicked() {
		ui.memory_mut(|m| m.toggle_popup(pid));
	}
	egui::popup_below_widget(ui, pid, &resp, egui::PopupCloseBehavior::CloseOnClick, |ui| {
		ui.set_min_width(220.0);
		add(ui);
	});
}

pub fn small_button(ui: &mut Ui, p: &Pal, text: &str, icon_name: Option<&str>, h: f32, enabled: bool, color: Color32) -> Response {
	let font = f_body(13.0);
	let tw = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font.clone(), color).size().x);
	let iw = if icon_name.is_some() { 16.0 + 6.0 } else { 0.0 };
	let (rect, resp) = ui.allocate_exact_size(Vec2::new(tw + iw + 24.0, h), if enabled { Sense::click() } else { Sense::hover() });
	resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, text));
	let fill = if resp.hovered() && enabled { p.div2 } else { Color32::TRANSPARENT };
	let alpha = if enabled { 1.0 } else { 0.4 };
	let c = color.gamma_multiply(alpha);
	ui.painter().rect(rect, 7, fill, Stroke::new(1.0_f32, p.div.gamma_multiply(alpha)), StrokeKind::Inside);
	let mut x = rect.min.x + 12.0;
	if let Some(n) = icon_name {
		paint_icon(ui, n, Rect::from_center_size(Pos2::new(x + 8.0, rect.center().y), Vec2::splat(16.0)), c);
		x += iw;
	}
	ui.painter().text(Pos2::new(x, rect.center().y), Align2::LEFT_CENTER, text, font, c);
	if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp }
}

pub fn page_tabs(ui: &Ui, p: &Pal, x0: f32, x1: f32, y: f32, labels: &[&str], sel: &mut usize) -> bool {
	let h = 34.0;
	let mut x = x0;
	let mut changed = false;
	ui.painter().line_segment([Pos2::new(x0, y + h), Pos2::new(x1, y + h)], Stroke::new(1.0_f32, p.div));
	for (i, t) in labels.iter().enumerate() {
		let on = *sel == i;
		let font = if on { f_semi(14.5) } else { f_body(14.5) };
		let tw = ui.fonts(|f| f.layout_no_wrap(t.to_string(), font.clone(), p.text).size().x) + 32.0;
		let r = Rect::from_min_size(Pos2::new(x, y), Vec2::new(tw, h));
		let resp = ui.interact(r, ui.id().with(("page", i)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
		if resp.hovered() && !on {
			ui.painter().rect_filled(r.shrink2(Vec2::new(0.0, 3.0)), 6, p.div2);
		}
		ui.painter().text(r.center(), Align2::CENTER_CENTER, *t, font, if on { p.text } else { p.muted });
		if on {
			ui.painter().rect_filled(Rect::from_min_size(Pos2::new(x, y + h - 2.0), Vec2::new(tw, 2.0)), 0, p.accent);
		}
		if resp.clicked() && !on {
			*sel = i;
			changed = true;
		}
		x += tw + 4.0;
	}
	changed
}

#[allow(clippy::too_many_arguments)]
pub fn action_button(ui: &Ui, p: &Pal, rect: Rect, id: &str, icon: &str, title: &str, subtitle: &str, busy: bool, running: bool, prog: Option<f32>) -> bool {
	let resp = ui.interact(rect, ui.id().with(("main", id)), if busy { Sense::hover() } else { Sense::click() });
	resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, !busy, title));
	let painter = ui.painter().with_clip_rect(rect.expand(1.0));
	painter.rect_filled(rect, 12, if resp.hovered() && !busy { p.accent_strong } else { p.accent });
	let ix = rect.min.x + 20.0;
	paint_icon(ui, icon, Rect::from_min_size(Pos2::new(ix, rect.center().y - 13.0), Vec2::splat(26.0)), p.on_accent);
	let tx = ix + 26.0 + 14.0;
	painter.text(Pos2::new(tx, rect.center().y - 10.0), Align2::LEFT_CENTER, title, f_semi(21.0), p.on_accent);
	painter.text(Pos2::new(tx, rect.center().y + 13.0), Align2::LEFT_CENTER, subtitle, f_body(13.0), p.on_accent.gamma_multiply(0.85));
	if running
		&& let Some(pr) = prog {
			let br = Rect::from_min_size(Pos2::new(rect.min.x, rect.max.y - 4.0), Vec2::new(rect.width() * pr.clamp(0.0, 1.0), 4.0));
			painter.with_clip_rect(rect.shrink(0.5)).rect_filled(br, CornerRadius { nw: 0, ne: 0, sw: 12, se: if pr >= 0.99 { 12 } else { 0 } }, p.ok_strong);
		}
	let clicked = resp.clicked();
	if !busy {
		resp.on_hover_cursor(egui::CursorIcon::PointingHand);
	}
	clicked
}

pub fn small_button_width(ui: &Ui, text: &str, icon: bool) -> f32 {
	ui.fonts(|f| f.layout_no_wrap(text.to_string(), f_body(13.0), Color32::WHITE).size().x) + if icon { 22.0 } else { 0.0 } + 24.0
}

#[allow(clippy::too_many_arguments)]
pub fn primary_button_icon(ui: &mut Ui, p: &Pal, text: &str, icon_name: Option<&str>, size: Vec2, radius: u8, font: FontId, enabled: bool) -> Response {
	let (rect, resp) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
	resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, text));
	let fill = if !enabled { p.accent.gamma_multiply(0.55) } else if resp.hovered() { p.accent_strong } else { p.accent };
	ui.painter().rect_filled(rect, radius, fill);
	let is = (font.size + 1.0).round();
	let tw = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font.clone(), p.on_accent).size().x);
	match icon_name {
		Some(n) if tw + is + 7.0 < size.x - 12.0 => {
			let x = rect.center().x - (tw + is + 7.0) / 2.0;
			paint_icon(ui, n, Rect::from_center_size(Pos2::new(x + is / 2.0, rect.center().y), Vec2::splat(is)), p.on_accent);
			ui.painter().text(Pos2::new(x + is + 7.0, rect.center().y), Align2::LEFT_CENTER, text, font, p.on_accent);
		}
		_ => {
			ui.painter().text(rect.center(), Align2::CENTER_CENTER, text, font, p.on_accent);
		}
	}
	if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp }
}

pub fn checkbox(ui: &mut Ui, p: &Pal, value: &mut bool, text: &str, size: f32, font: FontId) -> Response {
	let tw = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font.clone(), p.text).size().x);
	let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(size + 8.0 + tw, size.max(20.0)), Sense::click());
	let b = Rect::from_min_size(Pos2::new(rect.min.x, rect.center().y - size / 2.0), Vec2::splat(size));
	paint_tick(ui, b, if *value { 2 } else { 0 }, resp.hovered());
	ui.painter().text(Pos2::new(b.max.x + 8.0, rect.center().y), Align2::LEFT_CENTER, text, font, p.text);
	if resp.clicked() {
		*value = !*value;
		resp.mark_changed();
	}
	let v = *value;
	resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, v, text));
	resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn theme_button(ui: &mut Ui, p: &Pal, h: f32) -> Response {
	let (t, i) = if p.dark { ("Light theme", "sun") } else { ("Dark theme", "moon") };
	small_button(ui, p, t, Some(i), h, true, p.text)
}

#[allow(clippy::too_many_arguments)]
pub fn combo(ui: &mut Ui, p: &Pal, id: &str, items: &[String], sel: &mut usize, width: f32, height: f32, radius: u8, font: FontId) -> bool {
	let mut changed = false;
	let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
	let popup_id = ui.id().with(id).with("popup");
	let open = ui.memory(|m| m.is_popup_open(popup_id));
	ui.painter().rect(rect, radius, p.surface, Stroke::new(1.0_f32, if open { p.accent } else { p.div }), StrokeKind::Inside);
	let label = items.get(*sel).cloned().unwrap_or_default();
	paint_text_clipped(ui, Rect::from_min_max(Pos2::new(rect.min.x + 12.0, rect.min.y), Pos2::new(rect.max.x - 28.0, rect.max.y)), &label, font.clone(), p.text, Align2::LEFT_CENTER);
	let c = Pos2::new(rect.max.x - 15.0, rect.center().y);
	ui.painter().add(egui::Shape::line(vec![Pos2::new(c.x - 4.0, c.y - 2.0), Pos2::new(c.x, c.y + 2.0), Pos2::new(c.x + 4.0, c.y - 2.0)], Stroke::new(1.5_f32, p.text)));
	if resp.clicked() {
		ui.memory_mut(|m| m.toggle_popup(popup_id));
	}
	let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
	egui::popup::popup_below_widget(ui, popup_id, &resp, egui::PopupCloseBehavior::CloseOnClick, |ui| {
		ui.set_min_width(width - 12.0);
		egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
			for (i, t) in items.iter().enumerate() {
				let tw = ui.available_width();
				let (r, rr) = ui.allocate_exact_size(Vec2::new(tw, 28.0), Sense::click());
				if rr.hovered() || i == *sel {
					ui.painter().rect_filled(r, 7, if rr.hovered() { p.sel } else { p.div2 });
				}
				paint_text_clipped(ui, r.shrink2(Vec2::new(10.0, 0.0)), t, font.clone(), p.text, Align2::LEFT_CENTER);
				if rr.clicked()
					&& *sel != i {
						*sel = i;
						changed = true;
					}
			}
		});
	});
	changed
}

pub fn file_picker(ui: &mut Ui, p: &Pal, id: &str, files: &[(String, usize)], sel: &mut usize, width: f32, height: f32) -> bool {
	let font = f_body(13.0);
	let mut changed = false;
	let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
	let popup_id = ui.id().with(id).with("files");
	let filter_id = popup_id.with("filter");
	let open = ui.memory(|m| m.is_popup_open(popup_id));
	ui.painter().rect(rect, 7, p.surface, Stroke::new(1.0_f32, if open { p.accent } else { p.div }), StrokeKind::Inside);
	let short = |k: &str| -> String {
		let name = k.rsplit('/').next().unwrap_or(k);
		if is_table(k) { name.rsplit_once('.').map_or(name, |(a, _)| a).to_string() } else { name.to_string() }
	};
	fn is_table(k: &str) -> bool {
		let name = k.rsplit('/').next().unwrap_or(k);
		k.starts_with("table") || (name.starts_with("t_") && !k.ends_with(".yaml"))
	}
	let group = |k: &str| -> String {
		if is_table(k) {
			"Tables".to_string()
		} else if k.ends_with(".json") {
			format!("JSON · {}", k.split('/').next().unwrap_or(""))
		} else {
			"Other".to_string()
		}
	};
	let label = files.get(sel.wrapping_sub(1)).map(|(k, n)| format!("{}  ({n})", short(k)));
	let r = Rect::from_min_max(Pos2::new(rect.min.x + 12.0, rect.min.y), Pos2::new(rect.max.x - 28.0, rect.max.y));
	match &label {
		Some(t) => paint_text_clipped(ui, r, t, font.clone(), p.text, Align2::LEFT_CENTER),
		None => paint_text_clipped(ui, r, "choose a table or file", font.clone(), p.muted, Align2::LEFT_CENTER),
	}
	let c = Pos2::new(rect.max.x - 15.0, rect.center().y);
	ui.painter().add(egui::Shape::line(vec![Pos2::new(c.x - 4.0, c.y - 2.0), Pos2::new(c.x, c.y + 2.0), Pos2::new(c.x + 4.0, c.y - 2.0)], Stroke::new(1.5_f32, p.text)));
	if resp.clicked() {
		ui.memory_mut(|m| m.toggle_popup(popup_id));
	}
	let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
	let mut order: Vec<(String, Vec<usize>)> = Vec::new();
	for (i, (k, _)) in files.iter().enumerate() {
		let g = group(k);
		match order.iter_mut().find(|x| x.0 == g) {
			Some(x) => x.1.push(i),
			None => order.push((g, vec![i])),
		}
	}
	let rank = |g: &str| if g == "Tables" { (0, String::new()) } else if g == "JSON · scena" { (1, String::new()) } else if g.starts_with("JSON") { (2, g.to_string()) } else { (3, g.to_string()) };
	order.sort_by_key(|x| rank(&x.0));
	for g in &mut order {
		g.1.sort_by_key(|&i| short(&files[i].0));
	}
	egui::popup::popup_below_widget(ui, popup_id, &resp, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
		let pw = width.max(320.0) - 12.0;
		ui.set_width(pw);
		let mut q: String = ui.data(|d| d.get_temp(filter_id)).unwrap_or_default();
		let te = ui.add(egui::TextEdit::singleline(&mut q).hint_text(egui::RichText::new("filter: start typing a name").color(p.muted)).desired_width(pw));
		if !te.has_focus() && q.is_empty() {
			te.request_focus();
		}
		ui.data_mut(|d| d.insert_temp(filter_id, q.clone()));
		let ql = q.trim().to_lowercase();
		ui.add_space(4.0);
		egui::ScrollArea::vertical().max_height(440.0).show(ui, |ui| {
			for (g, list) in &order {
				let shown: Vec<usize> = list.iter().copied().filter(|&i| ql.is_empty() || short(&files[i].0).to_lowercase().contains(&ql)).collect();
				if shown.is_empty() {
					continue;
				}
				let (hr, _) = ui.allocate_exact_size(Vec2::new(pw - 14.0, 26.0), Sense::hover());
				ui.painter().text(Pos2::new(hr.min.x + 4.0, hr.center().y + 2.0), Align2::LEFT_CENTER, format!("{g}  {}", shown.len()), f_semi(12.5), p.muted);
				ui.painter().line_segment([Pos2::new(hr.min.x, hr.max.y - 0.5), Pos2::new(hr.max.x, hr.max.y - 0.5)], Stroke::new(1.0_f32, p.div));
				for i in shown {
					let tw = pw - 14.0;
					let (r, rr) = ui.allocate_exact_size(Vec2::new(tw, 26.0), Sense::click());
					let on = *sel == i + 1;
					if rr.hovered() || on {
						ui.painter().rect_filled(r, 6, if rr.hovered() { p.sel } else { p.div2 });
					}
					paint_text_clipped(ui, Rect::from_min_max(Pos2::new(r.min.x + 10.0, r.min.y), Pos2::new(r.max.x - 70.0, r.max.y)), &short(&files[i].0), font.clone(), p.text, Align2::LEFT_CENTER);
					ui.painter().text(Pos2::new(r.max.x - 10.0, r.center().y), Align2::RIGHT_CENTER, files[i].1.to_string(), f_body(12.0), p.muted);
					let rr = rr.on_hover_text(&files[i].0).on_hover_cursor(egui::CursorIcon::PointingHand);
					if rr.clicked() {
						if !on {
							*sel = i + 1;
							changed = true;
						}
						ui.memory_mut(|m| m.close_popup());
					}
				}
			}
		});
	});
	changed
}

#[derive(Clone)]
pub enum Dialog<A: Clone> {
	None,
	Info(String),
	Ask(String, A),
	Check(String, String, String, bool),
}

pub fn show_dialog<A: Clone>(ctx: &egui::Context, p: &Pal, title: &str, d: &mut Dialog<A>) -> Option<A> {
	let (text, ask) = match d {
		Dialog::None => return None,
		Dialog::Info(t) | Dialog::Check(t, ..) => (t.clone(), None),
		Dialog::Ask(t, a) => (t.clone(), Some(a.clone())),
	};
	let mut check = match d {
		Dialog::Check(_, label, note, on) => Some((label.clone(), note.clone(), *on)),
		_ => None,
	};
	let mut result = None;
	let mut close = false;
	let screen = ctx.screen_rect();
	egui::Area::new(egui::Id::new("dlg_dim")).order(egui::Order::Middle).fixed_pos(screen.min).show(ctx, |ui| {
		let (r, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
		ui.painter().rect_filled(r, 0, Color32::from_black_alpha(if p.dark { 120 } else { 70 }));
		let _ = resp;
	});
	egui::Area::new(egui::Id::new("dlg_box")).order(egui::Order::Foreground).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
		egui::Frame::new()
			.fill(p.bg)
			.stroke(Stroke::new(1.0_f32, p.div))
			.corner_radius(12)
			.inner_margin(egui::Margin::same(20))
			.shadow(egui::epaint::Shadow { offset: [0, 12], blur: 32, spread: 0, color: Color32::from_black_alpha(64) })
			.show(ui, |ui| {
				ui.set_max_width(520.0);
				ui.label(egui::RichText::new(title).font(f_semi(18.0)).color(p.text));
				ui.add_space(8.0);
				ui.label(egui::RichText::new(&text).font(f_body(14.0)).color(p.text));
				if let Some((label, note, on)) = check.as_mut() {
					ui.add_space(12.0);
					checkbox(ui, p, on, label, 18.0, f_semi(14.0));
					ui.add_space(4.0);
					ui.label(egui::RichText::new(note.as_str()).font(f_body(13.0)).color(p.muted));
				}
				ui.add_space(16.0);
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					if ask.is_some() {
						if primary_button_icon(ui, p, "Yes", Some("check"), Vec2::new(96.0, 34.0), 8, f_semi(15.0), true).clicked() {
							result = ask.clone();
							close = true;
						}
						if small_button(ui, p, "No", Some("x"), 34.0, true, p.text).clicked() {
							close = true;
						}
					} else if primary_button_icon(ui, p, "OK", Some("check"), Vec2::new(96.0, 34.0), 8, f_semi(15.0), true).clicked() {
						close = true;
					}
				});
			});
	});
	if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
		close = true;
	}
	if ask.is_none() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
		close = true;
	}
	if let (Some((_, _, v)), Dialog::Check(_, _, _, on)) = (check, &mut *d) {
		*on = v;
	}
	if close {
		*d = Dialog::None;
	}
	result
}

pub fn open_folder(path: &str) {
	let p = path.trim();
	if p.is_empty() {
		return;
	}
	let _ = std::fs::create_dir_all(p);
	#[cfg(windows)]
	let _ = std::process::Command::new("explorer").arg(p).spawn();
	#[cfg(not(windows))]
	let _ = std::process::Command::new("xdg-open").arg(p).spawn();
}

#[cfg(windows)]
static MAIN_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

pub fn set_dialog_owner(cc: &eframe::CreationContext) {
	#[cfg(windows)]
	{
		use raw_window_handle::{HasWindowHandle, RawWindowHandle};
		if let Ok(h) = cc.window_handle()
			&& let RawWindowHandle::Win32(w) = h.as_raw() {
				MAIN_HWND.store(w.hwnd.get(), std::sync::atomic::Ordering::Relaxed);
			}
	}
	#[cfg(not(windows))]
	let _ = cc;
}

pub fn notify_done(title: &str, text: &str) {
	#[cfg(windows)]
	{
		use windows_sys::Win32::UI::Shell::{ExtractIconW, Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_TIP, NIIF_INFO, NIIF_LARGE_ICON, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW};
		use windows_sys::Win32::UI::WindowsAndMessaging::{FlashWindowEx, DestroyIcon, FLASHWINFO, FLASHW_TIMERNOFG, FLASHW_TRAY};
		let hwnd = MAIN_HWND.load(std::sync::atomic::Ordering::Relaxed);
		if hwnd == 0 {
			return;
		}
		let wide = |s: &str, out: &mut [u16]| {
			let n = out.len() - 1;
			for (d, c) in out.iter_mut().zip(s.encode_utf16().take(n)) {
				*d = c;
			}
		};
		let exe: Vec<u16> = std::env::current_exe().unwrap_or_default().as_os_str().encode_wide_null();
		let (title, text) = (title.to_string(), text.to_string());
		std::thread::spawn(move || unsafe {
			let mut nd: NOTIFYICONDATAW = std::mem::zeroed();
			nd.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
			nd.hWnd = hwnd as _;
			nd.uID = 0x4B54;
			nd.uFlags = NIF_ICON | NIF_TIP | NIF_INFO;
			nd.hIcon = ExtractIconW(std::ptr::null_mut(), exe.as_ptr(), 0);
			nd.dwInfoFlags = NIIF_INFO | NIIF_LARGE_ICON;
			wide(&title, &mut nd.szTip);
			wide(&title, &mut nd.szInfoTitle);
			wide(&text, &mut nd.szInfo);
			Shell_NotifyIconW(NIM_DELETE, &nd);
			Shell_NotifyIconW(NIM_ADD, &nd);
			let fl = FLASHWINFO { cbSize: std::mem::size_of::<FLASHWINFO>() as u32, hwnd: hwnd as _, dwFlags: FLASHW_TRAY | FLASHW_TIMERNOFG, uCount: 0, dwTimeout: 0 };
			FlashWindowEx(&fl);
			std::thread::sleep(std::time::Duration::from_secs(20));
			Shell_NotifyIconW(NIM_DELETE, &nd);
			if !nd.hIcon.is_null() {
				DestroyIcon(nd.hIcon);
			}
		});
	}
	#[cfg(not(windows))]
	let _ = (title, text);
}

#[cfg(windows)]
trait WideNull {
	fn encode_wide_null(&self) -> Vec<u16>;
}
#[cfg(windows)]
impl WideNull for std::ffi::OsStr {
	fn encode_wide_null(&self) -> Vec<u16> {
		use std::os::windows::ffi::OsStrExt;
		self.encode_wide().chain([0]).collect()
	}
}

#[cfg(windows)]
struct Owner(isize);
#[cfg(windows)]
impl raw_window_handle::HasWindowHandle for Owner {
	fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
		let h = std::num::NonZeroIsize::new(self.0).ok_or(raw_window_handle::HandleError::Unavailable)?;
		let raw = raw_window_handle::RawWindowHandle::Win32(raw_window_handle::Win32WindowHandle::new(h));
		Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
	}
}
#[cfg(windows)]
impl raw_window_handle::HasDisplayHandle for Owner {
	fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
		Ok(raw_window_handle::DisplayHandle::windows())
	}
}
#[cfg(windows)]
fn with_owner(d: rfd::FileDialog) -> rfd::FileDialog {
	let h = MAIN_HWND.load(std::sync::atomic::Ordering::Relaxed);
	if h != 0 { d.set_parent(&Owner(h)) } else { d }
}

pub fn pick_folder_with_files(start: &str, title: &str) -> Option<PathBuf> {
	#[cfg(windows)]
	{
		use std::os::windows::ffi::{OsStrExt, OsStringExt};
		use windows_sys::Win32::UI::Controls::Dialogs::*;
		let wide = |s: &std::ffi::OsStr| s.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
		let mut name = vec![0u16; 4096];
		let here = wide("This folder".as_ref());
		name[..here.len()].copy_from_slice(&here);
		let filter: Vec<u16> = format!("{}\0*.*\0\0", "All files").encode_utf16().collect();
		let dir = wide(start.trim().as_ref());
		let title = wide(title.as_ref());
		let mut o: OPENFILENAMEW = unsafe { std::mem::zeroed() };
		o.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
		o.hwndOwner = MAIN_HWND.load(std::sync::atomic::Ordering::Relaxed) as _;
		o.lpstrFilter = filter.as_ptr();
		o.lpstrFile = name.as_mut_ptr();
		o.nMaxFile = name.len() as u32;
		if std::path::Path::new(start.trim()).is_dir() {
			o.lpstrInitialDir = dir.as_ptr();
		}
		o.lpstrTitle = title.as_ptr();
		o.Flags = OFN_EXPLORER | OFN_NOVALIDATE | OFN_HIDEREADONLY | OFN_NOCHANGEDIR | OFN_NOTESTFILECREATE;
		if unsafe { GetOpenFileNameW(&mut o) } == 0 {
			return None;
		}
		let n = name.iter().position(|&c| c == 0).unwrap_or(0);
		picked_path(std::ffi::OsString::from_wide(&name[..n]).into())
	}
	#[cfg(not(windows))]
	{
		let _ = (start, title);
		None
	}
}

pub fn picked_path(p: PathBuf) -> Option<PathBuf> {
	if p.exists() { Some(p) } else { p.parent().filter(|d| d.is_dir()).map(PathBuf::from) }
}

pub fn pick_folder(start: &str) -> Option<PathBuf> {
	#[cfg(windows)]
	{
		let mut d = with_owner(rfd::FileDialog::new());
		if std::path::Path::new(start).is_dir() {
			d = d.set_directory(start);
		}
		d.pick_folder()
	}
	#[cfg(not(windows))]
	{
		let _ = start;
		None
	}
}

pub fn pick_files(start: &str, title: &str, filter_name: &str, exts: &[&str], multi: bool) -> Vec<PathBuf> {
	#[cfg(windows)]
	{
		let mut d = with_owner(rfd::FileDialog::new()).set_title(title).add_filter(filter_name, exts).add_filter("All files", &["*"]);
		if std::path::Path::new(start).is_dir() {
			d = d.set_directory(start);
		}
		if multi { d.pick_files().unwrap_or_default() } else { d.pick_file().into_iter().collect() }
	}
	#[cfg(not(windows))]
	{
		let _ = (start, title, filter_name, exts, multi);
		Vec::new()
	}
}

#[cfg(windows)]
fn base64(b: &[u8]) -> String {
	const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
	let mut s = String::with_capacity(b.len().div_ceil(3) * 4);
	for ch in b.chunks(3) {
		let n = (ch[0] as u32) << 16 | (*ch.get(1).unwrap_or(&0) as u32) << 8 | *ch.get(2).unwrap_or(&0) as u32;
		for k in 0..4 {
			s.push(if k <= ch.len() { T[(n >> (18 - 6 * k) & 63) as usize] as char } else { '=' });
		}
	}
	s
}

fn vs_code() -> Option<PathBuf> {
	let mut c = Vec::new();
	if let Some(d) = std::env::var_os("LOCALAPPDATA") {
		c.push(PathBuf::from(d).join("Programs").join("Microsoft VS Code").join("Code.exe"));
	}
	for v in ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"] {
		if let Some(d) = std::env::var_os(v) {
			c.push(PathBuf::from(d).join("Microsoft VS Code").join("Code.exe"));
		}
	}
	c.into_iter().find(|p| p.is_file())
}

fn notepad_pp() -> Option<PathBuf> {
	let mut cands: Vec<PathBuf> = Vec::new();
	for v in ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"] {
		if let Some(d) = std::env::var_os(v) {
			cands.push(PathBuf::from(d).join("Notepad++").join("notepad++.exe"));
		}
	}
	if let Some(c) = cands.into_iter().find(|p| p.is_file()) {
		return Some(c);
	}
	#[cfg(windows)]
	{
		use std::os::windows::ffi::{OsStrExt, OsStringExt};
		use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
		let key: Vec<u16> = std::ffi::OsStr::new(r"SOFTWARE\Notepad++").encode_wide().chain([0]).collect();
		let mut buf = vec![0u16; 1024];
		let mut size = (buf.len() * 2) as u32;
		let r = unsafe { RegGetValueW(HKEY_LOCAL_MACHINE, key.as_ptr(), std::ptr::null(), RRF_RT_REG_SZ, std::ptr::null_mut(), buf.as_mut_ptr().cast(), &mut size) };
		if r != 0 {
			return None;
		}
		let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
		let p = PathBuf::from(std::ffi::OsString::from_wide(&buf[..n])).join("notepad++.exe");
		p.is_file().then_some(p)
	}
	#[cfg(not(windows))]
	None
}

pub fn save_file(start: &str, title: &str, name: &str, filter_name: &str, exts: &[&str]) -> Option<PathBuf> {
	#[cfg(windows)]
	{
		let mut d = with_owner(rfd::FileDialog::new()).set_title(title).set_file_name(name).add_filter(filter_name, exts);
		if std::path::Path::new(start).is_dir() {
			d = d.set_directory(start);
		}
		d.save_file()
	}
	#[cfg(not(windows))]
	{
		let _ = (start, title, name, filter_name, exts);
		None
	}
}

pub fn icon_from(png: &[u8]) -> egui::IconData {
	match image::load_from_memory(png) {
		Ok(im) => {
			let im = im.to_rgba8();
			egui::IconData { width: im.width(), height: im.height(), rgba: im.into_raw() }
		}
		Err(_) => window_icon(),
	}
}

pub fn window_icon() -> egui::IconData {
	let n = 32usize;
	let mut rgba = vec![0u8; n * n * 4];
	for y in 0..n {
		for x in 0..n {
			let inside = (3..29).contains(&x) && (3..29).contains(&y);
			let corner = |cx: f32, cy: f32| ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt() > 5.0;
			let cut = (x < 8 && y < 8 && corner(8.0, 8.0)) || (x >= 24 && y < 8 && corner(24.0, 8.0)) || (x < 8 && y >= 24 && corner(8.0, 24.0)) || (x >= 24 && y >= 24 && corner(24.0, 24.0));
			if inside && !cut {
				let i = (y * n + x) * 4;
				rgba[i..i + 4].copy_from_slice(&[0x59, 0x80, 0xa6, 255]);
			}
		}
	}
	egui::IconData { rgba, width: n as u32, height: n as u32 }
}

pub struct TreeLine {
	pub depth: u8,
	pub name: String,
	pub section: bool,
	pub note: String,
}

pub fn paint_tree(ui: &Ui, p: &Pal, r: Rect, lines: &[TreeLine]) {
	let painter = ui.painter();
	painter.rect(r, 10, Color32::TRANSPARENT, Stroke::new(1.0_f32, p.div), StrokeKind::Inside);
	let lh = 22.0;
	let fit = (((r.height() - 16.0) / lh).floor() as usize).max(1);
	let show = if lines.len() > fit { fit - 1 } else { lines.len() };
	let clip = painter.with_clip_rect(r.shrink(2.0));
	let mut y = r.min.y + 8.0;
	for l in &lines[..show] {
		let x = r.min.x + 14.0 + l.depth as f32 * 22.0;
		let cy = y + lh / 2.0;
		paint_icon(ui, "folder", Rect::from_center_size(Pos2::new(x + 7.0, cy), Vec2::splat(15.0)), if l.section { p.accent } else { p.muted });
		let g = clip.text(Pos2::new(x + 22.0, cy), Align2::LEFT_CENTER, &l.name, if l.section { f_semi(13.5) } else { f_body(13.5) }, if l.section { p.accent } else { p.text });
		if !l.note.is_empty() {
			clip.text(Pos2::new(g.max.x + 10.0, cy), Align2::LEFT_CENTER, &l.note, f_body(12.5), p.muted);
		}
		y += lh;
	}
	if lines.len() > show {
		clip.text(Pos2::new(r.min.x + 36.0, y + lh / 2.0), Align2::LEFT_CENTER, format!("… and {} more", lines.len() - show), f_body(12.5), p.muted);
	}
}

