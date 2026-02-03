use crate::config::{QueryHistory, QueryStore, RecentTablesStore};
use crate::state::*;
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
enum SwitcherItem {
    Table { name: String },
    Query { name: String, sql: String },
    History { sql: String, timestamp: String },
}

#[component]
pub fn QuickSwitcher() -> Element {
    let mut search_query = use_signal(|| String::new());
    let mut selected_index = use_signal(|| 0usize);
    let is_dark = *IS_DARK_MODE.read();
    let is_visible = *SHOW_QUICK_SWITCHER.read();

    // Build item list based on search
    let items = use_memo(move || {
        let query = search_query.read().to_lowercase();
        let mut items = Vec::new();

        // Add recent tables
        let recent_store = RecentTablesStore::new();
        for entry in recent_store.load() {
            if query.is_empty() || entry.table_name.to_lowercase().contains(&query) {
                items.push(SwitcherItem::Table {
                    name: entry.table_name,
                });
            }
        }

        // Add saved queries
        let query_store = QueryStore::new();
        for saved in query_store.load_queries() {
            if query.is_empty() || saved.name.to_lowercase().contains(&query) {
                items.push(SwitcherItem::Query {
                    name: saved.name,
                    sql: saved.sql,
                });
            }
        }

        // Add history
        let history = QueryHistory::new();
        for entry in history.get_entries() {
            if query.is_empty() || entry.sql.to_lowercase().contains(&query) {
                items.push(SwitcherItem::History {
                    sql: entry.sql.clone(),
                    timestamp: entry.executed_at.format("%H:%M").to_string(),
                });
            }
        }

        items.truncate(20); // Limit results
        items
    });

    // Reset selection when search changes
    use_effect(move || {
        let _ = search_query.read();
        selected_index.set(0);
    });

    // Handle keyboard navigation
    let handle_keydown = move |e: KeyboardEvent| match e.key() {
        Key::Escape => {
            *SHOW_QUICK_SWITCHER.write() = false;
        }
        Key::ArrowDown => {
            let len = items.read().len();
            if len > 0 {
                let current = *selected_index.read();
                selected_index.set((current + 1) % len);
            }
        }
        Key::ArrowUp => {
            let len = items.read().len();
            if len > 0 {
                let current = *selected_index.read();
                selected_index.set(if current == 0 { len - 1 } else { current - 1 });
            }
        }
        Key::Enter => {
            let idx = *selected_index.read();
            if let Some(item) = items.read().get(idx) {
                match item {
                    SwitcherItem::Table { name } => {
                        let sql = format!("SELECT * FROM \"{}\" LIMIT 100;", name);
                        *EDITOR_CONTENT.write() = sql;
                    }
                    SwitcherItem::Query { sql, .. } => {
                        *EDITOR_CONTENT.write() = sql.clone();
                    }
                    SwitcherItem::History { sql, .. } => {
                        *EDITOR_CONTENT.write() = sql.clone();
                    }
                }
                *SHOW_QUICK_SWITCHER.write() = false;
                search_query.set(String::new());
            }
        }
        _ => {}
    };

    if !is_visible {
        return rsx! {};
    }

    let bg_class = if is_dark { "bg-gray-900" } else { "bg-white" };
    let border_class = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-300"
    };
    let input_bg = if is_dark { "bg-black" } else { "bg-gray-50" };
    let text_class = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };
    let muted_class = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-400"
    };
    let selected_bg = if is_dark {
        "bg-blue-900"
    } else {
        "bg-blue-100"
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-start justify-center pt-32",
            onclick: move |_| *SHOW_QUICK_SWITCHER.write() = false,

            // Backdrop
            div { class: "absolute inset-0 bg-black bg-opacity-50" }

            // Modal
            div {
                class: "relative w-full max-w-2xl {bg_class} rounded-lg shadow-2xl border {border_class} overflow-hidden",
                onclick: move |e| e.stop_propagation(),

                // Search input
                div { class: "p-4 border-b {border_class}",
                    input {
                        class: "w-full px-4 py-3 text-lg rounded border {input_bg} {border_class} {text_class} focus:outline-none focus:ring-2 focus:ring-blue-500",
                        placeholder: "Search tables, queries, history...",
                        value: "{search_query}",
                        autofocus: true,
                        oninput: move |e| search_query.set(e.value().clone()),
                        onkeydown: handle_keydown,
                    }
                }

                // Results list
                div { class: "max-h-96 overflow-auto",
                    if items.read().is_empty() {
                        div { class: "p-8 text-center {muted_class}",
                            "No results found"
                        }
                    } else {
                        for (idx, item) in items.read().iter().enumerate() {
                            {
                                let is_selected = idx == *selected_index.read();
                                let item = item.clone();
                                rsx! {
                                    button {
                                        class: "w-full px-4 py-3 text-left flex items-center space-x-3 transition-colors",
                                        class: if is_selected { "{selected_bg}" } else { "hover:bg-gray-800" },
                                        onclick: move |_| {
                                            match &item {
                                                SwitcherItem::Table { name } => {
                                                    let sql = format!("SELECT * FROM \"{}\" LIMIT 100;", name);
                                                    *EDITOR_CONTENT.write() = sql;
                                                }
                                                SwitcherItem::Query { sql, .. } => {
                                                    *EDITOR_CONTENT.write() = sql.clone();
                                                }
                                                SwitcherItem::History { sql, .. } => {
                                                    *EDITOR_CONTENT.write() = sql.clone();
                                                }
                                            }
                                            *SHOW_QUICK_SWITCHER.write() = false;
                                            search_query.set(String::new());
                                        },

                                        // Icon based on type
                                        match &item {
                                            SwitcherItem::Table { .. } => rsx! {
                                                svg {
                                                    class: "w-5 h-5 {muted_class}",
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
                                            },
                                            SwitcherItem::Query { .. } => rsx! {
                                                svg {
                                                    class: "w-5 h-5 {muted_class}",
                                                    fill: "none",
                                                    stroke: "currentColor",
                                                    view_box: "0 0 24 24",
                                                    path {
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        stroke_width: "2",
                                                        d: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
                                                    }
                                                }
                                            },
                                            SwitcherItem::History { .. } => rsx! {
                                                svg {
                                                    class: "w-5 h-5 {muted_class}",
                                                    fill: "none",
                                                    stroke: "currentColor",
                                                    view_box: "0 0 24 24",
                                                    path {
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        stroke_width: "2",
                                                        d: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z",
                                                    }
                                                }
                                            },
                                        }

                                        // Content
                                        div { class: "flex-1 min-w-0",
                                            match &item {
                                                SwitcherItem::Table { name } => rsx! {
                                                    span { class: "{text_class} truncate", "{name}" }
                                                },
                                                SwitcherItem::Query { name, sql } => rsx! {
                                                    div {
                                                        span { class: "{text_class} font-medium", "{name}" }
                                                        span { class: "{muted_class} text-sm ml-2 truncate", "{sql}" }
                                                    }
                                                },
                                                SwitcherItem::History { sql, timestamp } => rsx! {
                                                    div { class: "flex items-center justify-between",
                                                        span { class: "{text_class} truncate flex-1", "{sql}" }
                                                        span { class: "{muted_class} text-xs", "{timestamp}" }
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Footer hints
                div { class: "px-4 py-2 border-t {border_class} {muted_class} text-xs flex items-center space-x-4",
                    span { "↑↓ to navigate" }
                    span { "Enter to select" }
                    span { "Esc to close" }
                }
            }
        }
    }
}
