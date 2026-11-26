use std::io::Read;
use std::path::Path;

pub fn table(name: &str) -> &'static str {
	use std::collections::HashMap;
	use std::sync::{Mutex, OnceLock};
	if let Some(t) = crate::pack::text(&format!("tables/{name}")) {
		return t;
	}
	static DEV: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
	let mut m = DEV.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap();
	m.entry(name.to_string())
		.or_insert_with(|| {
			let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("calvard_save").join(name);
			Box::leak(std::fs::read_to_string(p).unwrap_or_default().into_boxed_str())
		})
}

pub const DB1_SIZE: usize = 0x1903B0;

pub mod db1 {
	pub const CHARS: usize = 0x9BCE0;
	pub const CHAR_SIZE: usize = 1000;
	pub const CHAR_COUNT: usize = 21;
	pub const ITEMS: usize = 0xB7068;
	pub const ITEM_MAX_ID: usize = 4503;
	pub const SEPITH: usize = 0x190178;
	pub const MIRA: usize = 0x190198;
	pub const PLAYTIME: usize = 0x1901A8;
	pub const LINEUP: usize = 0x1154C;
	pub const LINEUP_MAX: usize = 64;
	pub const LINEUP_ACTIVE: usize = 4;
	pub const CHAR_WEAPON: usize = 672;
	pub const RECORD_CHR: [Option<u32>; 21] = [
		Some(0), Some(1), Some(2), Some(3), Some(6), Some(4), Some(10), Some(5), Some(8), Some(9), Some(7), Some(107), Some(13), Some(11), Some(63), Some(62),
		Some(12), Some(14), Some(108), Some(112), Some(15),
	];
	pub const QUESTS: usize = 0x18AB70;
	pub const QUEST_COUNT: usize = 256;
	pub const QUEST_SP: [(u16, u8); 73] = [
		(1, 4), (2, 4), (3, 4), (4, 3), (5, 3), (6, 5), (7, 4), (20, 4), (21, 3), (22, 5), (23, 4), (24, 4), (25, 4), (26, 4), (27, 3), (28, 3), (30, 4),
		(40, 4), (41, 4), (42, 4), (43, 5), (44, 4), (45, 4), (46, 3), (47, 4), (48, 4), (49, 3), (50, 4), (51, 4), (52, 4), (60, 3), (61, 4), (62, 4),
		(63, 3), (64, 4), (65, 4), (66, 3), (67, 4), (69, 4), (70, 4), (72, 4), (73, 4), (75, 5), (76, 4), (80, 3), (81, 4), (82, 5), (85, 3), (86, 4),
		(87, 3), (88, 3), (89, 4), (90, 4), (91, 4), (100, 4), (101, 5), (102, 5), (103, 5), (105, 5), (106, 4), (107, 5), (108, 4), (151, 4), (153, 6),
		(156, 8), (157, 4), (160, 10), (164, 12), (166, 5), (168, 9), (169, 4), (171, 15), (174, 5),
	];
	pub const RANKS: [(&str, u32); 15] = [("E", 0), ("E+", 5), ("E++", 20), ("D", 40), ("D+", 60), ("D++", 80), ("C", 110), ("C+", 140), ("C++", 170), ("B", 200), ("B+", 230), ("B++", 260), ("A", 290), ("AA", 320), ("AAA", 350)];
	pub const RANK_REWARDS: [[usize; 2]; 15] = [[0, 0], [3750, 0], [1759, 0], [703, 0], [3406, 15], [1761, 0], [4060, 0], [280, 15], [1760, 0], [3416, 16], [3315, 16], [3960, 0], [1219, 0], [1018, 0], [1763, 0]];
	pub const LGC: usize = 0x11044;
	pub const LGC_STEPS: [[u32; 6]; 3] = [[0, 10, 30, 60, 90, 120], [0, 8, 25, 50, 75, 100], [0, 5, 15, 30, 45, 75]];
	pub const ADDABLE: [(u32, usize); 12] = [(0, 0), (1, 1), (2, 2), (3, 3), (6, 4), (4, 5), (10, 6), (5, 7), (11, 13), (12, 16), (62, 15), (15, 20)];
	pub const NPC_GUESTS: [u32; 6] = [7, 8, 13, 14, 63, 107];
	pub const STATS: [&str; 8] = ["STR", "DEF", "ATS", "ADF", "SPD", "AGL", "DEX", "MOV"];
	pub const FACES: &str = "Daybreak-1/faces.bin";
	pub const ICONS: &str = "Daybreak-1/icons.png";
	pub const ITEMS_TSV: &str = "db1_items.tsv";
	pub const NAMES_TSV: &str = "db1_names.tsv";
	pub const SKILLS_TSV: &str = "db1_skills.tsv";
	pub const CHAR_CRAFTS: usize = 340;
	pub const CHAR_CRAFT_SLOTS: usize = 16;
	pub const CHAR_EQUIP: usize = 672;
	pub const EQUIP_SLOTS: [&str; 6] = ["Main", "Sub", "Armor", "Shoes", "Slot 1", "Slot 2"];
	pub const CHAR_DERIVED: [(&str, usize, &str); 9] = [
		("Attack", 72, ""),
		("Attack DEF", 76, ""),
		("Arts", 80, ""),
		("Arts DEF", 84, ""),
		("ACC", 92, "%"),
		("CRT", 104, "%"),
		("EVA", 96, "%"),
		("AEV", 100, "%"),
		("RNG", 64, ""),
	];
}

pub mod db2 {
	pub const SIZE: usize = 0x271918;
	pub const CHARS: usize = 0x17762C;
	pub const CHAR_SIZE: usize = 1132;
	pub const CHAR_COUNT: usize = 26;
	pub const ITEMS: usize = 0x196AD8;
	pub const ITEM_MAX_ID: usize = 4940;
	pub const SEPITH: usize = 0x270CB8;
	pub const MIRA: usize = 0x270CD8;
	pub const S_TOKEN: usize = 0x270CDC;
	pub const PLAYTIME: usize = 0x270CF0;
	pub const LINEUP: usize = 0x41874;
	pub const CHAR_EQUIP: usize = 800;
	pub const CHAR_CRAFTS: usize = 468;
	pub const QUESTS: usize = 0x26B1F0;
	pub const SP_EXTRA: usize = 0x4109C;
	pub const LGC: usize = 0x41044;
	pub const LGC_STEPS: [&[u32]; 3] = [&[0, 20, 40, 60], &[0, 16, 32, 48], &[0, 10, 20, 30]];
	pub const RANKS: [(&str, u32); 15] = [("E", 0), ("E+", 5), ("E++", 15), ("D", 30), ("D+", 50), ("D++", 75), ("C", 100), ("C+", 125), ("C++", 150), ("B", 180), ("B+", 210), ("B++", 240), ("A", 270), ("AA", 300), ("AAA", 330)];
	pub const RANK_REWARDS: [[usize; 2]; 15] = [[0, 0], [3750, 0], [1759, 0], [3640, 15], [3850, 0], [1761, 0], [3643, 15], [4060, 0], [1760, 0], [3318, 16], [3960, 0], [3646, 16], [1227, 0], [1027, 0], [1763, 0]];
	pub const QUEST_SP: [(u16, u8); 51] = [
		(151, 4), (152, 4), (153, 6), (154, 5), (155, 3), (178, 5), (156, 8), (179, 15), (184, 9), (159, 5), (180, 6), (183, 12), (161, 12), (162, 12),
		(181, 15), (1, 2), (2, 3), (10, 2), (11, 2), (12, 2), (13, 2), (16, 2), (14, 3), (15, 3), (17, 4), (20, 2), (21, 2), (22, 2), (23, 2), (27, 2),
		(24, 3), (25, 1), (28, 3), (26, 3), (29, 3), (30, 4), (32, 4), (31, 2), (33, 5), (40, 4), (44, 4), (45, 4), (41, 4), (46, 4), (42, 4), (47, 4),
		(43, 4), (48, 4), (50, 4), (51, 4), (52, 4),
	];
	pub const RECORD_CHR: [Option<u32>; 26] = [
		Some(0), Some(1), Some(2), Some(3), Some(6), Some(4), Some(10), Some(11), Some(9), Some(19), Some(12), Some(17), Some(18), Some(151), Some(152), Some(8),
		Some(63), Some(15), Some(7), Some(14), Some(149), Some(13), Some(159), Some(158), Some(104), None,
	];
	pub const ADDABLE: [(u32, usize); 17] = [(0, 0), (1, 1), (2, 2), (3, 3), (6, 4), (4, 5), (10, 6), (11, 7), (9, 8), (19, 9), (12, 10), (17, 11), (18, 12), (151, 13), (152, 14), (8, 15), (63, 16)];
	pub const NPC_GUESTS: [u32; 8] = [7, 14, 149, 13, 159, 158, 104, 15];
	pub const FACES: &str = "Daybreak-2/faces.bin";
	pub const ICONS: &str = "Daybreak-2/icons.png";
	pub const ITEMS_TSV: &str = "db2_items.tsv";
	pub const NAMES_TSV: &str = "db2_names.tsv";
	pub const SKILLS_TSV: &str = "db2_skills.tsv";
}

