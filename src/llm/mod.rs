use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::db::{ConstraintInfo, IndexInfo, SchemaInfo};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum LlmProvider {
    #[default]
    Ollama,
    OpenRouter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub ollama_url: String,
    pub ollama_model: String,
    pub openrouter_key: String,
    pub openrouter_model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Ollama,
            ollama_url: "http://localhost:11434".into(),
            ollama_model: "llama3.2".into(),
            openrouter_key: String::new(),
            openrouter_model: "openai/gpt-4o-mini".into(),
        }
    }
}

impl LlmConfig {
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("No config dir")?;
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "fbench", "fbench").map(|d| {
            let dir = d.config_dir().to_path_buf();
            fs::create_dir_all(&dir).ok();
            dir.join("llm.json")
        })
    }
}

#[derive(Debug)]
pub enum LlmRequest {
    Generate {
        prompt: String,
        schema: SchemaInfo,
        config: LlmConfig,
    },
    Explain {
        sql: String,
        config: LlmConfig,
    },
    #[allow(dead_code)]
    Optimize {
        sql: String,
        schema: SchemaInfo,
        config: LlmConfig,
    },
    #[allow(dead_code)]
    FixError {
        sql: String,
        error: String,
        schema: SchemaInfo,
        config: LlmConfig,
    },
    SuggestQueries {
        table: crate::db::TableInfo,
        config: LlmConfig,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySuggestion {
    pub label: String,
    pub sql: String,
}

#[derive(Debug)]
pub enum LlmResponse {
    Generated(String),
    Explanation(String),
    Optimization {
        explanation: String,
        sql: Option<String>,
    },
    ErrorFix {
        explanation: String,
        sql: Option<String>,
    },
    QuerySuggestions(Vec<QuerySuggestion>),
    Error(String),
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

pub struct LlmWorker {
    client: reqwest::Client,
    request_rx: mpsc::UnboundedReceiver<LlmRequest>,
    response_tx: mpsc::UnboundedSender<LlmResponse>,
}

impl LlmWorker {
    pub fn new(
        request_rx: mpsc::UnboundedReceiver<LlmRequest>,
        response_tx: mpsc::UnboundedSender<LlmResponse>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            request_rx,
            response_tx,
        }
    }

    pub async fn run(mut self) {
        while let Some(request) = self.request_rx.recv().await {
            let response = match request {
                LlmRequest::Generate {
                    prompt,
                    schema,
                    config,
                } => self.generate(&prompt, &schema, &config).await,
                LlmRequest::Explain { sql, config } => self.explain(&sql, &config).await,
                LlmRequest::Optimize {
                    sql,
                    schema,
                    config,
                } => self.optimize(&sql, &schema, &config).await,
                LlmRequest::FixError {
                    sql,
                    error,
                    schema,
                    config,
                } => self.fix_error(&sql, &error, &schema, &config).await,
                LlmRequest::SuggestQueries { table, config } => {
                    self.suggest_queries(&table, &config).await
                }
            };
            let _ = self.response_tx.send(response);
        }
    }

    async fn generate(
        &self,
        user_prompt: &str,
        schema: &SchemaInfo,
        config: &LlmConfig,
    ) -> LlmResponse {
        let prompt = self.build_prompt(user_prompt, schema);

        let result = match config.provider {
            LlmProvider::Ollama => self.call_ollama(&prompt, config).await,
            LlmProvider::OpenRouter => self.call_openrouter(&prompt, config).await,
        };

        match result {
            Ok(sql) => LlmResponse::Generated(sql),
            Err(e) => LlmResponse::Error(e),
        }
    }

    async fn explain(&self, sql: &str, config: &LlmConfig) -> LlmResponse {
        let prompt = format!(
            "Explain this SQL query in plain English. Be concise (2-3 sentences).\n\n\
             Query:\n{}\n\nExplanation:",
            sql
        );

        let result = match config.provider {
            LlmProvider::Ollama => self.call_ollama(&prompt, config).await,
            LlmProvider::OpenRouter => self.call_openrouter(&prompt, config).await,
        };

        match result {
            Ok(text) => LlmResponse::Explanation(text.trim().to_string()),
            Err(e) => LlmResponse::Error(e),
        }
    }

    async fn optimize(&self, sql: &str, schema: &SchemaInfo, config: &LlmConfig) -> LlmResponse {
        let schema_text = self.format_schema(schema);
        let prompt = format!(
            "Analyze this SQL query for performance improvements.\n\n\
             Schema:\n{}\n\n\
             Query:\n{}\n\n\
             Provide:\
             1. Brief explanation of potential issues (1-2 sentences)\
             2. Optimized query if applicable\n\n\
             Format your response as:\
             EXPLANATION: <your explanation>\n\
             SQL: <optimized query or 'NO_CHANGE' if already optimal>",
            schema_text, sql
        );

        let result = match config.provider {
            LlmProvider::Ollama => self.call_ollama(&prompt, config).await,
            LlmProvider::OpenRouter => self.call_openrouter(&prompt, config).await,
        };

        match result {
            Ok(text) => Self::parse_optimization_response(&text),
            Err(e) => LlmResponse::Error(e),
        }
    }

    fn parse_optimization_response(response: &str) -> LlmResponse {
        let explanation = response
            .lines()
            .find(|l| l.starts_with("EXPLANATION:"))
            .map(|l| l.trim_start_matches("EXPLANATION:").trim().to_string())
            .unwrap_or_else(|| response.lines().next().unwrap_or("").to_string());

        let sql_start = response.find("SQL:");
        let sql = sql_start.and_then(|i| {
            let after = response[i + 4..].trim();
            if after.is_empty() || after == "NO_CHANGE" {
                None
            } else {
                Some(Self::extract_sql(after))
            }
        });

        LlmResponse::Optimization { explanation, sql }
    }

    async fn fix_error(
        &self,
        sql: &str,
        error: &str,
        schema: &SchemaInfo,
        config: &LlmConfig,
    ) -> LlmResponse {
        let schema_text = self.format_schema(schema);
        let prompt = format!(
            "This SQL query failed with an error. Explain the problem and provide a fix.\n\n\
             Schema:\n{}\n\n\
             Query:\n{}\n\n\
             Error:\n{}\n\n\
             Format your response as:\
             EXPLANATION: <what went wrong>\n\
             SQL: <corrected query>",
            schema_text, sql, error
        );

        let result = match config.provider {
            LlmProvider::Ollama => self.call_ollama(&prompt, config).await,
            LlmProvider::OpenRouter => self.call_openrouter(&prompt, config).await,
        };

        match result {
            Ok(text) => Self::parse_fix_response(&text),
            Err(e) => LlmResponse::Error(e),
        }
    }

    fn parse_fix_response(response: &str) -> LlmResponse {
        let explanation = response
            .lines()
            .find(|l| l.starts_with("EXPLANATION:"))
            .map(|l| l.trim_start_matches("EXPLANATION:").trim().to_string())
            .unwrap_or_else(|| response.lines().next().unwrap_or("").to_string());

        let sql_start = response.find("SQL:");
        let sql = sql_start
            .map(|i| {
                let after = &response[i + 4..];
                Self::extract_sql(after.trim())
            })
            .filter(|s| !s.is_empty());

        LlmResponse::ErrorFix { explanation, sql }
    }

    async fn suggest_queries(
        &self,
        table: &crate::db::TableInfo,
        config: &LlmConfig,
    ) -> LlmResponse {
        let columns: Vec<String> = table
            .columns
            .iter()
            .map(|c| {
                let pk = if c.is_primary_key { " PK" } else { "" };
                format!("{} {}{}", c.name, c.data_type, pk)
            })
            .collect();

        let prompt = format!(
            "Suggest 3 useful SQL queries for this table. Keep labels short (3-5 words).\n\n\
             Table: {}\n\
             Columns:\n{}\n\
             Row estimate: {}\n\n\
             Format each suggestion as:\n\
             LABEL: <short description>\n\
             SQL: <query>\n\
             ---",
            table.name,
            columns.join("\n"),
            table.row_estimate
        );

        let result = match config.provider {
            LlmProvider::Ollama => self.call_ollama(&prompt, config).await,
            LlmProvider::OpenRouter => self.call_openrouter(&prompt, config).await,
        };

        match result {
            Ok(text) => Self::parse_suggestions_response(&text),
            Err(e) => LlmResponse::Error(e),
        }
    }

    fn parse_suggestions_response(response: &str) -> LlmResponse {
        let mut suggestions = Vec::new();
        let mut current_label = String::new();
        let mut current_sql = String::new();
        let mut in_sql = false;

        for line in response.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("LABEL:") {
                if !current_label.is_empty() && !current_sql.is_empty() {
                    suggestions.push(QuerySuggestion {
                        label: current_label.clone(),
                        sql: Self::extract_sql(&current_sql),
                    });
                }
                current_label = trimmed.trim_start_matches("LABEL:").trim().to_string();
                current_sql.clear();
                in_sql = false;
            } else if trimmed.starts_with("SQL:") {
                current_sql = trimmed.trim_start_matches("SQL:").trim().to_string();
                in_sql = true;
            } else if trimmed == "---" {
                if !current_label.is_empty() && !current_sql.is_empty() {
                    suggestions.push(QuerySuggestion {
                        label: current_label.clone(),
                        sql: Self::extract_sql(&current_sql),
                    });
                }
                current_label.clear();
                current_sql.clear();
                in_sql = false;
            } else if in_sql && !trimmed.is_empty() {
                current_sql.push('\n');
                current_sql.push_str(trimmed);
            }
        }

        // Don't forget the last one
        if !current_label.is_empty() && !current_sql.is_empty() {
            suggestions.push(QuerySuggestion {
                label: current_label,
                sql: Self::extract_sql(&current_sql),
            });
        }

        suggestions.truncate(3);
        LlmResponse::QuerySuggestions(suggestions)
    }

    fn format_constraint(&self, constraint: &ConstraintInfo) -> String {
        let mut text = format!("{}: {}", constraint.constraint_type, constraint.name);

        if !constraint.columns.is_empty() {
            text.push_str(&format!(" ({})", constraint.columns.join(", ")));
        }

        if let Some(foreign_table) = &constraint.foreign_table {
            text.push_str(&format!(" -> {}", foreign_table));
            if let Some(foreign_columns) = &constraint.foreign_columns {
                if !foreign_columns.is_empty() {
                    text.push_str(&format!(" ({})", foreign_columns.join(", ")));
                }
            }
        }

        if let Some(check_clause) = &constraint.check_clause {
            text.push_str(&format!(" [{}]", check_clause));
        }

        text
    }

    fn format_index(&self, index: &IndexInfo) -> String {
        let mut flags = Vec::new();
        if index.is_primary {
            flags.push("PRIMARY");
        } else if index.is_unique {
            flags.push("UNIQUE");
        }
        if !index.index_type.is_empty() {
            flags.push(index.index_type.as_str());
        }

        let flag_text = if flags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", flags.join(", "))
        };

        format!("{}{} ({})", index.name, flag_text, index.columns.join(", "))
    }

