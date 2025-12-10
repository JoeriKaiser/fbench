use eframe::egui;
use tokio::sync::mpsc;

use crate::db::{spawn_db_worker, DbRequest, DbResponse, SchemaInfo};
use crate::ui::{ConnectionDialog, Editor, QueriesPanel, Results, StatusBar, SchemaPanel, TableDetailPanel};

#[derive(Clone, Copy, PartialEq, Default)]
enum LeftPanelTab {
    #[default]
    Schema,
    Queries,
}

pub struct App {
    editor: Editor,
    results: Results,
    statusbar: StatusBar,
    connection_dialog: ConnectionDialog,
    queries_panel: QueriesPanel,
    schema_panel: SchemaPanel,
    table_detail: TableDetailPanel,
    left_panel_tab: LeftPanelTab,

    db_tx: mpsc::UnboundedSender<DbRequest>,
    db_rx: mpsc::UnboundedReceiver<DbResponse>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (db_tx, db_rx) = spawn_db_worker();

        Self {
            editor: Editor::default(),
            results: Results::default(),
            statusbar: StatusBar::default(),
            connection_dialog: ConnectionDialog::new(),
            queries_panel: QueriesPanel::new(),
            schema_panel: SchemaPanel::default(),
            table_detail: TableDetailPanel::default(),
            left_panel_tab: LeftPanelTab::default(),
            db_tx,
            db_rx,
        }
    }

    fn poll_db_responses(&mut self) {
        while let Ok(response) = self.db_rx.try_recv() {
            self.connection_dialog.handle_db_response(&response);
            
            match response {
                DbResponse::Connected => {
                    self.statusbar.connected = true;
                    self.results.clear();
                    let _ = self.db_tx.send(DbRequest::FetchSchema);
                }
                DbResponse::Schema(schema) => {
                    self.schema_panel.set_schema(schema.clone());
                    self.editor.set_schema(schema);
                }
                DbResponse::TableDetails(table) => {
                    self.table_detail.set_table(table);
                }
                DbResponse::QueryResult(result) => {
                    self.statusbar.row_count = Some(result.rows.len());
                    self.statusbar.exec_time_ms = Some(result.execution_time_ms);
                    self.results.set_result(result);
                }
                DbResponse::Error(e) => {
                    self.results.set_error(e);
                }
                DbResponse::Disconnected => {
                    self.statusbar.connected = false;
                    self.statusbar.db_name.clear();
                    self.schema_panel.set_schema(SchemaInfo::default());
                    self.editor.set_schema(SchemaInfo::default());
                }
                DbResponse::TestResult(_) => {}
            }
        }
    }

    fn execute_query(&self, sql: &str) {
        let _ = self.db_tx.send(DbRequest::Execute(sql.to_string()));
    }

    fn select_table_data(&mut self, table_name: &str) {
        let sql = format!("SELECT * FROM \"{}\" LIMIT 100;", table_name);
        self.editor.query = sql.clone();
        if self.statusbar.connected {
            self.execute_query(&sql);
        }
    }

    fn view_table_structure(&mut self, table_name: &str) {
        self.table_detail.open(table_name);
        let _ = self.db_tx.send(DbRequest::FetchTableDetails(table_name.to_string()));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_db_responses();

        if let Some(config) = self.connection_dialog.show(ctx, &self.db_tx) {
            self.statusbar.db_name = config.database.clone();
            let _ = self.db_tx.send(DbRequest::Connect(config));
        }
        
        self.queries_panel.show_save_popup(ctx, &self.editor.query);
        
        self.table_detail.show(ctx);

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Connect...").clicked() {
                        self.connection_dialog.show = true;
                        ui.close_menu();
                    }
                    if ui.button("Disconnect").clicked() {
                        let _ = self.db_tx.send(DbRequest::Disconnect);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Query", |ui| {
                    if ui.button("Execute (Ctrl+Enter)").clicked() {
                        if self.statusbar.connected {
                            self.execute_query(&self.editor.query.clone());
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("List Tables").clicked() {
                        let _ = self.db_tx.send(DbRequest::ListTables);
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    let selected_table = self.schema_panel.get_selected_table().map(|s| s.to_string());
                    
                    if let Some(table_name) = selected_table {
                        if ui.button(format!("Structure: {}", table_name)).clicked() {
                            self.view_table_structure(&table_name);
                            ui.close_menu();
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("Structure (select a table)"));
                    }
                    ui.separator();
                    if ui.button("Refresh Schema").clicked() {
                        let _ = self.db_tx.send(DbRequest::FetchSchema);
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            self.statusbar.show(ui);
        });

        egui::SidePanel::left("left_panel")
            .default_width(220.0)
            .min_width(150.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.left_panel_tab == LeftPanelTab::Schema, "▤ Schema").clicked() {
                        self.left_panel_tab = LeftPanelTab::Schema;
                    }
                    if ui.selectable_label(self.left_panel_tab == LeftPanelTab::Queries, "📝 Queries").clicked() {
                        self.left_panel_tab = LeftPanelTab::Queries;
                    }
                });
                ui.separator();

                match self.left_panel_tab {
                    LeftPanelTab::Schema => {
                        let action = self.schema_panel.show(ui);
                        
                        if let Some(table_name) = action.select_table_data {
                            self.select_table_data(&table_name);
                        }
                        if let Some(table_name) = action.view_table_structure {
                            self.view_table_structure(&table_name);
                        }
                    }
                    LeftPanelTab::Queries => {
                        if let Some(sql) = self.queries_panel.show_panel(ui) {
                            self.editor.query = sql;
                        }
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_height();
            let editor_height = available * 0.35;

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), editor_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    let action = self.editor.show(ui);
                    
                    if action.execute && self.statusbar.connected {
                        self.execute_query(&self.editor.query.clone());
                    }
                    
                    if action.save {
                        self.queries_panel.open_save_dialog();
                    }
                },
            );

            ui.separator();

            self.results.show(ctx, ui);
        });
    }
}
