use crate::components::layout::AppLayout;
use crate::config::SessionStore;
use crate::services::init_services;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    let (db_tx, llm_tx) = use_hook(init_services);

    use_context_provider(|| db_tx);
    use_context_provider(|| llm_tx);

    // Restore session state
    use_effect(move || {
        let store = SessionStore::new();
        let session = store.load();

        // Restore left tab
        match session.left_tab.as_str() {
            "Queries" => *LEFT_TAB.write() = LeftTab::Queries,
            "History" => *LEFT_TAB.write() = LeftTab::History,
            _ => *LEFT_TAB.write() = LeftTab::Schema,
        }

        // Restore panel height
        if session.editor_panel_height > 0.0 {
            *EDITOR_PANEL_HEIGHT.write() = session.editor_panel_height;
        }
    });

    // Auto-show connection modal when not connected
    use_effect(move || {
        if matches!(*CONNECTION.read(), ConnectionState::Disconnected) {
            *SHOW_CONNECTION_DIALOG.write() = true;
        }
    });

    rsx! {
        AppLayout {}
    }
}
