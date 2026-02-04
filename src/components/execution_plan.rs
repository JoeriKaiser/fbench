use crate::db::DbRequest;
use crate::services::DbSender;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn ExecutionPlanDialog() -> Element {
    let show = *SHOW_EXECUTION_PLAN.read();
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

    // Get execution plan from active tab
    let plan = {
        let tabs = EDITOR_TABS.read();
        tabs.active_tab()
            .and_then(|t| t.execution_plan.clone())
            .unwrap_or_else(|| "No execution plan available".to_string())
    };

    rsx! {
        div {
            class: "fixed inset-0 {bg_class} flex items-center justify-center z-50",
            onclick: move |_| *SHOW_EXECUTION_PLAN.write() = false,

            div {
                class: "{modal_bg} border {border_color} rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[80vh] flex flex-col",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    class: "flex items-center justify-between px-4 py-3 border-b {border_color}",

                    h3 {
                        class: "text-lg font-medium {text_color}",
                        "Execution Plan"
                    }

                    button {
                        class: "{text_color} hover:opacity-70",
                        onclick: move |_| *SHOW_EXECUTION_PLAN.write() = false,
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

                    pre {
                        class: "font-mono text-sm {text_color} whitespace-pre-wrap",
                        "{plan}"
                    }
                }

                // Footer
                div {
                    class: "flex items-center justify-end px-4 py-3 border-t {border_color}",

                    button {
                        class: "px-3 py-1.5 text-sm rounded bg-blue-600 hover:bg-blue-500 text-white",
                        onclick: move |_| *SHOW_EXECUTION_PLAN.write() = false,
                        "Close"
                    }
                }
            }
        }
    }
}

pub fn request_execution_plan() {
    let content = EDITOR_TABS
        .read()
        .active_tab()
        .map(|t| t.content.clone())
        .unwrap_or_default();

    if !content.is_empty() {
        if let Some(tx) = try_use_context::<DbSender>() {
            let _ = tx.send(DbRequest::Explain(content));
        }
    }
}
