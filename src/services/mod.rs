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
                // Close dialog and reset test status on successful connection
                *SHOW_CONNECTION_DIALOG.write() = false;
                *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Idle;
                let _ = db_tx.send(crate::db::DbRequest::FetchSchema);
            }
            DbResponse::ConnectionFailed(e) => {
                *CONNECTION.write() = ConnectionState::Error(e.clone());
                // Show error in test status area so user sees it
                *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Failed(e);
            }
            DbResponse::Schema(schema) => *SCHEMA.write() = schema,
            DbResponse::QueryResult(result) => {
                // Record in history
                query_history.add_entry(
                    result.sql.clone(),
                    Some(result.rows.len()),
                    Some(result.execution_time_ms),
                );
                // Clear draft after successful execution
                let store = crate::config::DraftStore::new();
                let _ = store.clear();
                // Notify UI that history changed
                *HISTORY_REVISION.write() += 1;
                // Update active tab with result
                if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
                    tab.result = Some(result.clone());
                    tab.last_error = None;
                    tab.execution_time_ms = Some(result.execution_time_ms);
                    tab.unsaved_changes = false;
                }
                // Also update global for backward compatibility during migration
                *QUERY_RESULT.write() = Some(result.clone());
                *EXECUTION_TIME_MS.write() = Some(result.execution_time_ms);
                *ROW_COUNT.write() = Some(result.rows.len());
                *LAST_ERROR.write() = None;
            }
            DbResponse::Error(e) => {
                // Update active tab with error
                if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
                    tab.last_error = Some(e.clone());
                    tab.result = None;
                }
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
            DbResponse::ExplainResult(plan) => {
                if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
                    tab.execution_plan = Some(plan);
                }
                *SHOW_EXECUTION_PLAN.write() = true;
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
            LlmResponse::Explanation(text) => {
                *AI_PANEL.write() = AiPanelState {
                    visible: true,
                    loading: false,
                    title: "Explanation".to_string(),
                    content: text,
                    suggested_sql: None,
                };
                *LLM_GENERATING.write() = false;
            }
            LlmResponse::Optimization { explanation, sql } => {
                *AI_PANEL.write() = AiPanelState {
                    visible: true,
                    loading: false,
                    title: "Optimization".to_string(),
                    content: explanation,
                    suggested_sql: sql,
                };
                *LLM_GENERATING.write() = false;
            }
            LlmResponse::ErrorFix { explanation, sql } => {
                *AI_PANEL.write() = AiPanelState {
                    visible: true,
                    loading: false,
                    title: "Error Fix".to_string(),
                    content: explanation,
                    suggested_sql: sql,
                };
                *LLM_GENERATING.write() = false;
            }
            LlmResponse::QuerySuggestions(suggestions) => {
                let table_name = SCHEMA_SUGGESTIONS.read().table_name.clone();
                *SCHEMA_SUGGESTIONS.write() = SuggestionsState {
                    suggestions,
                    loading: false,
                    table_name,
                };
                *LLM_GENERATING.write() = false;
            }
            LlmResponse::Error(e) => {
                *LLM_GENERATING.write() = false;
                *LLM_STATUS.write() = LlmStatus::Error(e.clone());
                // Also show error in AI panel if it's visible
                if AI_PANEL.read().visible {
                    *AI_PANEL.write() = AiPanelState {
                        visible: true,
                        loading: false,
                        title: "Error".to_string(),
                        content: e,
                        suggested_sql: None,
                    };
                }
            }
        }
    }
}
