use eframe::egui;

#[derive(Default)]
pub struct StatusBar {
    pub connected: bool,
    pub db_name: String,
    pub row_count: Option<usize>,
    pub exec_time_ms: Option<u64>,
}

impl StatusBar {
    pub fn show(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Connection status
            let (status_text, color) = if self.connected {
                (format!("● Connected to {}", self.db_name), egui::Color32::GREEN)
            } else {
                ("○ Disconnected".to_string(), egui::Color32::GRAY)
            };
            ui.colored_label(color, status_text);

            ui.separator();

            // Row count
            if let Some(count) = self.row_count {
                ui.label(format!("{} rows", count));
                ui.separator();
            }

            // Execution time
            if let Some(ms) = self.exec_time_ms {
                ui.label(format!("{}ms", ms));
            }
        });
    }
}
