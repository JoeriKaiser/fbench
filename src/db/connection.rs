use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow},
    mysql::{MySqlPool, MySqlRow},
    Column, Row, ValueRef,
};
use tokio::sync::mpsc;

use super::{
    ColumnInfo, ConnectionConfig, ConstraintInfo, DatabaseType, DbRequest, DbResponse,
    IndexInfo, QueryResult, SchemaInfo, TableInfo,
};

const MAX_VALUE_LEN: usize = 10_000;

enum DbPool {
    Postgres(PgPool),
    MySQL(MySqlPool),
}

pub struct DbWorker {
    pool: Option<DbPool>,
    db_type: Option<DatabaseType>,
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
            db_type: None,
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
        let result = match config.db_type {
            DatabaseType::PostgreSQL => {
                PgPool::connect(&config.connection_string()).await.map(|p| { let _ = p; })
            }
            DatabaseType::MySQL => {
                MySqlPool::connect(&config.connection_string()).await.map(|p| { let _ = p; })
            }
        };
        
        match result {
            Ok(_) => DbResponse::TestResult(Ok(())),
            Err(e) => DbResponse::TestResult(Err(e.to_string())),
        }
    }

    async fn connect(&mut self, config: ConnectionConfig) -> DbResponse {
        let db_type = config.db_type;
        let schema = config.schema.clone();

        let result = match db_type {
            DatabaseType::PostgreSQL => {
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
                pool_result.map(DbPool::Postgres)
            }
            DatabaseType::MySQL => {
                MySqlPool::connect(&config.connection_string()).await.map(DbPool::MySQL)
            }
        };

        match result {
            Ok(pool) => {
                self.pool = Some(pool);
                self.db_type = Some(db_type);
                DbResponse::Connected(db_type)
            }
            Err(e) => DbResponse::Error(e.to_string()),
        }
    }

    async fn fetch_schema(&self) -> DbResponse {
        match (&self.pool, self.db_type) {
            (Some(DbPool::Postgres(pool)), Some(DatabaseType::PostgreSQL)) => {
                self.fetch_schema_postgres(pool).await
            }
            (Some(DbPool::MySQL(pool)), Some(DatabaseType::MySQL)) => {
                self.fetch_schema_mysql(pool).await
            }
            _ => DbResponse::Error("Not connected".into()),
        }
    }

    async fn fetch_schema_postgres(&self, pool: &PgPool) -> DbResponse {
        let tables_sql = r#"
            SELECT 
                t.table_name::TEXT,
                COALESCE(s.n_live_tup, 0)::BIGINT as row_estimate
            FROM information_schema.tables t
            LEFT JOIN pg_stat_user_tables s 
                ON t.table_name = s.relname AND t.table_schema = s.schemaname
            WHERE t.table_type = 'BASE TABLE' 
              AND t.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY t.table_schema, t.table_name
        "#;

        let views_sql = r#"
            SELECT table_name::TEXT 
            FROM information_schema.views 
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_name
        "#;

        let columns_sql = r#"
            SELECT 
                c.table_name::TEXT,
                c.column_name::TEXT,
                c.data_type::TEXT,
                (c.is_nullable = 'YES') as nullable,
                c.column_default::TEXT,
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

    async fn fetch_schema_mysql(&self, pool: &MySqlPool) -> DbResponse {
        let db_name_sql = "SELECT DATABASE()";
        let db_name: Option<String> = match sqlx::query_scalar(db_name_sql).fetch_one(pool).await {
            Ok(name) => name,
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        let db_name = match db_name {
            Some(n) => n,
            None => return DbResponse::Error("No database selected".into()),
        };

        let tables_sql = r#"
            SELECT 
                t.TABLE_NAME as table_name,
                COALESCE(t.TABLE_ROWS, 0) as row_estimate
            FROM information_schema.TABLES t
            WHERE t.TABLE_SCHEMA = ?
              AND t.TABLE_TYPE = 'BASE TABLE'
            ORDER BY t.TABLE_NAME
        "#;

        let views_sql = r#"
            SELECT TABLE_NAME 
            FROM information_schema.VIEWS 
            WHERE TABLE_SCHEMA = ?
            ORDER BY TABLE_NAME
        "#;

        let columns_sql = r#"
            SELECT 
                c.TABLE_NAME as table_name,
                c.COLUMN_NAME as column_name,
                c.DATA_TYPE as data_type,
                (c.IS_NULLABLE = 'YES') as nullable,
                c.COLUMN_DEFAULT as default_value,
                (c.COLUMN_KEY = 'PRI') as is_primary_key
            FROM information_schema.COLUMNS c
            WHERE c.TABLE_SCHEMA = ?
            ORDER BY c.TABLE_NAME, c.ORDINAL_POSITION
        "#;

        let tables: Vec<(String, i64)> = match sqlx::query_as(tables_sql)
            .bind(&db_name)
            .fetch_all(pool)
            .await
        {
            Ok(t) => t,
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        let views: Vec<String> = match sqlx::query_scalar(views_sql)
            .bind(&db_name)
            .fetch_all(pool)
            .await
        {
            Ok(v) => v,
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        let columns: Vec<(String, String, String, bool, Option<String>, bool)> =
            match sqlx::query_as(columns_sql).bind(&db_name).fetch_all(pool).await {
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
        match (&self.pool, self.db_type) {
            (Some(DbPool::Postgres(pool)), Some(DatabaseType::PostgreSQL)) => {
                self.fetch_table_details_postgres(pool, table_name).await
            }
            (Some(DbPool::MySQL(pool)), Some(DatabaseType::MySQL)) => {
                self.fetch_table_details_mysql(pool, table_name).await
            }
            _ => DbResponse::Error("Not connected".into()),
        }
    }

    async fn fetch_table_details_postgres(&self, pool: &PgPool, table_name: &str) -> DbResponse {
        let columns_sql = r#"
            SELECT 
                c.column_name::TEXT,
                c.data_type::TEXT,
                (c.is_nullable = 'YES') as nullable,
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
                i.relname::TEXT as index_name,
                COALESCE(array_agg(a.attname::TEXT ORDER BY x.n), ARRAY[]::TEXT[]) as columns,
                ix.indisunique as is_unique,
                ix.indisprimary as is_primary,
                COALESCE(am.amname::TEXT, 'unknown') as index_type
            FROM pg_index ix
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            LEFT JOIN pg_am am ON am.oid = i.relam
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
                COALESCE(
                    array_agg(DISTINCT kcu.column_name::TEXT) FILTER (WHERE kcu.column_name IS NOT NULL),
                    ARRAY[]::TEXT[]
                ) as columns,
                ccu.table_name::TEXT as foreign_table,
                COALESCE(
                    array_agg(DISTINCT ccu.column_name::TEXT) FILTER (WHERE ccu.column_name IS NOT NULL AND tc.constraint_type = 'FOREIGN KEY'),
                    ARRAY[]::TEXT[]
                ) as foreign_columns,
                cc.check_clause::TEXT
            FROM information_schema.table_constraints tc
            LEFT JOIN information_schema.key_column_usage kcu 
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            LEFT JOIN information_schema.constraint_column_usage ccu 
                ON tc.constraint_name = ccu.constraint_name 
                AND tc.constraint_type = 'FOREIGN KEY'
            LEFT JOIN information_schema.check_constraints cc 
                ON tc.constraint_name = cc.constraint_name
            WHERE tc.table_name = $1
              AND tc.table_schema NOT IN ('pg_catalog', 'information_schema')
            GROUP BY tc.constraint_name, tc.constraint_type, ccu.table_name, cc.check_clause
            ORDER BY tc.constraint_type, tc.constraint_name
        "#;

        let columns: Vec<ColumnInfo> =
            match sqlx::query_as::<_, (String, String, bool, Option<String>, bool)>(columns_sql)
                .bind(table_name)
                .fetch_all(pool)
                .await
            {
                Ok(rows) => rows
                    .into_iter()
                    .map(|(name, data_type, nullable, default_value, is_primary_key)| ColumnInfo {
                        name,
                        data_type,
                        nullable,
                        default_value,
                        is_primary_key,
                    })
                    .collect(),
                Err(e) => return DbResponse::Error(e.to_string()),
            };

        let indexes: Vec<IndexInfo> =
            match sqlx::query_as::<_, (String, Vec<String>, bool, bool, String)>(indexes_sql)
                .bind(table_name)
                .fetch_all(pool)
                .await
            {
                Ok(rows) => rows
                    .into_iter()
                    .map(|(name, columns, is_unique, is_primary, index_type)| IndexInfo {
                        name,
                        columns,
                        is_unique,
                        is_primary,
                        index_type,
                    })
                    .collect(),
                Err(e) => return DbResponse::Error(e.to_string()),
            };

        let constraints: Vec<ConstraintInfo> = match sqlx::query_as::<
            _,
            (String, String, Vec<String>, Option<String>, Vec<String>, Option<String>),
        >(constraints_sql)
        .bind(table_name)
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(
                    |(name, constraint_type, columns, foreign_table, foreign_columns, check_clause)| {
                        let foreign_columns = if foreign_columns.is_empty() {
                            None
                        } else {
                            Some(foreign_columns)
                        };
                        
                        ConstraintInfo {
                            name,
                            constraint_type,
                            columns,
                            foreign_table,
                            foreign_columns,
                            check_clause,
                        }
                    },
                )
                .collect(),
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

    async fn fetch_table_details_mysql(&self, pool: &MySqlPool, table_name: &str) -> DbResponse {
        let db_name: Option<String> = match sqlx::query_scalar("SELECT DATABASE()")
            .fetch_one(pool)
            .await
        {
            Ok(name) => name,
            Err(e) => return DbResponse::Error(e.to_string()),
        };

        let db_name = match db_name {
            Some(n) => n,
            None => return DbResponse::Error("No database selected".into()),
        };

        let columns_sql = r#"
            SELECT 
                COLUMN_NAME as name,
                DATA_TYPE as data_type,
                (IS_NULLABLE = 'YES') as nullable,
                COLUMN_DEFAULT as default_value,
                (COLUMN_KEY = 'PRI') as is_primary_key
            FROM information_schema.COLUMNS
            WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
            ORDER BY ORDINAL_POSITION
        "#;

        let indexes_sql = r#"
            SELECT 
                INDEX_NAME as name,
                GROUP_CONCAT(COLUMN_NAME ORDER BY SEQ_IN_INDEX) as columns,
                (NON_UNIQUE = 0) as is_unique,
                (INDEX_NAME = 'PRIMARY') as is_primary,
                INDEX_TYPE as index_type
            FROM information_schema.STATISTICS
            WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
            GROUP BY INDEX_NAME, NON_UNIQUE, INDEX_TYPE
            ORDER BY INDEX_NAME
        "#;

        let constraints_sql = r#"
            SELECT 
                tc.CONSTRAINT_NAME as name,
                tc.CONSTRAINT_TYPE as constraint_type,
                GROUP_CONCAT(DISTINCT kcu.COLUMN_NAME) as columns,
                kcu.REFERENCED_TABLE_NAME as foreign_table,
                GROUP_CONCAT(DISTINCT kcu.REFERENCED_COLUMN_NAME) as foreign_columns,
                NULL as check_clause
            FROM information_schema.TABLE_CONSTRAINTS tc
            LEFT JOIN information_schema.KEY_COLUMN_USAGE kcu 
                ON tc.CONSTRAINT_NAME = kcu.CONSTRAINT_NAME 
                AND tc.TABLE_SCHEMA = kcu.TABLE_SCHEMA
                AND tc.TABLE_NAME = kcu.TABLE_NAME
            WHERE tc.TABLE_SCHEMA = ? AND tc.TABLE_NAME = ?
            GROUP BY tc.CONSTRAINT_NAME, tc.CONSTRAINT_TYPE, kcu.REFERENCED_TABLE_NAME
            ORDER BY tc.CONSTRAINT_TYPE, tc.CONSTRAINT_NAME
        "#;

        let columns: Vec<ColumnInfo> =
            match sqlx::query_as::<_, (String, String, bool, Option<String>, bool)>(columns_sql)
                .bind(&db_name)
                .bind(table_name)
                .fetch_all(pool)
                .await
            {
                Ok(rows) => rows
                    .into_iter()
                    .map(|(name, data_type, nullable, default_value, is_primary_key)| ColumnInfo {
                        name,
                        data_type,
                        nullable,
                        default_value,
                        is_primary_key,
                    })
                    .collect(),
                Err(e) => return DbResponse::Error(e.to_string()),
            };

        let indexes: Vec<IndexInfo> =
            match sqlx::query_as::<_, (String, String, bool, bool, String)>(indexes_sql)
                .bind(&db_name)
                .bind(table_name)
                .fetch_all(pool)
                .await
            {
                Ok(rows) => rows
                    .into_iter()
                    .map(|(name, columns_str, is_unique, is_primary, index_type)| IndexInfo {
                        name,
                        columns: columns_str.split(',').map(|s| s.to_string()).collect(),
                        is_unique,
                        is_primary,
                        index_type,
                    })
                    .collect(),
                Err(e) => return DbResponse::Error(e.to_string()),
            };

        let constraints: Vec<ConstraintInfo> = match sqlx::query_as::<
            _,
            (String, String, Option<String>, Option<String>, Option<String>, Option<String>),
        >(constraints_sql)
        .bind(&db_name)
        .bind(table_name)
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(
                    |(name, constraint_type, columns_str, foreign_table, foreign_columns_str, check_clause)| {
                        let columns = columns_str
                            .map(|s| s.split(',').map(|c| c.to_string()).collect())
                            .unwrap_or_default();
                        let foreign_columns = foreign_columns_str
                            .map(|s| s.split(',').map(|c| c.to_string()).collect());
                        ConstraintInfo {
                            name,
                            constraint_type,
                            columns,
                            foreign_table,
                            foreign_columns,
                            check_clause,
                        }
                    },
                )
                .collect(),
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
        match (&self.pool, self.db_type) {
            (Some(DbPool::Postgres(_)), Some(DatabaseType::PostgreSQL)) => {
                let sql = r#"
                    SELECT table_schema, table_name 
                    FROM information_schema.tables 
                    WHERE table_type = 'BASE TABLE' 
                      AND table_schema NOT IN ('pg_catalog', 'information_schema')
                    ORDER BY table_schema, table_name
                "#;
                self.execute(sql).await
            }
            (Some(DbPool::MySQL(_)), Some(DatabaseType::MySQL)) => {
                let sql = "SHOW TABLES";
                self.execute(sql).await
            }
            _ => DbResponse::Error("Not connected".into()),
        }
    }

    async fn execute(&self, sql: &str) -> DbResponse {
        match &self.pool {
            Some(DbPool::Postgres(pool)) => self.execute_postgres(pool, sql).await,
            Some(DbPool::MySQL(pool)) => self.execute_mysql(pool, sql).await,
            None => DbResponse::Error("Not connected".into()),
        }
    }

    async fn execute_postgres(&self, pool: &PgPool, sql: &str) -> DbResponse {
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
                        row_data.push(format_pg_value(row, i));
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

    async fn execute_mysql(&self, pool: &MySqlPool, sql: &str) -> DbResponse {
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
                        row_data.push(format_mysql_value(row, i));
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
            match pool {
                DbPool::Postgres(p) => p.close().await,
                DbPool::MySQL(p) => p.close().await,
            }
        }
        self.db_type = None;
        DbResponse::Disconnected
    }
}

fn format_pg_value(row: &PgRow, i: usize) -> String {
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
        .or_else(|| row.try_get::<serde_json::Value, _>(i).ok().map(|j| j.to_string()))
        .or_else(|| row.try_get::<Vec<f32>, _>(i).ok().map(|v| format_vector(&v)))
        .or_else(|| row.try_get::<Vec<f64>, _>(i).ok().map(|v| format_vector(&v)))
        .unwrap_or_else(|| "?".to_string());

    truncate_value(value)
}

fn format_mysql_value(row: &MySqlRow, i: usize) -> String {
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
        .or_else(|| row.try_get::<u32, _>(i).ok().map(|n| n.to_string()))
        .or_else(|| row.try_get::<u64, _>(i).ok().map(|n| n.to_string()))
        .or_else(|| row.try_get::<f64, _>(i).ok().map(|n| format_float(n)))
        .or_else(|| row.try_get::<f32, _>(i).ok().map(|n| format_float(n as f64)))
        .or_else(|| row.try_get::<bool, _>(i).ok().map(|b| b.to_string()))
        .or_else(|| row.try_get::<chrono::NaiveDateTime, _>(i).ok().map(|d| d.to_string()))
        .or_else(|| row.try_get::<chrono::NaiveDate, _>(i).ok().map(|d| d.to_string()))
        .or_else(|| row.try_get::<chrono::NaiveTime, _>(i).ok().map(|t| t.to_string()))
        .or_else(|| row.try_get::<serde_json::Value, _>(i).ok().map(|j| j.to_string()))
        .unwrap_or_else(|| "?".to_string());

    truncate_value(value)
}

fn truncate_value(value: String) -> String {
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
