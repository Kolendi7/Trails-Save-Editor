use super::*;

const INK: Color32 = Color32::from_rgb(60, 34, 12);
const PALE: Color32 = Color32::from_rgb(158, 146, 134);
const PAPER: Color32 = Color32::from_rgb(236, 233, 226);
const ROW: f32 = 46.0;
const BOOK: Vec2 = Vec2::new(736.0, 502.0);
const PAGE: Vec2 = Vec2::new(19.0, 19.0);

enum Edit {
	Mark(usize, bool),
	Record(usize, u32),
	Caught(usize, u32),
	Medal(usize, bool),
}

fn panel(c: &Canvas, x0: f32, y0: f32, x1: f32, y1: f32) {
	const E: f32 = 34.0;
	let xs = [(740.0, 740.0 + E, x0, x0 + E), (740.0 + E, 888.0 - E, x0 + E, x1 - E), (888.0 - E, 888.0, x1 - E, x1)];
	let ys = [(0.0, E, y0, y0 + E), (E, 148.0 - E, y0 + E, y1 - E), (148.0 - E, 148.0, y1 - E, y1)];
	for (a, b, x, x2) in xs {
		for (e, f, y, y2) in ys {
			c.img([a, e, b, f], c.rect(x, y, x2, y2));
		}
	}
}

fn vline(c: &Canvas, x: f32, y0: f32, y1: f32) {
	c.img([758.0, 154.0, 782.0, 210.0], c.rect(x - 12.0, y0, x + 12.0, y0 + 56.0));
	c.img([822.0, 170.0, 846.0, 200.0], c.rect(x - 12.0, y0 + 56.0, x + 12.0, y1 - 56.0));
	c.img([886.0, 164.0, 910.0, 220.0], c.rect(x - 12.0, y1 - 56.0, x + 12.0, y1));
}

impl App {
	pub(super) fn fish_tex(&mut self, ctx: &egui::Context, file: &str) -> Option<egui::TextureHandle> {
		let name = format!("{}/{file}", self.g.dir);
		if let Some(t) = self.art_tex.get(&name) {
			return t.clone();
		}
		let t = image::load_from_memory(&self.table(file)).ok().map(|img| ctx.load_texture(&name, premultiplied(&img.to_rgba8()), FACE_TEX));
		self.art_tex.insert(name, t.clone());
		t
	}

	fn fish_item(&self, id: u32) -> String {
		match id {
			990..=996 => format!("{} Sepith", SEPITH[id as usize - 990]),
			997 => "All Types of Sepith".into(),
			_ => self.item_of.get(&id).map_or(format!("#{id}"), |&k| self.items[k].name.clone()),
		}
	}

