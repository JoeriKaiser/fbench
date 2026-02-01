use dioxus::prelude::*;
use tokio::sync::mpsc;

pub type DbSender = mpsc::UnboundedSender<crate::db::DbRequest>;
pub type LlmSender = mpsc::UnboundedSender<crate::llm::LlmRequest>;

pub fn init_services() -> (DbSender, LlmSender) {
    let (db_tx, db_rx) = crate::db::spawn_db_worker();
    let (llm_tx, llm_rx) = crate::llm::spawn_llm_worker();

    let db_tx_clone = db_tx.clone();
    spawn(async move {
        handle_db_responses(db_rx, db_tx_clone).await;
    });
    spawn(async move {
        handle_llm_responses(llm_rx).await;
    });

    (db_tx, llm_tx)
}

async fn handle_db_responses(
    mut rx: mpsc::UnboundedReceiver<crate::db::DbResponse>,
    db_tx: DbSender,
) {
    use crate::config::QueryHistory;
    use crate::db::DbResponse;
    use crate::state::*;

    let mut query_history = QueryHistory::new();

    while let Some(response) = rx.recv().await {
        match response {
            DbResponse::Connected(db_type) => {
                let db_type_enum = match db_type {
                    crate::db::DatabaseType::PostgreSQL => DatabaseType::PostgreSQL,
                    crate::db::DatabaseType::MySQL => DatabaseType::MySQL,
                };
                *CONNECTION.write() = ConnectionState::Connected {
                    db_type: db_type_enum,
                    db_name: String::new(),
                };
                let _ = db_tx.send(crate::db::DbRequest::FetchSchema);
            }
            DbResponse::Schema(schema) => *SCHEMA.write() = schema,
            DbResponse::QueryResult(result) => {
                // Record in history
                query_history.add_entry(
                    result.sql.clone(),
                    Some(result.rows.len()),
                    Some(result.execution_time_ms),
                );
                // Notify UI that history changed
                *HISTORY_REVISION.write() += 1;
                *QUERY_RESULT.write() = Some(result.clone());
                *EXECUTION_TIME_MS.write() = Some(result.execution_time_ms);
                *ROW_COUNT.write() = Some(result.rows.len());
                *LAST_ERROR.write() = None;
            }
            DbResponse::Error(e) => {
                *LAST_ERROR.write() = Some(e);
                *QUERY_RESULT.write() = None;
            }
            DbResponse::Disconnected => {
                *CONNECTION.write() = ConnectionState::Disconnected;
                *SCHEMA.write() = Default::default();
                *CURRENT_DB_TYPE.write() = None;
            }
            DbResponse::ConnectionLost => {
                *CONNECTION.write() = ConnectionState::ConnectionLost;
            }
            DbResponse::TestResult(result) => {
                *TEST_CONNECTION_STATUS.write() = match result {
                    Ok(()) => TestConnectionStatus::Success,
                    Err(e) => TestConnectionStatus::Failed(e),
                };
            }
            _ => {}
        }
    }
}

async fn handle_llm_responses(mut rx: mpsc::UnboundedReceiver<crate::llm::LlmResponse>) {
    use crate::llm::LlmResponse;
    use crate::state::*;

    while let Some(response) = rx.recv().await {
        match response {
            LlmResponse::Generated(sql) => {
                // Replace editor content with generated SQL
                *EDITOR_CONTENT.write() = sql;
                *LLM_GENERATING.write() = false;
                *LLM_PROMPT.write() = String::new();
                *LLM_STATUS.write() = LlmStatus::Success("Query generated successfully".into());
            }
            LlmResponse::Error(e) => {
                *LLM_GENERATING.write() = false;
                *LLM_STATUS.write() = LlmStatus::Error(e);
            }
            _ => {
                // Other response types not handled yet
                *LLM_GENERATING.write() = false;
            }
        }
    }
}
