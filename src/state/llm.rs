use crate::llm::{LlmConfig, QuerySuggestion};
use dioxus::prelude::*;

/// A preset prompt template for quick selection
#[derive(Clone, Debug, PartialEq)]
pub struct LlmPreset {
    pub name: &'static str,
    pub prompt: &'static str,
}

/// AI action types for context menu
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AiAction {
    Explain,
    Optimize,
    FixError,
}

/// AI panel state for displaying results
#[derive(Clone, Debug, PartialEq)]
pub struct AiPanelState {
    pub visible: bool,
    pub loading: bool,
    pub title: String,
    pub content: String,
    pub suggested_sql: Option<String>,
}

impl Default for AiPanelState {
    fn default() -> Self {
        Self {
            visible: false,
            loading: false,
            title: String::new(),
            content: String::new(),
            suggested_sql: None,
        }
    }
}

/// Schema panel suggestions state
#[derive(Clone, Debug, Default)]
pub struct SuggestionsState {
    pub suggestions: Vec<QuerySuggestion>,
    pub loading: bool,
    pub table_name: Option<String>,
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

/// AI panel state for displaying explanations, optimizations, and fixes
pub static AI_PANEL: GlobalSignal<AiPanelState> = Signal::global(AiPanelState::default);

/// Schema-aware query suggestions
pub static SCHEMA_SUGGESTIONS: GlobalSignal<SuggestionsState> =
    Signal::global(SuggestionsState::default);