pub mod hz {
	pub const SIZE: usize = 0x298C18;
	pub const CHARS: usize = 0x197398;
	pub const CHAR_SIZE: usize = 1252;
	pub const CHAR_COUNT: usize = 40;
	pub const ITEMS: usize = 0x1BB4B0;
	pub const ITEM_MAX_ID: usize = 4921;
	pub const SEPITH: usize = 0x297960;
	pub const MIRA: usize = 0x297980;
	pub const G_TOKEN: usize = 0x297984;
	pub const CASINO: usize = 0x297988;
	pub const PLAYTIME: usize = 0x297998;
	pub const LINEUP: usize = 0x43194;
	pub const CHAR_EQUIP: usize = 920;
	pub const CHAR_CRAFTS: usize = 588;
	pub const QUESTS: usize = 0x290798;
	pub const SP_EXTRA: usize = 0x4209C;
	pub const LGC: usize = 0x42044;
	pub const LGC_STEPS: [&[u32]; 3] = [&[0, 12, 24, 36, 48, 72], &[0, 12, 24, 36, 48, 72], &[0, 10, 20, 30, 45, 60]];
	pub const RANKS: [(&str, u32); 15] = [("E", 0), ("E+", 25), ("E++", 50), ("D", 75), ("D+", 100), ("D++", 125), ("C", 150), ("C+", 175), ("C++", 200), ("B", 230), ("B+", 260), ("B++", 290), ("A", 320), ("AA", 350), ("AAA", 385)];
	pub const RANK_REWARDS: [[usize; 2]; 15] = [[0, 0], [4150, 0], [1741, 0], [3647, 15], [4250, 0], [1746, 0], [3649, 15], [3960, 0], [1743, 0], [3319, 16], [4361, 0], [3652, 16], [1236, 0], [1036, 0], [1748, 0]];
	pub const QUEST_SP: [(u16, u8); 79] = [
		(151, 4), (153, 3), (154, 2), (155, 1), (156, 3), (157, 3), (158, 1), (159, 4), (160, 2), (161, 5), (162, 6), (171, 5), (173, 5), (166, 5), (167, 3),
		(168, 3), (169, 1), (170, 3), (175, 3), (163, 5), (164, 7), (165, 9), (182, 5), (183, 5), (178, 4), (179, 3), (180, 2), (181, 5), (186, 5), (187, 8),
		(176, 4), (177, 7), (184, 8), (185, 12), (1, 3), (2, 2), (51, 3), (3, 1), (4, 2), (5, 2), (6, 2), (52, 3), (7, 2), (8, 3), (9, 2), (10, 2), (11, 2),
		(12, 3), (54, 3), (13, 2), (14, 4), (15, 3), (16, 2), (17, 2), (19, 2), (20, 2), (58, 3), (21, 2), (22, 2), (23, 2), (24, 3), (25, 2), (26, 2), (60, 4),
		(27, 2), (28, 3), (29, 4), (30, 2), (31, 3), (32, 4), (33, 4), (34, 2), (65, 3), (35, 1), (53, 3), (55, 3), (56, 3), (61, 3),
		(63, 3),
	];
	pub const RECORD_CHR: [Option<u32>; 40] = [
		Some(0), Some(1), Some(2), Some(3), Some(6), Some(4), Some(10), Some(5), Some(15), Some(8), Some(11), Some(9), Some(17), Some(18), Some(20), Some(21),
		Some(22), Some(24), Some(23), Some(63), Some(62), Some(19), Some(12), Some(152), Some(151), Some(159), Some(7), Some(104), Some(14), Some(149), Some(160),
		None, Some(169), None, Some(106), Some(173), Some(119), Some(107), None, None,
	];
	pub const ADDABLE: [(u32, usize); 26] = [
		(0, 0), (3, 3), (11, 10), (12, 22), (4, 5), (10, 6), (6, 4), (20, 14), (21, 15), (22, 16), (17, 12), (18, 13), (24, 17), (23, 18), (151, 24), (152, 23),
		(63, 19), (8, 9), (5, 7), (62, 20), (1, 1), (19, 21), (2, 2), (159, 25), (169, 32), (173, 35),
	];
	pub const NPC_GUESTS: [u32; 10] = [7, 104, 14, 149, 160, 106, 119, 107, 9, 15];
	pub const FACES: &str = "Horizon/faces.bin";
	pub const ICONS: &str = "Horizon/icons.png";
	pub const ITEMS_TSV: &str = "hz_items.tsv";
	pub const NAMES_TSV: &str = "hz_names.tsv";
	pub const SKILLS_TSV: &str = "hz_skills.tsv";
}

pub struct Profile {
	pub dir: &'static str,
	pub size: usize,
	pub chars: usize,
	pub char_size: usize,
	pub char_count: usize,
	pub items: usize,
	pub item_max_id: usize,
	pub item_u16: bool,
	pub sepith: usize,
	pub mira: usize,
	pub s_token: Option<usize>,
	pub casino: Option<usize>,
	pub token_name: &'static str,
	pub stat_shift: usize,
	pub playtime: usize,
	pub lineup: usize,
	pub party_sel: Option<(usize, usize, usize)>,
	pub difficulty: usize,
	pub equip: usize,
	pub crafts: usize,
	pub craft_lv: Option<(usize, usize, usize)>,
	pub craft_step: usize,
	pub craft_slots: usize,
	pub quests: usize,
	pub quest_sp: &'static [(u16, u8)],
	pub sp_extra: Option<usize>,
	pub ranks: &'static [(&'static str, u32)],
	pub rank_rewards: &'static [[usize; 2]],
	pub lgc: usize,
	pub lgc_steps: [&'static [u32]; 3],
	pub lgc_first: usize,
	pub lgc_digits: ([f32; 4], &'static str),
	pub record_chr: &'static [Option<u32>],
	pub addable: &'static [(u32, usize)],
	pub npc_guests: &'static [u32],
	pub faces: &'static str,
	pub icons: &'static str,
	pub items_tsv: &'static str,
	pub names_tsv: &'static str,
	pub skills_tsv: &'static str,
	pub kinds_tsv: &'static str,
	pub dlc_tsv: &'static str,
	pub qline_tsv: &'static str,
	pub tabs: &'static [&'static str],
	pub hidden: &'static [usize],
}

