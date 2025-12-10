use eframe::egui::{self, Color32, RichText};
use crate::db::{SchemaInfo, TableInfo};

#[derive(Clone, PartialEq)]
pub enum SchemaSelection {
    None,
    Table(String),
    View(String),
}

pub struct SchemaPanel {
    pub schema: SchemaInfo,
    expanded_tables: std::collections::HashSet<String>,
    expanded_sections: SectionState,
    pub selection: SchemaSelection,
    search_query: String,
}

struct SectionState {
    tables: bool,
    views: bool,
}

impl Default for SectionState {
    fn default() -> Self {
        Self { tables: true, views: true }
    }
}

impl Default for SchemaPanel {
    fn default() -> Self {
        Self {
            schema: SchemaInfo::default(),
            expanded_tables: std::collections::HashSet::new(),
            expanded_sections: SectionState::default(),
            selection: SchemaSelection::None,
            search_query: String::new(),
        }
    }
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
        if self.search_query.is_empty() {
            return true;
        }
        name.to_lowercase().contains(&self.search_query.to_lowercase())
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
                let table_count = self.schema.tables.len();
                let tables_header = egui::CollapsingHeader::new(
                    RichText::new(format!("📁 Tables ({})", table_count)).strong()
                )
                .default_open(self.expanded_sections.tables)
                .show(ui, |ui| {
                    self.show_tables(ui, &mut action);
                });
                self.expanded_sections.tables = tables_header.fully_open();

                ui.add_space(4.0);

                let view_count = self.schema.views.len();
                let views_header = egui::CollapsingHeader::new(
                    RichText::new(format!("◎ Views ({})", view_count)).strong()
                )
                .default_open(self.expanded_sections.views)
                .show(ui, |ui| {
                    self.show_views(ui, &mut action);
                });
                self.expanded_sections.views = views_header.fully_open();
            });

        action
    }

    fn show_tables(&mut self, ui: &mut egui::Ui, action: &mut SchemaPanelAction) {
        let filtered_tables: Vec<&TableInfo> = self.schema.tables
            .iter()
            .filter(|t| self.filter_matches(&t.name))
            .collect();

        if filtered_tables.is_empty() {
            ui.colored_label(Color32::GRAY, "No tables found");
            return;
        }

        for table in filtered_tables {
            let is_selected = self.selection == SchemaSelection::Table(table.name.clone());
            let is_expanded = self.expanded_tables.contains(&table.name);
            
            let header_id = ui.make_persistent_id(&table.name);
            
            ui.horizontal(|ui| {
                let toggle_icon = if is_expanded { "▼" } else { "▶" };
                if ui.small_button(toggle_icon).clicked() {
                    if is_expanded {
                        self.expanded_tables.remove(&table.name);
                    } else {
                        self.expanded_tables.insert(table.name.clone());
                    }
                }

                let row_text = format!("▪ {} ", table.name);
                let row_label = if is_selected {
                    RichText::new(&row_text).strong().color(ui.visuals().selection.stroke.color)
                } else {
                    RichText::new(&row_text)
                };
                
                let response = ui.selectable_label(is_selected, row_label);
                
                if response.clicked() {
                    self.selection = SchemaSelection::Table(table.name.clone());
                    action.select_table_data = Some(table.name.clone());
                }

                if table.row_estimate > 0 {
                    ui.colored_label(
                        Color32::GRAY, 
                        format!("~{}", format_row_count(table.row_estimate))
                    );
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
                ui.indent(header_id, |ui| {
                    for col in &table.columns {
                        ui.horizontal(|ui| {
                            let icon = if col.is_primary_key { "★" } else { "  " };
                            let nullable = if col.nullable { "?" } else { "" };
                            let col_text = format!("{} {}{}: {}", icon, col.name, nullable, col.data_type);
                            ui.colored_label(Color32::GRAY, col_text);
                        });
                    }
                });
            }
        }
    }

    fn show_views(&mut self, ui: &mut egui::Ui, action: &mut SchemaPanelAction) {
        let filtered_views: Vec<&String> = self.schema.views
            .iter()
            .filter(|v| self.filter_matches(v))
            .collect();

        if filtered_views.is_empty() {
            ui.colored_label(Color32::GRAY, "No views found");
            return;
        }

        for view in filtered_views {
            let is_selected = self.selection == SchemaSelection::View(view.clone());
            
            let row_text = format!("◎ {}", view);
            let row_label = if is_selected {
                RichText::new(&row_text).strong().color(ui.visuals().selection.stroke.color)
            } else {
                RichText::new(&row_text)
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
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}
