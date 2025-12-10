use sqlx::{postgres::{PgPool, PgPoolOptions}, Column, Row, ValueRef};
use tokio::sync::mpsc;

use super::{ConnectionConfig, DbRequest, DbResponse, QueryResult, SchemaInfo, TableInfo, ColumnInfo, IndexInfo, ConstraintInfo};
const MAX_VALUE_LEN: usize = 10_000;

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
                DbRequest::FetchTableDetails(table) => self.fetch_table_details(&table).await,
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
            SELECT 
                t.table_name,
                COALESCE(s.n_live_tup, 0) as row_estimate
            FROM information_schema.tables t
            LEFT JOIN pg_stat_user_tables s 
                ON t.table_name = s.relname AND t.table_schema = s.schemaname
            WHERE t.table_type = 'BASE TABLE' 
              AND t.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY t.table_schema, t.table_name
        "#;

        let views_sql = r#"
            SELECT table_name 
            FROM information_schema.views 
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_name
        "#;

        let columns_sql = r#"
            SELECT 
                c.table_name,
                c.column_name,
                c.data_type,
                c.is_nullable = 'YES' as nullable,
                c.column_default,
                COALESCE(pk.is_pk, false) as is_primary_key
            FROM information_schema.columns c
            LEFT JOIN (
                SELECT kcu.table_name, kcu.column_name, true as is_pk
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu 
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                WHERE tc.constraint_type = 'PRIMARY KEY'
            ) pk ON c.table_name = pk.table_name AND c.column_name = pk.column_name
            WHERE c.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY c.table_name, c.ordinal_position
        "#;

        let tables: Vec<(String, i64)> = match sqlx::query_as(tables_sql).fetch_all(pool).await {
            Ok(t) => t,
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        let views: Vec<String> = match sqlx::query_scalar(views_sql).fetch_all(pool).await {
            Ok(v) => v,
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        let columns: Vec<(String, String, String, bool, Option<String>, bool)> = 
            match sqlx::query_as(columns_sql).fetch_all(pool).await {
                Ok(c) => c,
                Err(e) => return DbResponse::Error(e.to_string()),
            };

        let mut table_infos: Vec<TableInfo> = tables
            .into_iter()
            .map(|(name, row_estimate)| TableInfo {
                name,
                row_estimate,
                columns: Vec::new(),
                indexes: Vec::new(),
                constraints: Vec::new(),
            })
            .collect();

        for (table_name, col_name, data_type, nullable, default_value, is_pk) in columns {
            if let Some(table) = table_infos.iter_mut().find(|t| t.name == table_name) {
                table.columns.push(ColumnInfo {
                    name: col_name,
                    data_type,
                    nullable,
                    default_value,
                    is_primary_key: is_pk,
                });
            }
        }

        DbResponse::Schema(SchemaInfo {
            tables: table_infos,
            views,
        })
    }

    async fn fetch_table_details(&self, table_name: &str) -> DbResponse {
        let Some(pool) = &self.pool else {
            return DbResponse::Error("Not connected".into());
        };

        let columns_sql = r#"
            SELECT 
                c.column_name::TEXT,
                c.data_type::TEXT,
                c.is_nullable = 'YES' as nullable,
                c.column_default::TEXT,
                COALESCE(pk.is_pk, false) as is_primary_key
            FROM information_schema.columns c
            LEFT JOIN (
                SELECT kcu.column_name, true as is_pk
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu 
                    ON tc.constraint_name = kcu.constraint_name
                WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_name = $1
            ) pk ON c.column_name = pk.column_name
            WHERE c.table_name = $1 
              AND c.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY c.ordinal_position
        "#;

        let indexes_sql = r#"
            SELECT 
                i.relname as index_name,
                array_agg(a.attname::TEXT ORDER BY x.n)::TEXT[] as columns,
                ix.indisunique as is_unique,
                ix.indisprimary as is_primary,
                am.amname as index_type
            FROM pg_index ix
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_am am ON am.oid = i.relam
            CROSS JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS x(attnum, n)
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = x.attnum
            WHERE t.relname = $1
            GROUP BY i.relname, ix.indisunique, ix.indisprimary, am.amname
            ORDER BY i.relname
        "#;

        let constraints_sql = r#"
            SELECT 
                tc.constraint_name::TEXT,
                tc.constraint_type::TEXT,
                array_agg(DISTINCT kcu.column_name::TEXT)::TEXT[] as columns,
                ccu.table_name::TEXT as foreign_table,
                array_agg(DISTINCT ccu.column_name::TEXT)::TEXT[] FILTER (WHERE ccu.column_name IS NOT NULL) as foreign_columns,
                cc.check_clause::TEXT
            FROM information_schema.table_constraints tc
            LEFT JOIN information_schema.key_column_usage kcu 
                ON tc.constraint_name = kcu.constraint_name
            LEFT JOIN information_schema.constraint_column_usage ccu 
                ON tc.constraint_name = ccu.constraint_name 
                AND tc.constraint_type = 'FOREIGN KEY'
            LEFT JOIN information_schema.check_constraints cc 
                ON tc.constraint_name = cc.constraint_name
            WHERE tc.table_name = $1
            GROUP BY tc.constraint_name, tc.constraint_type, ccu.table_name, cc.check_clause
            ORDER BY tc.constraint_type, tc.constraint_name
        "#;

        let columns: Vec<ColumnInfo> = match sqlx::query_as::<_, (String, String, bool, Option<String>, bool)>(columns_sql)
            .bind(table_name)
            .fetch_all(pool)
            .await
        {
            Ok(rows) => rows.into_iter().map(|(name, data_type, nullable, default_value, is_primary_key)| {
                ColumnInfo { name, data_type, nullable, default_value, is_primary_key }
            }).collect(),
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        let indexes: Vec<IndexInfo> = match sqlx::query_as::<_, (String, Vec<String>, bool, bool, String)>(indexes_sql)
            .bind(table_name)
            .fetch_all(pool)
            .await
        {
            Ok(rows) => rows.into_iter().map(|(name, columns, is_unique, is_primary, index_type)| {
                IndexInfo { name, columns, is_unique, is_primary, index_type }
            }).collect(),
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        let constraints: Vec<ConstraintInfo> = match sqlx::query_as::<_, (String, String, Vec<String>, Option<String>, Option<Vec<String>>, Option<String>)>(constraints_sql)
            .bind(table_name)
            .fetch_all(pool)
            .await
        {
            Ok(rows) => rows.into_iter().map(|(name, constraint_type, columns, foreign_table, foreign_columns, check_clause)| {
                ConstraintInfo { name, constraint_type, columns, foreign_table, foreign_columns, check_clause }
            }).collect(),
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        DbResponse::TableDetails(TableInfo {
            name: table_name.to_string(),
            columns,
            indexes,
            constraints,
            row_estimate: 0,
        })
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

                let mut data: Vec<Vec<String>> = Vec::with_capacity(rows.len());
                
                for row in &rows {
                    let mut row_data: Vec<String> = Vec::with_capacity(row.len());
                    for i in 0..row.len() {
                        row_data.push(format_value(row, i));
                    }
                    data.push(row_data);
                }

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
    let raw = match row.try_get_raw(i) {
        Ok(v) => v,
        Err(_) => return "?".to_string(),
    };

    if raw.is_null() {
        return "NULL".to_string();
    }

    let value = row.try_get::<String, _>(i).ok()
        .or_else(|| row.try_get::<i32, _>(i).ok().map(|n| n.to_string()))
        .or_else(|| row.try_get::<i64, _>(i).ok().map(|n| n.to_string()))
        .or_else(|| row.try_get::<i16, _>(i).ok().map(|n| n.to_string()))
        .or_else(|| row.try_get::<f64, _>(i).ok().map(|n| format_float(n)))
        .or_else(|| row.try_get::<f32, _>(i).ok().map(|n| format_float(n as f64)))
        .or_else(|| row.try_get::<bool, _>(i).ok().map(|b| b.to_string()))
        .or_else(|| row.try_get::<chrono::NaiveDateTime, _>(i).ok().map(|d| d.to_string()))
        .or_else(|| row.try_get::<chrono::DateTime<chrono::Utc>, _>(i).ok().map(|d| d.to_string()))
        .or_else(|| row.try_get::<chrono::NaiveDate, _>(i).ok().map(|d| d.to_string()))
        .or_else(|| row.try_get::<uuid::Uuid, _>(i).ok().map(|u| u.to_string()))
        .or_else(|| row.try_get::<serde_json::Value, _>(i).ok().map(|j| format_json(&j)))
        .or_else(|| row.try_get::<Vec<f32>, _>(i).ok().map(|v| format_vector(&v)))
        .or_else(|| row.try_get::<Vec<f64>, _>(i).ok().map(|v| format_vector(&v)))
        .unwrap_or_else(|| "?".to_string());

    if value.len() > MAX_VALUE_LEN {
        let mut truncated = value[..MAX_VALUE_LEN].to_string();
        truncated.push_str("...[truncated]");
        truncated
    } else {
        value
    }
}

#[inline]
fn format_float(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{:.0}", n)
    } else {
        format!("{:.6}", n).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn format_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "?".to_string())
}

fn format_vector<T: std::fmt::Display>(v: &[T]) -> String {
    if v.len() <= 5 {
        format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
    } else {
        format!(
            "[{}, {}, {}, ... ({} more) ..., {}, {}]",
            v[0], v[1], v[2],
            v.len() - 5,
            v[v.len() - 2], v[v.len() - 1]
        )
    }
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
