use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn ResultsTable() -> Element {
    let result = QUERY_RESULT.read();
    let error = LAST_ERROR.read();
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

                if let Some(error) = error.as_ref() {
                    span { class: "text-red-500 text-sm", "{error}" }
                } else if let Some(result) = result.as_ref() {
                    span { class: "{header_text} text-sm", "{result.rows.len()} rows" }
                } else {
                    span { class: "{muted_text} text-sm", "No results" }
                }

                if let Some(exec_time) = EXECUTION_TIME_MS.read().as_ref() {
                    span {
                        class: "text-xs {muted_text}",
                        "{exec_time}ms"
                    }
                }
            }

            div {
                class: "flex-1 overflow-auto",

                if let Some(result) = result.as_ref() {
                    table {
                        class: "w-full text-sm text-left",

                        thead {
                            class: "{header_bg} {header_text} sticky top-0",
                            tr {
                                for col in &result.columns {
                                    th {
                                        class: "px-4 py-2 font-medium border-b {header_border}",
                                        "{col}"
                                    }
                                }
                            }
                        }

                        tbody {
                            class: "{table_divider}",
                            for (idx, row) in result.rows.iter().enumerate() {
                                tr {
                                    class: if idx % 2 == 0 { "" } else { row_alt },
                                    for cell in row {
                                        td {
                                            class: "px-4 py-2 {cell_text} font-mono",
                                            "{cell}"
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
