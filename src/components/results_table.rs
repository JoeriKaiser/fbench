use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn ResultsTable() -> Element {
    let tabs = EDITOR_TABS.read();
    let active_tab = tabs.active_tab();
    let result = active_tab.and_then(|t| t.result.clone());
    let error = active_tab.and_then(|t| t.last_error.clone());
    let exec_time = active_tab.and_then(|t| t.execution_time_ms);
    let is_dark = *IS_DARK_MODE.read();

    // Theme-aware classes
    let header_bg = if is_dark { "bg-black" } else { "bg-gray-50" };
    let header_border = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };
    let header_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let row_alt = if is_dark { "bg-gray-950" } else { "bg-gray-50" };
    let cell_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-700"
    };
    let muted_text = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };
    let table_divider = if is_dark {
        "divide-gray-800"
    } else {
        "divide-gray-200"
    };

    rsx! {
        div {
            class: "flex flex-col h-full",

            div {
                class: "h-8 {header_bg} border-b {header_border} flex items-center px-3 justify-between",

                if let Some(error) = error {
                    span { class: "text-red-500 text-sm", "{error}" }
                } else if let Some(ref result) = result {
                    span { class: "{header_text} text-sm", "{result.rows.len()} rows" }
                } else {
                    span { class: "{muted_text} text-sm", "No results" }
                }

                div {
                    class: "flex items-center space-x-3",

                    if let Some(exec_time) = exec_time {
                        span {
                            class: "text-xs {muted_text}",
                            "{exec_time}ms"
                        }
                    }

                    // Explain button (only when we have results)
                    if result.is_some() {
                        button {
                            class: "text-xs px-2 py-1 rounded {header_bg} {header_text} hover:opacity-80",
                            onclick: move |_| show_execution_plan(),
                            "Explain"
                        }
                    }
                }
            }

            div {
                class: "flex-1 overflow-auto",

                if let Some(result) = result {
                    table {
                        class: "w-full text-sm text-left",

                        thead {
                            class: "{header_bg} {header_text} sticky top-0",
                            tr {
                                for col in result.columns.clone() {
                                    th {
                                        class: "px-4 py-2 font-medium border-b {header_border}",
                                        "{col}"
                                    }
                                }
                            }
                        }

                        tbody {
                            class: "{table_divider}",
                            for (idx, row) in result.rows.clone().into_iter().enumerate() {
                                tr {
                                    class: if idx % 2 == 0 { "" } else { row_alt },
                                    for cell in row.clone() {
                                        td {
                                            class: "px-4 py-2 {cell_text} font-mono cursor-pointer hover:bg-blue-500/10",
                                            onclick: move |_| view_cell_content(cell.clone()),
                                            title: "Click to view full content",
                                            {
                                                let display = if cell.len() > 100 {
                                                    format!("{}...", &cell[..100])
                                                } else {
                                                    cell.clone()
                                                };
                                                "{display}"
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

fn show_execution_plan() {
    // TODO: Show execution plan modal
    // For now, just log
    println!("Show execution plan");
}

fn view_cell_content(content: String) {
    // TODO: Open modal with full content
    // Check if JSON and pretty-print
    println!("View cell: {}", content);
}
