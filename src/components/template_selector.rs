use crate::config::{get_builtin_templates, QueryTemplate};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn TemplateSelector() -> Element {
    let templates = get_builtin_templates();
    let is_dark = *IS_DARK_MODE.read();

    let bg_class = if is_dark { "bg-gray-900" } else { "bg-white" };
    let border_class = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-300"
    };
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

    rsx! {
        div {
            class: "relative",

            // Dropdown trigger button
            button {
                class: "px-3 py-1.5 text-sm rounded flex items-center space-x-1.5 transition-colors",
                class: if is_dark {
                    "bg-gray-900 hover:bg-gray-800 text-gray-300"
                } else {
                    "bg-gray-100 hover:bg-gray-200 text-gray-700"
                },
                onclick: move |_| {
                    // Toggle dropdown - would need visibility signal
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
                        d: "M4 6h16M4 12h16M4 18h16",
                    }
                }
                span { "Templates" }
            }

            // Template list (simplified - would need visibility toggle)
            div { class: "mt-2 space-y-1",
                for template in &templates {
                    {
                        let template = template.clone();
                        rsx! {
                            button {
                                class: "w-full text-left px-3 py-2 rounded text-sm transition-colors",
                                class: if is_dark { "hover:bg-gray-800 {text_class}" } else { "hover:bg-gray-100 {text_class}" },
                                onclick: move |_| {
                                    // Apply template with default values
                                    let values: Vec<(String, String)> = template.variables.iter()
                                        .map(|v| (v.name.clone(), v.default_value.clone().unwrap_or_default()))
                                        .collect();
                                    let sql = template.apply(&values);
                                    *EDITOR_CONTENT.write() = sql;
                                },

                                div { class: "font-medium", "{template.name}" }
                                div { class: "text-xs {muted_class}", "{template.description}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
