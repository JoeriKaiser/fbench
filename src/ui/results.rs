use eframe::egui;
use crate::db::QueryResult;
use crate::export::{ExportFormat, export_results};

pub struct Results {
    pub result: Option<QueryResult>,
    pub error: Option<String>,
}

impl Default for Results {
    fn default() -> Self {
        Self {
            result: None,
            error: None,
        }
    }
}

impl Results {
    pub fn show(&self, ui: &mut egui::Ui) {
        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::RED, err);
            return;
        }

        let Some(result) = &self.result else {
            ui.label("No results");
            return;
        };

        ui.horizontal(|ui| {
            ui.label(format!("{} rows", result.rows.len()));
            ui.separator();
            
            ui.menu_button("📥 Export", |ui| {
                if ui.button("CSV").clicked() {
                    export_results(result, ExportFormat::Csv);
                    ui.close_menu();
                }
                if ui.button("JSON").clicked() {
                    export_results(result, ExportFormat::Json);
                    ui.close_menu();
                }
                if ui.button("XML").clicked() {
                    export_results(result, ExportFormat::Xml);
                    ui.close_menu();
                }
            });
        });

        ui.add_space(4.0);

        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("results_grid")
                .striped(true)
                .min_col_width(60.0)
                .show(ui, |ui| {
                    for col in &result.columns {
                        ui.strong(col);
                    }
                    ui.end_row();

                    for row in &result.rows {
                        for cell in row {
                            let label = if cell == "NULL" {
                                egui::RichText::new(cell).italics().color(egui::Color32::GRAY)
                            } else {
                                egui::RichText::new(cell).monospace()
                            };
                            ui.label(label);
                        }
                        ui.end_row();
                    }
                });
        });
    }
}