pub static DB1: Profile = Profile {
	dir: "Daybreak-1",
	size: DB1_SIZE,
	chars: db1::CHARS,
	char_size: db1::CHAR_SIZE,
	char_count: db1::CHAR_COUNT,
	items: db1::ITEMS,
	item_max_id: db1::ITEM_MAX_ID,
	item_u16: false,
	sepith: db1::SEPITH,
	mira: db1::MIRA,
	s_token: None,
	casino: None,
	token_name: "",
	stat_shift: 0,
	playtime: db1::PLAYTIME,
	lineup: db1::LINEUP,
	party_sel: None,
	difficulty: 0x11038,
	equip: db1::CHAR_EQUIP,
	crafts: db1::CHAR_CRAFTS,
	craft_lv: None,
	craft_step: 2,
	craft_slots: db1::CHAR_CRAFT_SLOTS,
	quests: db1::QUESTS,
	quest_sp: &db1::QUEST_SP,
	sp_extra: None,
	ranks: &db1::RANKS,
	rank_rewards: &db1::RANK_REWARDS,
	lgc: db1::LGC,
	lgc_steps: [&db1::LGC_STEPS[0], &db1::LGC_STEPS[1], &db1::LGC_STEPS[2]],
	lgc_first: 0,
	lgc_digits: ([339.0, 114.0, 218.0, 56.0], "123450"),
	record_chr: &db1::RECORD_CHR,
	addable: &db1::ADDABLE,
	npc_guests: &db1::NPC_GUESTS,
	faces: db1::FACES,
	icons: db1::ICONS,
	items_tsv: db1::ITEMS_TSV,
	names_tsv: db1::NAMES_TSV,
	skills_tsv: db1::SKILLS_TSV,
	kinds_tsv: "db1_kinds.tsv",
	dlc_tsv: "db1_dlc.tsv",
	qline_tsv: "db1_qline.tsv",
	tabs: &TABS,
	hidden: &[2603, 2614],
};

pub static DB2: Profile = Profile {
	dir: "Daybreak-2",
	size: db2::SIZE,
	chars: db2::CHARS,
	char_size: db2::CHAR_SIZE,
	char_count: db2::CHAR_COUNT,
	items: db2::ITEMS,
	item_max_id: db2::ITEM_MAX_ID,
	item_u16: true,
	sepith: db2::SEPITH,
	mira: db2::MIRA,
	s_token: Some(db2::S_TOKEN),
	casino: None,
	token_name: "S-Token",
	stat_shift: 0,
	playtime: db2::PLAYTIME,
	lineup: db2::LINEUP,
	party_sel: Some((0x41934, 0x41554, 0xC8)),
	difficulty: 0x41038,
	equip: db2::CHAR_EQUIP,
	crafts: db2::CHAR_CRAFTS,
	craft_lv: Some((340, 8, 4)),
	craft_step: 2,
	craft_slots: 16,
	quests: db2::QUESTS,
	quest_sp: &db2::QUEST_SP,
	sp_extra: Some(db2::SP_EXTRA),
	ranks: &db2::RANKS,
	rank_rewards: &db2::RANK_REWARDS,
	lgc: db2::LGC,
	lgc_steps: db2::LGC_STEPS,
	lgc_first: 3,
	lgc_digits: ([690.0, 618.0, 324.0, 56.0], "012345678"),
	record_chr: &db2::RECORD_CHR,
	addable: &db2::ADDABLE,
	npc_guests: &db2::NPC_GUESTS,
	faces: db2::FACES,
	icons: db2::ICONS,
	items_tsv: db2::ITEMS_TSV,
	names_tsv: db2::NAMES_TSV,
	skills_tsv: db2::SKILLS_TSV,
	kinds_tsv: "db2_kinds.tsv",
	dlc_tsv: "db2_dlc.tsv",
	qline_tsv: "db2_qline.tsv",
	tabs: &TABS_DB2,
	hidden: &[],
};

pub static HZ: Profile = Profile {
	dir: "Horizon",
	size: hz::SIZE,
	chars: hz::CHARS,
	char_size: hz::CHAR_SIZE,
	char_count: hz::CHAR_COUNT,
	items: hz::ITEMS,
	item_max_id: hz::ITEM_MAX_ID,
	item_u16: true,
	sepith: hz::SEPITH,
	mira: hz::MIRA,
	s_token: Some(hz::G_TOKEN),
	casino: Some(hz::CASINO),
	token_name: "G-Tokens",
	stat_shift: 4,
	playtime: hz::PLAYTIME,
	lineup: hz::LINEUP,
	party_sel: Some((0x43624, 0x42554, 0x188)),
	difficulty: 0x42038,
	equip: hz::CHAR_EQUIP,
	crafts: hz::CHAR_CRAFTS,
	craft_lv: Some((396, 12, 4)),
	craft_step: 2,
	craft_slots: 16,
	quests: hz::QUESTS,
	quest_sp: &hz::QUEST_SP,
	sp_extra: Some(hz::SP_EXTRA),
	ranks: &hz::RANKS,
	rank_rewards: &hz::RANK_REWARDS,
	lgc: hz::LGC,
	lgc_steps: hz::LGC_STEPS,
	lgc_first: 0,
	lgc_digits: ([690.0, 618.0, 324.0, 56.0], "012345678"),
	record_chr: &hz::RECORD_CHR,
	addable: &hz::ADDABLE,
	npc_guests: &hz::NPC_GUESTS,
	faces: hz::FACES,
	icons: hz::ICONS,
	items_tsv: hz::ITEMS_TSV,
	names_tsv: hz::NAMES_TSV,
	skills_tsv: hz::SKILLS_TSV,
	kinds_tsv: "hz_kinds.tsv",
	dlc_tsv: "hz_dlc.tsv",
	qline_tsv: "hz_qline.tsv",
	tabs: &TABS_HZ,
	hidden: &[],
};

pub fn stat(g: &Profile, off: usize) -> usize {
	if off >= 16 { off + g.stat_shift } else { off }
}

pub struct Skill {
	pub chr: u32,
	pub cp: u32,
	pub power: String,
	pub en: String,
	pub ru: String,
	pub kind: char,
	pub area: Option<u32>,
	pub power_num: u32,
	pub cp_lv: Vec<i32>,
	pub power_lv: Vec<i32>,
	pub effect: Option<u32>,
}

pub mod cells {
	pub const SEPITH: u32 = 5 * 21 + 7;
	pub fn craft_kind(k: char) -> u32 {
		13 * 21 + match k {
			'S' => 20,
			'P' => 17,
			'M' => 18,
			_ => 19,
		}
	}
	pub fn power_of(n: u32) -> Option<u32> {
		const T: [(u32, u32); 15] = [(80, 285), (90, 284), (100, 283), (110, 282), (120, 281), (130, 280), (150, 279), (170, 278), (200, 277), (230, 276), (270, 275), (310, 274), (360, 273), (400, 191), (440, 190)];
		T.iter().rev().find(|&&(t, _)| n >= t).map(|&(_, c)| c)
	}
	pub fn power(p: &str) -> Option<u32> {
		["SS", "S+", "S", "A+", "A", "B+", "B", "C+", "C", "D+", "D", "E+", "E"].iter().position(|&x| x == p).map(|i| 13 * 21 + i as u32)
	}
}