	pub(super) fn fishing(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let (g, p) = (self.g, self.pal);
		if self.fish.is_empty() {
			ui.label(egui::RichText::new("No fish list for this game in data.dat next to the program.").font(f_body(13.0)).color(p.muted));
			return;
		}
		let note = self.fish_tex(ctx, "fishnote.png");
		let pic = |i: usize| self.pics.pack(ctx, &format!("{}/fish/{i}.webp", g.dir), FACE_TEX, premultiplied);
		let Some(d) = self.data.as_ref() else { return };
		let n = self.fish.len();
		let top = usize::from(!fish::medals(g).is_empty());
		let (types, times) = fish::counts(g, d, &self.fish);
		let mut edits: Vec<Edit> = Vec::new();
		let mut sel = self.fish_sel.min(n + top - 1);
		ui.horizontal(|ui| {
			ui.label(egui::RichText::new(format!("Types Caught  {types} / {n}")).font(f_semi(14.0)).color(p.text));
			ui.add_space(16.0);
			ui.label(egui::RichText::new(format!("Times Caught  {times}")).font(f_semi(14.0)).color(p.text));
			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				if small_button(ui, &p, "Clear all", None, 28.0, true, p.muted).clicked() {
					edits.extend((0..n).map(|i| Edit::Mark(i, false)));
				}
				if small_button(ui, &p, "Catch all", None, 28.0, true, p.text).on_hover_text("Caught once at the smallest size; locations, bait and items revealed. Higher existing values stay.").clicked() {
					edits.extend((0..n).map(|i| Edit::Mark(i, true)));
				}
			});
		});
		ui.add_space(6.0);
		let h = ui.available_height();
		ui.horizontal_top(|ui| {
			ui.allocate_ui_with_layout(Vec2::new(290.0, h), egui::Layout::top_down(egui::Align::Min), |ui| {
				egui::Frame::new().fill(p.surface).corner_radius(8).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
					ui.set_min_size(Vec2::new(274.0, h - 16.0));
					egui::ScrollArea::vertical().id_salt(("flist", g.dir)).auto_shrink([false, false]).show(ui, |ui| {
						ui.spacing_mut().item_spacing.y = 4.0;
						if top == 1 {
							let (r, resp) = list_row(ui, &p, ROW - 4.0, sel == 0);
							ui.painter().text(r.center(), Align2::CENTER_CENTER, "TOP", f_semi(16.0), p.text);
							if resp.clicked() {
								sel = 0;
							}
						}
						for (i, x) in self.fish.iter().enumerate() {
							let on = fish::rec(g, d, x).caught > 0;
							ui.horizontal(|ui| {
								ui.set_height(ROW);
								let tip = if on { "Caught — click to remove" } else { "Not caught — click: caught once, everything revealed" };
								if tick(ui, if on { 2 } else { 0 }).on_hover_text(tip).clicked() {
									edits.push(Edit::Mark(i, !on));
								}
								let (r, resp) = list_row(ui, &p, ROW, sel == i + top);
								let fr = Rect::from_min_size(r.min + Vec2::new(4.0, 4.0), Vec2::new(84.0, ROW - 8.0));
								ui.painter().rect_filled(fr, 4, if on { PAPER } else { PAPER.gamma_multiply(0.45) });
								if ui.is_rect_visible(fr)
									&& let Some(t) = pic(x.idx)
								{
									let q = fr.shrink2(Vec2::new(4.0, 2.0));
									let z = Vec2::new(400.0, 225.0);
									let z = z * (q.width() / z.x).min(q.height() / z.y);
									ui.painter().image(t.id(), Rect::from_center_size(q.center(), z), UV_ALL, if on { Color32::WHITE } else { Color32::from_white_alpha(150) });
								}
								let x0 = fr.max.x + 10.0;
								let nm = self.fish_item(x.item);
								let w = ui.painter().layout_no_wrap(nm.clone(), f_body(15.0), p.text).size().x;
								let size = 15.0 * ((r.max.x - 8.0 - x0) / w.max(1.0)).min(1.0);
								ui.painter().text(Pos2::new(x0, r.center().y), Align2::LEFT_CENTER, nm, f_body(size), if on { p.text } else { p.muted });
								if resp.clicked() {
									sel = i + top;
								}
							});
						}
					}).middle_scroll(ui);
				});
			});
			let (area, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::hover());
			ui.painter().rect_filled(area, 8, p.surface);
			let Some(tex) = &note else {
				ui.painter().text(area.center(), Align2::CENTER_CENTER, "No notebook pictures (data.dat)", f_body(14.0), p.muted);
				return;
			};
			let k = (area.width() / BOOK.x).min(area.height() / BOOK.y);
			let book = Rect::from_center_size(area.center(), BOOK * k);
			let c = Canvas { pt: ui.painter_at(book), tex, o: book.min, base: Pos2::ZERO, k };
			c.img([0.0, 0.0, BOOK.x, BOOK.y], book);
			if top == 1 && sel == 0 {
				fish_top(ui, &c, d, (types, times), &self.items, &self.item_of, &mut edits);
			} else {
				let i = sel - top;
				let x = &self.fish[i];
				let c = Canvas { pt: c.pt.clone(), tex, o: book.min + PAGE * k, base: Pos2::ZERO, k: k / 2.0 };
				self.fish_card(ui, &c, pic(x.idx).as_ref(), d, i, &mut edits);
			}
		});
		self.fish_sel = sel;
		if edits.is_empty() {
			return;
		}
		let d = self.data.as_mut().unwrap();
		for e in edits {
			match e {
				Edit::Mark(i, on) => fish::mark(g, d, &self.fish[i], on),
				Edit::Record(i, v) => fish::set_record(g, d, &self.fish[i], v),
				Edit::Caught(i, v) => fish::set_caught(g, d, &self.fish[i], v),
				Edit::Medal(m, on) => fish::set_medal(g, d, &fish::medals(g)[m], on),
			}
		}
		let (types, times) = fish::counts(g, d, &self.fish);
		self.status = format!("Fish types: {types}, times caught: {times}");
	}

	fn fish_card(&self, ui: &mut egui::Ui, c: &Canvas, pic: Option<&egui::TextureHandle>, d: &[u8], i: usize, edits: &mut Vec<Edit>) {
		let (g, x) = (self.g, &self.fish[i]);
		let r = fish::rec(g, d, x);
		let on = r.caught > 0;
		let name = self.fish_item(x.item);
		c.text(114.0, 80.0, Align2::LEFT_CENTER, &name, 42.0, INK, false);
		let w = c.pt.layout_no_wrap(name, c.font(42.0, false), INK).size().x / c.k;
		for cx in [90.0, 138.0 + w] {
			let q = [(0.0, -20.0), (20.0, 0.0), (0.0, 20.0), (-20.0, 0.0)].map(|(a, b)| c.at(cx + a, 82.0 + b));
			c.pt.add(egui::Shape::convex_polygon(q.to_vec(), INK, Stroke::NONE));
		}
		for (a, b, x0, x1) in [(740.0, 780.0, 60.0, 100.0), (800.0, 860.0, 100.0, 1310.0), (894.0, 934.0, 1310.0, 1350.0)] {
			c.img([a, 232.0, b, 258.0], c.rect(x0, 108.0, x1, 134.0));
		}
		if let Some(t) = pic {
			let z = Vec2::new(400.0, 225.0) * (400.0 / 392.0) * c.k;
			c.pt.image(t.id(), Rect::from_center_size(c.at(275.0, 300.0), z), UV_ALL, if on { Color32::WHITE } else { Color32::from_black_alpha(130) });
		}
		panel(c, 490.0, 148.0, 1330.0, 460.0);
		vline(c, 800.0, 176.0, 432.0);
		let field = |ui: &mut egui::Ui, y: f32, dv: egui::DragValue<'_>| {
			let b = c.rect(548.0, y, 664.0, y + 46.0);
			let mut ch = ui.new_child(egui::UiBuilder::new().max_rect(b));
			let v = ch.visuals_mut();
			v.override_text_color = Some(INK);
			v.extreme_bg_color = Color32::from_rgb(250, 249, 246);
			for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active] {
				w.weak_bg_fill = Color32::from_rgb(250, 249, 246);
				w.bg_fill = w.weak_bg_fill;
				w.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(150, 135, 120));
			}
			ch.style_mut().override_font_id = Some(c.font(27.0, true));
			ch.spacing_mut().interact_size = Vec2::ZERO;
			ch.spacing_mut().button_padding = Vec2::new(4.0 * c.k, 0.0);
			ch.add_sized(b.size(), dv).changed()
		};
		c.text(528.0, 196.0, Align2::LEFT_CENTER, "[Record]", 32.0, INK, false);
		let mut v = r.record;
		if field(ui, 222.0, egui::DragValue::new(&mut v).clamp_existing_to_range(false).range(if on { x.min } else { 0 }..=x.max).speed(0.1)) && v != r.record {
			edits.push(Edit::Record(i, v));
		}
		c.text(678.0, 245.0, Align2::LEFT_CENTER, "rege", 26.0, INK, false);
		c.text(528.0, 286.0, Align2::LEFT_CENTER, &format!("{} ~ {}", x.min, x.max), 20.0, PALE, false);
		c.text(528.0, 326.0, Align2::LEFT_CENTER, "[Caught]", 32.0, INK, false);
		let mut v = r.caught;
		if field(ui, 352.0, egui::DragValue::new(&mut v).clamp_existing_to_range(false).range(0..=9999).speed(0.1)) && v != r.caught {
			edits.push(Edit::Caught(i, v));
		}
		c.text(678.0, 375.0, Align2::LEFT_CENTER, "times", 26.0, INK, false);
		let mut lines: Vec<(&str, bool)> = r.places.iter().map(|&p| (x.place(p), true)).collect();
		for (_, t) in &x.places {
			if !lines.iter().any(|l| l.0 == t) {
				lines.push((t, false));
			}
		}
		let step = (214.0 / lines.len() as f32).min(48.0);
		c.text(830.0, 196.0, Align2::LEFT_CENTER, "[Locations Found]", 32.0, INK, false);
		for (n, (t, known)) in lines.iter().enumerate() {
			c.fit(850.0, 220.0 + (n as f32 + 0.5) * step, Align2::LEFT_CENTER, t, 29.0f32.min(step * 0.72), 466.0, if *known { INK } else { PALE }, false);
		}
		panel(c, 66.0, 490.0, 1330.0, 850.0);
		vline(c, 608.0, 520.0, 822.0);
		vline(c, 942.0, 520.0, 822.0);
		let mut job = egui::text::LayoutJob::simple(x.desc.clone(), c.font(28.0, false), INK, 486.0 * c.k);
		job.sections.iter_mut().for_each(|s| s.format.line_height = Some(46.0 * c.k));
		let galley = c.pt.layout_job(job);
		c.pt.galley(c.at(104.0, 518.0), galley, INK);
		c.text(638.0, 540.0, Align2::LEFT_CENTER, "[Bait Used]", 32.0, INK, false);
		let mut baits: Vec<u32> = x.baits.iter().chain(&r.baits).copied().collect();
		baits.sort_unstable();
		baits.dedup();
		for (n, &b) in baits.iter().enumerate() {
			c.fit(658.0, 592.0 + n as f32 * 44.0, Align2::LEFT_CENTER, &self.fish_item(b), 29.0, 270.0, if r.baits.contains(&b) { INK } else { PALE }, false);
		}
		c.text(972.0, 540.0, Align2::LEFT_CENTER, "[Items Gained]", 32.0, INK, false);
		let mut gains: Vec<(u32, bool)> = Vec::new();
		for q in (0..4).filter(|&q| x.gains[q] != 0) {
			let got = r.gained >> (q + 1) & 1 == 1;
			match gains.iter_mut().find(|e| e.0 == x.gains[q]) {
				Some(e) => e.1 |= got,
				None => gains.push((x.gains[q], got)),
			}
		}
		for (n, &(id, got)) in gains.iter().enumerate() {
			let y = 592.0 + n as f32 * 48.0;
			let t = self.fish_item(id);
			c.fit(992.0, y, Align2::LEFT_CENTER, if got { &t } else { "?????" }, 29.0, 320.0, INK, false);
			if !got {
				ui.interact(c.rect(990.0, y - 22.0, 1310.0, y + 22.0), ui.id().with(("gain", i, n)), Sense::hover()).on_hover_text(t);
			}
		}
	}
}

