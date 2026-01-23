use eframe::egui::{self, Color32, RichText};
use crate::db::{SchemaInfo, TableInfo};
use crate::llm::{LlmResponse, QuerySuggestion};

#[derive(Clone, PartialEq, Default)]
pub enum SchemaSelection {
    #[default]
    None,
    Table(String),
    View(String),
}

#[derive(Default)]
pub struct SchemaPanel {
    pub schema: SchemaInfo,
    expanded_tables: std::collections::HashSet<String>,
    tables_expanded: bool,
    views_expanded: bool,
    pub selection: SchemaSelection,
    search_query: String,
    // AI suggestions state
    suggestions: Vec<QuerySuggestion>,
    suggestions_loading: bool,
    suggestions_table: Option<String>,
}

#[derive(Default)]
pub struct SchemaPanelAction {
    pub select_table_data: Option<String>,
    pub view_table_structure: Option<String>,
    pub request_suggestions: Option<TableInfo>,
    pub apply_suggestion: Option<String>,
}

impl SchemaPanel {
    pub fn set_schema(&mut self, schema: SchemaInfo) {
        self.schema = schema;
    }

    pub fn get_selected_table(&self) -> Option<&str> {
        match &self.selection {
            SchemaSelection::Table(name) => Some(name),
            _ => None,
        }
    }

    pub fn handle_llm_response(&mut self, response: &LlmResponse) {
        match response {
            LlmResponse::QuerySuggestions(suggestions) => {
                self.suggestions = suggestions.clone();
                self.suggestions_loading = false;
            }
            LlmResponse::Error(_) => {
                self.suggestions_loading = false;
                // Keep old suggestions on error
            }
            _ => {}
        }
    }

