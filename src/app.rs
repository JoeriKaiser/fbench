use eframe::egui;
use tokio::sync::mpsc;

use crate::db::{spawn_db_worker, DatabaseType, DbRequest, DbResponse, SchemaInfo};
use crate::llm::{spawn_llm_worker, LlmRequest, LlmResponse};
use crate::ui::{
    AiPrompt, ConnectionDialog, Editor, QueriesPanel, Results, SchemaPanel, StatusBar,
    TableDetailPanel,
};

#[derive(Clone, Copy, PartialEq, Default)]
enum LeftTab {
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
    ai_prompt: AiPrompt,
    left_tab: LeftTab,
    current_db_type: Option<DatabaseType>,
    schema: SchemaInfo,
    db_tx: mpsc::UnboundedSender<DbRequest>,
    db_rx: mpsc::UnboundedReceiver<DbResponse>,
    llm_tx: mpsc::UnboundedSender<LlmRequest>,
    llm_rx: mpsc::UnboundedReceiver<LlmResponse>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (db_tx, db_rx) = spawn_db_worker();
        let (llm_tx, llm_rx) = spawn_llm_worker();

        Self {
            editor: Editor::default(),
            results: Results::default(),
            statusbar: StatusBar::default(),
            connection_dialog: ConnectionDialog::new(),
            queries_panel: QueriesPanel::new(),
            schema_panel: SchemaPanel::default(),
            table_detail: TableDetailPanel::default(),
            ai_prompt: AiPrompt::default(),
            left_tab: LeftTab::default(),
            current_db_type: None,
            schema: SchemaInfo::default(),
            db_tx,
            db_rx,
            llm_tx,
            llm_rx,
        }
    }

    fn poll_responses(&mut self) {
        while let Ok(response) = self.db_rx.try_recv() {
            self.connection_dialog.handle_db_response(&response);

            match response {
                DbResponse::Connected(db_type) => {
                    self.statusbar.connected = true;
                    self.statusbar.db_type = Some(db_type);
                    self.current_db_type = Some(db_type);
                    self.results.clear();
                    let _ = self.db_tx.send(DbRequest::FetchSchema);
                }
                DbResponse::Schema(schema) => {
                    self.schema_panel.set_schema(schema.clone());
                    self.editor.set_schema(schema.clone());
                    self.schema = schema;
                }
                DbResponse::TableDetails(table) => self.table_detail.set_table(table),
                DbResponse::QueryResult(result) => {
                    self.statusbar.row_count = Some(result.rows.len());
                    self.statusbar.exec_time_ms = Some(result.execution_time_ms);
                    self.results.set_result(result);
                }
                DbResponse::Error(e) => self.results.set_error(e),
                DbResponse::Disconnected => {
                    self.statusbar.connected = false;
                    self.statusbar.db_type = None;
                    self.statusbar.db_name.clear();
                    self.current_db_type = None;
                    self.schema = SchemaInfo::default();
                    self.schema_panel.set_schema(SchemaInfo::default());
                    self.editor.set_schema(SchemaInfo::default());
                }
                DbResponse::TestResult(_) => {}
            }
        }

        while let Ok(response) = self.llm_rx.try_recv() {
            match response {
                LlmResponse::Generated(sql) => {
                    self.ai_prompt.set_generating(false);
                    self.ai_prompt.take_prompt();
                    self.editor.query = sql;
                }
                LlmResponse::Explanation(_) |
                LlmResponse::Optimization { .. } |
                LlmResponse::ErrorFix { .. } => {
                    // Will be handled by editor in future tasks
                }
                LlmResponse::QuerySuggestions(_) => {
                    // Will be handled by schema panel in future tasks
                }
                LlmResponse::Error(e) => {
                    self.ai_prompt.set_error(e);
                }
            }
        }
    }

    fn execute_query(&self, sql: &str) {
        let _ = self.db_tx.send(DbRequest::Execute(sql.to_string()));
    }

    fn select_table_data(&mut self, table: &str) {
        let sql = match self.current_db_type {
            Some(DatabaseType::MySQL) => format!("SELECT * FROM `{}` LIMIT 100;", table),
            _ => format!("SELECT * FROM \"{}\" LIMIT 100;", table),
        };
        self.editor.query = sql.clone();
        if self.statusbar.connected {
            self.execute_query(&sql);
        }
    }

    fn view_table_structure(&mut self, table: &str) {
        self.table_detail.open(table);
        let _ = self.db_tx.send(DbRequest::FetchTableDetails(table.to_string()));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_responses();

        if let Some(config) = self.connection_dialog.show(ctx, &self.db_tx) {
            self.statusbar.db_name = config.database.clone();
            let _ = self.db_tx.send(DbRequest::Connect(config));
        }

        self.queries_panel.show_save_popup(ctx, &self.editor.query);
        self.table_detail.show(ctx);
        self.ai_prompt.show_settings_window(ctx);

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
                    if let Some(table) =
                        self.schema_panel.get_selected_table().map(|s| s.to_string())
                    {
                        if ui.button(format!("Structure: {}", table)).clicked() {
                            self.view_table_structure(&table);
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

        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| self.statusbar.show(ui));

        egui::SidePanel::left("left_panel")
            .default_width(220.0)
            .min_width(150.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.left_tab == LeftTab::Schema, "▤ Schema")
                        .clicked()
                    {
                        self.left_tab = LeftTab::Schema;
                    }
                    if ui
                        .selectable_label(self.left_tab == LeftTab::Queries, "📝 Queries")
                        .clicked()
                    {
                        self.left_tab = LeftTab::Queries;
                    }
                });
                ui.separator();

                match self.left_tab {
                    LeftTab::Schema => {
                        let action = self.schema_panel.show(ui);
                        if let Some(t) = action.select_table_data {
                            self.select_table_data(&t);
                        }
                        if let Some(t) = action.view_table_structure {
                            self.view_table_structure(&t);
                        }
                    }
                    LeftTab::Queries => {
                        if let Some(sql) = self.queries_panel.show_panel(ui) {
                            self.editor.query = sql;
                        }
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let editor_height = ui.available_height() * 0.35;

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), editor_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    self.ai_prompt.show(
                        ui,
                        &self.llm_tx,
                        &self.schema,
                        self.statusbar.connected,
                    );
                    ui.add_space(4.0);

                    let action = self.editor.show(ui);
                    if let Some(sql) = action.execute_sql {
                        if self.statusbar.connected {
                            self.execute_query(&sql);
                        }
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
