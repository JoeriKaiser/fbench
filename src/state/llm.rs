use crate::llm::LlmConfig;
use dioxus::prelude::*;

pub static LLM_PROMPT: GlobalSignal<String> = Signal::global(String::new);

pub static LLM_GENERATING: GlobalSignal<bool> = Signal::global(|| false);

pub static SHOW_LLM_SETTINGS: GlobalSignal<bool> = Signal::global(|| false);

pub static LLM_CONFIG: GlobalSignal<LlmConfig> = Signal::global(LlmConfig::load);

#[derive(Clone, Debug, PartialEq)]
pub enum LlmStatus {
    None,
    Success(String),
    Error(String),
}

pub static LLM_STATUS: GlobalSignal<LlmStatus> = Signal::global(|| LlmStatus::None);
