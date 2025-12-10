use eframe::egui::{self, Color32, RichText};
use crate::db::TableInfo;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum DetailTab {
    #[default]
    Columns,
    Indexes,
    Constraints,
}

pub struct TableDetailPanel {
    pub table: Option<TableInfo>,
    pub show: bool,
    current_tab: DetailTab,
    loading: bool,
}

impl Default for TableDetailPanel {
    fn default() -> Self {
        Self {
            table: None,
            show: false,
            current_tab: DetailTab::Columns,
            loading: false,
        }
    }
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
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                    });
                    return;
                }

                let Some(table) = &self.table else {
                    ui.label("No table selected");
                    return;
                };

                ui.horizontal(|ui| {
                    if ui.selectable_label(self.current_tab == DetailTab::Columns, 
                        format!("▸ Columns ({})", table.columns.len())).clicked() {
                        self.current_tab = DetailTab::Columns;
                    }
                    if ui.selectable_label(self.current_tab == DetailTab::Indexes, 
                        format!("📑 Indexes ({})", table.indexes.len())).clicked() {
                        self.current_tab = DetailTab::Indexes;
                    }
                    if ui.selectable_label(self.current_tab == DetailTab::Constraints, 
                        format!("🔗 Constraints ({})", table.constraints.len())).clicked() {
                        self.current_tab = DetailTab::Constraints;
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match self.current_tab {
                            DetailTab::Columns => self.show_columns(ui, table),
                            DetailTab::Indexes => self.show_indexes(ui, table),
                            DetailTab::Constraints => self.show_constraints(ui, table),
                        }
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
                ui.label(RichText::new("Column").strong());
                ui.label(RichText::new("Type").strong());
                ui.label(RichText::new("Nullable").strong());
                ui.label(RichText::new("Default").strong());
                ui.label(RichText::new("Key").strong());
                ui.end_row();

                for col in &table.columns {
                    let name_text = if col.is_primary_key {
                        RichText::new(&col.name).color(Color32::from_rgb(255, 200, 100))
                    } else {
                        RichText::new(&col.name)
                    };
                    ui.label(name_text);

                    ui.colored_label(Color32::from_rgb(100, 200, 150), &col.data_type);

                    let nullable_text = if col.nullable { "YES" } else { "NO" };
                    let nullable_color = if col.nullable { Color32::GRAY } else { Color32::from_rgb(150, 150, 200) };
                    ui.colored_label(nullable_color, nullable_text);

                    let default_text = col.default_value.as_deref().unwrap_or("-");
                    ui.colored_label(Color32::GRAY, truncate(default_text, 30));

                    if col.is_primary_key {
                        ui.label("★ PK");
                    } else {
                        ui.label("");
                    }

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
                ui.label(RichText::new("Name").strong());
                ui.label(RichText::new("Columns").strong());
                ui.label(RichText::new("Type").strong());
                ui.label(RichText::new("Properties").strong());
                ui.end_row();

                for idx in &table.indexes {
                    let name_color = if idx.is_primary {
                        Color32::from_rgb(255, 200, 100)
                    } else if idx.is_unique {
                        Color32::from_rgb(150, 200, 255)
                    } else {
                        Color32::WHITE
                    };
                    ui.colored_label(name_color, &idx.name);

                    ui.label(idx.columns.join(", "));

                    ui.colored_label(Color32::GRAY, &idx.index_type);

                    let mut props = Vec::new();
                    if idx.is_primary { props.push("PRIMARY"); }
                    if idx.is_unique && !idx.is_primary { props.push("UNIQUE"); }
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
                ui.label(RichText::new("Name").strong());
                ui.label(RichText::new("Type").strong());
                ui.label(RichText::new("Columns").strong());
                ui.label(RichText::new("Details").strong());
                ui.end_row();

                for con in &table.constraints {
                    ui.label(&con.name);

                    let (type_text, type_color) = match con.constraint_type.as_str() {
                        "PRIMARY KEY" => ("★ PRIMARY KEY", Color32::from_rgb(255, 200, 100)),
                        "FOREIGN KEY" => ("🔗 FOREIGN KEY", Color32::from_rgb(150, 200, 255)),
                        "UNIQUE" => ("◈ UNIQUE", Color32::from_rgb(200, 150, 255)),
                        "CHECK" => ("✓ CHECK", Color32::from_rgb(150, 255, 150)),
                        _ => (&con.constraint_type as &str, Color32::WHITE),
                    };
                    ui.colored_label(type_color, type_text);

                    ui.label(con.columns.join(", "));

                    let detail = if let (Some(ft), Some(fc)) = (&con.foreign_table, &con.foreign_columns) {
                        format!("→ {}.{}", ft, fc.join(", "))
                    } else if let Some(check) = &con.check_clause {
                        truncate(check, 40).to_string()
                    } else {
                        "-".to_string()
                    };
                    ui.colored_label(Color32::GRAY, detail);

                    ui.end_row();
                }
            });
    }
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}
