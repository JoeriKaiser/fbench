use crate::export::{export_results, ExportFormat};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn MenuBar() -> Element {
    let is_dark = *IS_DARK_MODE.read();

    let bg_class = if is_dark { "bg-black" } else { "bg-white" };
    let border_class = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };
    let text_class = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let hover_class = if is_dark {
        "hover:text-white hover:bg-gray-900"
    } else {
        "hover:text-gray-900 hover:bg-gray-100"
    };
    let divider_class = if is_dark {
        "bg-gray-800"
    } else {
        "bg-gray-200"
    };

    rsx! {
        div {
            class: "h-10 {bg_class} border-b {border_class} flex items-center px-3 space-x-2",

            button {
                class: "px-3 py-1.5 text-sm {text_class} {hover_class} rounded flex items-center space-x-1.5 transition-colors",
                onclick: move |_| *SHOW_CONNECTION_DIALOG.write() = true,
                svg {
                    class: "w-4 h-4",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4",
                    }
                }
                span { "Connect" }
            }

            div { class: "w-px h-6 {divider_class} mx-2" }

            button {
                class: "px-3 py-1.5 text-sm {text_class} {hover_class} rounded flex items-center space-x-1.5 transition-colors",
                onclick: move |_| { *SHOW_SAVE_QUERY_DIALOG.write() = true; },
                svg {
                    class: "w-4 h-4",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-4 4m0 0l-4-4m4 4V4",
                    }
                }
                span { "Save Query" }
            }

            button {
                class: "px-3 py-1.5 text-sm {text_class} {hover_class} rounded flex items-center space-x-1.5 transition-colors",
                onclick: move |_| {
                    if let Some(ref result) = *QUERY_RESULT.read() {
                        export_results(result, ExportFormat::Csv);
                    }
                },
                svg {
                    class: "w-4 h-4",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12",
                    }
                }
                span { "Export" }
            }

            div { class: "flex-1" }

            ConnectionStatus {}
        }
    }
}

#[component]
fn ConnectionStatus() -> Element {
    let is_dark = *IS_DARK_MODE.read();
    let disconnected_bg = if is_dark {
        "bg-gray-700"
    } else {
        "bg-gray-400"
    };

    let (icon_class, text, color_class) = match *CONNECTION.read() {
        ConnectionState::Disconnected => (disconnected_bg, "Disconnected", "text-gray-500"),
        ConnectionState::Connecting => ("bg-yellow-500", "Connecting...", "text-yellow-500"),
        ConnectionState::Connected { db_type, .. } => {
            let db_name = match db_type {
                DatabaseType::PostgreSQL => "PostgreSQL",
                DatabaseType::MySQL => "MySQL",
            };
            ("bg-green-500", db_name, "text-green-500")
        }
        ConnectionState::ConnectionLost => ("bg-red-500", "Connection Lost", "text-red-500"),
        ConnectionState::Error(_) => ("bg-red-500", "Error", "text-red-500"),
    };

    rsx! {
        div {
            class: "flex items-center space-x-2",
            div {
                class: "w-2 h-2 rounded-full {icon_class}",
            }
            span {
                class: "text-xs font-medium {color_class}",
                "{text}"
            }
        }
    }
}