fn fish_top(ui: &mut egui::Ui, book: &Canvas, d: &[u8], (types, times): (usize, u32), items: &[save::Item], item_of: &HashMap<u32, usize>, edits: &mut Vec<Edit>) {
	let k = book.k * 699.0 / 692.0;
	let c = Canvas { pt: book.pt.clone(), tex: book.tex, o: book.o + PAGE * book.k, base: Pos2::ZERO, k };
	c.img([1.0, 505.0, 691.0, 967.0], c.rect(0.0, 0.0, 692.0, 461.0));
	let t = |x: f32, y: f32, a: Align2, s: &str, size: f32| c.text(x, y, a, s, size, INK, false);
	t(344.5, 46.5, Align2::LEFT_CENTER, "Types Caught", 19.0);
	t(631.0, 46.5, Align2::RIGHT_CENTER, &format!("{types} types"), 19.0);
	t(344.5, 85.5, Align2::LEFT_CENTER, "Times Caught", 19.0);
	t(631.0, 85.5, Align2::RIGHT_CENTER, &format!("{times} times"), 19.0);
	t(42.5, 326.0, Align2::LEFT_CENTER, "[Name]", 13.5);
	t(42.5, 352.0, Align2::LEFT_CENTER, "Lloyd Bannings", 16.5);
	t(42.5, 385.0, Align2::LEFT_CENTER, "[Affiliation]", 13.5);
	t(42.5, 411.0, Align2::LEFT_CENTER, "Fisherman's Guild, Crossbell Branch", 16.5);
	t(352.0, 265.0, Align2::LEFT_CENTER, "[Fisherman's Guidelines]", 16.0);
	for (n, (a, b)) in [("Be still as water.", "Let nothing move you."), ("Be dauntless.", "Cast away your regrets."), ("Be transcendent.", "Allow your heart to be free.")].into_iter().enumerate() {
		let y = 297.0 + n as f32 * 53.0;
		c.pt.circle_stroke(c.at(361.0, y + 0.5), 7.5 * k, Stroke::new((0.9 * k).max(1.0), INK));
		t(361.0, y + 0.5, Align2::CENTER_CENTER, &(n + 1).to_string(), 11.5);
		t(370.5, y, Align2::LEFT_CENTER, a, 16.0);
		t(370.5, y + 25.5, Align2::LEFT_CENTER, b, 16.0);
	}
	let cell = |n: usize| {
		let (x, y) = (740.0 + (n % 4) as f32 * 64.0, 264.0 + (n / 4) as f32 * 64.0);
		[x, y, x + 64.0, y + 64.0]
	};
	for (n, m) in fish::MEDALS.iter().enumerate() {
		let (x, y) = m.at;
		let r = c.rect(x - 30.5, y - 30.5, x + 30.5, y + 30.5);
		let f = cell(5);
		c.img([f[0], f[1], f[0] + 62.0, f[1] + 62.0], r);
		let on = fish::has_medal(d, m);
		let resp = ui.interact(r, ui.id().with(("medal", n)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
		if on || resp.hovered() {
			c.pt.image(c.tex.id(), c.rect(x - 27.0, y - 27.0, x + 27.0, y + 27.0), c.uv(cell(n)), if on { Color32::WHITE } else { Color32::from_white_alpha(70) });
		}
		let name = item_of.get(&m.item).map_or(format!("#{}", m.item), |&k| items[k].name.clone());
		let tip = if on {
			format!("{name} — click to remove the medal: the duel victory flag and the item")
		} else {
			format!("{name} — click to get the medal: the game treats this duel as won and the item is added")
		};
		if resp.on_hover_text(tip).clicked() {
			edits.push(Edit::Medal(n, !on));
		}
	}
}