pub fn skills(tsv: &str) -> std::collections::HashMap<u32, Skill> {
	tsv.lines()
		.filter(|l| !l.starts_with('#') && !l.trim().is_empty())
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			Some((
				f.first()?.parse().ok()?,
				Skill {
					chr: f.get(1)?.parse().ok()?,
					cp: f.get(2)?.parse().ok()?,
					power: f.get(3)?.to_string(),
					en: f.get(4)?.to_string(),
					ru: f.get(5).unwrap_or(&"").to_string(),
					kind: f.get(6).and_then(|s| s.chars().next()).unwrap_or('U'),
					area: f.get(7).and_then(|s| s.parse::<i64>().ok()).filter(|&v| v >= 0).map(|v| v as u32),
					power_num: f.get(8).and_then(|s| s.parse().ok()).unwrap_or(0),
					cp_lv: f.get(9).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default(),
					power_lv: f.get(10).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default(),
					effect: f.get(11).and_then(|s| s.parse::<u32>().ok()).filter(|&v| v > 0),
				},
			))
		})
		.collect()
}

pub fn crafts(g: &Profile, d: &[u8], k: usize) -> Vec<u32> {
	let o = g.chars + k * g.char_size + g.crafts;
	(0..g.craft_slots).map(|i| u16::from_le_bytes([d[o + i * g.craft_step], d[o + i * g.craft_step + 1]]) as u32).filter(|&v| (1000..0xFFFF).contains(&v)).collect()
}

pub fn craft_levels(g: &Profile, d: &[u8], k: usize) -> Vec<(u32, u32)> {
	let Some((at, step, lv)) = g.craft_lv else { return Vec::new() };
	let o = chr_at(g, k) + at;
	(0..16).map(|i| o + i * step).map(|r| (u16::from_le_bytes([d[r], d[r + 1]]) as u32, u16::from_le_bytes([d[r + lv], d[r + lv + 1]]) as u32)).filter(|&(id, l)| (1000..0xFFFF).contains(&id) && l > 0).collect()
}

pub fn chr_at(g: &Profile, k: usize) -> usize {
	g.chars + k * g.char_size
}

pub fn item_count(g: &Profile, d: &[u8], id: usize) -> u32 {
	let o = g.items + id * 4;
	if g.item_u16 { u16::from_le_bytes([d[o], d[o + 1]]) as u32 } else { d[o] as u32 }
}

pub fn set_item_count(g: &Profile, d: &mut [u8], id: usize, n: u32) {
	let o = g.items + id * 4;
	let flag = o + if g.item_u16 { 2 } else { 1 };
	if g.item_u16 {
		d[o..o + 2].copy_from_slice(&(n.min(u16::MAX as u32) as u16).to_le_bytes());
	} else {
		d[o] = n.min(255) as u8;
	}
	if d[flag] == 0 && n > 0 {
		d[flag] = 2;
	}
}

fn dlc_bit(g: &Profile, id: u32) -> (usize, u8) {
	(g.mira - 0xA0 + id as usize / 8, 1 << (id % 8))
}

pub fn dlc_opened(g: &Profile, d: &[u8], id: u32) -> bool {
	let (o, m) = dlc_bit(g, id);
	id < 1024 && d[o] & m != 0
}

pub fn set_dlc_opened(g: &Profile, d: &mut [u8], id: u32, opened: bool) {
	if id >= 1024 {
		return;
	}
	let (o, m) = dlc_bit(g, id);
	if opened { d[o] |= m } else { d[o] &= !m }
}

pub fn item_max(g: &Profile) -> u32 {
	if g.item_u16 { u16::MAX as u32 } else { 255 }
}

pub fn item_limit(g: &Profile, it: &Item) -> u32 {
	if it.max == 0 { item_max(g).min(9999) } else { it.max.min(item_max(g)) }
}

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

pub fn read_for(g: &Profile, path: &Path) -> Result<Vec<u8>, String> {
	let d = read(path)?;
	if d.len() != g.size {
		let other = match d.len() {
			DB1_SIZE => "Trails through Daybreak",
			db2::SIZE => "Trails through Daybreak II",
			hz::SIZE => "Trails beyond the Horizon",
			_ => "another game",
		};
		return Err(format!("{}: this save is from {other}", path.display()));
	}
	Ok(d)
}

pub fn read(path: &Path) -> Result<Vec<u8>, String> {
	let raw = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
	if raw.len() < 8 || raw[..4] != [0x28, 0xB5, 0x2F, 0xFD] {
		return Err(format!("{}: not a Calvard save (no zstd compression)", path.display()));
	}
	let mut dec = ruzstd::decoding::StreamingDecoder::new(&raw[..]).map_err(|e| format!("{}: {e}", path.display()))?;
	let mut d = Vec::new();
	dec.read_to_end(&mut d).map_err(|e| format!("{}: {e}", path.display()))?;
	if ![DB1_SIZE, db2::SIZE, hz::SIZE].contains(&d.len()) {
		return Err(format!("{}: data size {} — supported games are Trails through Daybreak, Daybreak II and beyond the Horizon", path.display(), d.len()));
	}
	if u32::from_le_bytes(d[8..12].try_into().unwrap()) != checksum(&d) {
		return Err(format!("{}: checksum mismatch — the save is damaged", path.display()));
	}
	Ok(d)
}

