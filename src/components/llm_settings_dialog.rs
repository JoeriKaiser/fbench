use crate::llm::{LlmConfig, LlmProvider};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn LlmSettingsDialog() -> Element {
    rsx! {
        if *SHOW_LLM_SETTINGS.read() {
            div {
                class: if *IS_DARK_MODE.read() {
                    "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
                } else {
                    "fixed inset-0 bg-black bg-opacity-30 flex items-center justify-center z-50"
                },
                onclick: move |_| {
                    *SHOW_LLM_SETTINGS.write() = false;
                },

                div {
                    class: if *IS_DARK_MODE.read() {
                        "bg-gray-900 border border-gray-700 rounded-lg shadow-xl p-6 w-full max-w-md"
                    } else {
                        "bg-white border border-gray-300 rounded-lg shadow-xl p-6 w-full max-w-md"
                    },
                    onclick: move |e| e.stop_propagation(),

                    LlmSettingsDialogContent {}
                }
            }
        }
    }
}

#[component]
fn LlmSettingsDialogContent() -> Element {
    let is_dark = *IS_DARK_MODE.read();

    // Mount this content only while the dialog is open so each open starts from
    // the current persisted/in-memory config instead of stale unsaved form state.
    let mut provider = use_signal(|| LLM_CONFIG.read().provider.clone());
    let mut ollama_url = use_signal(|| LLM_CONFIG.read().ollama_url.clone());
    let mut ollama_model = use_signal(|| LLM_CONFIG.read().ollama_model.clone());
    let mut openrouter_key = use_signal(|| LLM_CONFIG.read().openrouter_key.clone());
    let mut openrouter_model = use_signal(|| LLM_CONFIG.read().openrouter_model.clone());

    let text_color = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };
    let input_bg = if is_dark { "bg-black" } else { "bg-white" };
    let input_border = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-300"
    };
    let select_class = if is_dark {
        "bg-black border-gray-800 text-white focus:border-white appearance-none"
    } else {
        "bg-white border-gray-300 text-gray-900 focus:border-blue-500 appearance-none"
    };

    let on_save = move |_| {
        let new_config = LlmConfig {
            provider: provider.read().clone(),
            ollama_url: ollama_url.read().clone(),
            ollama_model: ollama_model.read().clone(),
            openrouter_key: openrouter_key.read().clone(),
            openrouter_model: openrouter_model.read().clone(),
        };

        if let Err(e) = new_config.save() {
            *LLM_STATUS.write() = LlmStatus::Error(format!("Failed to save: {}", e));
        } else {
            *LLM_CONFIG.write() = new_config;
            *LLM_STATUS.write() = LlmStatus::Success("Settings saved".into());
            *SHOW_LLM_SETTINGS.write() = false;
        }
    };

    let on_cancel = move |_| {
        *SHOW_LLM_SETTINGS.write() = false;
    };

    let current_provider = provider.read().clone();
    let ollama_url_value = ollama_url.read().clone();
    let ollama_model_value = ollama_model.read().clone();
    let openrouter_key_value = openrouter_key.read().clone();
    let openrouter_model_value = openrouter_model.read().clone();

    rsx! {
        h2 {
            class: "text-lg font-semibold {text_color} mb-4",
            "LLM Settings"
        }

        // Provider selection
        div {
            class: "mb-4",

            label {
                class: "block text-sm font-medium {text_color} mb-2",
                "Provider"
            }

            select {
                class: "w-full px-3 py-2 text-sm border rounded focus:outline-none focus:ring-2 focus:ring-blue-500 {select_class}",
                value: match current_provider {
                    LlmProvider::Ollama => "ollama",
                    LlmProvider::OpenRouter => "openrouter",
                },
                onchange: move |e| {
                    let new_provider = match e.value().as_str() {
                        "openrouter" => LlmProvider::OpenRouter,
                        _ => LlmProvider::Ollama,
                    };
                    provider.set(new_provider);
                },

                option {
                    class: if is_dark { "bg-black text-white" } else { "bg-white text-gray-900" },
                    value: "ollama",
                    "Ollama"
                }
                option {
                    class: if is_dark { "bg-black text-white" } else { "bg-white text-gray-900" },
                    value: "openrouter",
                    "OpenRouter"
                }
            }
        }

        // Provider-specific settings
        match current_provider {
            LlmProvider::Ollama => rsx! {
                // Ollama URL
                div {
                    class: "mb-4",

                    label {
                        class: "block text-sm font-medium {text_color} mb-2",
                        "URL"
                    }

                    input {
                        class: "w-full px-3 py-2 text-sm border rounded {input_bg} {input_border} {text_color} focus:outline-none focus:ring-2 focus:ring-blue-500",
                        r#type: "text",
                        value: "{ollama_url_value}",
                        oninput: move |e| {
                            ollama_url.set(e.value().clone());
                        },
                    }
                }

                // Ollama Model
                div {
                    class: "mb-4",

                    label {
                        class: "block text-sm font-medium {text_color} mb-2",
                        "Model"
                    }

                    input {
                        class: "w-full px-3 py-2 text-sm border rounded {input_bg} {input_border} {text_color} focus:outline-none focus:ring-2 focus:ring-blue-500",
                        r#type: "text",
                        value: "{ollama_model_value}",
                        oninput: move |e| {
                            ollama_model.set(e.value().clone());
                        },
                    }
                }
            },
            LlmProvider::OpenRouter => rsx! {
                // OpenRouter API Key
                div {
                    class: "mb-4",

                    label {
                        class: "block text-sm font-medium {text_color} mb-2",
                        "API Key"
                    }

                    input {
                        class: "w-full px-3 py-2 text-sm border rounded {input_bg} {input_border} {text_color} focus:outline-none focus:ring-2 focus:ring-blue-500",
                        r#type: "password",
                        value: "{openrouter_key_value}",
                        oninput: move |e| {
                            openrouter_key.set(e.value().clone());
                        },
                    }
                }

                // OpenRouter Model
                div {
                    class: "mb-4",

                    label {
                        class: "block text-sm font-medium {text_color} mb-2",
                        "Model"
                    }

                    input {
                        class: "w-full px-3 py-2 text-sm border rounded {input_bg} {input_border} {text_color} focus:outline-none focus:ring-2 focus:ring-blue-500",
                        r#type: "text",
                        value: "{openrouter_model_value}",
                        oninput: move |e| {
                            openrouter_model.set(e.value().clone());
                        },
                    }
                }
            },
        }

        // Buttons
        div {
            class: "flex justify-end space-x-3 mt-6",

            button {
                class: "px-4 py-2 text-sm font-medium text-gray-400 hover:text-gray-300 transition-colors",
                onclick: on_cancel,
                "Cancel"
            }

            button {
                class: "px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-500 rounded transition-colors",
                onclick: on_save,
                "Save"
            }
        }
    }
}
