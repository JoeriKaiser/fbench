use eframe::egui::{self, Color32, RichText};
use crate::db::TableInfo;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum DetailTab {
    #[default]
    Columns,
    Indexes,
    Constraints,
}

#[derive(Default)]
pub struct TableDetailPanel {
    pub table: Option<TableInfo>,
    pub show: bool,
    current_tab: DetailTab,
    loading: bool,
}

impl TableDetailPanel {
    pub fn open(&mut self, table_name: &str) {
        self.show = true;
        self.loading = true;
        self.table = Some(TableInfo {
            name: table_name.to_string(),
            ..Default::default()
        });
    }

    pub fn set_table(&mut self, table: TableInfo) {
        self.table = Some(table);
        self.loading = false;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.show {
            return;
        }

        let title = self.table.as_ref()
            .map(|t| format!("Structure: {}", t.name))
            .unwrap_or_else(|| "Table Structure".to_string());

        let mut open = self.show;

        egui::Window::new(title)
            .open(&mut open)
            .default_width(700.0)
            .default_height(500.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                if self.loading {
                    ui.centered_and_justified(|ui| ui.spinner());
                    return;
                }

                let Some(table) = &self.table else {
                    ui.label("No table selected");
                    return;
                };

                ui.horizontal(|ui| {
                    for (tab, label) in [
                        (DetailTab::Columns, format!("▸ Columns ({})", table.columns.len())),
                        (DetailTab::Indexes, format!("🔑 Indexes ({})", table.indexes.len())),
                        (DetailTab::Constraints, format!("🔗 Constraints ({})", table.constraints.len())),
                    ] {
                        if ui.selectable_label(self.current_tab == tab, label).clicked() {
                            self.current_tab = tab;
                        }
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.current_tab {
                        DetailTab::Columns => self.show_columns(ui, table),
                        DetailTab::Indexes => self.show_indexes(ui, table),
                        DetailTab::Constraints => self.show_constraints(ui, table),
                    });
            });

        self.show = open;
    }

    fn show_columns(&self, ui: &mut egui::Ui, table: &TableInfo) {
        if table.columns.is_empty() {
            ui.colored_label(Color32::GRAY, "No columns");
            return;
        }

        egui::Grid::new("columns_grid")
            .num_columns(5)
            .striped(true)
            .spacing([16.0, 4.0])
            .show(ui, |ui| {
                for h in ["Column", "Type", "Nullable", "Default", "Key"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();

                for col in &table.columns {
                    let name_text = if col.is_primary_key {
                        RichText::new(&col.name).color(Color32::from_rgb(255, 200, 100))
                    } else {
                        RichText::new(&col.name)
                    };
                    ui.label(name_text);
                    ui.colored_label(Color32::from_rgb(100, 200, 150), &col.data_type);
                    
                    let (text, color) = if col.nullable {
                        ("YES", Color32::GRAY)
                    } else {
                        ("NO", Color32::from_rgb(150, 150, 200))
                    };
                    ui.colored_label(color, text);
                    ui.colored_label(Color32::GRAY, truncate(col.default_value.as_deref().unwrap_or("-"), 30));
                    ui.label(if col.is_primary_key { "★ PK" } else { "" });
                    ui.end_row();
                }
            });
    }

    fn show_indexes(&self, ui: &mut egui::Ui, table: &TableInfo) {
        if table.indexes.is_empty() {
            ui.colored_label(Color32::GRAY, "No indexes");
            return;
        }

        egui::Grid::new("indexes_grid")
            .num_columns(4)
            .striped(true)
            .spacing([16.0, 4.0])
            .show(ui, |ui| {
                for h in ["Name", "Columns", "Type", "Properties"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();

                for idx in &table.indexes {
                    let color = if idx.is_primary {
                        Color32::from_rgb(255, 200, 100)
                    } else if idx.is_unique {
                        Color32::from_rgb(150, 200, 255)
                    } else {
                        Color32::WHITE
                    };
                    ui.colored_label(color, &idx.name);
                    ui.label(idx.columns.join(", "));
                    ui.colored_label(Color32::GRAY, &idx.index_type);

                    let props: Vec<_> = [
                        (idx.is_primary, "PRIMARY"),
                        (idx.is_unique && !idx.is_primary, "UNIQUE"),
                    ].iter().filter(|(c, _)| *c).map(|(_, s)| *s).collect();
                    ui.label(props.join(", "));
                    ui.end_row();
                }
            });
    }

    fn show_constraints(&self, ui: &mut egui::Ui, table: &TableInfo) {
        if table.constraints.is_empty() {
            ui.colored_label(Color32::GRAY, "No constraints");
            return;
        }

        egui::Grid::new("constraints_grid")
            .num_columns(4)
            .striped(true)
            .spacing([16.0, 4.0])
            .show(ui, |ui| {
                for h in ["Name", "Type", "Columns", "Details"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();

                for con in &table.constraints {
                    ui.label(&con.name);

                    let (text, color) = match con.constraint_type.as_str() {
                        "PRIMARY KEY" => ("★ PRIMARY KEY", Color32::from_rgb(255, 200, 100)),
                        "FOREIGN KEY" => ("🔗 FOREIGN KEY", Color32::from_rgb(150, 200, 255)),
                        "UNIQUE" => ("◈ UNIQUE", Color32::from_rgb(200, 150, 255)),
                        "CHECK" => ("✓ CHECK", Color32::from_rgb(150, 255, 150)),
                        _ => (con.constraint_type.as_str(), Color32::WHITE),
                    };
                    ui.colored_label(color, text);
                    ui.label(con.columns.join(", "));

                    let detail = match (&con.foreign_table, &con.foreign_columns) {
                        (Some(ft), Some(fc)) => format!("→ {}.{}", ft, fc.join(", ")),
                        _ => con.check_clause.as_ref()
                            .map(|c| truncate(c, 40).to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    };
                    ui.colored_label(Color32::GRAY, detail);
                    ui.end_row();
                }
            });
    }
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len { return s; }
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) { end -= 1; }
    &s[..end]
}
