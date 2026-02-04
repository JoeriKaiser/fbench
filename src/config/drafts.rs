use crate::state::QueryTab;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorDraft {
    pub content: String,
    pub saved_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabDraft {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftData {
    pub tabs: Vec<TabDraft>,
    pub active_tab_index: usize,
}

pub struct DraftStore {
    config_path: PathBuf,
}

impl DraftStore {
    pub fn new() -> Self {
        let config_dir = directories::ProjectDirs::from("com", "fbench", "fbench")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        fs::create_dir_all(&config_dir).ok();

        Self {
            config_path: config_dir.join("draft.json"),
        }
    }

    pub fn load(&self) -> EditorDraft {
        fs::read_to_string(&self.config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, content: &str) -> Result<(), String> {
        let draft = EditorDraft {
            content: content.to_string(),
            saved_at: Some(chrono::Utc::now()),
        };
        let json = serde_json::to_string_pretty(&draft).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }

    pub fn clear(&self) -> Result<(), String> {
        let draft = EditorDraft::default();
        let json = serde_json::to_string_pretty(&draft).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }

    // Multi-tab support
    pub fn save_tabs(&self, tabs: &[QueryTab], active_id: Option<&str>) -> Result<(), String> {
        let active_index = active_id
            .and_then(|id| tabs.iter().position(|t| t.id == id))
            .unwrap_or(0);

        let data = DraftData {
            tabs: tabs
                .iter()
                .map(|t| TabDraft {
                    title: t.title.clone(),
                    content: t.content.clone(),
                })
                .collect(),
            active_tab_index: active_index,
        };

        let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }

    pub fn load_tabs(&self) -> Option<DraftData> {
        fs::read_to_string(&self.config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }
}

impl Default for DraftStore {
    fn default() -> Self {
        Self::new()
    }
}