pub fn write(path: &Path, d: &mut [u8]) -> Result<(), String> {
	let c = checksum(d);
	d[8..12].copy_from_slice(&c.to_le_bytes());
	let out = pack(d);
	let tmp = path.with_extension("tmp");
	std::fs::write(&tmp, &out).map_err(|e| format!("{}: {e}", tmp.display()))?;
	let back = read(&tmp)?;
	if back != d {
		let _ = std::fs::remove_file(&tmp);
		return Err("write check failed: the file does not unpack to the same data".into());
	}
	std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
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

pub fn u32_at(d: &[u8], o: usize) -> u32 {
	u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}

pub fn set_u32(d: &mut [u8], o: usize, v: u32) {
	d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

fn quest_base(g: &Profile, id: usize) -> u32 {
	g.quest_sp.iter().find(|&&(q, _)| q as usize == id).map_or(0, |&(_, s)| s as u32)
}

fn quest_bonus_at(g: &Profile, id: usize) -> usize {
	g.quests + id * 80 + 0x4C
}

fn quest_done(g: &Profile, d: &[u8], id: usize) -> bool {
	d[g.quests + id * 80 + 0x0A] & 4 != 0
}

pub fn any_quest_done(g: &Profile, d: &[u8]) -> bool {
	g.sp_extra.is_some() || (0..db1::QUEST_COUNT).any(|q| quest_done(g, d, q))
}

fn sp_extra(g: &Profile, d: &[u8]) -> u32 {
	g.sp_extra.map_or(0, |o| u32_at(d, o) & 0x3FFF_FFFF)
}

pub fn total_sp(g: &Profile, d: &[u8]) -> u32 {
	(0..db1::QUEST_COUNT).filter(|&q| quest_done(g, d, q)).map(|q| quest_base(g, q) + u16::from_le_bytes([d[quest_bonus_at(g, q)], d[quest_bonus_at(g, q) + 1]]) as u32).sum::<u32>() + sp_extra(g, d)
}

pub fn set_total_sp(g: &Profile, d: &mut [u8], want: u32) -> u32 {
	let mut delta = want as i64 - total_sp(g, d) as i64;
	if let Some(o) = g.sp_extra {
		let cur = sp_extra(g, d) as i64;
		let nv = (cur + delta).max(0);
		let hi = u32_at(d, o) & 0xC000_0000;
		set_u32(d, o, if nv == 0 { 0 } else { (if hi == 0 { 0x4000_0000 } else { hi }) | nv as u32 });
		delta -= nv - cur;
	}
	let done: Vec<usize> = (0..db1::QUEST_COUNT).filter(|&q| quest_done(g, d, q)).collect();
	for q in done {
		if delta == 0 {
			break;
		}
		let o = quest_bonus_at(g, q);
		let b = u16::from_le_bytes([d[o], d[o + 1]]) as i64;
		let nb = (b + delta).clamp(0, u16::MAX as i64);
		delta -= nb - b;
		d[o..o + 2].copy_from_slice(&(nb as u16).to_le_bytes());
	}
	total_sp(g, d)
}

pub fn office_rank(g: &Profile, sp: u32) -> &'static str {
	g.ranks.iter().rev().find(|&&(_, t)| sp >= t).map_or("E", |&(r, _)| r)
}

pub fn lgc(g: &Profile, d: &[u8], k: usize) -> u32 {
	u32_at(d, g.lgc + k * 4) & 0x3FFF_FFFF
}

pub fn lgc_level(g: &Profile, k: usize, points: u32) -> (usize, Option<u32>) {
	let s = g.lgc_steps[k];
	let i = (0..s.len()).rev().find(|&i| points >= s[i]).unwrap_or(0);
	(g.lgc_first + i, s.get(i + 1).map(|&n| n - points))
}

pub fn lgc_max(g: &Profile) -> usize {
	g.lgc_first + g.lgc_steps[0].len() - 1
}

pub fn playtime(g: &Profile, d: &[u8]) -> f64 {
	f64::from_le_bytes(d[g.playtime..g.playtime + 8].try_into().unwrap())
}

pub fn set_playtime(g: &Profile, d: &mut [u8], secs: f64) {
	d[g.playtime..g.playtime + 8].copy_from_slice(&secs.to_le_bytes());
}

pub fn set_lgc(g: &Profile, d: &mut [u8], k: usize, v: u32) {
	let o = g.lgc + k * 4;
	let hi = u32_at(d, o) & 0xC000_0000;
	set_u32(d, o, hi | (v & 0x3FFF_FFFF));
}

fn lineup_at(g: &Profile, d: &[u8]) -> usize {
	match g.party_sel {
		Some((sel, base, step)) => {
			let k = (u32_at(d, sel) & 0x3FFF_FFFF) as usize;
			let o = base + k * step;
			if k < 16 && o + db1::LINEUP_MAX * 2 <= d.len() { o } else { g.lineup }
		}
		None => g.lineup,
	}
}

pub fn difficulty(g: &Profile, d: &[u8]) -> (u32, u32) {
	(u32_at(d, g.difficulty) & 0xFF, u32_at(d, g.difficulty - 4) & 0xFF)
}

pub fn set_difficulty(g: &Profile, d: &mut [u8], command: u32, field: u32) {
	let keep = |o: usize, v: u32| (u32_at(d, o) & !0xFF) | v;
	let (c, f) = (keep(g.difficulty, command.min(4)), keep(g.difficulty - 4, field.clamp(1, 3)));
	set_u32(d, g.difficulty, c);
	set_u32(d, g.difficulty - 4, f);
}

pub fn lineup(g: &Profile, d: &[u8]) -> Vec<u32> {
	let at = lineup_at(g, d);
	(0..db1::LINEUP_MAX).map(|i| u16::from_le_bytes([d[at + i * 2], d[at + i * 2 + 1]])).take_while(|&v| v != 0xFFFF).map(u32::from).collect()
}

pub fn set_lineup(g: &Profile, d: &mut [u8], ids: &[u32]) {
	for i in 0..db1::LINEUP_MAX {
		let v = ids.get(i).map_or(0xFFFF, |&c| c as u16);
		let at = lineup_at(g, d);
		d[at + i * 2..at + i * 2 + 2].copy_from_slice(&v.to_le_bytes());
	}
}

#[derive(Clone, Debug)]
pub struct Item {
	pub id: usize,
	pub kind: u8,
	pub owner: u32,
	pub icon: Option<u32>,
	pub en: String,
	pub ru: String,
	pub cat: u8,
	pub max: u32,
	pub eq: [i32; 4],
	pub sex: char,
	pub desc_en: String,
	pub desc_ru: String,
	pub stats: [i32; 16],
}

impl Item {
	pub fn service(&self) -> bool {
		[&self.en, &self.ru, &self.desc_en, &self.desc_ru].iter().any(|s| s.contains('◆') || s.contains('◇'))
	}
}

pub const ITEM_STATS: [(&str, usize); 16] = [("HP", 128), ("EP", 132), ("DMG", 136), ("RES", 140), ("ADMG", 144), ("ARES", 148), ("STR", 152), ("DEF", 156), ("ATS", 160), ("ADF", 164), ("ACC", 176), ("EVA", 180), ("AEV", 184), ("CRT", 188), ("SPD", 192), ("MOV", 196)];

pub fn items(tsv: &str) -> Vec<Item> {
	tsv.lines()
		.filter(|l| !l.starts_with('#') && !l.trim().is_empty())
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			Some(Item {
				id: f.first()?.parse().ok()?,
				kind: f.get(1)?.parse().ok()?,
				owner: f.get(2)?.parse().ok()?,
				icon: f.get(3)?.parse::<i64>().ok().filter(|&v| v >= 0).map(|v| v as u32),
				en: f.get(4)?.to_string(),
				ru: f.get(5).unwrap_or(&"").to_string(),
				cat: f.get(6).and_then(|s| s.parse().ok()).unwrap_or(0),
				max: f.get(7).and_then(|s| s.parse().ok()).unwrap_or(0),
				sex: match f.get(9).copied() {
					Some("M") => 'M',
					Some("F") => 'F',
					_ => ' ',
				},
				eq: {
					let v: Vec<i32> = f.get(8).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default();
					std::array::from_fn(|i| v.get(i).copied().unwrap_or(0))
				},
				desc_en: f.get(10).map(|s| s.replace("\\n", "\n")).unwrap_or_default(),
				desc_ru: f.get(11).map(|s| s.replace("\\n", "\n")).unwrap_or_default(),
				stats: {
					let v: Vec<i32> = f.get(12).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default();
					std::array::from_fn(|i| v.get(i).copied().unwrap_or(0))
				},
			})
		})
		.collect()
}

pub fn kinds(tsv: &str) -> std::collections::HashMap<u8, String> {
	tsv.lines()
		.filter(|l| !l.starts_with('#'))
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			Some((f.first()?.parse().ok()?, f.get(1)?.to_string()))
		})
		.collect()
}

pub fn qlines(tsv: &str) -> std::collections::HashMap<(u32, usize), ([u8; 4], usize)> {
	tsv.lines()
		.filter(|l| !l.starts_with('#') && !l.trim().is_empty())
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			let e: Vec<u8> = f.get(2)?.split(',').filter_map(|x| x.parse::<u8>().ok()).collect();
			let line: usize = f.get(1)?.parse().ok()?;
			Some(((f.first()?.parse().ok()?, line.checked_sub(1)?), (std::array::from_fn(|i| e.get(i).copied().unwrap_or(0)), f.get(3)?.parse().ok()?)))
		})
		.collect()
}

pub fn quartz_element(it: &Item) -> Option<u8> {
	match it.icon? {
		c @ 84..=90 => Some((c - 83) as u8),
		c @ 105..=111 => Some((c - 104) as u8),
		_ => None,
	}
}

pub struct Dlc {
	pub id: u32,
	pub en: String,
	pub ru: String,
	pub items: Vec<(usize, u32)>,
}

pub fn dlcs(tsv: &str) -> Vec<Dlc> {
	tsv.lines()
		.filter(|l| !l.starts_with('#') && !l.trim().is_empty())
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			let items = f.get(3).unwrap_or(&"").split(',').filter_map(|p| p.split_once(':')).filter_map(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?))).collect();
			Some(Dlc { id: f.first()?.parse().ok()?, en: f.get(1)?.to_string(), ru: f.get(2).unwrap_or(&"").to_string(), items })
		})
		.collect()
}

pub fn names(tsv: &str) -> std::collections::HashMap<u32, (String, String)> {
	let mut m = std::collections::HashMap::new();
	for l in tsv.lines().filter(|l| !l.starts_with('#')) {
		let f: Vec<&str> = l.split('\t').collect();
		if let (Some(id), Some(en)) = (f.first().and_then(|s| s.parse::<u32>().ok()), f.get(1)) {
			m.entry(id).or_insert_with(|| (en.to_string(), f.get(2).unwrap_or(&"").to_string()));
		}
	}
	m
}

