use crate::state::{EDITOR_CONTENT, IS_DARK_MODE};
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

    let Some(state) = menu_state else {
        return rsx! {};
    };

    let is_dark = *IS_DARK_MODE.read();
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
                    "Table: {state.table_name}"
                }

                button {
                    class: "w-full text-left px-3 py-2 text-sm {text_class} {hover_class} transition-colors flex items-center space-x-2",
                    onclick: move |_| {
                        let sql = format!("SELECT * FROM \"{}\" LIMIT 100;", state.table_name);
                        *EDITOR_CONTENT.write() = sql;
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
                    span { "SELECT * FROM {state.table_name}" }
                }
            }
        }
    }
}
