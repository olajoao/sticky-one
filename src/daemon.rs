use crate::clipboard::read_as_entry;
use crate::config::{pid_path, POLL_INTERVAL_MS};
use crate::error::{Result, StickyError};
use crate::storage::Storage;
use std::fs;
use std::time::Duration;
use tokio::signal;
use tokio::time::interval;

pub struct Daemon {
    storage: Storage,
    last_hash: Option<String>,
}

impl Daemon {
    pub fn new() -> Result<Self> {
        let storage = Storage::open()?;
        let last_hash = storage.get_latest_hash()?;
        Ok(Self { storage, last_hash })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.storage.cleanup_old()?;

        let mut poll = interval(Duration::from_millis(POLL_INTERVAL_MS));

        loop {
            tokio::select! {
                _ = poll.tick() => {
                    if let Err(e) = self.poll_clipboard() {
                        eprintln!("Clipboard poll error: {}", e);
                    }
                }
                _ = signal::ctrl_c() => {
                    self.cleanup()?;
                    break;
                }
            }
        }

        Ok(())
    }

    fn poll_clipboard(&mut self) -> Result<()> {
        let entry = match read_as_entry() {
            Ok(Some(e)) => e,
            Ok(None) => return Ok(()),
            Err(StickyError::ImageTooLarge { .. }) => return Ok(()),
            Err(e) => return Err(e),
        };

        // Skip if same as last entry (dedup)
        if self.last_hash.as_ref() == Some(&entry.hash) {
            return Ok(());
        }

        self.storage.insert(&entry)?;
        self.last_hash = Some(entry.hash);

        // Periodic cleanup
        self.storage.cleanup_old()?;

        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        let path = pid_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

pub fn is_running() -> Option<u32> {
    let path = pid_path();
    if !path.exists() {
        return None;
    }

    let pid_str = fs::read_to_string(&path).ok()?;
    let pid: u32 = pid_str.trim().parse().ok()?;

    // Check if process exists
    #[cfg(unix)]
    {
        use std::process::Command;
        let status = Command::new("kill").args(["-0", &pid.to_string()]).status();
        if status.map(|s| s.success()).unwrap_or(false) {
            return Some(pid);
        }
    }

    #[cfg(windows)]
    {
        // On Windows, just trust the PID file for now
        return Some(pid);
    }

    // Stale PID file, remove it
    let _ = fs::remove_file(&path);
    None
}

pub fn stop() -> Result<()> {
    let pid = is_running().ok_or(StickyError::DaemonNotRunning)?;

    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .args([&pid.to_string()])
            .status()
            .map_err(|e| StickyError::Io(e))?;
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()
            .map_err(|e| StickyError::Io(e))?;
    }

    // Remove PID file
    let path = pid_path();
    if path.exists() {
        fs::remove_file(&path)?;
    }

    Ok(())
}
