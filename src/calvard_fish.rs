use super::*;

const INK: Color32 = Color32::from_rgb(46, 44, 42);
const GREY: Color32 = Color32::from_rgb(120, 118, 114);
const FRAME: Color32 = Color32::from_rgb(142, 124, 108);
const PAGE: Color32 = Color32::from_rgb(229, 229, 227);
const LINE: Color32 = Color32::from_rgb(150, 148, 144);
const ROW: f32 = 46.0;

impl Canvas<'_> {
	fn grid(&self, x0: f32, y0: f32, x1: f32, y1: f32) {
		self.pt.rect_filled(self.rect(x0, y0, x1, y1), 0, Color32::from_rgb(230, 230, 228));
		let mut y = y0;
		while y < y1 {
			let mut x = x0;
			while x < x1 {
				let (w, h) = ((x1 - x).min(64.0), (y1 - y).min(64.0));
				self.img([1100.0, 1040.0, 1100.0 + w, 1040.0 + h], self.rect(x, y, x + w, y + h));
				x += 64.0;
			}
			y += 64.0;
		}
		self.pt.rect_stroke(self.rect(x0, y0, x1, y1), 0, Stroke::new((3.0 * self.k).max(1.0), FRAME), StrokeKind::Inside);
	}
	fn hline(&self, x0: f32, x1: f32, y: f32) {
		self.pt.line_segment([self.at(x0, y), self.at(x1, y)], Stroke::new((1.5 * self.k).max(1.0), LINE));
	}
	fn bar(&self, s: [f32; 4], x: f32, y: f32, w: f32, h: f32) {
		let e = if w < 200.0 { 50.0 } else { 70.0 };
		for (a, b, xa, xb) in [(s[0], s[0] + e, x, x + e), (s[0] + e, s[2] - e, x + e, x + w - e), (s[2] - e, s[2], x + w - e, x + w)] {
			self.img([a, s[1], b, s[3]], self.rect(xa, y, xb, y + h));
		}
	}
}

enum Edit {
	Mark(usize, bool),
	RankA(usize, bool),
	Rec(usize, f32, u32, u16),
	Gear(usize, bool),
}

impl App {
	pub(super) fn has_fishing(&self) -> bool {
		save::fishing(self.g).is_some() && !self.fish.is_empty()
	}

