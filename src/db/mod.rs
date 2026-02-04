mod connection;
mod query;

pub use connection::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DatabaseType {
    #[default]
    PostgreSQL,
    MySQL,
}

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub schema: String,
}

impl ConnectionConfig {
    pub fn connection_string(&self) -> String {
        match self.db_type {
            DatabaseType::PostgreSQL => format!(
                "postgres://{}:{}@{}:{}/{}",
                self.user, self.password, self.host, self.port, self.database
            ),
            DatabaseType::MySQL => format!(
                "mysql://{}:{}@{}:{}/{}",
                self.user, self.password, self.host, self.port, self.database
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintInfo {
    pub name: String,
    pub constraint_type: String,
    pub columns: Vec<String>,
    pub foreign_table: Option<String>,
    pub foreign_columns: Option<Vec<String>>,
    pub check_clause: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
    pub constraints: Vec<ConstraintInfo>,
    pub row_estimate: i64,
}

#[derive(Debug, Clone, Default)]
pub struct SchemaInfo {
    pub tables: Vec<TableInfo>,
    pub views: Vec<String>,
}

#[derive(Debug)]
pub enum DbRequest {
    Connect(ConnectionConfig),
    TestConnection(ConnectionConfig),
    Execute(String),
    Explain(String),
    ListTables,
    FetchSchema,
    FetchTableDetails(String),
    Disconnect,
}

#[derive(Debug)]
pub enum DbResponse {
    Connected(DatabaseType),
    ConnectionFailed(String),
    TestResult(Result<(), String>),
    QueryResult(QueryResult),
    ExplainResult(String),
    Schema(SchemaInfo),
    TableDetails(TableInfo),
    Error(String),
    Disconnected,
    ConnectionLost,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub sql: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub execution_time_ms: u64,
}
