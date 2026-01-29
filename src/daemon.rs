use crate::clipboard::read_as_entry;
use crate::config::{pid_path, Config, POLL_INTERVAL_MS};
use crate::error::{Result, StickyError};
use crate::hotkey::HotkeyListener;
use crate::storage::Storage;
use std::fs;
use std::process::Command;
use std::time::Duration;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::time::interval;

pub struct Daemon {
    storage: Storage,
    last_hash: Option<String>,
    config: Config,
}

impl Daemon {
    pub fn new() -> Result<Self> {
        let storage = Storage::open()?;
        let last_hash = storage.get_latest_hash()?;
        let config = Config::load();
        Ok(Self {
            storage,
            last_hash,
            config,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.storage.cleanup_old()?;

        let mut poll = interval(Duration::from_millis(POLL_INTERVAL_MS));

        // Setup hotkey listener
        let (hotkey_tx, mut hotkey_rx) = mpsc::channel::<()>(1);
        let hotkey_listener = HotkeyListener::new(&self.config.hotkey)?;

        tokio::spawn(async move {
            if let Err(e) = hotkey_listener.listen(hotkey_tx).await {
                eprintln!("Hotkey listener error: {}", e);
            }
        });

        loop {
            tokio::select! {
                _ = poll.tick() => {
                    if let Err(e) = self.poll_clipboard() {
                        eprintln!("Clipboard poll error: {}", e);
                    }
                }
                Some(()) = hotkey_rx.recv() => {
                    self.spawn_popup();
                }
                _ = signal::ctrl_c() => {
                    self.cleanup()?;
                    break;
                }
            }
        }

        Ok(())
    }

    fn spawn_popup(&self) {
        // Get current executable path
        if let Ok(exe) = std::env::current_exe() {
            let _ = Command::new(exe).arg("popup").spawn();
        }
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
