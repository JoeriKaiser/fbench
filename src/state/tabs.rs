use crate::config::DraftStore;
use dioxus::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct QueryTab {
    pub id: String,
    pub title: String,
    pub content: String,
    pub result: Option<crate::db::QueryResult>,
    pub execution_plan: Option<String>,
    pub last_error: Option<String>,
    pub execution_time_ms: Option<u64>,
    pub unsaved_changes: bool,
}

impl QueryTab {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            content: String::new(),
            result: None,
            execution_plan: None,
            last_error: None,
            execution_time_ms: None,
            unsaved_changes: false,
        }
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }
}

#[derive(Debug, Clone)]
pub struct TabState {
    pub tabs: Vec<QueryTab>,
    pub active_tab_id: Option<String>,
}

impl TabState {
    pub fn new() -> Self {
        // Try to load from draft store first
        let store = DraftStore::new();
        if let Some(draft) = store.load_tabs() {
            let tabs: Vec<QueryTab> = draft
                .tabs
                .iter()
                .map(|d| QueryTab::new(d.title.clone()).with_content(d.content.clone()))
                .collect();

            let active_id = tabs
                .get(draft.active_tab_index)
                .or_else(|| tabs.first())
                .map(|t| t.id.clone());

            return Self {
                tabs,
                active_tab_id: active_id,
            };
        }

        // Default: single tab with sample query
        let default_tab = QueryTab::new("Query 1").with_content("SELECT * FROM users LIMIT 10;");
        let id = default_tab.id.clone();
        Self {
            tabs: vec![default_tab],
            active_tab_id: Some(id),
        }
    }

    pub fn active_tab(&self) -> Option<&QueryTab> {
        self.active_tab_id
            .as_ref()
            .and_then(|id| self.tabs.iter().find(|t| t.id == *id))
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut QueryTab> {
        self.active_tab_id
            .as_ref()
            .and_then(|id| self.tabs.iter_mut().find(|t| t.id == *id))
    }

    pub fn add_tab(&mut self, title: impl Into<String>) -> String {
        let tab = QueryTab::new(title);
        let id = tab.id.clone();
        self.tabs.push(tab);
        self.active_tab_id = Some(id.clone());
        id
    }

    pub fn close_tab(&mut self, id: &str) {
        if self.tabs.len() <= 1 {
            return; // Don't close last tab
        }
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(pos);
            if self.active_tab_id.as_ref() == Some(&id.to_string()) {
                // Switch to previous tab or first tab
                let new_pos = pos.saturating_sub(1);
                self.active_tab_id = self.tabs.get(new_pos).map(|t| t.id.clone());
            }
        }
    }

    pub fn set_active(&mut self, id: &str) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_tab_id = Some(id.to_string());
        }
    }
}

pub static EDITOR_TABS: GlobalSignal<TabState> = Signal::global(TabState::new);