    fn format_schema(&self, schema: &SchemaInfo) -> String {
        let mut text = String::new();

        for table in &schema.tables {
            text.push_str(&format!("Table: {}\n", table.name));
            for col in &table.columns {
                let pk = if col.is_primary_key { " PK" } else { "" };
                let null = if col.nullable { " nullable" } else { "" };
                text.push_str(&format!(
                    "  Column: {} {}{}{}\n",
                    col.name, col.data_type, null, pk
                ));
            }

            if !table.constraints.is_empty() {
                text.push_str("  Constraints:\n");
                for constraint in &table.constraints {
                    text.push_str(&format!("    {}\n", self.format_constraint(constraint)));
                }
            }

            if !table.indexes.is_empty() {
                text.push_str("  Indexes:\n");
                for index in &table.indexes {
                    text.push_str(&format!("    {}\n", self.format_index(index)));
                }
            }

            text.push('\n');
        }

        if !schema.views.is_empty() {
            text.push_str("Views:\n");
            for view in &schema.views {
                text.push_str(&format!("  {}\n", view));
            }
        }

        text
    }

    fn build_prompt(&self, user_prompt: &str, schema: &SchemaInfo) -> String {
        let schema_text = self.format_schema(schema);

        format!(
            "You are a SQL expert. Generate a SQL query based on the user's request.\n\
             Only output the raw SQL query, no explanations, no markdown.\n\
             Use only tables, views, columns, and relationships listed below.\n\
             When a join is needed, prefer the foreign-key relationships from the schema.\n\n\
             Database schema:\n{}\n\
             User request: {}\n\nSQL:",
            schema_text, user_prompt
        )
    }

