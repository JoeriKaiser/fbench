use crate::components::context_menu::show_table_context_menu;
use crate::config::RecentTablesStore;
use crate::services::LlmSender;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn SchemaPanel() -> Element {
    let schema = SCHEMA.read();
    let is_dark = *IS_DARK_MODE.read();
    let is_connected = matches!(*CONNECTION.read(), ConnectionState::Connected { .. });
    let llm_tx = use_context::<LlmSender>();

    let muted_text = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };
    let header_text = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-500"
    };
    let item_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let item_hover = if is_dark {
        "hover:bg-gray-900 hover:text-white"
    } else {
        "hover:bg-gray-100 hover:text-gray-900"
    };

    rsx! {
        div {
            class: "space-y-2",

            if !is_connected {
                div {
                    class: "{muted_text} text-sm text-center py-8",
                    "Connect to a database to view schema"
                }
            } else if schema.tables.is_empty() {
                div {
                    class: "{muted_text} text-sm text-center py-8",
                    "No tables found"
                }
            } else {
                h3 {
                    class: "text-xs font-semibold {header_text} uppercase tracking-wider mb-2",
                    "Tables ({schema.tables.len()})"
                }

                for table in &schema.tables {
                    TableItem { table: table.clone() }
                }

                if !schema.views.is_empty() {
                    h3 {
                        class: "text-xs font-semibold {header_text} uppercase tracking-wider mb-2 mt-4",
                        "Views ({schema.views.len()})"
                    }

                    for view in &schema.views {
                        ViewItem { view: view.clone() }
                    }
                }

                // AI Suggestions section
                SuggestionsSection {}
            }
        }
    }
}

