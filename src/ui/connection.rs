use eframe::egui;
use tokio::sync::mpsc;
use crate::db::{ConnectionConfig, DatabaseType, DbRequest, DbResponse};
use crate::config::{SavedConnection, ConnectionStore};

pub struct ConnectionDialog {
    pub db_type: DatabaseType,
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub schema: String,
    pub connection_name: String,
    pub save_password: bool,
    pub show: bool,
    
    store: ConnectionStore,
    saved_connections: Vec<SavedConnection>,
    selected_index: Option<usize>,
    status_message: Option<(String, bool)>,
    testing: bool,
}

impl Default for ConnectionDialog {
    fn default() -> Self { Self::new() }
}

impl ConnectionDialog {
    pub fn new() -> Self {
        let store = ConnectionStore::new();
        let saved_connections = store.load_connections();
        let last_used = store.get_last_used();
        
        let mut dialog = Self {
            db_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: "5432".to_string(),
            user: "postgres".to_string(),
            password: String::new(),
            database: "postgres".to_string(),
            schema: String::new(),
            connection_name: String::new(),
            save_password: false,
            show: true,
            store,
            saved_connections,
            selected_index: None,
            status_message: None,
            testing: false,
        };

        if let Some(last_name) = last_used {
            if let Some(idx) = dialog.saved_connections.iter().position(|c| c.name == last_name) {
                dialog.selected_index = Some(idx);
                dialog.load_selected();
            }
        }
        dialog
    }

    pub fn handle_db_response(&mut self, response: &DbResponse) {
        if let DbResponse::TestResult(result) = response {
            self.testing = false;
            self.status_message = Some(match result {
                Ok(()) => ("✓ Connection successful!".into(), false),
                Err(e) => (format!("✗ {}", e), true),
            });
        }
    }

    fn load_selected(&mut self) {
        let Some(conn) = self.selected_index.and_then(|i| self.saved_connections.get(i)) else { return };
        
        self.connection_name = conn.name.clone();
        self.db_type = conn.db_type;
        self.host = conn.host.clone();
        self.port = conn.port.to_string();
        self.user = conn.user.clone();
        self.database = conn.database.clone();
        self.schema = conn.schema.clone();
        self.save_password = conn.save_password;
        self.password = if conn.save_password {
            conn.password.clone().unwrap_or_default()
        } else {
            self.store.get_password(&conn.name).unwrap_or_default()
        };
    }

    fn clear_form(&mut self) {
        self.connection_name.clear();
        self.db_type = DatabaseType::PostgreSQL;
        self.host = "localhost".to_string();
        self.port = self.db_type.default_port().to_string();
        self.user = "postgres".to_string();
        self.password.clear();
        self.database = "postgres".to_string();
        self.schema.clear();
        self.save_password = false;
        self.selected_index = None;
    }

    fn on_db_type_changed(&mut self) {
        let ports = [DatabaseType::PostgreSQL.default_port(), DatabaseType::MySQL.default_port()];
        if let Ok(p) = self.port.parse::<u16>() {
            if ports.contains(&p) {
                self.port = self.db_type.default_port().to_string();
            }
        }
        
        self.user = match (self.db_type, self.user.as_str()) {
            (DatabaseType::PostgreSQL, "root") => "postgres".to_string(),
            (DatabaseType::MySQL, "postgres") => "root".to_string(),
            _ => return,
        };
    }

    fn save_current(&mut self) {
        let name = self.connection_name.trim();
        if name.is_empty() {
            self.status_message = Some(("Enter a connection name".into(), true));
            return;
        }

        let Ok(port) = self.port.parse() else {
            self.status_message = Some(("Invalid port".into(), true));
            return;
        };

        let conn = SavedConnection {
            name: name.to_string(),
            db_type: self.db_type,
            host: self.host.clone(),
            port,
            user: self.user.clone(),
            database: self.database.clone(),
            schema: self.schema.clone(),
            save_password: self.save_password,
            password: self.save_password.then(|| self.password.clone()),
        };

        if !self.save_password {
            if let Err(e) = self.store.set_password(&conn.name, &self.password) {
                self.status_message = Some((format!("Failed to save password: {}", e), true));
                return;
            }
        }

        match self.saved_connections.iter().position(|c| c.name == conn.name) {
            Some(idx) => self.saved_connections[idx] = conn,
            None => self.saved_connections.push(conn),
        }

        match self.store.save_connections(&self.saved_connections) {
            Ok(()) => self.status_message = Some(("Connection saved".into(), false)),
            Err(e) => self.status_message = Some((format!("Failed to save: {}", e), true)),
        }
    }

    fn delete_selected(&mut self) {
        let Some(idx) = self.selected_index else { return };
        let name = self.saved_connections[idx].name.clone();
        let _ = self.store.delete_password(&name);
        self.saved_connections.remove(idx);
        let _ = self.store.save_connections(&self.saved_connections);
        self.clear_form();
        self.status_message = Some(("Connection deleted".into(), false));
    }

    fn build_config(&self) -> Option<ConnectionConfig> {
        Some(ConnectionConfig {
            db_type: self.db_type,
            host: self.host.clone(),
            port: self.port.parse().ok()?,
            user: self.user.clone(),
            password: self.password.clone(),
            database: self.database.clone(),
            schema: self.schema.trim().to_string(),
        })
    }

