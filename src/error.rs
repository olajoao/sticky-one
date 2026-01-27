use thiserror::Error;

#[derive(Error, Debug)]
pub enum StickyError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Daemon already running (pid: {0})")]
    DaemonRunning(u32),

    #[error("Daemon not running")]
    DaemonNotRunning,

    #[error("Daemon error: {0}")]
    Daemon(String),

    #[error("Entry not found: {0}")]
    NotFound(i64),

    #[error("Image too large: {size} bytes (max: {max})")]
    ImageTooLarge { size: usize, max: usize },
}

pub type Result<T> = std::result::Result<T, StickyError>;
