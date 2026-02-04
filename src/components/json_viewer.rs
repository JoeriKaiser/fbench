use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn JsonViewer() -> Element {
    let show = *SHOW_JSON_VIEWER.read();
    let content = JSON_VIEWER_CONTENT.read().clone();
    let is_dark = *IS_DARK_MODE.read();

    if !show {
        return rsx! {};
    }

    let bg_class = if is_dark {
        "bg-black/80"
    } else {
        "bg-white/80"
    };
    let modal_bg = if is_dark { "bg-gray-900" } else { "bg-white" };
    let border_color = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-200"
    };
    let text_color = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };

    // Try to parse and pretty-print JSON
    let formatted = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        serde_json::to_string_pretty(&json).unwrap_or(content.clone())
    } else {
        content.clone()
    };

    let is_json = serde_json::from_str::<serde_json::Value>(&content).is_ok();

    rsx! {
        div {
            class: "fixed inset-0 {bg_class} flex items-center justify-center z-50",
            onclick: move |_| *SHOW_JSON_VIEWER.write() = false,

            div {
                class: "{modal_bg} border {border_color} rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[80vh] flex flex-col",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    class: "flex items-center justify-between px-4 py-3 border-b {border_color}",

                    h3 {
                        class: "text-lg font-medium {text_color}",
                        if is_json { "JSON Viewer" } else { "Cell Content" }
                    }

                    button {
                        class: "{text_color} hover:opacity-70",
                        onclick: move |_| *SHOW_JSON_VIEWER.write() = false,
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M6 18L18 6M6 6l12 12",
                            }
                        }
                    }
                }

                // Content
                div {
                    class: "flex-1 overflow-auto p-4",

                    if is_json {
                        pre {
                            class: "font-mono text-sm {text_color} whitespace-pre-wrap",
                            "{formatted}"
                        }
                    } else {
                        div {
                            class: "font-mono text-sm {text_color} whitespace-pre-wrap break-all",
                            "{content}"
                        }
                    }
                }

                // Footer
                div {
                    class: "flex items-center justify-end px-4 py-3 border-t {border_color} space-x-2",

                    button {
                        class: "px-3 py-1.5 text-sm rounded transition-colors",
                        class: if is_dark {
                            "bg-gray-800 hover:bg-gray-700 text-gray-300"
                        } else {
                            "bg-gray-100 hover:bg-gray-200 text-gray-700"
                        },
                        onclick: move |_| {
                            // Copy to clipboard
                            let _ = document::eval(&format!(
                                r#"navigator.clipboard.writeText(`{}`)"#,
                                formatted.replace('`', "\\`")
                            ));
                        },
                        "Copy"
                    }

                    button {
                        class: "px-3 py-1.5 text-sm rounded bg-blue-600 hover:bg-blue-500 text-white",
                        onclick: move |_| *SHOW_JSON_VIEWER.write() = false,
                        "Close"
                    }
                }
            }
        }
    }
}
