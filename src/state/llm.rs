use crate::llm::LlmConfig;
use dioxus::prelude::*;

/// A preset prompt template for quick selection
#[derive(Clone, Debug, PartialEq)]
pub struct LlmPreset {
    pub name: &'static str,
    pub prompt: &'static str,
}

/// Built-in preset prompts for common SQL generation tasks
pub const LLM_PRESETS: &[LlmPreset] = &[
    LlmPreset {
        name: "Custom",
        prompt: "",
    },
    LlmPreset {
        name: "Find all records",
        prompt: "Find all records from the main table",
    },
    LlmPreset {
        name: "Count records",
        prompt: "Count total records in the main table",
    },
    LlmPreset {
        name: "Recent records",
        prompt: "Get the 10 most recent records",
    },
    LlmPreset {
        name: "Join tables",
        prompt: "Join the main tables and show related data",
    },
    LlmPreset {
        name: "Aggregation",
        prompt: "Show aggregation statistics grouped by category",
    },
];

pub static LLM_PROMPT: GlobalSignal<String> = Signal::global(String::new);
pub static SELECTED_PRESET_INDEX: GlobalSignal<usize> = Signal::global(|| 0);

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
