mod connection;
mod query;

pub use connection::*;

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub schema: String,
}

impl ConnectionConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub table_name: String,
    pub column_name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct SchemaInfo {
    pub tables: Vec<String>,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug)]
pub enum DbRequest {
    Connect(ConnectionConfig),
    TestConnection(ConnectionConfig),
    Execute(String),
    ListTables,
    FetchSchema,
    Disconnect,
}

#[derive(Debug)]
pub enum DbResponse {
    Connected,
    TestResult(Result<(), String>),
    QueryResult(QueryResult),
    Schema(SchemaInfo),
    Error(String),
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub execution_time_ms: u64,
}
