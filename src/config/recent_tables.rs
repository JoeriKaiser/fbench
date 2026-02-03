use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_RECENT_TABLES: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentTableEntry {
    pub table_name: String,
    pub accessed_at: DateTime<Utc>,
}

pub struct RecentTablesStore {
    config_path: PathBuf,
}

impl RecentTablesStore {
    pub fn new() -> Self {
        let config_dir = directories::ProjectDirs::from("com", "fbench", "fbench")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        fs::create_dir_all(&config_dir).ok();

        Self {
            config_path: config_dir.join("recent_tables.json"),
        }
    }

    pub fn load(&self) -> Vec<RecentTableEntry> {
        fs::read_to_string(&self.config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn add(&self, table_name: &str) -> Result<(), String> {
        let mut entries = self.load();

        // Remove if already exists
        entries.retain(|e| e.table_name != table_name);

        // Add to front
        entries.insert(
            0,
            RecentTableEntry {
                table_name: table_name.to_string(),
                accessed_at: Utc::now(),
            },
        );

        // Trim to max
        entries.truncate(MAX_RECENT_TABLES);

        self.save(&entries)
    }

    fn save(&self, entries: &[RecentTableEntry]) -> Result<(), String> {
        let json = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }
}

impl Default for RecentTablesStore {
    fn default() -> Self {
        Self::new()
    }
}
