use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::db::SchemaInfo;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LlmProvider {
    Ollama,
    OpenRouter,
}

impl Default for LlmProvider {
    fn default() -> Self {
        Self::Ollama
    }
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
}

#[derive(Debug)]
pub enum LlmResponse {
    Generated(String),
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
                LlmRequest::Generate { prompt, schema, config } => {
                    self.generate(&prompt, &schema, &config).await
                }
            };
            let _ = self.response_tx.send(response);
        }
    }

    async fn generate(&self, user_prompt: &str, schema: &SchemaInfo, config: &LlmConfig) -> LlmResponse {
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

    fn build_prompt(&self, user_prompt: &str, schema: &SchemaInfo) -> String {
        let mut prompt = String::from(
            "You are a SQL expert. Generate a SQL query based on the user's request.\n\
             Only output the raw SQL query, no explanations, no markdown.\n\n\
             Database schema:\n",
        );

        for table in &schema.tables {
            prompt.push_str(&format!("\nTable: {}\n", table.name));
            for col in &table.columns {
                let pk = if col.is_primary_key { " PK" } else { "" };
                let null = if col.nullable { "?" } else { "" };
                prompt.push_str(&format!("  {} {}{}{}\n", col.name, col.data_type, null, pk));
            }
        }

        for view in &schema.views {
            prompt.push_str(&format!("\nView: {}\n", view));
        }

        prompt.push_str(&format!("\nUser request: {}\n\nSQL:", user_prompt));
        prompt
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
