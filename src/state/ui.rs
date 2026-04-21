use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum LeftTab {
    #[default]
    Schema,
    Queries,
    History,
}

pub static LEFT_TAB: GlobalSignal<LeftTab> = Signal::global(|| LeftTab::Schema);

pub static SHOW_CONNECTION_DIALOG: GlobalSignal<bool> = Signal::global(|| false);

pub static SHOW_SAVE_QUERY_DIALOG: GlobalSignal<bool> = Signal::global(|| false);

/// Test connection status
#[derive(Clone, Debug, PartialEq, Default)]
pub enum TestConnectionStatus {
    #[default]
    Idle,
    Testing,
    Connecting,
    Success,
    Failed(String),
}

pub static TEST_CONNECTION_STATUS: GlobalSignal<TestConnectionStatus> =
    Signal::global(|| TestConnectionStatus::Idle);

pub static IS_DARK_MODE: GlobalSignal<bool> = Signal::global(|| true);

/// Panel resize state - stores the height of the SQL editor panel (in pixels)
/// Results panel takes remaining space
pub static EDITOR_PANEL_HEIGHT: GlobalSignal<f64> = Signal::global(|| 300.0);

/// Increments when saved queries are updated (for UI reactivity)
pub static QUERIES_REVISION: GlobalSignal<u64> = Signal::global(|| 0);

/// Whether we're currently resizing panels
pub static IS_RESIZING_PANELS: GlobalSignal<bool> = Signal::global(|| false);

/// Quick switcher visibility
pub static SHOW_QUICK_SWITCHER: GlobalSignal<bool> = Signal::global(|| false);

/// JSON viewer modal state
pub static SHOW_JSON_VIEWER: GlobalSignal<bool> = Signal::global(|| false);
pub static JSON_VIEWER_CONTENT: GlobalSignal<String> = Signal::global(String::new);

/// Execution plan modal state
pub static SHOW_EXECUTION_PLAN: GlobalSignal<bool> = Signal::global(|| false);

/// Import progress state: (inserted, total)
pub static IMPORT_PROGRESS: GlobalSignal<Option<(usize, usize)>> = Signal::global(|| None);

/// Import completion/error message shown in the import dialog
pub static IMPORT_MESSAGE: GlobalSignal<Option<String>> = Signal::global(|| None);

/// Import dialog visibility
pub static SHOW_IMPORT_DIALOG: GlobalSignal<bool> = Signal::global(|| false);
