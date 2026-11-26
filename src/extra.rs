use eframe::egui;

const BASIS_ID: u64 = 0xCBF2_9CE4_8422_2325;
const BASIS_KEY: u64 = 0x8422_2325_CBF2_9CE4;

static DATA: &[(u64, &[u8])] = &[];

fn fnv(b: &[u8], basis: u64) -> u64 {
	let mut h = basis;
	for &x in b {
		h ^= x as u64;
		h = h.wrapping_mul(0x0000_0100_0000_01B3);
	}
	h
}

fn stream(seed: u64, n: usize) -> Vec<u8> {
	let mut s = if seed == 0 { 1 } else { seed };
	let mut out = Vec::with_capacity(n + 8);
	while out.len() < n {
		s ^= s >> 12;
		s ^= s << 25;
		s ^= s >> 27;
		out.extend_from_slice(&s.wrapping_mul(0x2545_F491_4F6C_DD1D).to_le_bytes());
	}
	out.truncate(n);
	out
}

fn caps(text: &str) -> bool {
	text.chars().any(char::is_uppercase) && !text.chars().any(char::is_lowercase)
}

fn open(text: &str) -> Option<(String, String)> {
	let text = text.trim();
	if !caps(text) {
		return None;
	}
	let b = text.as_bytes();
	let id = fnv(b, BASIS_ID);
	let (_, enc) = DATA.iter().find(|(h, _)| *h == id)?;
	let ks = stream(fnv(b, BASIS_KEY), enc.len());
	let plain: Vec<u8> = enc.iter().zip(ks).map(|(a, k)| a ^ k).collect();
	let s = String::from_utf8(plain).ok()?;
	let rest = s.strip_prefix("ok\n")?;
	let (title, body) = rest.split_once('\n')?;
	Some((title.to_string(), body.to_string()))
}

const SEAL: u64 = 0x4FA2_8059_E0CE_52FE;

#[allow(dead_code)]
fn seal() -> u64 {
	let data: &[(u64, &[u8])] = std::hint::black_box(&DATA);
	let mut h = BASIS_ID ^ BASIS_KEY;
	for (id, enc) in data {
		h = fnv(&id.to_le_bytes(), h);
		h = fnv(enc, h);
	}
	h
}

#[allow(dead_code)]
pub fn init() {
	if std::hint::black_box(DATA).len() != 2 || seal() != std::hint::black_box(SEAL) {
		std::process::exit(0x5E);
	}
}

pub fn watch(ctx: &egui::Context, text: &mut String) {
	let id = egui::Id::new(0x5EC2_E7u32);
	if let Some(v) = open(text) {
		text.clear();
		ctx.data_mut(|d| d.insert_temp(id, Some(v)));
	}
	let cur: Option<(String, String)> = ctx.data(|d| d.get_temp(id)).flatten();
	let Some((title, body)) = cur else { return };
	let mut keep = true;
	egui::Window::new(egui::RichText::new(&title).strong())
		.id(id.with("w"))
		.collapsible(false)
		.resizable(false)
		.anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
		.fixed_size([460.0, 0.0])
		.show(ctx, |ui| {
			ui.add_space(4.0);
			ui.add(egui::Label::new(egui::RichText::new(&body).size(15.0)).wrap());
			ui.add_space(10.0);
			ui.vertical_centered(|ui| {
				if ui.button("  OK  ").clicked() {
					keep = false;
				}
			});
		});
	if !keep || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
		ctx.data_mut(|d| d.insert_temp::<Option<(String, String)>>(id, None));
	}
}

