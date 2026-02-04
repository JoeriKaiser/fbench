use crate::import::{self, ImportData};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn ImportDialog() -> Element {
    let show = *SHOW_IMPORT_DIALOG.read();
    if !show {
        return rsx! {};
    }

    let mut step = use_signal(|| 0usize);
    let mut import_data = use_signal(|| None::<ImportData>);
    let mut target_table = use_signal(|| String::new());
    let mut column_mapping = use_signal(|| Vec::<(usize, String)>::new());
    let mut error_msg = use_signal(|| None::<String>);

    let is_dark = *IS_DARK_MODE.read();
    let progress = IMPORT_PROGRESS.read().clone();

    let bg = if is_dark { "bg-gray-900" } else { "bg-white" };
    let text = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };
    let muted = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-400"
    };
    let input_bg = if is_dark { "bg-gray-800" } else { "bg-gray-50" };
    let input_border = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-300"
    };

    rsx! {
        // Modal overlay
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| close_dialog(),

            div {
                class: "{bg} rounded-lg shadow-xl w-11/12 max-w-3xl max-h-screen overflow-auto p-6",
                style: "max-height: 80vh",
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    class: "flex items-center justify-between mb-4",
                    h2 { class: "text-lg font-semibold {text}", "Import Data" }
                    button {
                        class: "{muted} hover:opacity-80 text-xl",
                        onclick: move |_| close_dialog(),
                        "✕"
                    }
                }

                // Error message
                if let Some(err) = error_msg.read().as_ref() {
                    div {
                        class: "mb-4 p-2 rounded bg-red-900 bg-opacity-30 text-red-400 text-sm",
                        "{err}"
                    }
                }

                // Step 0: File selection
                if *step.read() == 0 {
                    div {
                        class: "space-y-4",
                        p { class: "{muted} text-sm", "Select a CSV or JSON file to import." }
                        button {
                            class: "px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-500",
                            onclick: move |_| {
                                spawn(async move {
                                    let file = rfd::AsyncFileDialog::new()
                                        .add_filter("Data files", &["csv", "json"])
                                        .pick_file()
                                        .await;

                                    if let Some(file) = file {
                                        let path = file.path().to_path_buf();
                                        match import::parse_file(&path) {
                                            Ok(data) => {
                                                *import_data.write() = Some(data);
                                                *error_msg.write() = None;
                                                *step.write() = 1;
                                            }
                                            Err(e) => {
                                                *error_msg.write() = Some(e.to_string());
                                            }
                                        }
                                    }
                                });
                            },
                            "Choose File..."
                        }
                    }
                }

                // Step 1: Target table selection
                if *step.read() == 1 {
                    {
                        let schema = SCHEMA.read();
                        let data = import_data.read();
                        let file_cols = data.as_ref().map(|d| d.columns.len()).unwrap_or(0);
                        let file_rows = data.as_ref().map(|d| d.rows.len()).unwrap_or(0);

                        rsx! {
                            div {
                                class: "space-y-4",
                                p {
                                    class: "{muted} text-sm",
                                    "File has {file_cols} columns and {file_rows} rows. Select a target table."
                                }

                                select {
                                    class: "w-full px-3 py-2 rounded {input_bg} {input_border} {text} border",
                                    value: "{target_table}",
                                    onchange: move |evt: FormEvent| {
                                        *target_table.write() = evt.value();
                                        // Auto-map columns
                                        let table_name = evt.value();
                                        let schema = SCHEMA.read();
                                        if let Some(table_info) = schema.tables.iter().find(|t| t.name == table_name) {
                                            if let Some(data) = import_data.read().as_ref() {
                                                let mapping = import::auto_map_columns(&data.columns, &table_info.columns);
                                                *column_mapping.write() = mapping;
                                            }
                                        }
                                    },
                                    option { value: "", "Select table..." }
                                    for table in schema.tables.iter() {
                                        option {
                                            value: "{table.name}",
                                            "{table.name} ({table.columns.len()} cols)"
                                        }
                                    }
                                }

                                div {
                                    class: "flex justify-between",
                                    button {
                                        class: "px-3 py-1 rounded {muted} hover:opacity-80",
                                        onclick: move |_| *step.write() = 0,
                                        "Back"
                                    }
                                    button {
                                        class: "px-3 py-1 rounded bg-blue-600 text-white hover:bg-blue-500 disabled:opacity-50",
                                        disabled: target_table.read().is_empty(),
                                        onclick: move |_| *step.write() = 2,
                                        "Next"
                                    }
                                }
                            }
                        }
                    }
                }

                // Step 2: Column mapping
                if *step.read() == 2 {
                    {
                        let schema = SCHEMA.read();
                        let data = import_data.read();
                        let table_name = target_table.read().clone();
                        let table_info = schema.tables.iter().find(|t| t.name == table_name);
                        let table_columns: Vec<String> = table_info
                            .map(|t| t.columns.iter().map(|c| c.name.clone()).collect())
                            .unwrap_or_default();
                        let file_columns = data.as_ref().map(|d| d.columns.clone()).unwrap_or_default();
                        let current_mapping = column_mapping.read().clone();

                        rsx! {
                            div {
                                class: "space-y-4",
                                p { class: "{muted} text-sm", "Map file columns to table columns." }

                                div {
                                    class: "space-y-2 max-h-64 overflow-auto",
                                    for (idx, file_col) in file_columns.iter().enumerate() {
                                        {
                                            let mapped_to = current_mapping
                                                .iter()
                                                .find(|(i, _)| *i == idx)
                                                .map(|(_, c)| c.clone())
                                                .unwrap_or_default();
                                            rsx! {
                                                div {
                                                    class: "flex items-center space-x-3",
                                                    span { class: "w-40 text-sm {text} truncate", "{file_col}" }
                                                    span { class: "{muted}", "\u{2192}" }
                                                    select {
                                                        class: "flex-1 px-2 py-1 rounded text-sm {input_bg} {input_border} {text} border",
                                                        value: "{mapped_to}",
                                                        onchange: move |evt: FormEvent| {
                                                            let val = evt.value();
                                                            let mut mapping = column_mapping.write();
                                                            mapping.retain(|(i, _)| *i != idx);
                                                            if !val.is_empty() {
                                                                mapping.push((idx, val));
                                                            }
                                                        },
                                                        option { value: "", "Skip" }
                                                        for table_col in &table_columns {
                                                            option {
                                                                value: "{table_col}",
                                                                selected: mapped_to == *table_col,
                                                                "{table_col}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div {
                                    class: "flex justify-between",
                                    button {
                                        class: "px-3 py-1 rounded {muted} hover:opacity-80",
                                        onclick: move |_| *step.write() = 1,
                                        "Back"
                                    }
                                    button {
                                        class: "px-3 py-1 rounded bg-blue-600 text-white hover:bg-blue-500 disabled:opacity-50",
                                        disabled: column_mapping.read().is_empty(),
                                        onclick: move |_| *step.write() = 3,
                                        "Next"
                                    }
                                }
                            }
                        }
                    }
                }

                // Step 3: Preview and execute
                if *step.read() == 3 {
                    {
                        let data = import_data.read();
                        let mapping = column_mapping.read().clone();
                        let table_name = target_table.read().clone();

                        // Build preview: mapped columns and first 10 rows
                        let mapped_cols: Vec<String> = mapping.iter().map(|(_, c)| c.clone()).collect();
                        let preview_rows: Vec<Vec<String>> = data
                            .as_ref()
                            .map(|d| {
                                d.rows
                                    .iter()
                                    .take(10)
                                    .map(|row| {
                                        mapping
                                            .iter()
                                            .map(|(idx, _)| {
                                                row.get(*idx).cloned().unwrap_or_else(|| "NULL".to_string())
                                            })
                                            .collect()
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        let total_rows = data.as_ref().map(|d| d.rows.len()).unwrap_or(0);

                        rsx! {
                            div {
                                class: "space-y-4",
                                p {
                                    class: "{muted} text-sm",
                                    "Preview: {mapped_cols.len()} columns, {total_rows} rows total (showing first 10)"
                                }

                                // Preview table
                                div {
                                    class: "overflow-auto max-h-48 border rounded {input_border}",
                                    table {
                                        class: "w-full text-xs text-left",
                                        thead {
                                            class: "{input_bg}",
                                            tr {
                                                for col in &mapped_cols {
                                                    th { class: "px-2 py-1 font-medium {text}", "{col}" }
                                                }
                                            }
                                        }
                                        tbody {
                                            for row in &preview_rows {
                                                tr {
                                                    for cell in row {
                                                        td { class: "px-2 py-1 font-mono {text}", "{cell}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Progress bar
                                if let Some((inserted, total)) = progress {
                                    div {
                                        class: "space-y-1",
                                        div {
                                            class: "text-sm {text}",
                                            "Importing... {inserted}/{total}"
                                        }
                                        div {
                                            class: "w-full h-2 rounded {input_bg}",
                                            {
                                                let pct = if total > 0 { (inserted * 100) / total } else { 0 };
                                                rsx! {
                                                    div {
                                                        class: "h-2 rounded bg-blue-600",
                                                        style: "width: {pct}%",
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div {
                                    class: "flex justify-between",
                                    button {
                                        class: "px-3 py-1 rounded {muted} hover:opacity-80",
                                        onclick: move |_| *step.write() = 2,
                                        "Back"
                                    }
                                    button {
                                        class: "px-4 py-2 rounded bg-green-700 text-white hover:bg-green-600",
                                        disabled: progress.is_some(),
                                        onclick: {
                                            let table_name = table_name.clone();
                                            let mapping = mapping.clone();
                                            move |_| {
                                                execute_import(
                                                    &table_name,
                                                    &mapping,
                                                    &import_data.read(),
                                                );
                                            }
                                        },
                                        "Import {total_rows} rows"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn execute_import(table_name: &str, mapping: &[(usize, String)], data: &Option<ImportData>) {
    let data = match data {
        Some(d) => d,
        None => return,
    };

    let columns: Vec<String> = mapping.iter().map(|(_, c)| c.clone()).collect();
    let rows: Vec<Vec<String>> = data
        .rows
        .iter()
        .map(|row| {
            mapping
                .iter()
                .map(|(idx, _)| row.get(*idx).cloned().unwrap_or_else(|| "NULL".to_string()))
                .collect()
        })
        .collect();

    let batch_size = 100;
    send_db_request(crate::db::DbRequest::ImportData {
        table: table_name.to_string(),
        columns,
        rows,
        batch_size,
    });
}

fn close_dialog() {
    *SHOW_IMPORT_DIALOG.write() = false;
}
