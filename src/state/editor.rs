use crate::config::DraftStore;
use crate::db::QueryResult;
use dioxus::prelude::*;

// DEPRECATED: Use EDITOR_TABS instead for multi-tab support
// Keeping temporarily for backward compatibility during migration
pub static EDITOR_CONTENT: GlobalSignal<String> = Signal::global(|| {
    // Load from draft store
    let store = DraftStore::new();
    let draft = store.load();
    if draft.content.is_empty() {
        "SELECT * FROM users LIMIT 10;".to_string()
    } else {
        draft.content
    }
});

pub static QUERY_RESULT: GlobalSignal<Option<QueryResult>> = Signal::global(|| None);

pub static LAST_ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);

pub static EXECUTION_TIME_MS: GlobalSignal<Option<u64>> = Signal::global(|| None);

pub static ROW_COUNT: GlobalSignal<Option<usize>> = Signal::global(|| None);

// Increments when query history is updated (for UI reactivity)
pub static HISTORY_REVISION: GlobalSignal<u64> = Signal::global(|| 0);