pub fn models(tsv: &str) -> std::collections::HashMap<u32, u32> {
	let mut m = std::collections::HashMap::new();
	for l in tsv.lines().filter(|l| !l.starts_with('#')) {
		let f: Vec<&str> = l.split('\t').collect();
		if let (Some(id), Some(md)) = (f.first().and_then(|s| s.parse::<u32>().ok()), f.get(3).and_then(|s| s.strip_prefix("chr")).and_then(|s| s.parse::<u32>().ok())) {
			m.entry(id).or_insert(md);
		}
	}
	m
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

pub fn faces_atlas(atlas: &[u8], tsv: &str, cell: u32) -> std::collections::HashMap<u32, Vec<u8>> {
	let mut m = std::collections::HashMap::new();
	let Ok(img) = image::load_from_memory(atlas).map(|i| i.to_rgba8()) else { return m };
	let mut done: std::collections::HashMap<u32, Vec<u8>> = std::collections::HashMap::new();
	for l in tsv.lines() {
		let mut c = l.split('\t');
		let (Some(Ok(id)), Some(Ok(k))) = (c.next().map(|x| x.trim().parse::<u32>()), c.next().map(|x| x.trim().parse::<u32>())) else { continue };
		let (x, y) = ((k % 16) * cell, (k / 16) * cell);
		if x + cell > img.width() || y + cell > img.height() {
			continue;
		}
		let png = done.entry(k).or_insert_with(|| {
			let mut out = Vec::new();
			let _ = image::imageops::crop_imm(&img, x, y, cell, cell).to_image().write_to(&mut std::io::Cursor::new(&mut out), image::ImageOutputFormat::Png);
			out
		});
		m.insert(id, png.clone());
	}
	m
}

pub fn record_chr(g: &Profile, d: &[u8], k: usize, owners: &std::collections::HashMap<usize, u32>) -> Option<u32> {
	let o = chr_at(g, k) + g.equip;
	let w = u16::from_le_bytes([d[o], d[o + 1]]) as usize;
	let own = owners.get(&w).copied().filter(|&c| c != 0xFFFF);
	let base = g.record_chr.get(k).copied().flatten();
	let has_own_record = |c: u32| g.record_chr.iter().enumerate().any(|(j, &r)| j != k && r == Some(c) && u32_at(d, chr_at(g, j)) > 0);
	match own {
		Some(c) if base != Some(c) && has_own_record(c) => base,
		_ => own.or(base),
	}
}

pub struct Notes {
	pub log: usize,
	pub wide: bool,
	pub ach: usize,
	pub counters: usize,
	pub first_type: u32,
	pub mons_type: u32,
	pub mons_tsv: &'static str,
	pub ach_tsv: &'static str,
}

pub static NOTES_DB1: Notes = Notes { log: 0x18FBB8, wide: false, ach: 0x18FFF8, counters: 0x110D4, first_type: 50, mons_type: 55, mons_tsv: "db1_mons.tsv", ach_tsv: "db1_ach.tsv" };
pub static NOTES_DB2: Notes = Notes { log: 0x270538, wide: false, ach: 0x270A38, counters: 0x410E8, first_type: 55, mons_type: 55, mons_tsv: "db2_mons.tsv", ach_tsv: "db2_ach.tsv" };
pub static NOTES_HZ: Notes = Notes { log: 0x295AE0, wide: true, ach: 0x2976E0, counters: 0x420CC, first_type: 48, mons_type: 48, mons_tsv: "hz_mons.tsv", ach_tsv: "hz_ach.tsv" };

pub fn notes(g: &Profile) -> &'static Notes {
	match g.size {
		db2::SIZE => &NOTES_DB2,
		hz::SIZE => &NOTES_HZ,
		_ => &NOTES_DB1,
	}
}

pub struct Mon {
	pub id: usize,
	pub group: u32,
	pub group_name: String,
	pub name: String,
	pub level: u32,
	pub ex: bool,
	pub uncounted: bool,
	pub notes: String,
	pub elems: Vec<(String, i32)>,
	pub stats: Vec<(String, i32)>,
	pub drops: Vec<(usize, u32)>,
}

pub fn mons(tsv: &str) -> Vec<Mon> {
	tsv.lines()
		.filter(|l| !l.starts_with('#') && !l.trim().is_empty())
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			if f.get(6) == Some(&"1") {
				return None;
			}
			Some(Mon { id: f.first()?.parse().ok()?, group: f.get(1)?.parse().ok()?, group_name: f.get(2)?.trim().to_string(), name: f.get(3)?.to_string(), level: f.get(4)?.parse().unwrap_or(0), ex: f.get(5) == Some(&"1"), uncounted: f.get(5).is_some_and(|x| *x != "0"), notes: f.get(7).unwrap_or(&"").to_string(), elems: Vec::new(), stats: Vec::new(), drops: Vec::new() })
		})
		.collect()
}

pub struct Enemy {
	pub pic: String,
	pub boss: bool,
	pub level: u32,
	p: [(f32, f32); 6],
	pub exp: u32,
	pub elems: [i32; 7],
	pub tsm: bool,
	pub sepith: [u32; 8],
	pub items: Vec<(usize, u32)>,
}

pub fn enemies(tsv: &str) -> std::collections::HashMap<usize, Enemy> {
	let pair = |s: &str| s.split_once(':').and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)));
	let nums = |s: &str| s.split(',').filter_map(|x| x.parse().ok()).collect::<Vec<i64>>();
	tsv.lines()
		.filter(|l| !l.starts_with('#'))
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			if f.len() < 15 {
				return None;
			}
			let mut p = [(0.0, 0.0); 6];
			for (k, x) in p.iter_mut().enumerate() {
				*x = pair(f[4 + k])?;
			}
			let (el, sp) = (nums(f[11]), nums(f[13]));
			Some((
				f[0].parse().ok()?,
				Enemy {
					pic: f[1].to_string(),
					boss: f[2] == "1",
					level: f[3].parse().ok()?,
					p,
					exp: f[10].parse().ok()?,
					elems: std::array::from_fn(|k| el.get(k).copied().unwrap_or(100) as i32),
					tsm: f[12] == "1",
					sepith: std::array::from_fn(|k| sp.get(k).copied().unwrap_or(0) as u32),
					items: f[14].split(',').filter_map(|x| x.split_once(':')).filter_map(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?))).collect(),
				},
			))
		})
		.collect()
}

#[derive(Default)]
pub struct Battle {
	turn: std::collections::HashMap<u32, [u32; 2]>,
	adj: std::collections::HashMap<(Option<u32>, u32), [u32; 4]>,
}

pub fn battle(tsv: &str) -> Battle {
	let mut b = Battle::default();
	for l in tsv.lines().filter(|l| !l.starts_with('#')) {
		let f: Vec<&str> = l.split('\t').collect();
		let n = |k: usize| f.get(k).and_then(|x| x.parse::<u32>().ok()).unwrap_or(0);
		match f[0] {
			"T" => {
				b.turn.insert(n(1), [n(2), n(3)]);
			}
			"A" => {
				b.adj.insert((f.get(1).and_then(|x| x.parse().ok()), n(2)), [n(3), n(4), n(5), n(6)]);
			}
			_ => {}
		}
	}
	b
}

#[derive(Debug, PartialEq)]
pub struct EnemyNums {
	pub hp: u32,
	pub str: u32,
	pub def: u32,
	pub ats: u32,
	pub adf: u32,
	pub spd: u32,
}