#[component]
fn SuggestionsSection() -> Element {
    let suggestions = SCHEMA_SUGGESTIONS.read();
    let schema = SCHEMA.read();
    let llm_tx = use_context::<LlmSender>();
    let is_dark = *IS_DARK_MODE.read();

    let header_text = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-500"
    };
    let item_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let item_hover = if is_dark {
        "hover:bg-gray-900 hover:text-white"
    } else {
        "hover:bg-gray-100 hover:text-gray-900"
    };
    let muted_text = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };

    let Some(table_name) = suggestions.table_name.clone() else {
        return rsx! {};
    };

    let suggestions_list: Vec<_> = suggestions.suggestions.clone();
    let is_loading = suggestions.loading;

    rsx! {
        div {
            class: "mt-4 pt-4 border-t",
            class: if is_dark { "border-gray-800" } else { "border-gray-200" },

            div {
                class: "flex items-center justify-between mb-2",

                h3 {
                    class: "text-xs font-semibold {header_text} uppercase tracking-wider",
                    "Suggested Queries"
                }

                if is_loading {
                    div {
                        class: "animate-spin h-3 w-3 border-2 border-blue-500 border-t-transparent rounded-full",
                    }
                } else {
                    button {
                        class: "text-xs {item_text} hover:text-blue-500 transition-colors",
                        onclick: move |_| {
                            if let Some(table) = schema.tables.iter().find(|t| t.name == table_name) {
                                *SCHEMA_SUGGESTIONS.write() = SuggestionsState {
                                    suggestions: Vec::new(),
                                    loading: true,
                                    table_name: Some(table_name.clone()),
                                };
                                let config = LLM_CONFIG.read().clone();
                                let _ = llm_tx.send(crate::llm::LlmRequest::SuggestQueries {
                                    table: table.clone(),
                                    config,
                                });
                            }
                        },
                        "↻ Refresh"
                    }
                }
            }

            if suggestions_list.is_empty() && !is_loading {
                div {
                    class: "text-xs {muted_text}",
                    "No suggestions available"
                }
            } else {
                div {
                    class: "space-y-1",

                    for suggestion in suggestions_list {
                        SuggestionButton { suggestion: suggestion.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn SuggestionButton(suggestion: crate::llm::QuerySuggestion) -> Element {
    let is_dark = *IS_DARK_MODE.read();
    let item_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let item_hover = if is_dark {
        "hover:bg-gray-900 hover:text-white"
    } else {
        "hover:bg-gray-100 hover:text-gray-900"
    };

    let sql = suggestion.sql.clone();
    let label = suggestion.label.clone();

    rsx! {
        button {
            class: "w-full text-left px-2 py-1.5 text-xs {item_text} {item_hover} rounded transition-colors",
            onclick: move |_| {
                *EDITOR_CONTENT.write() = sql.clone();
            },
            title: "{suggestion.sql}",
            "▸ {label}"
        }
    }
}

#[component]
fn TableItem(table: crate::db::TableInfo) -> Element {
    let mut is_expanded = use_signal(|| false);
    let is_dark = *IS_DARK_MODE.read();

    let item_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let item_hover = if is_dark {
        "hover:bg-gray-900 hover:text-white"
    } else {
        "hover:bg-gray-100 hover:text-gray-900"
    };
    let icon_color = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-400"
    };
    let chevron_color = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };
    let col_name_color = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-700"
    };
    let col_muted = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };
    let row_estimate_color = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };
    let pk_color = if is_dark {
        "text-white"
    } else {
        "text-yellow-500"
    };

    // Clone table name for use in closures
    let table_name_for_context_menu = table.name.clone();
    let table_name_for_tracking = table.name.clone();
    let table_name_for_select = table.name.clone();

    rsx! {
        div {
            class: "space-y-1",

                button {
                    class: "w-full flex items-center space-x-2 px-2 py-1.5 rounded text-sm {item_text} {item_hover} text-left transition-colors",
                    onclick: move |_| {
                        let current = *is_expanded.read();
                        is_expanded.set(!current);
                        // Track table access
                        let store = RecentTablesStore::new();
                        let _ = store.add(&table_name_for_tracking);
                        // Update recent tables signal
                        let recent = store.load();
                        *RECENT_TABLES.write() = recent.into_iter().map(|e| e.table_name).collect();
                    },
                    oncontextmenu: move |e| {
                        e.prevent_default();
                        let coords = e.data.client_coordinates();
                        show_table_context_menu(table_name_for_context_menu.clone(), coords.x as i32, coords.y as i32);
                    },

                svg {
                    class: "w-3.5 h-3.5 {chevron_color} transition-transform",
                    style: if *is_expanded.read() { "transform: rotate(90deg)" } else { "" },
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M9 5l7 7-7 7",
                    }
                }

                svg {
                    class: "w-4 h-4 {icon_color}",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z",
                    }
                }

                span { "{table.name}" }

                if table.row_estimate > 0 {
                    span {
                        class: "text-xs {row_estimate_color} ml-auto",
                        "~{table.row_estimate}"
                    }
                }
            }

            if *is_expanded.read() {
                div {
                    class: "ml-6 space-y-0.5",

                    for col in &table.columns {
                        div {
                            class: "flex items-center space-x-2 px-2 py-1 text-xs",

                            if col.is_primary_key {
                                svg {
                                    class: "w-3 h-3 {pk_color}",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z",
                                    }
                                }
                            } else {
                                div { class: "w-3" }
                            }

                            span {
                                class: if col.nullable { "" } else { "font-medium {col_name_color}" },
                                "{col.name}"
                            }
                            span {
                                class: col_muted,
                                "{col.data_type}"
                            }
                        }
                    }

                    button {
                        class: "mt-2 px-2 py-1 text-xs {item_text} hover:text-blue-500 text-left transition-colors",
                        onclick: move |_| {
                            let sql = format!("SELECT * FROM \"{}\" LIMIT 100;", table_name_for_select);
                            *EDITOR_CONTENT.write() = sql;
                            // Track table access when generating SELECT
                            let store = RecentTablesStore::new();
                            let _ = store.add(&table_name_for_select);
                            let recent = store.load();
                            *RECENT_TABLES.write() = recent.into_iter().map(|e| e.table_name).collect();
                        },
                        "SELECT * FROM {table.name}"
                    }
                }
            }
        }
    }
}

#[component]
fn ViewItem(view: String) -> Element {
    let is_dark = *IS_DARK_MODE.read();
    let item_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let item_hover = if is_dark {
        "hover:bg-gray-900 hover:text-white"
    } else {
        "hover:bg-gray-100 hover:text-gray-900"
    };
    let icon_color = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-400"
    };

    rsx! {
            button {
                class: "w-full flex items-center space-x-2 px-2 py-1.5 rounded text-sm {item_text} {item_hover} text-left transition-colors",
                onclick: move |_| {
                    let sql = format!("SELECT * FROM \"{}\" LIMIT 100;", view);
                    *EDITOR_CONTENT.write() = sql;
                },

                svg {
                    class: "w-4 h-4 {icon_color}",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
                    }
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z",
                    }
                }

                span { "{view}" }
            }
    }
}
