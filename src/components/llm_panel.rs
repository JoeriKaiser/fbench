use crate::services::LlmSender;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn LlmPanel() -> Element {
    let llm_tx = use_context::<LlmSender>();
    let is_dark = *IS_DARK_MODE.read();
    let is_connected = matches!(*CONNECTION.read(), ConnectionState::Connected { .. });
    let is_generating = *LLM_GENERATING.read();
    let prompt = LLM_PROMPT.read().clone();
    let schema = SCHEMA.read().clone();

    let status = LLM_STATUS.read().clone();
    let status_visible = !matches!(status, LlmStatus::None);
    let (status_msg, status_is_error) = match status {
        LlmStatus::Success(msg) => (msg, false),
        LlmStatus::Error(msg) => (msg, true),
        LlmStatus::None => (String::new(), false),
    };

    let can_generate =
        is_connected && !is_generating && !prompt.trim().is_empty() && !schema.tables.is_empty();

    let bg_color = if is_dark { "bg-black" } else { "bg-gray-50" };
    let border_color = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };
    let input_bg = if is_dark { "bg-black" } else { "bg-white" };
    let input_border = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-300"
    };
    let input_text = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };
    let hint_color = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };

    // Generate callback - captures llm_tx by clone for FnMut
    let llm_tx_clone = llm_tx.clone();
    let on_generate = move |_| {
        if can_generate {
            let prompt_text = LLM_PROMPT.read().clone();
            let config = LLM_CONFIG.read().clone();
            let schema = SCHEMA.read().clone();

            *LLM_GENERATING.write() = true;
            *LLM_STATUS.write() = LlmStatus::None;

            let _ = llm_tx_clone.send(crate::llm::LlmRequest::Generate {
                prompt: prompt_text,
                schema,
                config,
            });
        }
    };

    // Key down callback - also clones llm_tx
    let llm_tx_clone2 = llm_tx.clone();
    let on_key_down = move |e: KeyboardEvent| {
        if e.key() == Key::Enter && !e.modifiers().contains(keyboard_types::Modifiers::CONTROL) {
            e.prevent_default();
            if can_generate {
                let prompt_text = LLM_PROMPT.read().clone();
                let config = LLM_CONFIG.read().clone();
                let schema = SCHEMA.read().clone();

                *LLM_GENERATING.write() = true;
                *LLM_STATUS.write() = LlmStatus::None;

                let _ = llm_tx_clone2.send(crate::llm::LlmRequest::Generate {
                    prompt: prompt_text,
                    schema,
                    config,
                });
            }
        }
    };

    let on_settings_click = move |_| {
        *SHOW_LLM_SETTINGS.write() = true;
    };

    rsx! {
        div {
            class: "px-4 py-3 {bg_color} border-b {border_color}",

            div {
                class: "flex items-center space-x-3",

                // Robot icon
                span {
                    class: "text-xl",
                    "🤖"
                }

                // Prompt input
                input {
                    class: "flex-1 px-3 py-2 text-sm rounded border {input_bg} {input_border} {input_text} focus:outline-none focus:ring-2 focus:ring-blue-500",
                    r#type: "text",
                    placeholder: "Describe the query you want...",
                    value: "{prompt}",
                    disabled: is_generating || !is_connected,
                    oninput: move |e| {
                        *LLM_PROMPT.write() = e.value().clone();
                    },
                    onkeydown: on_key_down,
                }

                // Generate button
                button {
                    class: if can_generate {
                        "px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-500 rounded transition-colors"
                    } else {
                        "px-4 py-2 text-sm font-medium text-gray-400 bg-gray-700 rounded cursor-not-allowed"
                    },
                    disabled: !can_generate,
                    onclick: on_generate,
                    "Generate"
                }

                // Settings button
                button {
                    class: "p-2 text-lg hover:bg-gray-700 rounded transition-colors",
                    onclick: on_settings_click,
                    title: "LLM Settings",
                    "⚙"
                }

                // Loading spinner
                if is_generating {
                    div {
                        class: "animate-spin h-5 w-5 border-2 border-blue-500 border-t-transparent rounded-full",
                    }
                }
            }

            // Status message
            if status_visible {
                div {
                    class: "mt-2 text-sm",
                    class: if status_is_error { "text-red-400" } else { "text-green-400" },
                    "{status_msg}"
                }
            }

            // Connection hint
            if !is_connected {
                div {
                    class: "mt-2 text-xs {hint_color}",
                    "Connect to a database to use AI query generation"
                }
            }
        }
    }
}
