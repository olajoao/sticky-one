use directories::ProjectDirs;
use evdev::KeyCode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{collections::HashSet, fs};

pub const APP_NAME: &str = "sticky_one";
pub const RETENTION_HOURS: i64 = 12;
pub const POLL_INTERVAL_MS: u64 = 500;
pub const MAX_IMAGE_SIZE_BYTES: usize = 5 * 1024 * 1024; // 5MB
pub const PID_FILE: &str = "daemon.pid";
pub const CONFIG_FILE: &str = "config.toml";

pub fn data_dir() -> PathBuf {
    ProjectDirs::from("", "", APP_NAME)
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn config_dir() -> PathBuf {
    ProjectDirs::from("", "", APP_NAME)
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn db_path() -> PathBuf {
    data_dir().join("clipboard.db")
}

pub fn pid_path() -> PathBuf {
    data_dir().join(PID_FILE)
}

pub fn config_path() -> PathBuf {
    config_dir().join(CONFIG_FILE)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub hotkey: HotkeyConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: HotkeyConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: vec!["Alt".to_string(), "Shift".to_string()],
            key: "C".to_string(),
        }
    }
}

impl HotkeyConfig {
    pub fn modifier_keys(&self) -> HashSet<KeyCode> {
        self.modifiers
            .iter()
            .filter_map(|m| parse_modifier(m))
            .collect()
    }

    pub fn trigger_key(&self) -> Option<KeyCode> {
        parse_key(&self.key)
    }
}

fn parse_modifier(name: &str) -> Option<KeyCode> {
    match name.to_lowercase().as_str() {
        "alt" | "left_alt" => Some(KeyCode::KEY_LEFTALT),
        "right_alt" | "altgr" => Some(KeyCode::KEY_RIGHTALT),
        "shift" | "left_shift" => Some(KeyCode::KEY_LEFTSHIFT),
        "right_shift" => Some(KeyCode::KEY_RIGHTSHIFT),
        "ctrl" | "control" | "left_ctrl" => Some(KeyCode::KEY_LEFTCTRL),
        "right_ctrl" => Some(KeyCode::KEY_RIGHTCTRL),
        "super" | "meta" | "win" | "left_meta" => Some(KeyCode::KEY_LEFTMETA),
        "right_meta" => Some(KeyCode::KEY_RIGHTMETA),
        _ => None,
    }
}

fn parse_key(name: &str) -> Option<KeyCode> {
    match name.to_uppercase().as_str() {
        "A" => Some(KeyCode::KEY_A),
        "B" => Some(KeyCode::KEY_B),
        "C" => Some(KeyCode::KEY_C),
        "D" => Some(KeyCode::KEY_D),
        "E" => Some(KeyCode::KEY_E),
        "F" => Some(KeyCode::KEY_F),
        "G" => Some(KeyCode::KEY_G),
        "H" => Some(KeyCode::KEY_H),
        "I" => Some(KeyCode::KEY_I),
        "J" => Some(KeyCode::KEY_J),
        "K" => Some(KeyCode::KEY_K),
        "L" => Some(KeyCode::KEY_L),
        "M" => Some(KeyCode::KEY_M),
        "N" => Some(KeyCode::KEY_N),
        "O" => Some(KeyCode::KEY_O),
        "P" => Some(KeyCode::KEY_P),
        "Q" => Some(KeyCode::KEY_Q),
        "R" => Some(KeyCode::KEY_R),
        "S" => Some(KeyCode::KEY_S),
        "T" => Some(KeyCode::KEY_T),
        "U" => Some(KeyCode::KEY_U),
        "V" => Some(KeyCode::KEY_V),
        "W" => Some(KeyCode::KEY_W),
        "X" => Some(KeyCode::KEY_X),
        "Y" => Some(KeyCode::KEY_Y),
        "Z" => Some(KeyCode::KEY_Z),
        "0" => Some(KeyCode::KEY_0),
        "1" => Some(KeyCode::KEY_1),
        "2" => Some(KeyCode::KEY_2),
        "3" => Some(KeyCode::KEY_3),
        "4" => Some(KeyCode::KEY_4),
        "5" => Some(KeyCode::KEY_5),
        "6" => Some(KeyCode::KEY_6),
        "7" => Some(KeyCode::KEY_7),
        "8" => Some(KeyCode::KEY_8),
        "9" => Some(KeyCode::KEY_9),
        "SPACE" => Some(KeyCode::KEY_SPACE),
        "ENTER" | "RETURN" => Some(KeyCode::KEY_ENTER),
        "ESCAPE" | "ESC" => Some(KeyCode::KEY_ESC),
        "TAB" => Some(KeyCode::KEY_TAB),
        "BACKSPACE" => Some(KeyCode::KEY_BACKSPACE),
        "F1" => Some(KeyCode::KEY_F1),
        "F2" => Some(KeyCode::KEY_F2),
        "F3" => Some(KeyCode::KEY_F3),
        "F4" => Some(KeyCode::KEY_F4),
        "F5" => Some(KeyCode::KEY_F5),
        "F6" => Some(KeyCode::KEY_F6),
        "F7" => Some(KeyCode::KEY_F7),
        "F8" => Some(KeyCode::KEY_F8),
        "F9" => Some(KeyCode::KEY_F9),
        "F10" => Some(KeyCode::KEY_F10),
        "F11" => Some(KeyCode::KEY_F11),
        "F12" => Some(KeyCode::KEY_F12),
        _ => None,
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            let config = Config::default();
            let _ = config.save();
            config
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).unwrap_or_default();
        fs::write(path, content)
    }
}