    async fn call_ollama(&self, prompt: &str, config: &LlmConfig) -> Result<String, String> {
        let url = format!("{}/api/generate", config.ollama_url);

        let response = self
            .client
            .post(&url)
            .json(&OllamaRequest {
                model: config.ollama_model.clone(),
                prompt: prompt.to_string(),
                stream: false,
            })
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama error: {}", response.status()));
        }

        let result: OllamaResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(Self::extract_sql(&result.response))
    }

    async fn call_openrouter(&self, prompt: &str, config: &LlmConfig) -> Result<String, String> {
        if config.openrouter_key.is_empty() {
            return Err("OpenRouter API key not configured".into());
        }

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", config.openrouter_key))
            .header("Content-Type", "application/json")
            .json(&OpenRouterRequest {
                model: config.openrouter_model.clone(),
                messages: vec![ChatMessage {
                    role: "user",
                    content: prompt.to_string(),
                }],
            })
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OpenRouter error {}: {}", status, body));
        }

        let result: OpenRouterResponse = response.json().await.map_err(|e| e.to_string())?;
        let content = result
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(Self::extract_sql(&content))
    }

    fn extract_sql(response: &str) -> String {
        let trimmed = response.trim();

        if let Some(start) = trimmed.find("```sql") {
            let after = &trimmed[start + 6..];
            if let Some(end) = after.find("```") {
                return after[..end].trim().to_string();
            }
        }

        if let Some(start) = trimmed.find("```") {
            let after = &trimmed[start + 3..];
            if let Some(end) = after.find("```") {
                return after[..end].trim().to_string();
            }
        }

        trimmed.to_string()
    }
}

