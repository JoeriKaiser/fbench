use crate::db::SchemaInfo;
use dioxus::prelude::*;

// Re-export DatabaseType from db module
pub use crate::db::DatabaseType;

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected {
        db_type: DatabaseType,
        db_name: String,
    },
    ConnectionLost,
    Error(String),
}

pub static CONNECTION: GlobalSignal<ConnectionState> =
    Signal::global(|| ConnectionState::Disconnected);

pub static SCHEMA: GlobalSignal<SchemaInfo> = Signal::global(SchemaInfo::default);

pub static CURRENT_DB_TYPE: GlobalSignal<Option<DatabaseType>> = Signal::global(|| None);

pub static RECENT_TABLES: GlobalSignal<Vec<String>> = Signal::global(|| Vec::new());
