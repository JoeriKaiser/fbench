use crate::config::{ConnectionStore, SavedConnection};
use crate::db::{ConnectionConfig, DatabaseType as DbType};
use crate::services::DbSender;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn ConnectionDialog() -> Element {
    rsx! {
        if *SHOW_CONNECTION_DIALOG.read() {
        div {
            class: "fixed inset-0 bg-black bg-opacity-80 flex items-center justify-center z-50",
            onclick: move |_| {
                *SHOW_CONNECTION_DIALOG.write() = false;
                *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Idle;
            },

            div {
                class: "rounded-lg shadow-2xl w-[500px] max-w-[90vw]",
                class: if *IS_DARK_MODE.read() { "bg-black border border-gray-800" } else { "bg-white border border-gray-300" },
                onclick: move |e| e.stop_propagation(),

                ConnectionDialogContent {}
            }
        }
        }
    }
}

#[component]
fn ConnectionDialogContent() -> Element {
    let is_dark = *IS_DARK_MODE.read();
    let mut store = use_signal(ConnectionStore::new);
    let mut saved_connections = use_signal(|| store.read().load_connections());

    let mut db_type = use_signal(|| DbType::PostgreSQL);
    let mut host = use_signal(|| "localhost".to_string());
    let mut port = use_signal(|| 5432u16);
    let mut user = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut database = use_signal(String::new);
    let mut schema = use_signal(String::new);
    let mut save_password = use_signal(|| false);
    let mut connection_name = use_signal(String::new);

    // Theme-aware classes
    let _bg_class = if is_dark { "bg-black" } else { "bg-white" };
    let text_class = if is_dark {
        "text-white"
    } else {
        "text-gray-900"
    };
    let label_class = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-600"
    };
    let input_class = if is_dark {
        "bg-black border-gray-800 text-white focus:border-white"
    } else {
        "bg-white border-gray-300 text-gray-900 focus:border-blue-500"
    };
    let select_class = if is_dark {
        "bg-black border-gray-800 text-white focus:border-white appearance-none"
    } else {
        "bg-white border-gray-300 text-gray-900 focus:border-blue-500 appearance-none"
    };
    let secondary_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let divider_class = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };

    // Update port when db_type changes
    use_effect(move || {
        let new_port = match db_type() {
            DbType::PostgreSQL => 5432,
            DbType::MySQL => 3306,
        };
        port.set(new_port);
    });

    // Reset test status when dialog opens
    use_effect(move || {
        *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Idle;
    });

    let validate_inputs = move || -> Result<(), String> {
        if host.read().trim().is_empty() {
            return Err("Host is required".to_string());
        }
        if user.read().trim().is_empty() {
            return Err("Username is required".to_string());
        }
        if database.read().trim().is_empty() {
            return Err("Database name is required".to_string());
        }
        Ok(())
    };

    let connect = move || {
        if let Err(e) = validate_inputs() {
            *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Failed(e);
            return;
        }

        let config = ConnectionConfig {
            db_type: db_type(),
            host: host.read().clone(),
            port: *port.read(),
            user: user.read().clone(),
            password: password.read().clone(),
            database: database.read().clone(),
            schema: schema.read().clone(),
        };

        *CONNECTION.write() = ConnectionState::Connecting;

        if let Some(tx) = try_use_context::<DbSender>() {
            let _ = tx.send(crate::db::DbRequest::Connect(config));
        }

        *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Idle;
        *SHOW_CONNECTION_DIALOG.write() = false;
    };

    let mut save_and_connect = move || {
        if let Err(e) = validate_inputs() {
            *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Failed(e);
            return;
        }

        let name = connection_name.read().trim().to_string();
        if name.is_empty() {
            *TEST_CONNECTION_STATUS.write() =
                TestConnectionStatus::Failed("Please enter a connection name to save".to_string());
            return;
        }

        let config = ConnectionConfig {
            db_type: db_type(),
            host: host.read().clone(),
            port: *port.read(),
            user: user.read().clone(),
            password: password.read().clone(),
            database: database.read().clone(),
            schema: schema.read().clone(),
        };

        *CONNECTION.write() = ConnectionState::Connecting;

        if let Some(tx) = try_use_context::<DbSender>() {
            let _ = tx.send(crate::db::DbRequest::Connect(config));
        }

        // Save connection (update existing or add new)
        let saved = SavedConnection {
            name: name.clone(),
            db_type: db_type(),
            host: host.read().clone(),
            port: *port.read(),
            user: user.read().clone(),
            database: database.read().clone(),
            schema: schema.read().clone(),
            save_password: save_password(),
            password: if save_password() {
                Some(password.read().clone())
            } else {
                None
            },
        };

        let st = store.write();
        let mut conns = st.load_connections();

        // Check if connection with this name already exists
        if let Some(existing) = conns.iter_mut().find(|c| c.name == name) {
            // Update existing connection
            *existing = saved;
        } else {
            // Add new connection
            conns.push(saved);
        }

        let _ = st.save_connections(&conns);

        if save_password() {
            let _ = st.set_password(&name, &password.read());
        }

        // Refresh saved connections list
        saved_connections.set(conns);

        *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Idle;
        *SHOW_CONNECTION_DIALOG.write() = false;
    };

    let test_connection = move || {
        if let Err(e) = validate_inputs() {
            *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Failed(e);
            return;
        }

        let config = ConnectionConfig {
            db_type: db_type(),
            host: host.read().clone(),
            port: *port.read(),
            user: user.read().clone(),
            password: password.read().clone(),
            database: database.read().clone(),
            schema: schema.read().clone(),
        };

        *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Testing;

        if let Some(tx) = try_use_context::<DbSender>() {
            let _ = tx.send(crate::db::DbRequest::TestConnection(config));
        }
    };

    rsx! {
        div {
            class: "p-6 space-y-4",

            h2 {
                class: "text-lg font-semibold {text_class} mb-4",
                "Database Connection"
            }

            // Saved connections dropdown
            if !saved_connections.read().is_empty() {
                div {
                    class: "mb-4",
                    label {
                        class: "block text-sm font-medium {label_class} mb-1",
                        "Saved Connections"
                    }
                    select {
                        class: "w-full px-3 py-2 border rounded text-sm focus:outline-none {select_class}",
                        onchange: move |e| {
                            let value = e.value();
                            if let Some(conn) = saved_connections.read().iter().find(|c| c.name == value) {
                                db_type.set(conn.db_type);
                                host.set(conn.host.clone());
                                port.set(conn.port);
                                user.set(conn.user.clone());
                                database.set(conn.database.clone());
                                schema.set(conn.schema.clone());
                                connection_name.set(conn.name.clone());

                                // Load password from keyring if not saved
                                if conn.save_password {
                                    if let Some(pwd) = conn.password.clone() {
                                        password.set(pwd);
                                    }
                                } else {
                                    let st = store.read();
                                    if let Some(pwd) = st.get_password(&conn.name) {
                                        password.set(pwd);
                                    }
                                }

                                // Reset test status when loading a saved connection
                                *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Idle;
                            }
                        },
                        option {
                            class: if is_dark { "bg-black text-white" } else { "bg-white text-gray-900" },
                            value: "",
                            "Select a saved connection..."
                        }
                        for conn in (*saved_connections.read()).iter() {
                            option {
                                class: if is_dark { "bg-black text-white" } else { "bg-white text-gray-900" },
                                value: "{conn.name}",
                                "{conn.name}"
                            }
                        }
                    }
                }
            }

            // Database type
            div {
                label {
                    class: "block text-sm font-medium {label_class} mb-1",
                    "Database Type"
                }
                div {
                    class: "flex space-x-4",

                    label {
                        class: "flex items-center space-x-2 cursor-pointer",
                        input {
                            r#type: "radio",
                            name: "db_type",
                            checked: db_type() == DbType::PostgreSQL,
                            onchange: move |_| db_type.set(DbType::PostgreSQL),
                        }
                        span { class: "text-sm {secondary_text}", "PostgreSQL" }
                    }

                    label {
                        class: "flex items-center space-x-2 cursor-pointer",
                        input {
                            r#type: "radio",
                            name: "db_type",
                            checked: db_type() == DbType::MySQL,
                            onchange: move |_| db_type.set(DbType::MySQL),
                        }
                        span { class: "text-sm {secondary_text}", "MySQL" }
                    }
                }
            }

            // Host and port
            div {
                class: "grid grid-cols-3 gap-4",

                div {
                    class: "col-span-2",
                    label {
                        class: "block text-sm font-medium {label_class} mb-1",
                        "Host *"
                    }
                    input {
                        class: "w-full px-3 py-2 border rounded text-sm focus:outline-none {input_class}",
                        r#type: "text",
                        value: "{host}",
                        oninput: move |e| host.set(e.value().clone()),
                    }
                }

                div {
                    label {
                        class: "block text-sm font-medium {label_class} mb-1",
                        "Port"
                    }
                    input {
                        class: "w-full px-3 py-2 border rounded text-sm focus:outline-none {input_class}",
                        r#type: "number",
                        value: "{port}",
                        oninput: move |e| {
                            if let Ok(p) = e.value().parse::<u16>() {
                                port.set(p);
                            }
                        },
                    }
                }
            }

            // User and password
            div {
                class: "grid grid-cols-2 gap-4",

                div {
                    label {
                        class: "block text-sm font-medium {label_class} mb-1",
                        "Username *"
                    }
                    input {
                        class: "w-full px-3 py-2 border rounded text-sm focus:outline-none {input_class}",
                        r#type: "text",
                        value: "{user}",
                        oninput: move |e| user.set(e.value().clone()),
                    }
                }

                div {
                    label {
                        class: "block text-sm font-medium {label_class} mb-1",
                        "Password"
                    }
                    input {
                        class: "w-full px-3 py-2 border rounded text-sm focus:outline-none {input_class}",
                        r#type: "password",
                        value: "{password}",
                        oninput: move |e| password.set(e.value().clone()),
                    }
                }
            }

            // Database
            div {
                label {
                    class: "block text-sm font-medium {label_class} mb-1",
                    "Database *"
                }
                input {
                    class: "w-full px-3 py-2 border rounded text-sm focus:outline-none {input_class}",
                    r#type: "text",
                    value: "{database}",
                    oninput: move |e| database.set(e.value().clone()),
                }
            }

            // Schema (PostgreSQL only)
            if db_type() == DbType::PostgreSQL {
                div {
                    label {
                        class: "block text-sm font-medium {label_class} mb-1",
                        "Schema (optional)"
                    }
                    input {
                        class: "w-full px-3 py-2 border rounded text-sm focus:outline-none {input_class}",
                        r#type: "text",
                        value: "{schema}",
                        oninput: move |e| schema.set(e.value().clone()),
                    }
                }
            }

            // Divider
            div {
                class: "border-t pt-4 mt-4 {divider_class}",
            }

            // Save connection section
            div {
                label {
                    class: "block text-sm font-medium {label_class} mb-1",
                    "Save Connection Name (optional)"
                }
                input {
                    class: "w-full px-3 py-2 border rounded text-sm focus:outline-none {input_class}",
                    r#type: "text",
                    placeholder: "My Production DB",
                    value: "{connection_name}",
                    oninput: move |e| connection_name.set(e.value().clone()),
                }

                label {
                    class: "flex items-center space-x-2 mt-2 cursor-pointer",
                    input {
                        r#type: "checkbox",
                        checked: save_password(),
                        onchange: move |_| save_password.set(!save_password()),
                    }
                    span { class: "text-sm {secondary_text}", "Save password in keychain" }
                }
            }

            // Test status
            TestStatusMessage {}

            // Buttons
            div {
                class: "flex justify-end space-x-3 pt-4 border-t {divider_class}",

                button {
                    class: if is_dark {
                        "px-4 py-2 text-sm rounded transition-colors bg-gray-900 hover:bg-gray-800 text-white"
                    } else {
                        "px-4 py-2 text-sm rounded transition-colors bg-gray-100 hover:bg-gray-200 text-gray-700"
                    },
                    onclick: move |_| {
                        *SHOW_CONNECTION_DIALOG.write() = false;
                        *TEST_CONNECTION_STATUS.write() = TestConnectionStatus::Idle;
                    },
                    "Cancel"
                }

                button {
                    class: if is_dark {
                        "px-4 py-2 text-sm rounded transition-colors bg-gray-900 hover:bg-gray-800 text-white"
                    } else {
                        "px-4 py-2 text-sm rounded transition-colors bg-gray-100 hover:bg-gray-200 text-gray-700"
                    },
                    disabled: matches!(*TEST_CONNECTION_STATUS.read(), TestConnectionStatus::Testing),
                    onclick: move |_| test_connection(),
                    "Test"
                }

                button {
                    class: if is_dark {
                        "px-4 py-2 text-sm rounded transition-colors bg-white hover:bg-gray-200 text-black"
                    } else {
                        "px-4 py-2 text-sm rounded transition-colors bg-gray-800 hover:bg-gray-700 text-white"
                    },
                    onclick: move |_| connect(),
                    "Connect"
                }

                button {
                    class: "px-4 py-2 text-sm rounded transition-colors bg-blue-600 hover:bg-blue-500 text-white",
                    onclick: move |_| save_and_connect(),
                    "Save & Connect"
                }
            }
        }
    }
}

#[component]
fn TestStatusMessage() -> Element {
    let status = TEST_CONNECTION_STATUS.read().clone();

    if matches!(status, TestConnectionStatus::Idle) {
        return rsx! {};
    }

    rsx! {
        div {
            class: "text-sm",
            match status {
                TestConnectionStatus::Testing => rsx! {
                    span { class: "text-yellow-500", "Testing connection..." }
                },
                TestConnectionStatus::Success => rsx! {
                    span { class: "text-green-500", "Connection successful!" }
                },
                TestConnectionStatus::Failed(ref e) => rsx! {
                    span { class: "text-red-500", "{e}" }
                },
                _ => rsx! {}
            }
        }
    }
}