pub fn spawn_llm_worker() -> (
    mpsc::UnboundedSender<LlmRequest>,
    mpsc::UnboundedReceiver<LlmResponse>,
) {
    let (request_tx, request_rx) = mpsc::unbounded_channel();
    let (response_tx, response_rx) = mpsc::unbounded_channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(LlmWorker::new(request_rx, response_tx).run());
    });

    (request_tx, response_rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{ColumnInfo, SchemaInfo, TableInfo};

    fn worker() -> LlmWorker {
        let (_request_tx, request_rx) = mpsc::unbounded_channel();
        let (response_tx, _response_rx) = mpsc::unbounded_channel();
        LlmWorker::new(request_rx, response_tx)
    }

    #[test]
    fn build_prompt_includes_relationships_and_indexes() {
        let worker = worker();
        let schema = SchemaInfo {
            tables: vec![TableInfo {
                name: "orders".into(),
                columns: vec![
                    ColumnInfo {
                        name: "id".into(),
                        data_type: "integer".into(),
                        nullable: false,
                        default_value: None,
                        is_primary_key: true,
                    },
                    ColumnInfo {
                        name: "customer_id".into(),
                        data_type: "integer".into(),
                        nullable: false,
                        default_value: None,
                        is_primary_key: false,
                    },
                ],
                indexes: vec![IndexInfo {
                    name: "orders_customer_id_idx".into(),
                    columns: vec!["customer_id".into()],
                    is_unique: false,
                    is_primary: false,
                    index_type: "btree".into(),
                }],
                constraints: vec![ConstraintInfo {
                    name: "orders_customer_id_fkey".into(),
                    constraint_type: "FOREIGN KEY".into(),
                    columns: vec!["customer_id".into()],
                    foreign_table: Some("customers".into()),
                    foreign_columns: Some(vec!["id".into()]),
                    check_clause: None,
                }],
                row_estimate: 0,
            }],
            views: vec!["recent_orders".into()],
        };

        let prompt = worker.build_prompt("list recent orders with customer names", &schema);

        assert!(
            prompt.contains("FOREIGN KEY: orders_customer_id_fkey (customer_id) -> customers (id)")
        );
        assert!(prompt.contains("orders_customer_id_idx [btree] (customer_id)"));
        assert!(prompt.contains("Views:\n  recent_orders"));
    }
}
