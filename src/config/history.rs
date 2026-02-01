use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_HISTORY_ITEMS: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    pub sql: String,
    pub executed_at: DateTime<Local>,
    pub row_count: Option<usize>,
    pub execution_time_ms: Option<u64>,
}

pub struct QueryHistory {
    config_path: PathBuf,
    entries: Vec<HistoryEntry>,
}

impl QueryHistory {
    pub fn new() -> Self {
        let config_dir = directories::ProjectDirs::from("com", "fbench", "fbench")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        fs::create_dir_all(&config_dir).ok();

        let config_path = config_dir.join("history.json");
        let entries = Self::load_entries(&config_path);

        Self {
            config_path,
            entries,
        }
    }

    fn load_entries(path: &PathBuf) -> Vec<HistoryEntry> {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_entries(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.entries).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }

    pub fn add_entry(
        &mut self,
        sql: String,
        row_count: Option<usize>,
        execution_time_ms: Option<u64>,
    ) {
        let entry = HistoryEntry {
            sql: sql.trim().to_string(),
            executed_at: Local::now(),
            row_count,
            execution_time_ms,
        };

        // Don't add duplicates at the top
        if let Some(first) = self.entries.first() {
            if first.sql == entry.sql {
                // Update the existing entry with new execution info
                self.entries[0] = entry;
                let _ = self.save_entries();
                return;
            }
        }

        self.entries.insert(0, entry);

        // Keep only the most recent MAX_HISTORY_ITEMS
        if self.entries.len() > MAX_HISTORY_ITEMS {
            self.entries.truncate(MAX_HISTORY_ITEMS);
        }

        let _ = self.save_entries();
    }

    pub fn get_entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        let _ = self.save_entries();
    }
}

impl Default for QueryHistory {
    fn default() -> Self {
        Self::new()
    }
}
