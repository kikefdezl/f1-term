use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata, Record};

pub const APP: &str = "f1-term";
const LOGFILE: &str = "f1-term.log";

struct FileLogger {
    file: Mutex<File>,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata())
            && let Ok(mut file) = self.file.lock()
        {
            let now = chrono::Utc::now();
            let _ = writeln!(
                file,
                "{} [{}] {}: {}",
                now.format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

pub fn get_log_dir() -> std::path::PathBuf {
    let mut path = if let Some(state_home) = std::env::var_os("XDG_STATE_HOME") {
        std::path::PathBuf::from(state_home)
    } else if let Some(data_local) = std::env::var_os("LOCALAPPDATA") {
        std::path::PathBuf::from(data_local)
    } else if let Some(home) = std::env::var_os("HOME") {
        let mut p = std::path::PathBuf::from(home);
        p.push(".local");
        p.push("state");
        p
    } else {
        return std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    };

    path.push(APP);
    let _ = std::fs::create_dir_all(&path);
    path
}

pub fn init() -> std::path::PathBuf {
    let log_dir = get_log_dir();
    let log_path = log_dir.join(LOGFILE);

    if let Ok(file) = File::create(&log_path) {
        let logger = FileLogger {
            file: Mutex::new(file),
        };
        let _ = log::set_boxed_logger(Box::new(logger));
        log::set_max_level(LevelFilter::Debug);
    }

    log_dir
}
