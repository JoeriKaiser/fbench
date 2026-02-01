use crate::components::layout::AppLayout;
use crate::services::init_services;
use crate::state::{ConnectionState, CONNECTION, SHOW_CONNECTION_DIALOG};
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    let (db_tx, llm_tx) = use_hook(init_services);

    use_context_provider(|| db_tx);
    use_context_provider(|| llm_tx);

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
