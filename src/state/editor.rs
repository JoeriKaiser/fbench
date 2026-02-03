use crate::config::DraftStore;
use crate::db::QueryResult;
use dioxus::prelude::*;

// Load draft content or use default
fn get_initial_content() -> String {
    let store = DraftStore::new();
    let draft = store.load();
    if draft.content.is_empty() {
        "SELECT * FROM users LIMIT 10;".to_string()
    } else {
        draft.content
    }
}

pub static EDITOR_CONTENT: GlobalSignal<String> = Signal::global(|| get_initial_content());

pub static QUERY_RESULT: GlobalSignal<Option<QueryResult>> = Signal::global(|| None);

pub static LAST_ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);

pub static EXECUTION_TIME_MS: GlobalSignal<Option<u64>> = Signal::global(|| None);

pub static ROW_COUNT: GlobalSignal<Option<usize>> = Signal::global(|| None);

// Increments when query history is updated (for UI reactivity)
pub static HISTORY_REVISION: GlobalSignal<u64> = Signal::global(|| 0);
