use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn AiResultsPanel() -> Element {
    let ai_panel = AI_PANEL.read();
    let is_dark = *IS_DARK_MODE.read();

    if !ai_panel.visible {
        return rsx! {};
    }

    let bg_class = if is_dark {
        "bg-gray-900 border-gray-800"
    } else {
        "bg-gray-50 border-gray-200"
    };
    let text_class = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };
    let title_class = if is_dark {
        "text-gray-200"
    } else {
        "text-gray-800"
    };
    let code_bg = if is_dark {
        "bg-black border-gray-800"
    } else {
        "bg-white border-gray-200"
    };
    let secondary_button = if is_dark {
        "px-3 py-1.5 text-sm border border-gray-600 hover:bg-gray-800 text-gray-300 rounded transition-colors"
    } else {
        "px-3 py-1.5 text-sm border border-gray-300 hover:bg-gray-100 text-gray-700 rounded transition-colors"
    };

    let title = ai_panel.title.clone();
    let content = ai_panel.content.clone();
    let suggested_sql = ai_panel.suggested_sql.clone();
    let is_loading = ai_panel.loading;

    rsx! {
        div {
            class: "border-t {bg_class} p-4",

            div {
                class: "flex items-center justify-between mb-3",

                h3 {
                    class: "font-semibold {title_class}",
                    "{title}"
                }

                button {
                    class: "text-gray-500 hover:text-gray-700 transition-colors",
                    onclick: move |_| {
                        *AI_PANEL.write() = AiPanelState::default();
                    },
                    "✕"
                }
            }

            if is_loading {
                div {
                    class: "flex items-center space-x-2 {text_class}",
                    div {
                        class: "animate-spin h-4 w-4 border-2 border-blue-500 border-t-transparent rounded-full",
                    }
                    span { "Thinking..." }
                }
            } else {
                div {
                    class: "space-y-3",

                    p {
                        class: "text-sm {text_class}",
                        "{content}"
                    }

                    if let Some(sql) = suggested_sql {
                        {
                            let apply_sql = sql.clone();
                            let copy_sql = sql.clone();
                            rsx! {
                                div {
                                    class: "mt-3",

                                    div {
                                        class: "border rounded {code_bg} p-3 max-h-64 overflow-auto",

                                        pre {
                                            class: "text-sm font-mono text-blue-400 whitespace-pre min-w-max",
                                            "{sql}"
                                        }
                                    }

                                    div {
                                        class: "flex space-x-2 mt-3",

                                        button {
                                            class: "px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 text-white rounded transition-colors",
                                            onclick: move |_| {
                                                if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
                                                    tab.content = apply_sql.clone();
                                                    tab.unsaved_changes = true;
                                                }
                                                *AI_PANEL.write() = AiPanelState::default();
                                            },
                                            "Apply SQL"
                                        }

                                        button {
                                            class: secondary_button,
                                            onclick: move |_| copy_to_clipboard(&copy_sql),
                                            "Copy"
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

fn copy_to_clipboard(text: &str) {
    let escaped = text.replace('\\', "\\\\").replace('`', "\\`");
    let _ = document::eval(&format!(r#"navigator.clipboard.writeText(`{}`)"#, escaped));
}
