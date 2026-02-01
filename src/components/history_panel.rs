use crate::config::{HistoryEntry, QueryHistory};
use crate::state::{EDITOR_CONTENT, HISTORY_REVISION, IS_DARK_MODE};
use dioxus::prelude::*;

#[component]
pub fn HistoryPanel() -> Element {
    let mut entries: Signal<Vec<HistoryEntry>> = use_signal(Vec::new);
    let is_dark = *IS_DARK_MODE.read();

    // Reload history when HISTORY_REVISION changes (indicating new query executed)
    use_effect(move || {
        let _revision = *HISTORY_REVISION.read();
        let history = QueryHistory::new();
        entries.set(history.get_entries().to_vec());
    });

    // Theme-aware classes
    let header_text = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-500"
    };
    let clear_text = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-400"
    };
    let clear_hover = if is_dark {
        "hover:text-white"
    } else {
        "hover:text-red-600"
    };
    let muted_text = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };
    let item_hover = if is_dark {
        "hover:bg-gray-900"
    } else {
        "hover:bg-gray-100"
    };
    let sql_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };

    rsx! {
        div {
            class: "space-y-2",

            div {
                class: "flex items-center justify-between mb-3",
                h3 {
                    class: "text-xs font-semibold {header_text} uppercase tracking-wider",
                    "Query History"
                }

                if !entries.read().is_empty() {
                    button {
                        class: "text-xs {clear_text} {clear_hover} transition-colors",
                        onclick: move |_| {
                            let mut history = QueryHistory::new();
                            history.clear();
                            entries.set(history.get_entries().to_vec());
                        },
                        "Clear All"
                    }
                }
            }

            if entries.read().is_empty() {
                div {
                    class: "{muted_text} text-sm text-center py-8",
                    "No query history"
                }
            } else {
                div {
                    class: "space-y-1",

                    for entry in (*entries.read()).iter() {
                        {
                            let entry_sql = entry.sql.clone();
                            let entry_time = entry.executed_at.format("%H:%M").to_string();
                            let entry_row_count = entry.row_count;
                            let entry_exec_time = entry.execution_time_ms;
                            rsx! {
                                button {
                                    class: "w-full text-left px-2 py-2 rounded {item_hover} group transition-colors",
                                    onclick: move |_| {
                                        *EDITOR_CONTENT.write() = entry_sql.clone();
                                    },

                                    div {
                                        class: "flex items-center justify-between",

                                        span {
                                            class: "text-xs {sql_text} truncate flex-1 mr-2",
                                            "{entry_sql}"
                                        }

                                        span {
                                            class: "text-xs {muted_text} whitespace-nowrap",
                                            "{entry_time}"
                                        }
                                    }

                                    div {
                                        class: "flex items-center space-x-2 mt-1",

                                        if let Some(count) = entry_row_count {
                                            span {
                                                class: "text-xs {muted_text}",
                                                "{count} rows"
                                            }
                                        }

                                        if let Some(time) = entry_exec_time {
                                            span {
                                                class: "text-xs {muted_text}",
                                                "{time}ms"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