pub fn enemy_nums(b: &Battle, e: &Enemy, diff: u32, level: u32) -> EnemyNums {
	let x = |p: (f32, f32)| (p.1 * level as f32) as u64 + p.0 as u64;
	let pct = |v: u64, p: u32| v * p as u64 / 100;
	let t = b.turn.get(&diff).copied().unwrap_or([100, 100]);
	let key = b.adj.keys().any(|k| k.0 == Some(diff)).then_some(diff);
	let (lo, hi) = b.adj.keys().filter(|k| k.0 == key).fold((u32::MAX, 0), |(a, z), k| (a.min(k.1), z.max(k.1)));
	let a = b.adj.get(&(key, level.clamp(lo.min(hi), hi))).copied();
	let [hp, st, df, at, ad, sp] = e.p;
	let hp = match a {
		Some(a) => ((hp.0 + hp.1 * level as f32) * a[0] as f32 / 100.0) as u64,
		None => x(hp),
	};
	let a = a.unwrap_or([100; 4]);
	EnemyNums { hp: pct(hp, t[0]) as u32, str: pct(x(st), a[1]) as u32, def: x(df) as u32, ats: pct(x(at), a[2]) as u32, adf: x(ad) as u32, spd: pct(pct(x(sp), t[1]), a[3]).min(999) as u32 }
}

pub struct Ach {
	pub id: usize,
	pub cat: u32,
	pub cat_name: String,
	pub kind: u32,
	pub goal: u32,
	pub name: String,
	pub desc: String,
	pub rewards: Vec<(usize, u32)>,
}

pub fn achs(tsv: &str) -> Vec<Ach> {
	tsv.lines()
		.filter(|l| !l.starts_with('#') && !l.trim().is_empty())
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			Some(Ach { id: f.first()?.parse().ok()?, cat: f.get(1)?.parse().ok()?, cat_name: f.get(2)?.to_string(), kind: f.get(3)?.parse().ok()?, goal: f.get(4)?.parse().ok()?, name: f.get(5)?.to_string(), desc: f.get(6).unwrap_or(&"").to_string(), rewards: [7, 9].iter().filter_map(|&k| Some((f.get(k)?.parse::<usize>().ok()? & 0xFFFF, f.get(k + 1)?.parse().ok()?))).filter(|r: &(usize, u32)| r.0 > 0 && r.1 > 0).collect() })
		})
		.collect()
}

pub fn mon_logged(g: &Profile, d: &[u8], id: usize) -> bool {
	let n = notes(g);
	if n.wide { d[n.log + id * 4] != 0 || d[n.log + id * 4 + 1] != 0 } else { d[n.log + id] != 0 }
}

pub fn mon_level(g: &Profile, d: &[u8], id: usize) -> Option<u16> {
	let n = notes(g);
	n.wide.then(|| u16::from_le_bytes([d[n.log + id * 4 + 2], d[n.log + id * 4 + 3]]))
}

pub fn area_level(g: &Profile, d: &[u8], list: &[Mon], group: u32) -> u16 {
	let lv = |m: &Mon| mon_level(g, d, m.id).filter(|_| mon_logged(g, d, m.id)).unwrap_or(0);
	let area = list.iter().filter(|m| m.group == group).map(lv).max().unwrap_or(0);
	let all = list.iter().map(lv).max().unwrap_or(0);
	[area, all, u32_at(d, g.chars).min(u16::MAX as u32) as u16, 1].into_iter().find(|&v| v > 0).unwrap_or(1)
}

pub fn set_mon_logged(g: &Profile, d: &mut [u8], m: &Mon, on: bool, new_lv: u16) {
	let n = notes(g);
	if n.wide {
		let o = n.log + m.id * 4;
		if on {
			let old = u16::from_le_bytes([d[o + 2], d[o + 3]]);
			let lv = if old > 0 { old } else { new_lv.max(1) };
			d[o..o + 2].copy_from_slice(&1u16.to_le_bytes());
			d[o + 2..o + 4].copy_from_slice(&lv.to_le_bytes());
		} else {
			d[o..o + 4].fill(0);
		}
	} else {
		d[n.log + m.id] = u8::from(on);
	}
}

pub fn ach_counter_at(g: &Profile, kind: u32) -> Option<usize> {
	let n = notes(g);
	(kind >= n.first_type && (kind - n.first_type < 64 || fish_kind(g, kind))).then(|| n.counters + (kind - n.first_type) as usize * 4)
}

pub fn ach_counter(g: &Profile, d: &[u8], kind: u32) -> Option<u32> {
	ach_counter_at(g, kind).map(|o| u32_at(d, o) & 0x3FFF_FFFF)
}

pub fn set_ach_counter(g: &Profile, d: &mut [u8], kind: u32, v: u32) {
	if let Some(o) = ach_counter_at(g, kind) {
		set_u32(d, o, if v == 0 && u32_at(d, o) == 0 { 0 } else { 0x4000_0000 | v.min(0x3FFF_FFFF) });
	}
}

pub fn ach_state(g: &Profile, d: &[u8], id: usize) -> u8 {
	d[notes(g).ach + id]
}

pub fn set_ach_state(g: &Profile, d: &mut [u8], id: usize, v: u8) {
	d[notes(g).ach + id] = v;
}

pub fn mons_logged_count(g: &Profile, d: &[u8], list: &[Mon]) -> u32 {
	list.iter().filter(|m| !m.uncounted && mon_logged(g, d, m.id)).count() as u32
}

pub fn sync_mons_achs(g: &Profile, d: &mut [u8], list: &[Mon], achs: &[Ach], before: u32) {
	sync_achs(g, d, notes(g).mons_type, mons_logged_count(g, d, list), achs, before);
}

fn sync_achs(g: &Profile, d: &mut [u8], k: u32, c: u32, achs: &[Ach], before: u32) {
	set_ach_counter(g, d, k, c);
	for a in achs.iter().filter(|a| a.kind == k) {
		if c >= a.goal && ach_state(g, d, a.id) == 0 {
			set_ach_state(g, d, a.id, 1);
		} else if c < a.goal && before >= a.goal {
			set_ach_state(g, d, a.id, 0);
		}
	}
}

pub struct Fishing {
	pub at: usize,
	pub tsv: &'static str,
	pub first: &'static str,
	pub of_total: bool,
	pub by_level: bool,
	pub kinds: [Option<u32>; 3],
	pub gear: &'static [(&'static str, &'static [usize])],
	pub counted: std::ops::RangeInclusive<usize>,
	pub faces: &'static [(u16, u32, &'static str)],
}

pub static FISHING_DB2: Fishing = Fishing {
	at: 0x2701F8,
	tsv: "db2_fish.tsv",
	first: "Caught",
	of_total: true,
	by_level: false,
	kinds: [Some(146), Some(147), Some(148)],
	gear: &[("Rod", &[349, 359, 350]), ("Reel", &[351, 352]), ("Line", &[353, 354]), ("Hook", &[355, 356]), ("Outfit", &[357, 358])],
	counted: 349..=358,
	faces: &[(0, 6, "Van"), (1, 0, "Agnès"), (2, 1, "Feri"), (3, 2, "Aaron"), (6, 3, "Risette"), (4, 4, "Quatre"), (10, 5, "Judith"), (17, 7, "Swin"), (18, 8, "Nadia"), (151, 9, "Celis"), (152, 10, "Leon"), (19, 11, "Renne"), (11, 12, "Elaine"), (9, 13, "Kincaid"), (12, 14, "Shizuna"), (8, 15, "Zin"), (63, 16, "Fie")],
};

