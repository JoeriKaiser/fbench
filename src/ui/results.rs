use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::cmp::Ordering;
use crate::db::QueryResult;
use crate::export::{ExportFormat, export_results};

const MAX_CELL_LEN: usize = 100;
const DEFAULT_COL_WIDTH: f32 = 120.0;
const MIN_COL_WIDTH: f32 = 60.0;
const MAX_COL_WIDTH: f32 = 400.0;
const ROW_HEIGHT: f32 = 24.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortDir { Asc, Desc }

impl SortDir {
    fn toggle(self) -> Self { match self { Self::Asc => Self::Desc, Self::Desc => Self::Asc } }
    fn symbol(self) -> &'static str { match self { Self::Asc => " ▲", Self::Desc => " ▼" } }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct SortState { col: usize, dir: SortDir }

struct CellDetail { col_name: String, value: String, copied: bool }

#[derive(Default)]
pub struct Results {
    result: Option<QueryResult>,
    sorted_indices: Vec<usize>,
    sort_state: Option<SortState>,
    col_widths: Vec<f32>,
    pub error: Option<String>,
    detail_modal: Option<CellDetail>,
}

impl Results {
    pub fn set_result(&mut self, result: QueryResult) {
        self.col_widths = Self::calc_col_widths(&result);
        self.sorted_indices = (0..result.rows.len()).collect();
        self.sort_state = None;
        self.result = Some(result);
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.result = None;
        self.sorted_indices.clear();
        self.sort_state = None;
    }

    pub fn clear(&mut self) {
        self.result = None;
        self.error = None;
        self.col_widths.clear();
        self.sorted_indices.clear();
        self.sort_state = None;
    }

