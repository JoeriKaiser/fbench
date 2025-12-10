use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::db::DatabaseType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConnection {
    pub name: String,
    #[serde(default)]
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub database: String,
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub save_password: bool,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ConnectionsFile {
    connections: Vec<SavedConnection>,
    #[serde(default)]
    last_used: Option<String>,
}

pub struct ConnectionStore {
    config_path: PathBuf,
}

impl ConnectionStore {
    pub fn new() -> Self {
        let config_dir = directories::ProjectDirs::from("com", "fbench", "fbench")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        
        fs::create_dir_all(&config_dir).ok();
        
        Self {
            config_path: config_dir.join("connections.json"),
        }
    }

    fn load_file(&self) -> ConnectionsFile {
        fs::read_to_string(&self.config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_file(&self, file: &ConnectionsFile) -> Result<(), String> {
        let json = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }

    pub fn load_connections(&self) -> Vec<SavedConnection> {
        self.load_file().connections
    }

    pub fn get_last_used(&self) -> Option<String> {
        self.load_file().last_used
    }

    pub fn set_last_used(&self, name: &str) -> Result<(), String> {
        let mut file = self.load_file();
        file.last_used = Some(name.to_string());
        self.save_file(&file)
    }

    pub fn save_connections(&self, connections: &[SavedConnection]) -> Result<(), String> {
        let mut file = self.load_file();
        file.connections = connections.to_vec();
        self.save_file(&file)
    }

    pub fn get_password(&self, connection_name: &str) -> Option<String> {
        let entry = keyring::Entry::new("fbench", connection_name).ok()?;
        entry.get_password().ok()
    }

    pub fn set_password(&self, connection_name: &str, password: &str) -> Result<(), String> {
        let entry = keyring::Entry::new("fbench", connection_name)
            .map_err(|e| e.to_string())?;
        entry.set_password(password).map_err(|e| e.to_string())
    }

    pub fn delete_password(&self, connection_name: &str) -> Result<(), String> {
        let entry = keyring::Entry::new("fbench", connection_name)
            .map_err(|e| e.to_string())?;
        entry.delete_credential().map_err(|e| e.to_string())
    }
}

impl Default for ConnectionStore {
    fn default() -> Self {
        Self::new()
    }
}
