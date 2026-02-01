use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedQuery {
    pub name: String,
    pub sql: String,
}

pub struct QueryStore {
    config_path: PathBuf,
}

impl QueryStore {
    pub fn new() -> Self {
        let config_dir = directories::ProjectDirs::from("com", "fbench", "fbench")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        fs::create_dir_all(&config_dir).ok();

        Self {
            config_path: config_dir.join("queries.json"),
        }
    }

    pub fn load_queries(&self) -> Vec<SavedQuery> {
        fs::read_to_string(&self.config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_queries(&self, queries: &[SavedQuery]) -> Result<(), String> {
        let json = serde_json::to_string_pretty(queries).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }
}

impl Default for QueryStore {
    fn default() -> Self {
        Self::new()
    }
}
