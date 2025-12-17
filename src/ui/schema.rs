use eframe::egui::{self, Color32, RichText};
use crate::db::{SchemaInfo};

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
}

#[derive(Default)]
pub struct SchemaPanelAction {
    pub select_table_data: Option<String>,
    pub view_table_structure: Option<String>,
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

        action
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
