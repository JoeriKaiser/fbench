use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn StatusBar() -> Element {
    let is_dark = *IS_DARK_MODE.read();

    let bg_class = if is_dark { "bg-black" } else { "bg-gray-50" };
    let border_class = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };
    let muted_text = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-500"
    };

    let status_text = match *CONNECTION.read() {
        ConnectionState::Connected {
            db_type,
            ref db_name,
        } => {
            let db_label = match db_type {
                DatabaseType::PostgreSQL => "PostgreSQL",
                DatabaseType::MySQL => "MySQL",
            };
            if db_name.is_empty() {
                format!("Connected to {}", db_label)
            } else {
                format!("Connected to {} / {}", db_label, db_name)
            }
        }
        ConnectionState::ConnectionLost => "Connection lost".to_string(),
        ConnectionState::Disconnected => "Not connected".to_string(),
        ConnectionState::Connecting => "Connecting...".to_string(),
        ConnectionState::Error(ref e) => format!("Error: {}", e),
    };

    let status_color = match *CONNECTION.read() {
        ConnectionState::Connected { .. } => "text-green-500",
        ConnectionState::ConnectionLost | ConnectionState::Error(_) => "text-red-500",
        _ => muted_text,
    };

    let tabs = EDITOR_TABS.read();
    let active_tab = tabs.active_tab();
    let row_count = active_tab
        .and_then(|tab| tab.result.as_ref())
        .map(|result| result.rows.len());
    let execution_time_ms = active_tab.and_then(|tab| tab.execution_time_ms);
    let import_message = IMPORT_MESSAGE.read().clone();

    rsx! {
        div {
            class: "h-7 {bg_class} border-t {border_class} flex items-center px-3 justify-between text-xs",

            div {
                class: "flex items-center space-x-4 min-w-0",
                span {
                    class: status_color,
                    "{status_text}"
                }

                if let Some(message) = import_message {
                    span {
                        class: "text-green-500 truncate",
                        "{message}"
                    }
                }
            }

            div {
                class: "flex items-center space-x-4",

                if let Some(count) = row_count {
                    span { class: muted_text, "{count} rows" }
                }

                if let Some(time) = execution_time_ms {
                    span { class: muted_text, "{time}ms" }
                }
            }
        }
    }
}
