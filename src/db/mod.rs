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
    // Phase 2: Data mutations
    ExecuteMutation(String),
    ExecuteBatch(Vec<String>),
    ImportData {
        table: String,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        batch_size: usize,
    },
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
    // Phase 2: Mutation responses
    MutationResult {
        affected_rows: u64,
    },
    BatchResult {
        affected_rows: u64,
        statement_count: usize,
    },
    ImportProgress {
        inserted: usize,
        total: usize,
    },
    ImportComplete {
        total: usize,
    },
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub sql: String,
    pub columns: Vec<String>,
    pub column_types: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub execution_time_ms: u64,
    pub source_table: Option<String>,
    pub primary_keys: Vec<String>,
}

/// Extract table name from simple SELECT queries.
/// Returns None for JOINs, subqueries, UNIONs, CTEs, or multi-table queries.
pub fn extract_source_table(sql: &str) -> Option<String> {
    let normalized = sql
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    // Reject queries with JOINs, UNIONs, subqueries, CTEs
    if normalized.contains(" join ")
        || normalized.contains(" union ")
        || normalized.contains(" intersect ")
        || normalized.contains(" except ")
        || normalized.contains("with ")
        || normalized.matches("select").count() > 1
    {
        return None;
    }

    // Match: SELECT ... FROM table_name [WHERE ...] [ORDER ...] [LIMIT ...]
    let from_pos = normalized.find(" from ")?;
    let after_from = &normalized[from_pos + 6..].trim_start();

    // Take the first word after FROM (the table name)
    let table_end = after_from
        .find(|c: char| c.is_whitespace() || c == ';' || c == ')')
        .unwrap_or(after_from.len());
    let table = &after_from[..table_end];

    if table.is_empty() {
        return None;
    }

    // Return the original-case version by finding it in the original SQL
    let orig_lower = sql.to_lowercase();
    let from_pos_orig = orig_lower.find(" from ")?;
    let after_from_orig = sql[from_pos_orig + 6..].trim_start();
    let table_orig = &after_from_orig[..after_from_orig
        .find(|c: char| c.is_whitespace() || c == ';' || c == ')')
        .unwrap_or(after_from_orig.len())];

    Some(table_orig.to_string())
}