    fn filter_matches(&self, name: &str) -> bool {
        self.search_query.is_empty() 
            || name.to_lowercase().contains(&self.search_query.to_lowercase())
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> SchemaPanelAction {
        let mut action = SchemaPanelAction::default();

        ui.horizontal(|ui| {
            ui.label("⌕");
            ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Filter...")
                    .desired_width(ui.available_width() - 8.0)
            );
        });
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let tables_header = egui::CollapsingHeader::new(
                    RichText::new(format!("📁 Tables ({})", self.schema.tables.len())).strong()
                )
                .default_open(self.tables_expanded)
                .show(ui, |ui| self.show_tables(ui, &mut action));
                self.tables_expanded = tables_header.fully_open();

                ui.add_space(4.0);

                let views_header = egui::CollapsingHeader::new(
                    RichText::new(format!("◎ Views ({})", self.schema.views.len())).strong()
                )
                .default_open(self.views_expanded)
                .show(ui, |ui| self.show_views(ui, &mut action));
                self.views_expanded = views_header.fully_open();
            });

        self.show_suggestions(ui, &mut action);

        action
    }

    fn show_suggestions(&mut self, ui: &mut egui::Ui, action: &mut SchemaPanelAction) {
        let selected_table = match &self.selection {
            SchemaSelection::Table(name) => Some(name.clone()),
            _ => None,
        };

        // Request suggestions when table changes
        if let Some(table_name) = &selected_table {
            if self.suggestions_table.as_ref() != Some(table_name) {
                self.suggestions_table = Some(table_name.clone());
                self.suggestions_loading = true;
                self.suggestions.clear();

                if let Some(table) = self.schema.tables.iter().find(|t| &t.name == table_name) {
                    action.request_suggestions = Some(table.clone());
                }
            }
        }

        if selected_table.is_none() {
            return;
        }

        ui.add_space(8.0);
        ui.separator();

        ui.horizontal(|ui| {
            ui.strong("Suggested Queries");
            if self.suggestions_loading {
                ui.spinner();
            } else if ui.small_button("↻").on_hover_text("Refresh suggestions").clicked() {
                if let Some(table_name) = &self.suggestions_table {
                    if let Some(table) = self.schema.tables.iter().find(|t| &t.name == table_name) {
                        self.suggestions_loading = true;
                        action.request_suggestions = Some(table.clone());
                    }
                }
            }
        });

        ui.add_space(4.0);

        if self.suggestions.is_empty() && !self.suggestions_loading {
            ui.colored_label(Color32::GRAY, "No suggestions available");
        } else {
            for suggestion in &self.suggestions {
                let response = ui.selectable_label(false, format!("▸ {}", suggestion.label));
                if response.clicked() {
                    action.apply_suggestion = Some(suggestion.sql.clone());
                }
                response.on_hover_text(&suggestion.sql);
            }
        }
    }

    fn show_tables(&mut self, ui: &mut egui::Ui, action: &mut SchemaPanelAction) {
        let filtered: Vec<_> = self.schema.tables.iter()
            .filter(|t| self.filter_matches(&t.name))
            .collect();

        if filtered.is_empty() {
            ui.colored_label(Color32::GRAY, "No tables found");
            return;
        }

        for table in filtered {
            let is_selected = self.selection == SchemaSelection::Table(table.name.clone());
            let is_expanded = self.expanded_tables.contains(&table.name);
            
            ui.horizontal(|ui| {
                if ui.small_button(if is_expanded { "▼" } else { "▶" }).clicked() {
                    if is_expanded {
                        self.expanded_tables.remove(&table.name);
                    } else {
                        self.expanded_tables.insert(table.name.clone());
                    }
                }

                let row_label = if is_selected {
                    RichText::new(format!("▪ {} ", table.name))
                        .strong()
                        .color(ui.visuals().selection.stroke.color)
                } else {
                    RichText::new(format!("▪ {} ", table.name))
                };
                
                let response = ui.selectable_label(is_selected, row_label);
                
                if response.clicked() {
                    self.selection = SchemaSelection::Table(table.name.clone());
                    action.select_table_data = Some(table.name.clone());
                }

                if table.row_estimate > 0 {
                    ui.colored_label(Color32::GRAY, format!("~{}", format_row_count(table.row_estimate)));
                }

                response.context_menu(|ui| {
                    if ui.button("▸ Select Data").clicked() {
                        action.select_table_data = Some(table.name.clone());
                        ui.close_menu();
                    }
                    if ui.button("⚙ View Structure").clicked() {
                        action.view_table_structure = Some(table.name.clone());
                        ui.close_menu();
                    }
                });
            });

            if is_expanded {
                ui.indent(ui.make_persistent_id(&table.name), |ui| {
                    for col in &table.columns {
                        let icon = if col.is_primary_key { "★" } else { "  " };
                        let nullable = if col.nullable { "?" } else { "" };
                        ui.colored_label(
                            Color32::GRAY,
                            format!("{} {}{}: {}", icon, col.name, nullable, col.data_type)
                        );
                    }
                });
            }
        }
    }

    fn show_views(&mut self, ui: &mut egui::Ui, action: &mut SchemaPanelAction) {
        let filtered: Vec<_> = self.schema.views.iter()
            .filter(|v| self.filter_matches(v))
            .collect();

        if filtered.is_empty() {
            ui.colored_label(Color32::GRAY, "No views found");
            return;
        }

        for view in filtered {
            let is_selected = self.selection == SchemaSelection::View(view.clone());
            
            let row_label = if is_selected {
                RichText::new(format!("◎ {}", view))
                    .strong()
                    .color(ui.visuals().selection.stroke.color)
            } else {
                RichText::new(format!("◎ {}", view))
            };

            let response = ui.selectable_label(is_selected, row_label);
            
            if response.clicked() {
                self.selection = SchemaSelection::View(view.clone());
                action.select_table_data = Some(view.clone());
            }

            response.context_menu(|ui| {
                if ui.button("▸ Select Data").clicked() {
                    action.select_table_data = Some(view.clone());
                    ui.close_menu();
                }
            });
        }
    }
}

fn format_row_count(count: i64) -> String {
    match count {
        c if c >= 1_000_000 => format!("{:.1}M", c as f64 / 1_000_000.0),
        c if c >= 1_000 => format!("{:.1}K", c as f64 / 1_000.0),
        c => c.to_string(),
    }
}
