use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedQuery {
    pub name: String,
    pub sql: String,
    pub is_bookmarked: bool,
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

    pub fn toggle_bookmark(&self, name: &str) -> Result<(), String> {
        let mut queries = self.load_queries();
        if let Some(query) = queries.iter_mut().find(|q| q.name == name) {
            query.is_bookmarked = !query.is_bookmarked;
            self.save_queries(&queries)
        } else {
            Err(format!("Query '{}' not found", name))
        }
    }

    pub fn get_bookmarked_queries(&self) -> Vec<SavedQuery> {
        self.load_queries()
            .into_iter()
            .filter(|q| q.is_bookmarked)
            .collect()
    }
}

impl Default for QueryStore {
    fn default() -> Self {
        Self::new()
    }
}
