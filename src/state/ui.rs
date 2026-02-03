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

pub static SHOW_TABLE_DETAIL: GlobalSignal<Option<String>> = Signal::global(|| None);

/// Test connection status
#[derive(Clone, Debug, PartialEq, Default)]
pub enum TestConnectionStatus {
    #[default]
    Idle,
    Testing,
    Success,
    Failed(String),
}

pub static TEST_CONNECTION_STATUS: GlobalSignal<TestConnectionStatus> =
    Signal::global(|| TestConnectionStatus::Idle);

/// System theme preference
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Dark,
    Light,
}

pub static THEME_PREFERENCE: GlobalSignal<ThemePreference> =
    Signal::global(|| ThemePreference::System);

/// Current effective theme (resolved from system preference if set to System)
pub static IS_DARK_MODE: GlobalSignal<bool> = Signal::global(|| true);

/// Panel resize state - stores the height of the SQL editor panel (in pixels)
/// Results panel takes remaining space
pub static EDITOR_PANEL_HEIGHT: GlobalSignal<f64> = Signal::global(|| 300.0);

/// Whether we're currently resizing panels
pub static IS_RESIZING_PANELS: GlobalSignal<bool> = Signal::global(|| false);

/// Quick switcher visibility
pub static SHOW_QUICK_SWITCHER: GlobalSignal<bool> = Signal::global(|| false);