    fn calc_col_widths(result: &QueryResult) -> Vec<f32> {
        let mut widths: Vec<f32> = result.columns.iter()
            .map(|c| (c.len() as f32 * 8.0 + 24.0).clamp(MIN_COL_WIDTH, MAX_COL_WIDTH))
            .collect();

        for row in result.rows.iter().take(50) {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    let w = (cell.len().min(MAX_CELL_LEN) as f32 * 7.5 + 16.0).clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
                    widths[i] = widths[i].max(w);
                }
            }
        }
        widths
    }

    fn sort_by_column(&mut self, col: usize) {
        let Some(result) = &self.result else { return };
        if col >= result.columns.len() { return; }

        let dir = self.sort_state
            .filter(|s| s.col == col)
            .map(|s| s.dir.toggle())
            .unwrap_or(SortDir::Asc);

        self.sort_state = Some(SortState { col, dir });

        let rows = &result.rows;
        self.sorted_indices.sort_by(|&a, &b| {
            let va = rows.get(a).and_then(|r| r.get(col)).map(|s| s.as_str()).unwrap_or("");
            let vb = rows.get(b).and_then(|r| r.get(col)).map(|s| s.as_str()).unwrap_or("");

            let cmp = match (va.parse::<f64>(), vb.parse::<f64>()) {
                (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(Ordering::Equal),
                _ => va.cmp(vb),
            };
            if dir == SortDir::Desc { cmp.reverse() } else { cmp }
        });
    }

    fn truncate(s: &str) -> &str {
        if s.len() <= MAX_CELL_LEN { return s; }
        let mut end = MAX_CELL_LEN;
        while end > 0 && !s.is_char_boundary(end) { end -= 1; }
        &s[..end]
    }

    fn show_detail_modal(&mut self, ctx: &egui::Context) {
        let Some(detail) = &mut self.detail_modal else { return };
        let mut open = true;

        egui::Window::new(&detail.col_name)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("📋 Copy to Clipboard").clicked() {
                        ui.ctx().copy_text(detail.value.clone());
                        detail.copied = true;
                    }
                    if detail.copied { ui.colored_label(egui::Color32::GREEN, "✓ Copied!"); }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{} characters", detail.value.len()));
                    });
                });

                ui.add_space(8.0);
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut detail.value.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .interactive(true));
                });
            });

        if !open { self.detail_modal = None; }
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.show_detail_modal(ctx);

        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::RED, err);
            return;
        }

        let Some(result) = &self.result else {
            ui.colored_label(egui::Color32::GRAY, "Run a query to see results");
            return;
        };

        ui.horizontal(|ui| {
            ui.label(format!("{} rows", result.rows.len()));
            ui.separator();
            ui.label(format!("{} columns", result.columns.len()));
            ui.separator();

            if let Some(s) = self.sort_state {
                if let Some(col_name) = result.columns.get(s.col) {
                    ui.label(format!("Sorted by: {}{}", col_name, s.dir.symbol()));
                    ui.separator();
                }
            }

            ui.menu_button("📥 Export", |ui| {
                for (label, fmt) in [("CSV", ExportFormat::Csv), ("JSON", ExportFormat::Json), ("XML", ExportFormat::Xml)] {
                    if ui.button(label).clicked() {
                        export_results(result, fmt);
                        ui.close_menu();
                    }
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.colored_label(egui::Color32::GRAY, "Double-click cell to view full content");
            });
        });

        ui.add_space(4.0);

        if result.columns.is_empty() {
            ui.label("Query returned no columns");
            return;
        }

        let available_height = ui.available_height();
        let columns = result.columns.clone();
        let col_count = columns.len();
        let row_count = self.sorted_indices.len();
        let sort_state = self.sort_state;

        let mut clicked_header: Option<usize> = None;
        let mut double_clicked: Option<(usize, usize)> = None;

        egui::ScrollArea::horizontal().auto_shrink([false, false]).show(ui, |ui| {
            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height)
                .sense(egui::Sense::click());

            for i in 0..col_count {
                let w = self.col_widths.get(i).copied().unwrap_or(DEFAULT_COL_WIDTH);
                table = table.column(Column::initial(w).range(MIN_COL_WIDTH..=MAX_COL_WIDTH).clip(true));
            }

            table.header(ROW_HEIGHT + 4.0, |mut header| {
                for (i, name) in columns.iter().enumerate() {
                    header.col(|ui| {
                        let sorted = sort_state.map(|s| s.col == i).unwrap_or(false);
                        let indicator = if sorted { sort_state.unwrap().dir.symbol() } else { "" };
                        let resp = ui.add(egui::Label::new(
                            egui::RichText::new(format!("{}{}", name, indicator)).strong()
                        ).sense(egui::Sense::click()));
                        if resp.clicked() { clicked_header = Some(i); }
                        resp.on_hover_text("Click to sort");
                    });
                }
            }).body(|body| {
                body.rows(ROW_HEIGHT, row_count, |mut row| {
                    let visual_idx = row.index();
                    let actual_idx = self.sorted_indices.get(visual_idx).copied().unwrap_or(0);

                    if let Some(row_data) = self.result.as_ref().and_then(|r| r.rows.get(actual_idx)) {
                        for (col_idx, cell) in row_data.iter().enumerate() {
                            row.col(|ui| {
                                let display = Self::truncate(cell);
                                let truncated = display.len() < cell.len();
                                let is_null = cell == "NULL";

                                let text = if is_null {
                                    egui::RichText::new(display).italics().color(egui::Color32::GRAY).monospace()
                                } else if truncated {
                                    egui::RichText::new(format!("{}…", display)).monospace()
                                } else {
                                    egui::RichText::new(display).monospace()
                                };

                                let resp = ui.add(egui::Label::new(text).sense(egui::Sense::click()));
                                if resp.double_clicked() { double_clicked = Some((actual_idx, col_idx)); }
                                if truncated && resp.hovered() { resp.on_hover_text("Double-click to view full content"); }
                            });
                        }
                    }
                });
            });
        });

        if let Some(col) = clicked_header { self.sort_by_column(col); }

        if let Some((row_idx, col_idx)) = double_clicked {
            if let Some(result) = &self.result {
                if let Some(cell) = result.rows.get(row_idx).and_then(|r| r.get(col_idx)) {
                    self.detail_modal = Some(CellDetail {
                        col_name: result.columns.get(col_idx).cloned().unwrap_or_else(|| format!("Column {}", col_idx)),
                        value: cell.clone(),
                        copied: false,
                    });
                }
            }
        }
    }
}
