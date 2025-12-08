use eframe::egui;
use tokio::sync::mpsc;

use crate::db::{spawn_db_worker, DbRequest, DbResponse, SchemaInfo};
use crate::ui::{ConnectionDialog, Editor, QueriesPanel, Results, StatusBar};

pub struct App {
    editor: Editor,
    results: Results,
    statusbar: StatusBar,
    connection_dialog: ConnectionDialog,
    queries_panel: QueriesPanel,
    schema: SchemaInfo,

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
            schema: SchemaInfo::default(),
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
                    self.results.error = None;
                    // Fetch schema for autocomplete
                    let _ = self.db_tx.send(DbRequest::FetchSchema);
                }
                DbResponse::Schema(schema) => {
                    self.schema = schema.clone();
                    self.editor.set_schema(schema);
                }
                DbResponse::QueryResult(result) => {
                    self.statusbar.row_count = Some(result.rows.len());
                    self.statusbar.exec_time_ms = Some(result.execution_time_ms);
                    self.results.result = Some(result);
                    self.results.error = None;
                }
                DbResponse::Error(e) => {
                    self.results.error = Some(e);
                }
                DbResponse::Disconnected => {
                    self.statusbar.connected = false;
                    self.statusbar.db_name.clear();
                    self.schema = SchemaInfo::default();
                    self.editor.set_schema(SchemaInfo::default());
                }
                DbResponse::TestResult(_) => {}
            }
        }
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
                    if ui.button("List Tables").clicked() {
                        let _ = self.db_tx.send(DbRequest::ListTables);
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            self.statusbar.show(ui);
        });

        egui::SidePanel::left("queries_panel")
            .default_width(180.0)
            .min_width(120.0)
            .show(ctx, |ui| {
                if let Some(sql) = self.queries_panel.show_panel(ui) {
                    self.editor.query = sql;
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
                        let _ = self.db_tx.send(DbRequest::Execute(self.editor.query.clone()));
                    }
                    
                    if action.save {
                        self.queries_panel.open_save_dialog();
                    }
                },
            );

            ui.separator();

            self.results.show(ui);
        });
    }
}
