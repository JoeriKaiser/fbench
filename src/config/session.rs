use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionState {
    pub left_tab: String,
    pub sidebar_scroll_position: f64,
    pub editor_panel_height: f64,
}

pub struct SessionStore {
    config_path: PathBuf,
}

impl SessionStore {
    pub fn new() -> Self {
        let config_dir = directories::ProjectDirs::from("com", "fbench", "fbench")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        fs::create_dir_all(&config_dir).ok();

        Self {
            config_path: config_dir.join("session.json"),
        }
    }

    pub fn load(&self) -> SessionState {
        fs::read_to_string(&self.config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, state: &SessionState) -> Result<(), String> {
        let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
