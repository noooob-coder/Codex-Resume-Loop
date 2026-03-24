use chrono::Local;
use directories::ProjectDirs;
use std::backtrace::Backtrace;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn log_path() -> Option<PathBuf> {
    let dirs = ProjectDirs::from("dev", "shcem", "crl-desktop")?;
    let dir = dirs.config_dir();
    if fs::create_dir_all(dir).is_err() {
        return None;
    }
    Some(dir.join("crl-desktop.log"))
}

pub fn append_log(message: &str) {
    let Some(path) = log_path() else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(
        file,
        "[{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        message
    );
}

pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_else(|| "unknown".to_owned());
        let payload = if let Some(message) = info.payload().downcast_ref::<&str>() {
            (*message).to_owned()
        } else if let Some(message) = info.payload().downcast_ref::<String>() {
            message.clone()
        } else {
            "non-string panic payload".to_owned()
        };

        append_log(&format!("panic at {location}: {payload}"));
        append_log(&format!("backtrace: {:?}", Backtrace::force_capture()));
        previous(info);
    }));
}
