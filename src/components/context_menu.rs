use crate::services::LlmSender;
use crate::state::*;
use dioxus::prelude::*;

/// Global signal to track context menu state
pub static CONTEXT_MENU: GlobalSignal<Option<ContextMenuState>> = Signal::global(|| None);

#[derive(Clone, Debug)]
pub struct ContextMenuState {
    pub x: i32,
    pub y: i32,
    pub table_name: String,
}

/// Show context menu at specified position for a table
pub fn show_table_context_menu(table_name: String, x: i32, y: i32) {
    *CONTEXT_MENU.write() = Some(ContextMenuState { x, y, table_name });
}

/// Hide the context menu
pub fn hide_context_menu() {
    *CONTEXT_MENU.write() = None;
}

#[component]
pub fn ContextMenu() -> Element {
    let menu_state = CONTEXT_MENU.read().clone();
    let llm_tx = use_context::<LlmSender>();

    let Some(state) = menu_state else {
        return rsx! {};
    };

    let is_dark = *IS_DARK_MODE.read();
    let is_connected = matches!(*CONNECTION.read(), ConnectionState::Connected { .. });
    let schema = SCHEMA.read().clone();
    let table_name = state.table_name.clone();

    let bg_class = if is_dark {
        "bg-black border-gray-800"
    } else {
        "bg-white border-gray-200"
    };
    let text_class = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };
    let hover_class = if is_dark {
        "hover:bg-gray-900"
    } else {
        "hover:bg-gray-100"
    };
    let muted_class = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };

    // Clone for closures
    let table_name_for_select = table_name.clone();
    let table_name_for_explain = table_name.clone();
    let table_name_for_suggest = table_name.clone();
    let llm_tx_explain = llm_tx.clone();
    let llm_tx_suggest = llm_tx.clone();

    rsx! {
        // Backdrop to close menu when clicking outside
        div {
            class: "fixed inset-0 z-50",
            onclick: move |_| hide_context_menu(),

            // Menu positioned at click coordinates
            div {
                class: "fixed rounded-lg shadow-xl border py-1 min-w-[200px] z-50 {bg_class}",
                style: "left: {state.x}px; top: {state.y}px;",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "px-3 py-1.5 text-xs font-medium {text_class} border-b opacity-60",
                    class: if is_dark { "border-gray-800" } else { "border-gray-200" },
                    "Table: {table_name}"
                }

                button {
                    class: "w-full text-left px-3 py-2 text-sm {text_class} {hover_class} transition-colors flex items-center space-x-2",
                    onclick: move |_| {
                        let sql = format!("SELECT * FROM \"{}\" LIMIT 100;", table_name_for_select);
                        if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
                            tab.content = sql;
                            tab.unsaved_changes = true;
                        }
                        hide_context_menu();
                    },

                    svg {
                        class: "w-4 h-4 opacity-70",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
                        }
                    }
                    span { "SELECT * FROM {table_name}" }
                }

                // AI Actions section
                if is_connected {
                    div {
                        class: "my-1 border-t",
                        class: if is_dark { "border-gray-800" } else { "border-gray-200" },
                    }

                    div {
                        class: "px-3 py-1 text-xs {muted_class} uppercase tracking-wider",
                        "AI Actions"
                    }

                    // Explain action
                    button {
                        class: "w-full text-left px-3 py-2 text-sm {text_class} {hover_class} transition-colors flex items-center space-x-2",
                        onclick: move |_| {
                            let sql = format!("SELECT * FROM \"{}\" LIMIT 100;", table_name_for_explain);
                            *AI_PANEL.write() = AiPanelState {
                                visible: true,
                                loading: true,
                                title: "Explaining...".to_string(),
                                content: String::new(),
                                suggested_sql: None,
                            };
                            let config = LLM_CONFIG.read().clone();
                            let _ = llm_tx_explain.send(crate::llm::LlmRequest::Explain {
                                sql,
                                config,
                            });
                            hide_context_menu();
                        },

                        svg {
                            class: "w-4 h-4 opacity-70",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                            }
                        }
                        span { "Explain Query" }
                    }

                    // Suggest queries action
                    button {
                        class: "w-full text-left px-3 py-2 text-sm {text_class} {hover_class} transition-colors flex items-center space-x-2",
                        onclick: move |_| {
                            if let Some(table) = schema.tables.iter().find(|t| t.name == table_name_for_suggest) {
                                *SCHEMA_SUGGESTIONS.write() = SuggestionsState {
                                    suggestions: Vec::new(),
                                    loading: true,
                                    table_name: Some(table_name_for_suggest.clone()),
                                };
                                let config = LLM_CONFIG.read().clone();
                                let _ = llm_tx_suggest.send(crate::llm::LlmRequest::SuggestQueries {
                                    table: table.clone(),
                                    config,
                                });
                            }
                            hide_context_menu();
                        },

                        svg {
                            class: "w-4 h-4 opacity-70",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z",
                            }
                        }
                        span { "Suggest Queries" }
                    }
                }
            }
        }
    }
}
