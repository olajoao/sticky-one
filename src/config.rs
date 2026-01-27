use directories::ProjectDirs;
use std::path::PathBuf;

pub const APP_NAME: &str = "sticky_one";
pub const RETENTION_HOURS: i64 = 12;
pub const POLL_INTERVAL_MS: u64 = 500;
pub const MAX_IMAGE_SIZE_BYTES: usize = 5 * 1024 * 1024; // 5MB
pub const PID_FILE: &str = "daemon.pid";

pub fn data_dir() -> PathBuf {
    ProjectDirs::from("", "", APP_NAME)
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn db_path() -> PathBuf {
    data_dir().join("clipboard.db")
}

pub fn pid_path() -> PathBuf {
    data_dir().join(PID_FILE)
}
