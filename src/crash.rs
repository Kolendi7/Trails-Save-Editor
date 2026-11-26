use std::io::Write;
use std::path::PathBuf;

pub const FILE: &str = "crash_report.txt";

fn report_path() -> PathBuf {
	let near = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join(FILE)));
	if let Some(p) = near {
		if std::fs::OpenOptions::new().create(true).append(true).open(&p).is_ok() {
			return p;
		}
	}
	std::env::temp_dir().join(FILE)
}

pub fn report_text(program: &str, version: &str, thread: &str, message: &str, location: &str) -> String {
	let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
	format!(
		"=== {program} {version} — crash\r\ntime (UTC, seconds since 1970): {secs}\r\nsystem: {} {}\r\nthread: {thread}\r\nreason: {message}\r\nlocation: {location}\r\n",
		std::env::consts::OS,
		std::env::consts::ARCH
	)
}

pub fn install(program: &'static str, version: &'static str) {
	std::panic::set_hook(Box::new(move |info| {
		let message = info
			.payload()
			.downcast_ref::<&str>()
			.map(|s| s.to_string())
			.or_else(|| info.payload().downcast_ref::<String>().cloned())
			.unwrap_or_else(|| "(no text)".into());
		let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
		let cur = std::thread::current();
		let thread = cur.name().unwrap_or("unnamed").to_string();
		let mut text = report_text(program, version, &thread, &message, &location);
		text.push_str("stack:\r\n");
		text.push_str(&std::backtrace::Backtrace::force_capture().to_string().replace('\n', "\r\n"));
		text.push_str("\r\n\r\n");
		let path = report_path();
		let saved = std::fs::OpenOptions::new().create(true).append(true).open(&path).and_then(|mut f| f.write_all(text.as_bytes())).is_ok();
		eprintln!("{program}: {message} ({location})");
		if thread == "main" {
			show_box(program, &message, saved.then_some(&path));
		}
	}));
}

#[cfg(windows)]
fn show_box(program: &str, message: &str, path: Option<&PathBuf>) {
	use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
	let where_ = match path {
		Some(p) => format!("Report saved:\n{}\n\nPlease send this file to the author.", p.display()),
		None => "The report could not be saved.".into(),
	};
	let body = format!("{program} crashed and will close.\n\nReason: {message}\n\n{where_}");
	let w = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
	let (b, t) = (w(&body), w(program));
	unsafe { MessageBoxW(std::ptr::null_mut(), b.as_ptr(), t.as_ptr(), MB_OK | MB_ICONERROR) };
}

#[cfg(not(windows))]
fn show_box(_program: &str, _message: &str, _path: Option<&PathBuf>) {}

