use crate::hooks::use_shiki::use_shiki;
use crate::services::DbSender;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn SqlEditor() -> Element {
    let shiki = use_shiki();
    let mut highlighted = use_signal(String::new);
    let is_dark = *IS_DARK_MODE.read();

    // Theme-aware classes
    let toolbar_bg = if is_dark { "bg-black" } else { "bg-gray-50" };
    let toolbar_border = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };
    let editor_bg = if is_dark { "bg-black" } else { "bg-white" };
    let hint_text = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };

    // Track both content changes AND shiki readiness
    use_effect(move || {
        // Read both signals to track them
        let code = EDITOR_CONTENT.read().clone();
        let is_ready = shiki.is_ready();
        let shiki = shiki;
        spawn(async move {
            // Only highlight if shiki is ready
            if is_ready {
                if let Some(html) = shiki.highlight(&code).await {
                    highlighted.set(html);
                }
            }
        });
    });

    rsx! {
        div {
            class: "flex flex-col h-full",

            div {
                class: "h-10 {toolbar_bg} border-b {toolbar_border} flex items-center px-3 space-x-3",

                button {
                    class: "px-3 py-1.5 text-sm rounded flex items-center space-x-1.5 transition-colors",
                    class: if is_dark { "bg-white hover:bg-gray-200 text-black" } else { "bg-blue-600 hover:bg-blue-500 text-white" },
                    onclick: move |_| execute_query(),
                    svg {
                        class: "w-3.5 h-3.5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z",
                        }
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                        }
                    }
                    span { "Run" }
                }

                button {
                    class: "px-3 py-1.5 text-sm rounded flex items-center space-x-1.5 transition-colors",
                    class: if is_dark {
                        "bg-gray-900 hover:bg-gray-800 text-gray-300"
                    } else {
                        "bg-gray-100 hover:bg-gray-200 text-gray-700"
                    },
                    onclick: move |_| execute_statement(),
                    svg {
                        class: "w-3.5 h-3.5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M13 10V3L4 14h7v7l9-11h-7z",
                        }
                    }
                    span { "Run Statement" }
                }

                div { class: "flex-1" }

                span {
                    class: "text-xs {hint_text}",
                    "Ctrl+Enter to run"
                }
            }

            div {
                class: "flex-1 relative overflow-hidden {editor_bg}",

                // Highlighted code layer (behind textarea)
                div {
                    class: "absolute inset-0 p-4 font-mono text-sm leading-6 overflow-auto pointer-events-none select-none",
                    dangerous_inner_html: "{highlighted}",
                }

                // Textarea for input (on top) - transparent text when Shiki ready
                textarea {
                    class: if shiki.is_ready() {
                        "absolute inset-0 w-full h-full p-4 bg-transparent text-transparent caret-blue-500 font-mono text-sm leading-6 resize-none focus:outline-none border-0"
                    } else {
                        "absolute inset-0 w-full h-full p-4 bg-transparent text-gray-700 caret-blue-500 font-mono text-sm leading-6 resize-none focus:outline-none border-0"
                    },
                    value: "{EDITOR_CONTENT}",
                    oninput: move |e| {
                        *EDITOR_CONTENT.write() = e.value().clone();
                    },
                    onkeydown: move |e| {
                        if e.data.key() == Key::Enter && e.data.modifiers().contains(keyboard_types::Modifiers::CONTROL) {
                            e.prevent_default();
                            execute_query();
                        }
                    },
                    spellcheck: "false",
                    placeholder: "Enter your SQL query here...",
                }
            }
        }
    }
}

fn execute_query() {
    let content = EDITOR_CONTENT.read();
    if let Some(tx) = try_use_context::<DbSender>() {
        let _ = tx.send(crate::db::DbRequest::Execute(content.clone()));
    }
}

fn execute_statement() {
    execute_query();
}
