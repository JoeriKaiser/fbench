use crate::components::{HistoryPanel, QueriesPanel, SchemaPanel};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn Sidebar() -> Element {
    let is_dark = *IS_DARK_MODE.read();

    let bg_class = if is_dark { "bg-black" } else { "bg-white" };
    let border_class = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };

    rsx! {
        div {
            class: "w-64 {bg_class} border-r {border_class} flex flex-col",

            div {
                class: "h-10 {bg_class} border-b {border_class} flex",

                TabButton {
                    tab: LeftTab::Schema,
                    label: "Schema",
                    icon: "M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4",
                }
                TabButton {
                    tab: LeftTab::Queries,
                    label: "Queries",
                    icon: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
                }
                TabButton {
                    tab: LeftTab::History,
                    label: "History",
                    icon: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z",
                }
            }

            div {
                class: "flex-1 overflow-auto p-2",

                match *LEFT_TAB.read() {
                    LeftTab::Schema => rsx! { SchemaPanel {} },
                    LeftTab::Queries => rsx! { QueriesPanel {} },
                    LeftTab::History => rsx! { HistoryPanel {} },
                }
            }
        }
    }
}

#[component]
fn TabButton(tab: LeftTab, label: String, icon: String) -> Element {
    let is_active = *LEFT_TAB.read() == tab;
    let is_dark = *IS_DARK_MODE.read();

    let active_class = if is_active {
        if is_dark {
            "text-white border-b-2 border-white"
        } else {
            "text-gray-900 border-b-2 border-gray-900"
        }
    } else if is_dark {
        "text-gray-500 hover:text-gray-300 hover:bg-gray-900"
    } else {
        "text-gray-500 hover:text-gray-700 hover:bg-gray-100"
    };

    rsx! {
        button {
            class: "flex-1 flex items-center justify-center space-x-1.5 text-xs font-medium {active_class} transition-colors",
                onclick: move |_| *LEFT_TAB.write() = tab,
            svg {
                class: "w-3.5 h-3.5",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "{icon}",
                }
            }
            span { "{label}" }
        }
    }
}