    pub fn show(&mut self, ctx: &egui::Context, db_tx: &mpsc::UnboundedSender<DbRequest>) -> Option<ConnectionConfig> {
        if !self.show { return None; }
        
        let mut result = None;

        egui::Window::new("Connect to Database")
            .collapsible(false)
            .resizable(true)
            .default_width(550.0)
            .min_width(450.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(150.0);
                        ui.label("Saved Connections");
                        ui.add_space(4.0);
                        
                        egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                            let mut clicked_idx = None;
                            for (idx, conn) in self.saved_connections.iter().enumerate() {
                                let icon = match conn.db_type {
                                    DatabaseType::PostgreSQL => "🐘",
                                    DatabaseType::MySQL => "🐬",
                                };
                                if ui.selectable_label(
                                    self.selected_index == Some(idx),
                                    format!("{} {}", icon, conn.name)
                                ).clicked() {
                                    clicked_idx = Some(idx);
                                }
                            }
                            if let Some(idx) = clicked_idx {
                                self.selected_index = Some(idx);
                                self.load_selected();
                                self.status_message = None;
                            }
                        });
                        
                        ui.add_space(8.0);
                        if ui.button("New Connection").clicked() {
                            self.clear_form();
                            self.status_message = None;
                        }
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.set_min_width(320.0);
                        
                        egui::Grid::new("connection_grid")
                            .num_columns(2)
                            .spacing([12.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("Name:");
                                ui.add(egui::TextEdit::singleline(&mut self.connection_name)
                                    .desired_width(200.0).hint_text("My Database"));
                                ui.end_row();

                                ui.label("Database Type:");
                                let mut type_changed = false;
                                egui::ComboBox::from_id_salt("db_type")
                                    .selected_text(self.db_type.display_name())
                                    .width(200.0)
                                    .show_ui(ui, |ui| {
                                        for db_type in DatabaseType::all() {
                                            let icon = match db_type {
                                                DatabaseType::PostgreSQL => "🐘",
                                                DatabaseType::MySQL => "🐬",
                                            };
                                            if ui.selectable_value(&mut self.db_type, *db_type, 
                                                format!("{} {}", icon, db_type.display_name())).changed() {
                                                type_changed = true;
                                            }
                                        }
                                    });
                                ui.end_row();
                                if type_changed { self.on_db_type_changed(); }

                                for (label, value, hint) in [
                                    ("Host:", &mut self.host, None::<&str>),
                                    ("Port:", &mut self.port, None),
                                    ("User:", &mut self.user, None),
                                ] {
                                    ui.label(label);
                                    let mut edit = egui::TextEdit::singleline(value).desired_width(200.0);
                                    if let Some(h) = hint { edit = edit.hint_text(h); }
                                    ui.add(edit);
                                    ui.end_row();
                                }

                                ui.label("Password:");
                                ui.add(egui::TextEdit::singleline(&mut self.password)
                                    .desired_width(200.0).password(true));
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(&mut self.save_password, "Save password (insecure)");
                                ui.end_row();

                                ui.label("Database:");
                                ui.add(egui::TextEdit::singleline(&mut self.database).desired_width(200.0));
                                ui.end_row();

                                if self.db_type == DatabaseType::PostgreSQL {
                                    ui.label("Schema:");
                                    ui.add(egui::TextEdit::singleline(&mut self.schema)
                                        .desired_width(200.0).hint_text("optional"));
                                    ui.end_row();
                                }
                            });

                        ui.add_space(12.0);

                        if let Some((msg, is_error)) = &self.status_message {
                            let color = if *is_error {
                                egui::Color32::from_rgb(255, 100, 100)
                            } else {
                                egui::Color32::from_rgb(100, 255, 100)
                            };
                            ui.colored_label(color, msg);
                            ui.add_space(4.0);
                        }

                        ui.horizontal(|ui| {
                            if ui.button("💾 Save").clicked() { self.save_current(); }
                            if self.selected_index.is_some() && ui.button("🗑 Delete").clicked() {
                                self.delete_selected();
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            let label = if self.testing { "Testing..." } else { "Test Connection" };
                            if ui.add_enabled(!self.testing, egui::Button::new(label)).clicked() {
                                if let Some(config) = self.build_config() {
                                    self.testing = true;
                                    self.status_message = Some(("Testing connection...".into(), false));
                                    let _ = db_tx.send(DbRequest::TestConnection(config));
                                } else {
                                    self.status_message = Some(("Invalid port".into(), true));
                                }
                            }
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new("Connect").min_size(egui::vec2(80.0, 28.0))).clicked() {
                                if let Some(config) = self.build_config() {
                                    let name = self.connection_name.trim();
                                    if !name.is_empty() { let _ = self.store.set_last_used(name); }
                                    result = Some(config);
                                    self.show = false;
                                    self.status_message = None;
                                } else {
                                    self.status_message = Some(("Invalid port".into(), true));
                                }
                            }
                            if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(80.0, 28.0))).clicked() {
                                self.show = false;
                                self.status_message = None;
                            }
                        });
                    });
                });
            });

        result
    }
}
