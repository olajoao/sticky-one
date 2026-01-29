use crate::config::HotkeyConfig;
use crate::error::{Result, StickyError};
use evdev::{Device, EventSummary, EventType, KeyCode};
use std::collections::HashSet;
use std::path::Path;
use tokio::sync::mpsc;

pub struct HotkeyListener {
    modifiers: HashSet<KeyCode>,
    trigger_key: KeyCode,
}

impl HotkeyListener {
    pub fn new(config: &HotkeyConfig) -> Result<Self> {
        let modifiers = config.modifier_keys();
        let trigger_key = config
            .trigger_key()
            .ok_or_else(|| StickyError::Hotkey(format!("invalid trigger key: {}", config.key)))?;

        if modifiers.is_empty() {
            return Err(StickyError::Hotkey("no valid modifiers configured".into()));
        }

        Ok(Self {
            modifiers,
            trigger_key,
        })
    }

    pub async fn listen(&self, tx: mpsc::Sender<()>) -> Result<()> {
        let devices = find_keyboards()?;
        if devices.is_empty() {
            return Err(StickyError::Hotkey(
                "no keyboard devices found (are you in the input group?)".into(),
            ));
        }

        let (event_tx, mut event_rx) = mpsc::channel::<(KeyCode, bool)>(100);

        for device in devices {
            let tx = event_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = listen_device(device, tx).await {
                    eprintln!("device listener error: {}", e);
                }
            });
        }

        drop(event_tx);

        let mut pressed: HashSet<KeyCode> = HashSet::new();

        while let Some((key, is_press)) = event_rx.recv().await {
            if is_press {
                pressed.insert(key);
            } else {
                pressed.remove(&key);
            }

            if key == self.trigger_key
                && is_press
                && self.modifiers.iter().all(|m| pressed.contains(m))
            {
                let _ = tx.send(()).await;
            }
        }

        Ok(())
    }
}

fn find_keyboards() -> Result<Vec<Device>> {
    let mut keyboards = Vec::new();
    let input_dir = Path::new("/dev/input");

    if !input_dir.exists() {
        return Err(StickyError::Hotkey("/dev/input not found".into()));
    }

    for entry in std::fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("event"))
            .unwrap_or(false)
        {
            continue;
        }

        if let Ok(device) = Device::open(&path) {
            if device
                .supported_keys()
                .map(|keys| keys.contains(KeyCode::KEY_A))
                .unwrap_or(false)
            {
                keyboards.push(device);
            }
        }
    }

    Ok(keyboards)
}

async fn listen_device(device: Device, tx: mpsc::Sender<(KeyCode, bool)>) -> Result<()> {
    let mut stream = device
        .into_event_stream()
        .map_err(|e| StickyError::Hotkey(e.to_string()))?;

    loop {
        let event = stream
            .next_event()
            .await
            .map_err(|e| StickyError::Hotkey(e.to_string()))?;

        if event.event_type() != EventType::KEY {
            continue;
        }

        if let EventSummary::Key(_, key, value) = event.destructure() {
            let is_press = value == 1;
            let is_release = value == 0;

            if is_press || is_release {
                let _ = tx.send((key, is_press)).await;
            }
        }
    }
}
