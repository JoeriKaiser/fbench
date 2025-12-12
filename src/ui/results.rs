use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::cmp::Ordering;
use crate::db::QueryResult;
use crate::export::{ExportFormat, export_results};

const MAX_CELL_DISPLAY_LEN: usize = 100;
const DEFAULT_COL_WIDTH: f32 = 120.0;
const MIN_COL_WIDTH: f32 = 60.0;
const MAX_COL_WIDTH: f32 = 400.0;
const ROW_HEIGHT: f32 = 24.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
    fn symbol(self) -> &'static str {
        match self {
            Self::Ascending => " ▲",
            Self::Descending => " ▼",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct SortState {
    column: usize,
    direction: SortDirection,
}

pub struct Results {
    result: Option<QueryResult>,
    sorted_indices: Vec<usize>,
    sort_state: Option<SortState>,
    col_widths: Vec<f32>,
    pub error: Option<String>,
    detail_modal: Option<CellDetail>,
}

struct CellDetail {
    column_name: String,
    value: String,
    copied: bool,
}

impl Default for Results {
    fn default() -> Self {
        Self {
            result: None,
            sorted_indices: Vec::new(),
            sort_state: None,
            col_widths: Vec::new(),
            error: None,
            detail_modal: None,
        }
    }
}

impl Results {
    pub fn set_result(&mut self, result: QueryResult) {
        self.col_widths = Self::calculate_column_widths(&result);
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

    fn calculate_column_widths(result: &QueryResult) -> Vec<f32> {
        let mut widths: Vec<f32> = result
            .columns
            .iter()
            .map(|c| (c.len() as f32 * 8.0 + 24.0).clamp(MIN_COL_WIDTH, MAX_COL_WIDTH))
            .collect();

        for row in result.rows.iter().take(50) {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    let display_len = cell.len().min(MAX_CELL_DISPLAY_LEN);
                    let cell_width =
                        (display_len as f32 * 7.5 + 16.0).clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
                    widths[i] = widths[i].max(cell_width);
                }
            }
        }

        widths
    }

    fn sort_by_column(&mut self, col_idx: usize) {
        let Some(result) = &self.result else { return };
        if col_idx >= result.columns.len() {
            return;
        }

        let new_direction = if let Some(state) = self.sort_state {
            if state.column == col_idx {
                state.direction.toggle()
            } else {
                SortDirection::Ascending
            }
        } else {
            SortDirection::Ascending
        };

        self.sort_state = Some(SortState {
            column: col_idx,
            direction: new_direction,
        });

        let rows = &result.rows;
        self.sorted_indices.sort_by(|&a, &b| {
            let val_a = rows
                .get(a)
                .and_then(|r| r.get(col_idx))
                .map(|s| s.as_str())
                .unwrap_or("");
            let val_b = rows
                .get(b)
                .and_then(|r| r.get(col_idx))
                .map(|s| s.as_str())
                .unwrap_or("");

            let cmp = match (val_a.parse::<f64>(), val_b.parse::<f64>()) {
                (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(Ordering::Equal),
                _ => val_a.cmp(val_b),
            };

            match new_direction {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
    }

    #[inline]
    fn truncate_for_display(value: &str) -> &str {
        if value.len() <= MAX_CELL_DISPLAY_LEN {
            value
        } else {
            let mut end = MAX_CELL_DISPLAY_LEN;
            while end > 0 && !value.is_char_boundary(end) {
                end -= 1;
            }
            &value[..end]
        }
    }

    fn show_detail_modal(&mut self, ctx: &egui::Context) {
        let Some(detail) = &mut self.detail_modal else {
            return;
        };

        let mut open = true;

        egui::Window::new(&detail.column_name)
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
                    if detail.copied {
                        ui.colored_label(egui::Color32::GREEN, "✓ Copied!");
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{} characters", detail.value.len()));
                    });
                });

                ui.add_space(8.0);

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut detail.value.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .interactive(true),
                        );
                    });
            });

        if !open {
            self.detail_modal = None;
        }
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

            if let Some(sort_state) = self.sort_state {
                if let Some(col_name) = result.columns.get(sort_state.column) {
                    ui.label(format!(
                        "Sorted by: {}{}",
                        col_name,
                        sort_state.direction.symbol()
                    ));
                    ui.separator();
                }
            }

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

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.colored_label(
                    egui::Color32::GRAY,
                    "Double-click cell to view full content",
                );
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
        let mut double_clicked_cell: Option<(usize, usize)> = None;

        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .min_scrolled_height(0.0)
                    .max_scroll_height(available_height)
                    .sense(egui::Sense::click());

                for i in 0..col_count {
                    let width = self.col_widths.get(i).copied().unwrap_or(DEFAULT_COL_WIDTH);
                    table = table.column(
                        Column::initial(width)
                            .range(MIN_COL_WIDTH..=MAX_COL_WIDTH)
                            .clip(true),
                    );
                }

                table
                    .header(ROW_HEIGHT + 4.0, |mut header| {
                        for (col_idx, col_name) in columns.iter().enumerate() {
                            header.col(|ui| {
                                let is_sorted =
                                    sort_state.map(|s| s.column == col_idx).unwrap_or(false);
                                let sort_indicator = if is_sorted {
                                    sort_state.unwrap().direction.symbol()
                                } else {
                                    ""
                                };

                                let text = format!("{}{}", col_name, sort_indicator);
                                let response = ui.add(
                                    egui::Label::new(egui::RichText::new(&text).strong())
                                        .sense(egui::Sense::click()),
                                );

                                if response.clicked() {
                                    clicked_header = Some(col_idx);
                                }

                                response.on_hover_text("Click to sort");
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(ROW_HEIGHT, row_count, |mut row| {
                            let visual_row_idx = row.index();
                            let actual_row_idx =
                                self.sorted_indices.get(visual_row_idx).copied().unwrap_or(0);

                            if let Some(row_data) =
                                self.result.as_ref().and_then(|r| r.rows.get(actual_row_idx))
                            {
                                for (col_idx, cell) in row_data.iter().enumerate() {
                                    row.col(|ui| {
                                        let display_value = Self::truncate_for_display(cell);
                                        let is_truncated = display_value.len() < cell.len();
                                        let is_null = cell == "NULL";

                                        let text = if is_null {
                                            egui::RichText::new(display_value)
                                                .italics()
                                                .color(egui::Color32::GRAY)
                                                .monospace()
                                        } else if is_truncated {
                                            egui::RichText::new(format!("{}…", display_value))
                                                .monospace()
                                        } else {
                                            egui::RichText::new(display_value).monospace()
                                        };

                                        let response = ui.add(
                                            egui::Label::new(text).sense(egui::Sense::click()),
                                        );

                                        if response.double_clicked() {
                                            double_clicked_cell = Some((actual_row_idx, col_idx));
                                        }

                                        if is_truncated && response.hovered() {
                                            response
                                                .on_hover_text("Double-click to view full content");
                                        }
                                    });
                                }
                            }
                        });
                    });
            });

        if let Some(col_idx) = clicked_header {
            self.sort_by_column(col_idx);
        }

        if let Some((row_idx, col_idx)) = double_clicked_cell {
            if let Some(result) = &self.result {
                if let Some(row) = result.rows.get(row_idx) {
                    if let Some(cell) = row.get(col_idx) {
                        let col_name = result
                            .columns
                            .get(col_idx)
                            .cloned()
                            .unwrap_or_else(|| format!("Column {}", col_idx));

                        self.detail_modal = Some(CellDetail {
                            column_name: col_name,
                            value: cell.clone(),
                            copied: false,
                        });
                    }
                }
            }
        }
    }
}
