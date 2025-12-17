use eframe::egui;
use crate::config::{SavedQuery, QueryStore};

pub struct QueriesPanel {
    store: QueryStore,
    saved_queries: Vec<SavedQuery>,
    selected_index: Option<usize>,
    show_save_dialog: bool,
    save_name: String,
    status_message: Option<(String, bool)>,
}

impl Default for QueriesPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl QueriesPanel {
    pub fn new() -> Self {
        let store = QueryStore::new();
        let saved_queries = store.load_queries();
        Self {
            store,
            saved_queries,
            selected_index: None,
            show_save_dialog: false,
            save_name: String::new(),
            status_message: None,
        }
    }

    pub fn open_save_dialog(&mut self) {
        self.show_save_dialog = true;
        self.save_name.clear();
        self.status_message = None;
    }

    fn save_query(&mut self, sql: &str) {
        let name = self.save_name.trim();
        if name.is_empty() {
            self.status_message = Some(("Enter a name".into(), true));
            return;
        }

        let query = SavedQuery { name: name.to_string(), sql: sql.to_string() };

        if let Some(idx) = self.saved_queries.iter().position(|q| q.name == query.name) {
            self.saved_queries[idx] = query;
        } else {
            self.saved_queries.push(query);
        }

        match self.store.save_queries(&self.saved_queries) {
            Ok(()) => {
                self.show_save_dialog = false;
                self.save_name.clear();
            }
            Err(e) => self.status_message = Some((format!("Failed: {}", e), true)),
        }
    }

    fn delete_query(&mut self, idx: usize) {
        self.saved_queries.remove(idx);
        let _ = self.store.save_queries(&self.saved_queries);
        self.selected_index = None;
    }

    pub fn show_save_popup(&mut self, ctx: &egui::Context, current_sql: &str) {
        if !self.show_save_dialog { return; }

        egui::Window::new("Save Query")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.save_name)
                            .desired_width(200.0)
                            .hint_text("My query")
                    );
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.save_query(current_sql);
                    }
                });

                if let Some((msg, is_error)) = &self.status_message {
                    let color = if *is_error {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else {
                        egui::Color32::from_rgb(100, 255, 100)
                    };
                    ui.colored_label(color, msg);
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() { self.save_query(current_sql); }
                    if ui.button("Cancel").clicked() {
                        self.show_save_dialog = false;
                        self.status_message = None;
                    }
                });
            });
    }

    pub fn show_panel(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let mut result = None;

        ui.heading("Queries");
        ui.add_space(4.0);

        if self.saved_queries.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No saved queries");
            return None;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_delete = None;
            
            for (idx, query) in self.saved_queries.iter().enumerate() {
                let selected = self.selected_index == Some(idx);
                
                ui.horizontal(|ui| {
                    let response = ui.selectable_label(selected, &query.name);
                    
                    if response.clicked() {
                        self.selected_index = Some(idx);
                        result = Some(query.sql.clone());
                    }
                    
                    response.context_menu(|ui| {
                        if ui.button("🗑 Delete").clicked() {
                            to_delete = Some(idx);
                            ui.close_menu();
                        }
                    });
                });
            }

            if let Some(idx) = to_delete { self.delete_query(idx); }
        });

        result
    }
}