pub static FISHING_HZ: Fishing = Fishing {
	at: 0x2957A0,
	tsv: "hz_fish.tsv",
	first: "Catch List",
	of_total: false,
	by_level: true,
	kinds: [Some(132), None, None],
	gear: &[("Rod", &[345, 346, 347, 348, 349, 350])],
	counted: 345..=350,
	faces: &[(0, 6, "Van"), (1, 0, "Agnès"), (2, 1, "Feri"), (3, 2, "Aaron"), (6, 3, "Risette"), (4, 4, "Quatre"), (10, 5, "Judith"), (5, 17, "Bergard"), (17, 7, "Swin"), (18, 8, "Nadia"), (151, 9, "Celis"), (152, 10, "Leon"), (19, 11, "Renne"), (11, 12, "Elaine"), (9, 13, "Kincaid"), (12, 14, "Shizuna"), (8, 15, "Zin"), (63, 16, "Fie"), (62, 18, "Rixia"), (159, 19, "Jorda"), (20, 20, "Rean"), (21, 21, "Altina"), (22, 22, "Crow"), (23, 23, "Kevin"), (24, 24, "Rufus")],
};

pub fn fishing(g: &Profile) -> Option<&'static Fishing> {
	match g.size {
		db2::SIZE => Some(&FISHING_DB2),
		hz::SIZE => Some(&FISHING_HZ),
		_ => None,
	}
}

pub fn fish_kind(g: &Profile, kind: u32) -> bool {
	fishing(g).is_some_and(|f| f.kinds.contains(&Some(kind)))
}

pub struct Fish {
	pub item: usize,
	pub max: f32,
	pub min: f32,
	pub items: [usize; 3],
	pub counts: [u32; 3],
	pub sepith: [[u32; 8]; 3],
	pub desc: String,
}

impl Fish {
	fn single(&self) -> bool {
		self.items[1] == 0 && self.items[2] == 0
	}
	pub fn rank_a(&self) -> f32 {
		if self.single() { self.max } else { self.min + (self.max - self.min) * 2.0 / 3.0 }
	}
	fn rec_a(&self) -> f32 {
		(self.rank_a() * 10.0 - 0.001).ceil() / 10.0
	}
	pub fn is_a(&self, rec: f32, count: u32) -> bool {
		count > 0 && rec + 0.0001 >= self.rank_a()
	}
	pub fn has_rank(&self, n: usize, rec: f32, count: u32) -> bool {
		match n {
			0 => self.is_a(rec, count),
			1 => count > 0 && rec + 0.0001 >= self.min + (self.max - self.min) / 3.0,
			_ => count > 0,
		}
	}
	pub fn ranks(&self) -> Vec<(&'static str, f32, Option<f32>)> {
		let round = |v: f32| (v * 10.0).round() / 10.0;
		let (a, b) = (round(self.rank_a()), round(self.min + (self.max - self.min) / 3.0));
		if self.single() { vec![("A", a, None)] } else { vec![("A", a, None), ("B", b, Some(a - 0.1)), ("C", self.min, Some(b - 0.1))] }
	}
}

pub fn fishes(tsv: &str) -> Vec<Fish> {
	fn list<T: std::str::FromStr + Copy + Default, const N: usize>(s: &str) -> Option<[T; N]> {
		let v: Vec<T> = s.split(',').filter_map(|x| x.parse().ok()).collect();
		v.try_into().ok()
	}
	tsv.lines()
		.filter(|l| !l.starts_with('#'))
		.filter_map(|l| {
			let f: Vec<&str> = l.split('\t').collect();
			let s: [u32; 24] = list(f.get(5)?)?;
			let mut sepith = [[0; 8]; 3];
			for (k, v) in s.into_iter().enumerate() {
				sepith[k / 8][k % 8] = v;
			}
			Some(Fish { item: f[0].parse().ok()?, max: f[1].parse().ok()?, min: f[2].parse().ok()?, items: list(f[3])?, counts: list(f[4])?, sepith, desc: f.get(6).unwrap_or(&"").to_string() })
		})
		.collect()
}

pub fn fish_rec(f: &Fishing, d: &[u8], i: usize) -> (f32, u32, u16) {
	let o = f.at + i * 12;
	(f32::from_bits(u32_at(d, o)), u32_at(d, o + 4), u16::from_le_bytes([d[o + 8], d[o + 9]]))
}

pub fn set_fish(f: &Fishing, d: &mut [u8], i: usize, rec: f32, count: u32, who: u16) {
	let o = f.at + i * 12;
	if count == 0 {
		d[o..o + 12].fill(0);
		return;
	}
	set_u32(d, o, rec.to_bits());
	set_u32(d, o + 4, count);
	d[o + 8..o + 10].copy_from_slice(&who.to_le_bytes());
	d[o + 10..o + 12].copy_from_slice(&1u16.to_le_bytes());
}

pub fn mark_fish(f: &Fishing, d: &mut [u8], i: usize, x: &Fish, on: bool) {
	let (rec, count, who) = fish_rec(f, d, i);
	if !on {
		set_fish(f, d, i, 0.0, 0, 0);
	} else {
		set_fish(f, d, i, if x.is_a(rec, count) { rec } else { x.rec_a() }, count.max(1), if count > 0 { who } else { 0 });
	}
}

pub fn set_rank_a(f: &Fishing, d: &mut [u8], i: usize, x: &Fish, on: bool) {
	let (rec, count, who) = fish_rec(f, d, i);
	if on {
		mark_fish(f, d, i, x, true);
	} else if x.is_a(rec, count) && !x.single() {
		set_fish(f, d, i, x.rec_a() - 0.1, count, who);
	}
}

pub fn fish_counts(g: &Profile, f: &Fishing, d: &[u8], list: &[Fish]) -> [u32; 3] {
	let mut c = [0; 3];
	for (i, x) in list.iter().enumerate() {
		let (rec, n, _) = fish_rec(f, d, i);
		c[0] += u32::from(n > 0);
		c[1] += u32::from(x.is_a(rec, n));
	}
	c[2] = f.counted.clone().filter(|&id| item_count(g, d, id) > 0).count() as u32;
	c
}

pub fn sync_fish(g: &Profile, f: &Fishing, d: &mut [u8], list: &[Fish], achs: &[Ach], before: [u32; 3]) {
	let now = fish_counts(g, f, d, list);
	for k in 0..3 {
		if let Some(kind) = f.kinds[k]
			&& now[k] != before[k]
		{
			sync_achs(g, d, kind, now[k], achs, before[k]);
		}
	}
}

pub fn give_mare_form(g: &Profile, d: &mut [u8], k: usize) {
	if g.size != DB1_SIZE || k > 2 {
		return;
	}
	let id = 3301 + k;
	if lgc_level(g, k, lgc(g, d, k)).0 >= 3 && item_count(g, d, id) == 0 {
		set_item_count(g, d, id, 1);
	}
}

pub const TABS: [&str; 9] = ["Items", "Cooking", "Weapons", "Armor", "Acc.", "Quartz", "Mats.", "Key", "Holo Cores"];
pub const TABS_DB2: [&str; 10] = ["Items", "Droplets", "Cooking", "Weapons", "Armor", "Acc.", "Quartz", "Mats.", "Key", "Holo Cores"];
pub const TABS_HZ: [&str; 10] = TABS_DB2;

pub fn item_tab(g: &Profile, it: &Item) -> Option<usize> {
	if it.kind == 0 || (310..=319).contains(&it.id) || g.hidden.contains(&it.id) {
		return None;
	}
	let name = match it.cat {
		1 if it.kind == 36 => "Droplets",
		1 if matches!(it.kind, 3 | 4) => "Cooking",
		1 => "Items",
		12 | 13 => "Weapons",
		14 | 15 => "Armor",
		16 => "Acc.",
		22 => "Quartz",
		30 => "Mats.",
		20 | 21 | 23 | 25 => "Holo Cores",
		_ => "Key",
	};
	g.tabs.iter().position(|&t| t == name)
}