	pub(super) fn fishing(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let Some(f) = save::fishing(self.g) else { return };
		let (g, p) = (self.g, self.pal);
		let icons = self.icons(ctx);
		let note = self.art(ctx, "ui:fishnote.png");
		let pic = |i: usize| self.pics.pack(ctx, &format!("{}/ui/fish/{i}.webp", g.dir), FACE_TEX, premultiplied);
		let Some(d) = self.data.as_ref() else { return };
		let n = self.fish.len();
		self.fish_sel = self.fish_sel.min(n);
		let counts = save::fish_counts(g, f, d, &self.fish);
		let mut edits: Vec<Edit> = Vec::new();
		let mut sel = self.fish_sel;
		ui.horizontal(|ui| {
			ui.label(egui::RichText::new(format!("Types of Fish Caught  {} / {n}", counts[0])).font(f_semi(14.0)).color(p.text));
			ui.add_space(16.0);
			ui.label(egui::RichText::new(format!("Rank A  {} / {n}", counts[1])).font(f_semi(14.0)).color(p.text));
			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				if small_button(ui, &p, "Clear all", None, 28.0, true, p.muted).clicked() {
					edits.extend((0..n).map(|i| Edit::Mark(i, false)));
				}
				if small_button(ui, &p, "Catch all", None, 28.0, true, p.text).on_hover_text("Caught once, rank A. Higher existing values stay.").clicked() {
					edits.extend((0..n).map(|i| Edit::Mark(i, true)));
				}
			});
		});
		ui.add_space(6.0);
		let h = ui.available_height();
		let name = |id: usize| self.items.iter().find(|it| it.id == id).map_or(format!("#{id}"), |it| it.en.clone());
		ui.horizontal_top(|ui| {
			ui.allocate_ui_with_layout(Vec2::new(290.0, h), egui::Layout::top_down(egui::Align::Min), |ui| {
				egui::Frame::new().fill(p.surface).corner_radius(8).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
					ui.set_min_size(Vec2::new(274.0, h - 16.0));
					egui::ScrollArea::vertical().id_salt("flist").auto_shrink([false, false]).show(ui, |ui| {
						ui.spacing_mut().item_spacing.y = 4.0;
						let (r, resp) = list_row(ui, &p, ROW - 4.0, sel == 0);
						ui.painter().text(r.center(), Align2::CENTER_CENTER, f.first, f_semi(16.0), p.text);
						if resp.clicked() {
							sel = 0;
						}
						for (i, x) in self.fish.iter().enumerate() {
							let (rec, cnt, _) = save::fish_rec(f, d, i);
							ui.horizontal(|ui| {
								ui.set_height(ROW);
								let tip = if cnt > 0 { "Caught — click to remove" } else { "Not caught — click: caught once, rank A" };
								if tick(ui, if cnt > 0 { 2 } else { 0 }).on_hover_text(tip).clicked() {
									edits.push(Edit::Mark(i, cnt == 0));
								}
								let (r, resp) = list_row(ui, &p, ROW, sel == i + 1);
								let fr = Rect::from_min_size(r.min + Vec2::new(4.0, 4.0), Vec2::new(84.0, ROW - 8.0));
								ui.painter().rect_filled(fr, 4, if cnt > 0 { PAGE } else { PAGE.gamma_multiply(0.45) });
								if ui.is_rect_visible(fr)
									&& let Some(t) = pic(i)
								{
									let q = fr.shrink2(Vec2::new(4.0, 2.0));
									let z = Vec2::new(512.0, 288.0);
									let z = z * (q.width() / z.x).min(q.height() / z.y);
									ui.painter().image(t.id(), Rect::from_center_size(q.center(), z), UV_ALL, if cnt > 0 { Color32::WHITE } else { Color32::from_white_alpha(150) });
								}
								let x0 = fr.max.x + 10.0;
								let nm = name(x.item);
								let w = ui.painter().layout_no_wrap(nm.clone(), f_body(15.0), p.text).size().x;
								let size = 15.0 * ((r.max.x - 24.0 - x0) / w.max(1.0)).min(1.0);
								ui.painter().text(Pos2::new(x0, r.center().y), Align2::LEFT_CENTER, nm, f_body(size), if cnt > 0 { p.text } else { p.muted });
								if x.is_a(rec, cnt) {
									ui.painter().text(Pos2::new(r.max.x - 12.0, r.center().y), Align2::CENTER_CENTER, "A", f_semi(14.0), p.gold);
								}
								if resp.clicked() {
									sel = i + 1;
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
			if sel == 0 {
				self.fish_page(ui, area, tex, f, d, counts, &mut sel, &mut edits);
			} else {
				let i = sel - 1;
				self.fish_card(ui, area, tex, pic(i).as_ref(), &icons, f, d, i, &mut edits);
			}
		});
		self.fish_sel = sel;
		if edits.is_empty() {
			return;
		}
		let d = self.data.as_mut().unwrap();
		let before: Vec<u8> = self.achs.iter().map(|a| save::ach_state(g, d, a.id)).collect();
		for e in edits {
			match e {
				Edit::Mark(i, on) => save::mark_fish(f, d, i, &self.fish[i], on),
				Edit::RankA(i, on) => save::set_rank_a(f, d, i, &self.fish[i], on),
				Edit::Rec(i, rec, cnt, who) => save::set_fish(f, d, i, rec, cnt, who),
				Edit::Gear(id, on) => save::set_item_count(g, d, id, u32::from(on)),
			}
		}
		save::sync_fish(g, f, d, &self.fish, &self.achs, counts);
		let c = save::fish_counts(g, f, d, &self.fish);
		let got = self.achs.iter().zip(&before).filter(|(a, b)| **b == 0 && save::ach_state(g, d, a.id) == 1).count();
		self.status = if got > 0 { format!("Fish types: {}, rank A: {}; achievements completed: {got} — claim the rewards in game", c[0], c[1]) } else { format!("Fish types: {}, rank A: {}", c[0], c[1]) };
	}

	#[allow(clippy::too_many_arguments)]
	fn fish_page(&self, ui: &mut egui::Ui, area: Rect, tex: &egui::TextureHandle, f: &save::Fishing, d: &[u8], counts: [u32; 3], sel: &mut usize, edits: &mut Vec<Edit>) {
		let g = self.g;
		let z = Vec2::new(1242.0, 828.0);
		let k = ((area.width() - 12.0) / z.x).min((area.height() - 12.0) / z.y);
		let page = Rect::from_center_size(area.center(), z * k);
		let c = Canvas { pt: ui.painter_at(page), tex, o: page.min, base: Pos2::new(530.0, 152.0), k };
		c.pt.rect_filled(page, 0, PAGE);
		let name = |id: usize| self.items.iter().find(|it| it.id == id).map_or(format!("#{id}"), |it| it.en.clone());
		c.piece([322.0, 4.0, 480.0, 32.0], 557.0, 174.0);
		c.hline(541.0, 1136.0, 201.0);
		c.hline(1172.0, 1767.0, 201.0);
		c.piece([8.0, 124.0, 572.0, 292.0], 543.0, 213.0);
		let n = self.fish.len();
		for (y, v) in [(275.0, counts[0]), (325.0, counts[1])] {
			c.text(968.0, y, Align2::CENTER_CENTER, &if f.of_total && v as usize == n { format!("{v} / {n}") } else { v.to_string() }, 24.0, INK, true);
		}
		c.piece([12.0, 326.0, 516.0, 588.0], 583.0, 389.0);
		let owned = |id: usize| save::item_count(g, d, id) > 0;
		let gear: Vec<usize> = f.gear.iter().flat_map(|x| x.1.iter().copied()).collect();
		let lv = if f.by_level { gear.iter().rposition(|&id| owned(id)).map_or(0, |x| x + 1) } else { gear.iter().filter(|&&id| owned(id)).count() };
		if lv == gear.len() {
			c.piece([18.0, 614.0, 80.0, 652.0], 1011.0, 401.0);
		} else {
			c.text(1042.0, 421.0, Align2::CENTER_CENTER, &lv.to_string(), 38.0, Color32::from_rgb(0, 74, 170), true);
		}
		let mut slot = |ui: &egui::Ui, id: usize, x: f32, y: f32, w: f32, dark: bool, label: &str| {
			let on = owned(id);
			let src = if !on { 6.0 } else if dark { 102.0 } else { 54.0 };
			c.bar([598.0, src, 844.0, src + 36.0], x, y, w, 33.0);
			let r = c.rect(x, y, x + w, y + 33.0);
			c.check(c.rect(x + 10.0, y + 8.0, x + 27.0, y + 25.0), on);
			let mut left = 34.0;
			if !label.is_empty() {
				c.text(x + 36.0, y + 17.0, Align2::LEFT_CENTER, label, 14.0, Color32::WHITE, true);
				left = 84.0;
			}
			c.fit(x + (w + left) / 2.0 - 4.0, y + 16.5, Align2::CENTER_CENTER, &name(id), 20.0, w - left - 14.0, Color32::WHITE, false);
			let resp = ui.interact(r, ui.id().with(("gear", id)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
			let tip = self.items.iter().find(|it| it.id == id).map(|it| it.desc_en.replace("\\n", "\n")).unwrap_or_default();
			if resp.on_hover_text(tip).clicked() {
				edits.push(Edit::Gear(id, !on));
			}
		};
		if f.by_level {
			c.piece([597.0, 152.0, 620.0, 170.0], 583.0, 656.0);
			c.text(606.0, 666.0, Align2::LEFT_CENTER, f.gear[0].0, 15.0, GREY, true);
			c.hline(668.0, 1086.0, 667.0);
			for (n, &id) in f.gear[0].1.iter().enumerate() {
				slot(ui, id, 584.0, 697.0 + n as f32 * 41.0, 495.0, true, &format!("Lv. {}", n + 1));
			}
		} else {
			for (n, (title, ids)) in f.gear.iter().enumerate() {
				let y = 666.0 + n as f32 * 62.0;
				c.piece([597.0, 152.0, 620.0, 170.0], 583.0, y - 9.0);
				c.text(606.0, y, Align2::LEFT_CENTER, title, 15.0, GREY, true);
				c.hline(668.0, 1086.0, y);
				let w = (495.0 - 9.0 * (ids.len() - 1) as f32) / ids.len() as f32;
				for (k, &id) in ids.iter().enumerate() {
					slot(ui, id, 584.0 + k as f32 * (w + 9.0), y + 12.0, w, n + 1 < f.gear.len() && k + 1 < ids.len(), "");
				}
			}
		}
		c.piece([322.0, 66.0, 480.0, 94.0], 1184.0, 219.0);
		c.grid(1186.0, 249.0, 1749.0, 963.0);
		c.text(1302.0, 269.0, Align2::CENTER_CENTER, "Fish Name", 14.0, GREY, true);
		for (t, y) in [("Rank", 263.0), ("A", 275.0)] {
			c.text(1426.0, y, Align2::CENTER_CENTER, t, 13.0, GREY, true);
		}
		let caught: Vec<usize> = (0..n).filter(|&i| save::fish_rec(f, d, i).1 > 0).collect();
		for (pos, &i) in caught.iter().enumerate() {
			let x = &self.fish[i];
			let (rec, cnt, _) = save::fish_rec(f, d, i);
			let (a, b) = (1199.0 + (pos % 2) as f32 * 268.0, 285.0 + (pos / 2) as f32 * 48.0);
			c.piece([695.0, 227.0, 953.0, 269.0], a, b);
			c.fit(a + 30.0, b + 21.0, Align2::LEFT_CENTER, &name(x.item), 20.0, 182.0, INK, false);
			let is_a = x.is_a(rec, cnt);
			c.check_paper(c.rect(a + 220.0, b + 13.0, a + 238.0, b + 31.0), is_a, true);
			let resp = ui.interact(c.rect(a, b, a + 212.0, b + 42.0), ui.id().with(("ribbon", i)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
			if resp.clicked() {
				*sel = i + 1;
			}
			let resp = ui.interact(c.rect(a + 212.0, b, a + 246.0, b + 42.0), ui.id().with(("ribbon_a", i)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
			let tip = if is_a { "Rank A — click: record just below rank A" } else { "Click — record at the rank A minimum" };
			if resp.on_hover_text(tip).clicked() {
				edits.push(Edit::RankA(i, !is_a));
			}
		}
		ui.painter().rect_stroke(page, 0, Stroke::new(2.0_f32, FRAME), StrokeKind::Inside);
	}

	#[allow(clippy::too_many_arguments)]
	fn fish_card(&self, ui: &mut egui::Ui, area: Rect, tex: &egui::TextureHandle, pic: Option<&egui::TextureHandle>, icons: &Option<egui::TextureHandle>, f: &save::Fishing, d: &[u8], i: usize, edits: &mut Vec<Edit>) {
		let p = self.pal;
		let x = &self.fish[i];
		let (rec, cnt, who) = save::fish_rec(f, d, i);
		let k = (area.width() / 1344.0).min(area.height() / 860.0);
		let o = area.min + Vec2::new(((area.width() - 1344.0 * k) / 2.0).max(0.0), 0.0);
		let c = Canvas { pt: ui.painter_at(area), tex, o, base: Pos2::ZERO, k };
		let item = |id: usize| self.items.iter().find(|it| it.id == id);
		let name = item(x.item).map_or(format!("#{}", x.item), |it| it.en.clone());
		let q = Canvas { pt: ui.painter_at(area), tex, o: c.at(36.0, 30.0), base: Pos2::ZERO, k: k * 1.1 };
		q.grid(0.0, 0.0, 563.0, 502.0);
		q.piece([1285.0, 533.0, 1816.0, 986.0], 9.0, 22.0);
		q.fit(46.0, 52.0, Align2::LEFT_CENTER, &name, 34.0, 400.0, Color32::WHITE, true);
		if let Some(t) = pic {
			let z = Vec2::new(512.0, 288.0) * (460.0 / 512.0) * q.k;
			q.pt.image(t.id(), Rect::from_center_size(q.at(285.0, 208.0), z), UV_ALL, if cnt > 0 { Color32::WHITE } else { Color32::from_black_alpha(150) });
		}
		let (mut new_rec, mut new_cnt, mut new_who) = (rec, cnt, who);
		let field = |ui: &mut egui::Ui, r: Rect, dv: egui::DragValue<'_>| {
			let mut ch = ui.new_child(egui::UiBuilder::new().max_rect(r));
			let v = ch.visuals_mut();
			v.override_text_color = Some(INK);
			v.extreme_bg_color = Color32::from_rgb(250, 249, 244);
			for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active] {
				w.weak_bg_fill = Color32::from_rgb(238, 236, 228);
				w.bg_fill = w.weak_bg_fill;
				w.bg_stroke = Stroke::new(1.5_f32, GREY);
			}
			ch.style_mut().override_font_id = Some(f_semi(20.0 * k * 1.1));
			ch.spacing_mut().interact_size = Vec2::ZERO;
			ch.spacing_mut().button_padding = Vec2::new(4.0 * k, 0.0);
			ch.add_sized(r.size(), dv)
		};
		q.text(46.0, 407.0, Align2::LEFT_CENTER, "Caught:", 23.0, INK, false);
		field(ui, q.rect(142.0, 391.0, 232.0, 423.0), egui::DragValue::new(&mut new_cnt).range(0..=9999).speed(0.0));
		q.text(46.0, 446.0, Align2::LEFT_CENTER, "Record:", 23.0, INK, false);
		field(ui, q.rect(142.0, 430.0, 232.0, 462.0), egui::DragValue::new(&mut new_rec).clamp_existing_to_range(false).range(if cnt > 0 { x.min } else { 0.0 }..=x.max).speed(0.0).fixed_decimals(1));
		if !f.of_total {
			q.text(242.0, 407.0, Align2::LEFT_CENTER, "x", 21.0, INK, false);
		}
		q.text(242.0, 446.0, Align2::LEFT_CENTER, "rege", 21.0, INK, false);
		let stamp = |r: Rect, s: u32| q.img([22.0 + (s % 9) as f32 * 112.0, 1028.0 + (s / 9) as f32 * 112.0, 124.0 + (s % 9) as f32 * 112.0, 1128.0 + (s / 9) as f32 * 112.0], r);
		q.text(412.0, 443.0, Align2::RIGHT_CENTER, "Record Holder", 14.0, GREY, true);
		let fr = q.rect(420.0, 356.0, 516.0, 450.0);
		if cnt > 0 {
			match f.faces.iter().find(|x| x.0 == who) {
				Some(x) => stamp(fr, x.1),
				None => q.text(468.0, 403.0, Align2::CENTER_CENTER, &format!("#{who}"), 22.0, GREY, true),
			}
			let pop = ui.id().with("holder");
			let resp = ui.interact(fr.expand2(Vec2::new(14.0 * k, 0.0)), ui.id().with(("holder_btn", i)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
			let a = q.at(522.0, 403.0);
			q.pt.add(egui::Shape::convex_polygon(vec![a, a + Vec2::new(14.0, 0.0) * k, a + Vec2::new(7.0, 10.0) * k], GREY, Stroke::NONE));
			let who_name = f.faces.iter().find(|x| x.0 == who).map_or(format!("#{who}"), |x| x.2.to_string());
			let resp = resp.on_hover_text(format!("{who_name}. Click to pick the record holder"));
			if resp.clicked() {
				ui.memory_mut(|m| m.toggle_popup(pop));
			}
			egui::popup_below_widget(ui, pop, &resp, egui::PopupCloseBehavior::CloseOnClick, |ui| {
				ui.set_min_width(6.0 * 62.0);
				for row in f.faces.chunks(6) {
					ui.horizontal(|ui| {
						for x in row {
							let (r, b) = ui.allocate_exact_size(Vec2::splat(56.0), Sense::click());
							ui.painter().rect(r, 4, Color32::from_rgb(214, 211, 200), Stroke::new(2.0_f32, if x.0 == who { p.accent } else { GREY }), StrokeKind::Inside);
							let s = x.1;
							let z = tex.size_vec2();
							let (u, v) = (22.0 + (s % 9) as f32 * 112.0, 1028.0 + (s / 9) as f32 * 112.0);
							ui.painter().image(tex.id(), r.shrink(3.0), Rect::from_min_max(Pos2::new(u / z.x, v / z.y), Pos2::new((u + 102.0) / z.x, (v + 100.0) / z.y)), Color32::WHITE);
							if b.on_hover_text(x.2).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
								new_who = x.0;
							}
						}
					});
				}
			});
		}
		if (new_rec, new_cnt, new_who) != (rec, cnt, who) {
			if new_cnt == cnt && new_rec > 0.0 {
				new_cnt = new_cnt.max(1);
			}
			if new_cnt > 0 {
				new_rec = new_rec.clamp(x.min, x.max.max(rec));
			}
			edits.push(Edit::Rec(i, new_rec, new_cnt, new_who));
		}
		let y = 30.0 + 502.0 * 1.1 + 14.0;
		c.text(40.0, y, Align2::LEFT_TOP, "Ecological Info", 20.0, p.muted, false);
		let text = c.pt.layout(x.desc.clone(), f_body(22.0 * k), p.text, 615.0 * k);
		c.pt.galley(c.at(42.0, y + 32.0), text, p.text);
		let white = Color32::from_rgb(250, 250, 248);
		for (n, (letter, lo, hi)) in x.ranks().into_iter().enumerate() {
			let z = 1.12;
			let (x3, y3) = (700.0, 24.0 + n as f32 * 274.0);
			let (w, h) = (563.0 * z, 233.0 * z);
			let src = [290.0, 530.0, 770.0][n];
			c.img([695.0, src, 1258.0, src + 233.0], c.rect(x3, y3, x3 + w, y3 + h));
			if n == 0 {
				let on = x.is_a(rec, cnt);
				let r = c.rect(x3 + 18.0, y3 + 22.0, x3 + 40.0, y3 + 44.0);
				c.check(r, on);
				let resp = ui.interact(r.expand(6.0 * k), ui.id().with(("rank_a", i)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
				if resp.on_hover_text("The game derives rank A from the record: the tick sets the record to the minimum").clicked() {
					edits.push(Edit::RankA(i, !on));
				}
			}
			c.text(x3 + 56.0, y3 + 38.0, Align2::LEFT_CENTER, "RANK", 18.0, white, true);
			c.text(x3 + 150.0, y3 + 32.0, Align2::CENTER_CENTER, letter, 48.0, white, true);
			let len = hi.map_or(format!("Length: {lo:.1} rege and up"), |hi| format!("Length: {lo:.1} ~ {hi:.1} rege"));
			c.text(x3 + 206.0, y3 + 38.0, Align2::LEFT_CENTER, &len, 22.0, white, true);
			let it = item(x.items[n]);
			if !x.has_rank(n, rec, cnt) {
				c.img([546.0, 656.0, 622.0, 734.0], c.rect(x3 + 66.0, y3 + 116.0, x3 + 151.0, y3 + 203.0));
				c.text(x3 + 152.0, y3 + 159.0, Align2::LEFT_CENTER, &format!("Catch a Rank {letter} Fish"), 25.0, INK, true);
				let tip = format!("{}  x {}", it.map_or(format!("#{}", x.items[n]), |it| it.en.clone()), x.counts[n]);
				ui.interact(c.rect(x3, y3 + 70.0, x3 + w, y3 + h), ui.id().with(("rank_lock", i, n)), Sense::hover()).on_hover_text(tip);
				continue;
			}
			c.text(x3 + 24.0, y3 + 114.0, Align2::LEFT_CENTER, "Items Gained", 17.0, GREY, false);
			if let (Some(t), Some(cell)) = (icons, it.and_then(|it| it.icon)) {
				c.pt.image(t.id(), Rect::from_center_size(c.at(x3 + 176.0, y3 + 94.0), Vec2::splat(38.0 * k)), atlas_uv(t, cell), Color32::WHITE);
			}
			c.fit(x3 + 202.0, y3 + 94.0, Align2::LEFT_CENTER, &it.map_or(format!("#{}", x.items[n]), |it| it.en.clone()), 25.0, 330.0, INK, true);
			c.text(x3 + w - 40.0, y3 + 94.0, Align2::RIGHT_CENTER, &format!("x  {}", x.counts[n]), 25.0, INK, true);
			c.hline(x3 + 20.0, x3 + w - 24.0, y3 + 128.0);
			c.pt.rect_filled(c.rect(x3 + 20.0, y3 + 138.0, x3 + w - 22.0, y3 + 248.0), 6.0 * k, Color32::from_rgb(140, 138, 134));
			for (j, &v) in x.sepith[n].iter().enumerate() {
				let (a, b) = (x3 + 52.0 + (j % 4) as f32 * 146.0, y3 + 166.0 + (j / 4) as f32 * 52.0);
				let col = if v > 0 { Color32::WHITE } else { Color32::from_rgb(196, 194, 190) };
				if let Some(t) = icons {
					c.pt.image(t.id(), Rect::from_center_size(c.at(a, b), Vec2::splat(36.0 * k)), atlas_uv(t, save::cells::SEPITH + j as u32), if v > 0 { Color32::WHITE } else { Color32::from_white_alpha(140) });
				}
				c.text(a + 24.0, b, Align2::LEFT_CENTER, "×", 20.0, col, false);
				c.text(a + 96.0, b, Align2::RIGHT_CENTER, &v.to_string(), 22.0, col, true);
			}
		}
	}
}

