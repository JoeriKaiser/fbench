use crate::config::{QueryStore, SavedQuery};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn SaveQueryDialog() -> Element {
    rsx! {
        if *SHOW_SAVE_QUERY_DIALOG.read() {
            SaveQueryDialogContent {}
        }
    }
}

#[component]
fn SaveQueryDialogContent() -> Element {
    let is_dark = *IS_DARK_MODE.read();
    let mut query_name = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);

    let overlay_bg = if is_dark {
        "bg-black bg-opacity-80"
    } else {
        "bg-black bg-opacity-50"
    };
    let dialog_bg = if is_dark { "bg-black" } else { "bg-white" };
    let dialog_border = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-300"
    };
    let text_color = if is_dark {
        "text-white"
    } else {
        "text-gray-900"
    };
    let label_color = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-600"
    };
    let input_bg = if is_dark { "bg-black" } else { "bg-white" };
    let input_border = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-300"
    };
    let input_text = if is_dark {
        "text-white"
    } else {
        "text-gray-900"
    };

    let on_save = move |_| {
        tracing::info!("Save button clicked in dialog");
        let name = query_name.read().trim().to_string();
        if name.is_empty() {
            error_message.set(Some("Please enter a query name".to_string()));
            return;
        }

        let sql = EDITOR_TABS
            .read()
            .active_tab()
            .map(|t| t.content.clone())
            .unwrap_or_default();
        if sql.trim().is_empty() {
            error_message.set(Some("Query content is empty".to_string()));
            return;
        }

        let store = QueryStore::new();
        let mut queries = store.load_queries();

        // Check if query with this name already exists
        if queries.iter().any(|q| q.name == name) {
            error_message.set(Some("A query with this name already exists".to_string()));
            return;
        }

        queries.push(SavedQuery {
            name,
            sql,
            is_bookmarked: false,
        });

        if let Err(e) = store.save_queries(&queries) {
            error_message.set(Some(format!("Failed to save query: {}", e)));
            return;
        }

        tracing::info!("Query saved successfully");

        // Trigger refresh of queries panel
        *QUERIES_REVISION.write() += 1;

        // Close dialog and reset
        *SHOW_SAVE_QUERY_DIALOG.write() = false;
        query_name.set(String::new());
        error_message.set(None);
    };

    let on_cancel = move |_| {
        *SHOW_SAVE_QUERY_DIALOG.write() = false;
        query_name.set(String::new());
        error_message.set(None);
    };

    let on_overlay_click = move |_| {
        *SHOW_SAVE_QUERY_DIALOG.write() = false;
        query_name.set(String::new());
        error_message.set(None);
    };

    let on_dialog_click = move |e: MouseEvent| {
        e.stop_propagation();
    };

    rsx! {
        div {
            class: "fixed inset-0 {overlay_bg} flex items-center justify-center z-50",
            onclick: on_overlay_click,

            div {
                class: "{dialog_bg} border {dialog_border} rounded-lg shadow-2xl w-[400px] max-w-[90vw]",
                onclick: on_dialog_click,

                div {
                    class: "p-6 space-y-4",

                    h2 {
                        class: "text-lg font-semibold {text_color}",
                        "Save Query"
                    }

                    div {
                        label {
                            class: "block text-sm font-medium {label_color} mb-1",
                            "Query Name"
                        }
                        input {
                            class: "w-full px-3 py-2 border rounded text-sm focus:outline-none {input_bg} {input_border} {input_text}",
                            r#type: "text",
                            placeholder: "My Query",
                            value: "{query_name}",
                            oninput: move |e| {
                                query_name.set(e.value().clone());
                                error_message.set(None);
                            },
                        }
                    }

                    if let Some(ref error) = *error_message.read() {
                        div {
                            class: "text-sm text-red-500",
                            "{error}"
                        }
                    }

                    div {
                        class: "flex justify-end space-x-3 pt-4",

                        button {
                            class: if is_dark {
                                "px-4 py-2 text-sm rounded transition-colors bg-gray-900 hover:bg-gray-800 text-white"
                            } else {
                                "px-4 py-2 text-sm rounded transition-colors bg-gray-100 hover:bg-gray-200 text-gray-700"
                            },
                            onclick: on_cancel,
                            "Cancel"
                        }

                        button {
                            class: "px-4 py-2 text-sm rounded transition-colors bg-blue-600 hover:bg-blue-500 text-white",
                            onclick: on_save,
                            "Save"
                        }
                    }
                }
            }
        }
    }
}
