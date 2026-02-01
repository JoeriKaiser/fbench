use crate::llm::{LlmConfig, LlmProvider};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn LlmSettingsDialog() -> Element {
    let show = *SHOW_LLM_SETTINGS.read();

    if !show {
        return rsx! {};
    }

    let is_dark = *IS_DARK_MODE.read();

    // Use signals for form state so closures can update them
    let mut provider = use_signal(|| LLM_CONFIG.read().provider.clone());
    let mut ollama_url = use_signal(|| LLM_CONFIG.read().ollama_url.clone());
    let mut ollama_model = use_signal(|| LLM_CONFIG.read().ollama_model.clone());
    let mut openrouter_key = use_signal(|| LLM_CONFIG.read().openrouter_key.clone());
    let mut openrouter_model = use_signal(|| LLM_CONFIG.read().openrouter_model.clone());

    let overlay_bg = if is_dark {
        "bg-black bg-opacity-50"
    } else {
        "bg-black bg-opacity-30"
    };
    let dialog_bg = if is_dark { "bg-gray-900" } else { "bg-white" };
    let dialog_border = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-300"
    };
    let text_color = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };
    let input_bg = if is_dark { "bg-black" } else { "bg-gray-50" };
    let input_border = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-300"
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
        // Reset to saved values
        let saved = LlmConfig::load();
        provider.set(saved.provider);
        ollama_url.set(saved.ollama_url);
        ollama_model.set(saved.ollama_model);
        openrouter_key.set(saved.openrouter_key);
        openrouter_model.set(saved.openrouter_model);
        *SHOW_LLM_SETTINGS.write() = false;
    };

    let on_overlay_click = move |_| {
        *SHOW_LLM_SETTINGS.write() = false;
    };

    let on_dialog_click = move |e: MouseEvent| {
        e.stop_propagation();
    };

    let current_provider = provider.read().clone();

    rsx! {
        // Modal overlay
        div {
            class: "fixed inset-0 {overlay_bg} flex items-center justify-center z-50",
            onclick: on_overlay_click,

            // Dialog
            div {
                class: "{dialog_bg} border {dialog_border} rounded-lg shadow-xl p-6 w-full max-w-md",
                onclick: on_dialog_click,

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
                        class: "w-full px-3 py-2 text-sm border rounded {input_bg} {input_border} {text_color} focus:outline-none focus:ring-2 focus:ring-blue-500",
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

                        option { value: "ollama", "Ollama" }
                        option { value: "openrouter", "OpenRouter" }
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
                                value: "{ollama_url}",
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
                                value: "{ollama_model}",
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
                                value: "{openrouter_key}",
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
                                value: "{openrouter_model}",
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
    }
}
