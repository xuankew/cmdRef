use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Write a debug log entry to ~/.config/cmdref/debug.log
/// Only active when CMDREF_DEBUG=1 environment variable is set.
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if std::env::var("CMDREF_DEBUG").is_ok() {
            $crate::debug::write_log(&format!($($arg)*));
        }
    };
}

fn log_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("cmdref");
    std::fs::create_dir_all(&path).ok();
    path.push("debug.log");
    path
}

pub fn write_log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

/// Initialize the debug log file (truncate if exists)
pub fn init() {
    if std::env::var("CMDREF_DEBUG").is_ok() {
        let path = log_path();
        let _ = File::create(&path);
        write_log("=== CmdRef debug session started ===");
    }
}
