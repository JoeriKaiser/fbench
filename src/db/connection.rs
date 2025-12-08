use sqlx::{postgres::{PgPool, PgPoolOptions}, Column, Row, ValueRef};
use tokio::sync::mpsc;

use super::{ConnectionConfig, DbRequest, DbResponse, QueryResult, SchemaInfo, ColumnInfo};

pub struct DbWorker {
    pool: Option<PgPool>,
    request_rx: mpsc::UnboundedReceiver<DbRequest>,
    response_tx: mpsc::UnboundedSender<DbResponse>,
}

impl DbWorker {
    pub fn new(
        request_rx: mpsc::UnboundedReceiver<DbRequest>,
        response_tx: mpsc::UnboundedSender<DbResponse>,
    ) -> Self {
        Self {
            pool: None,
            request_rx,
            response_tx,
        }
    }

    pub async fn run(mut self) {
        while let Some(request) = self.request_rx.recv().await {
            let response = match request {
                DbRequest::Connect(config) => self.connect(config).await,
                DbRequest::TestConnection(config) => self.test_connection(config).await,
                DbRequest::Execute(sql) => self.execute(&sql).await,
                DbRequest::ListTables => self.list_tables().await,
                DbRequest::FetchSchema => self.fetch_schema().await,
                DbRequest::Disconnect => self.disconnect().await,
            };
            let _ = self.response_tx.send(response);
        }
    }

    async fn test_connection(&self, config: ConnectionConfig) -> DbResponse {
        let result = PgPool::connect(&config.connection_string()).await;
        match result {
            Ok(pool) => {
                pool.close().await;
                DbResponse::TestResult(Ok(()))
            }
            Err(e) => DbResponse::TestResult(Err(e.to_string())),
        }
    }

    async fn connect(&mut self, config: ConnectionConfig) -> DbResponse {
        let schema = config.schema.clone();
        
        let pool_result = if !schema.is_empty() {
            let search_path = format!("SET search_path TO \"{}\", public", schema);
            PgPoolOptions::new()
                .after_connect(move |conn, _meta| {
                    let sql = search_path.clone();
                    Box::pin(async move {
                        sqlx::query(&sql).execute(&mut *conn).await?;
                        Ok(())
                    })
                })
                .connect(&config.connection_string())
                .await
        } else {
            PgPool::connect(&config.connection_string()).await
        };

        match pool_result {
            Ok(pool) => {
                self.pool = Some(pool);
                DbResponse::Connected
            }
            Err(e) => DbResponse::Error(e.to_string()),
        }
    }

    async fn fetch_schema(&self) -> DbResponse {
        let Some(pool) = &self.pool else {
            return DbResponse::Error("Not connected".into());
        };

        let tables_sql = r#"
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_type = 'BASE TABLE' 
              AND table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_name
        "#;

        let columns_sql = r#"
            SELECT table_name, column_name, data_type
            FROM information_schema.columns
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_name, ordinal_position
        "#;

        let tables: Vec<String> = match sqlx::query_scalar(tables_sql).fetch_all(pool).await {
            Ok(t) => t,
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        let columns: Vec<ColumnInfo> = match sqlx::query_as::<_, (String, String, String)>(columns_sql)
            .fetch_all(pool)
            .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(|(table_name, column_name, data_type)| ColumnInfo {
                    table_name,
                    column_name,
                    data_type,
                })
                .collect(),
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        DbResponse::Schema(SchemaInfo { tables, columns })
    }

    async fn list_tables(&self) -> DbResponse {
        let sql = r#"
            SELECT table_schema, table_name 
            FROM information_schema.tables 
            WHERE table_type = 'BASE TABLE' 
              AND table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_schema, table_name
        "#;
        self.execute(sql).await
    }

    async fn execute(&self, sql: &str) -> DbResponse {
        let Some(pool) = &self.pool else {
            return DbResponse::Error("Not connected".into());
        };

        let start = std::time::Instant::now();
        match sqlx::query(sql).fetch_all(pool).await {
            Ok(rows) => {
                let columns: Vec<String> = if rows.is_empty() {
                    vec![]
                } else {
                    rows[0].columns().iter().map(|c| c.name().to_string()).collect()
                };

                let data: Vec<Vec<String>> = rows
                    .iter()
                    .map(|row| {
                        (0..row.len())
                            .map(|i| format_value(row, i))
                            .collect()
                    })
                    .collect();

                DbResponse::QueryResult(QueryResult {
                    columns,
                    rows: data,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                })
            }
            Err(e) => DbResponse::Error(e.to_string()),
        }
    }

    async fn disconnect(&mut self) -> DbResponse {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
        }
        DbResponse::Disconnected
    }
}

fn format_value(row: &sqlx::postgres::PgRow, i: usize) -> String {
    row.try_get_raw(i)
        .ok()
        .and_then(|v| {
            if v.is_null() {
                Some("NULL".to_string())
            } else {
                row.try_get::<String, _>(i).ok()
                    .or_else(|| row.try_get::<i16, _>(i).ok().map(|n| n.to_string()))
                    .or_else(|| row.try_get::<i32, _>(i).ok().map(|n| n.to_string()))
                    .or_else(|| row.try_get::<i64, _>(i).ok().map(|n| n.to_string()))
                    .or_else(|| row.try_get::<f32, _>(i).ok().map(|n| n.to_string()))
                    .or_else(|| row.try_get::<f64, _>(i).ok().map(|n| n.to_string()))
                    .or_else(|| row.try_get::<bool, _>(i).ok().map(|b| b.to_string()))
                    .or_else(|| row.try_get::<chrono::NaiveDateTime, _>(i).ok().map(|d| d.to_string()))
                    .or_else(|| row.try_get::<chrono::DateTime<chrono::Utc>, _>(i).ok().map(|d| d.to_string()))
                    .or_else(|| row.try_get::<chrono::NaiveDate, _>(i).ok().map(|d| d.to_string()))
                    .or_else(|| row.try_get::<serde_json::Value, _>(i).ok().map(|j| j.to_string()))
                    .or_else(|| row.try_get::<uuid::Uuid, _>(i).ok().map(|u| u.to_string()))
            }
        })
        .unwrap_or_else(|| "?".to_string())
}

pub fn spawn_db_worker() -> (mpsc::UnboundedSender<DbRequest>, mpsc::UnboundedReceiver<DbResponse>) {
    let (request_tx, request_rx) = mpsc::unbounded_channel();
    let (response_tx, response_rx) = mpsc::unbounded_channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(DbWorker::new(request_rx, response_tx).run());
    });

    (request_tx, response_rx)
}
