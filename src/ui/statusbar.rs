use eframe::egui;
use crate::db::DatabaseType;

#[derive(Default)]
pub struct StatusBar {
    pub connected: bool,
    pub db_type: Option<DatabaseType>,
    pub db_name: String,
    pub row_count: Option<usize>,
    pub exec_time_ms: Option<u64>,
}

impl StatusBar {
    pub fn show(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let (text, color) = if self.connected {
                let icon = match self.db_type {
                    Some(DatabaseType::PostgreSQL) => "🐘",
                    Some(DatabaseType::MySQL) => "🐬",
                    None => "●",
                };
                let type_name = self.db_type.map(|t| t.display_name()).unwrap_or("Database");
                (format!("{} Connected to {} ({})", icon, self.db_name, type_name), egui::Color32::GREEN)
            } else {
                ("○ Disconnected".to_string(), egui::Color32::GRAY)
            };
            ui.colored_label(color, text);
            ui.separator();

            if let Some(count) = self.row_count {
                ui.label(format!("{} rows", count));
                ui.separator();
            }
            if let Some(ms) = self.exec_time_ms {
                ui.label(format!("{}ms", ms));
            }
        });
    }
}
