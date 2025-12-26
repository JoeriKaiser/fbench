use eframe::egui::{self, Color32, RichText};
use tokio::sync::mpsc;

use crate::db::SchemaInfo;
use crate::llm::{LlmConfig, LlmProvider, LlmRequest};

pub struct AiPrompt {
    prompt: String,
    generating: bool,
    show_settings: bool,
    config: LlmConfig,
    status: Option<(String, bool)>,
}

impl Default for AiPrompt {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            generating: false,
            show_settings: false,
            config: LlmConfig::load(),
            status: None,
        }
    }
}

impl AiPrompt {
    pub fn set_generating(&mut self, val: bool) {
        self.generating = val;
    }

    pub fn set_error(&mut self, err: String) {
        self.generating = false;
        self.status = Some((err, true));
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        llm_tx: &mpsc::UnboundedSender<LlmRequest>,
        schema: &SchemaInfo,
        connected: bool,
    ) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🤖").size(16.0));

            let response = ui.add_sized(
                [ui.available_width() - 120.0, 24.0],
                egui::TextEdit::singleline(&mut self.prompt)
                    .hint_text("Describe the query you want...")
                    .interactive(!self.generating),
            );

            let can_generate = connected
                && !self.generating
                && !self.prompt.trim().is_empty()
                && !schema.tables.is_empty();

            let enter_pressed =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            let generate_clicked = ui
                .add_enabled(can_generate, egui::Button::new("Generate"))
                .clicked();

            if generate_clicked || (enter_pressed && can_generate) {
                self.generating = true;
                self.status = None;
                let _ = llm_tx.send(LlmRequest::Generate {
                    prompt: self.prompt.clone(),
                    schema: schema.clone(),
                    config: self.config.clone(),
                });
            }

            if ui.button("⚙").on_hover_text("LLM Settings").clicked() {
                self.show_settings = true;
            }

            if self.generating {
                ui.spinner();
            }
        });

        if let Some((msg, is_error)) = &self.status {
            let color = if *is_error {
                Color32::from_rgb(255, 100, 100)
            } else {
                Color32::from_rgb(100, 255, 100)
            };
            ui.colored_label(color, msg);
        }
    }

    pub fn take_prompt(&mut self) -> String {
        std::mem::take(&mut self.prompt)
    }

    pub fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.show_settings {
            return;
        }

        let mut open = self.show_settings;

        egui::Window::new("LLM Settings")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Provider:");
                    egui::ComboBox::from_id_salt("llm_provider")
                        .selected_text(match self.config.provider {
                            LlmProvider::Ollama => "Ollama",
                            LlmProvider::OpenRouter => "OpenRouter",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.config.provider,
                                LlmProvider::Ollama,
                                "Ollama",
                            );
                            ui.selectable_value(
                                &mut self.config.provider,
                                LlmProvider::OpenRouter,
                                "OpenRouter",
                            );
                        });
                });

                ui.add_space(8.0);

                match self.config.provider {
                    LlmProvider::Ollama => {
                        egui::Grid::new("ollama_settings")
                            .num_columns(2)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("URL:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.config.ollama_url)
                                        .desired_width(250.0),
                                );
                                ui.end_row();

                                ui.label("Model:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.config.ollama_model)
                                        .desired_width(250.0),
                                );
                                ui.end_row();
                            });
                    }
                    LlmProvider::OpenRouter => {
                        egui::Grid::new("openrouter_settings")
                            .num_columns(2)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("API Key:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.config.openrouter_key)
                                        .desired_width(250.0)
                                        .password(true),
                                );
                                ui.end_row();

                                ui.label("Model:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.config.openrouter_model)
                                        .desired_width(250.0),
                                );
                                ui.end_row();
                            });
                    }
                }

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        if let Err(e) = self.config.save() {
                            self.status = Some((format!("Failed to save: {}", e), true));
                        } else {
                            self.status = Some(("Settings saved".into(), false));
                            self.show_settings = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.config = LlmConfig::load();
                        self.show_settings = false;
                    }
                });
            });

        self.show_settings = open;
    }
}
