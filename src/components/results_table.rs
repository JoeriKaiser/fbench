use crate::components::filter_panel::{toggle_sort, FilterPanel};
use crate::filter::SortDirection;
use crate::state::tabs::CellEdit;
use crate::state::*;
use dioxus::prelude::*;
use std::collections::HashMap;

pub static EDITING_CELL: GlobalSignal<Option<(usize, usize)>> = Signal::global(|| None);
pub static SELECTED_ROWS: GlobalSignal<std::collections::HashSet<usize>> =
    Signal::global(Default::default);
pub static INSERTING_ROW: GlobalSignal<bool> = Signal::global(|| false);

struct FkLink {
    foreign_table: String,
    column_mapping: Vec<(String, String)>,
}

fn detect_fk_columns(source_table: &str, result_columns: &[String]) -> HashMap<usize, FkLink> {
    let schema = SCHEMA.read();
    let mut fk_map = HashMap::new();

    let table_info = schema.tables.iter().find(|t| t.name == source_table);
    let Some(table) = table_info else {
        return fk_map;
    };

    for constraint in &table.constraints {
        if constraint.constraint_type != "FOREIGN KEY" {
            continue;
        }
        let Some(ref foreign_table) = constraint.foreign_table else {
            continue;
        };
        let Some(ref foreign_columns) = constraint.foreign_columns else {
            continue;
        };

        for (local_col, _foreign_col) in constraint.columns.iter().zip(foreign_columns.iter()) {
            if let Some(col_idx) = result_columns.iter().position(|c| c == local_col) {
                let mapping: Vec<(String, String)> = constraint
                    .columns
                    .iter()
                    .zip(foreign_columns.iter())
                    .map(|(l, f)| (l.clone(), f.clone()))
                    .collect();
                fk_map.insert(
                    col_idx,
                    FkLink {
                        foreign_table: foreign_table.clone(),
                        column_mapping: mapping,
                    },
                );
                break;
            }
        }
    }

    fk_map
}

fn navigate_fk(
    foreign_table: &str,
    column_mapping: &[(String, String)],
    row: &[String],
    result_columns: &[String],
) {
    let conditions: Vec<String> = column_mapping
        .iter()
        .filter_map(|(local_col, foreign_col)| {
            let col_idx = result_columns.iter().position(|c| c == local_col)?;
            let value = row.get(col_idx)?;
            if value == "NULL" {
                None
            } else {
                Some(format!("{} = '{}'", foreign_col, value.replace('\'', "''")))
            }
        })
        .collect();

    if conditions.is_empty() {
        return;
    }

    let sql = format!(
        "SELECT * FROM {} WHERE {}",
        foreign_table,
        conditions.join(" AND ")
    );

    let tab_title = format!(
        "{} [{}]",
        foreign_table,
        conditions
            .first()
            .map(|c| c.split(" = ").last().unwrap_or("?").trim_matches('\''))
            .unwrap_or("?")
    );

    let tab_sql = sql.clone();
    {
        let mut tabs = EDITOR_TABS.write();
        let id = tabs.add_tab(tab_title);
        if let Some(tab) = tabs.tabs.iter_mut().find(|t| t.id == id) {
            tab.content = tab_sql;
        }
    }

    send_db_request(crate::db::DbRequest::Execute(sql));
}

#[component]
pub fn ResultsTable() -> Element {
    let tabs = EDITOR_TABS.read();
    let active_tab = tabs.active_tab();
    let result = active_tab.and_then(|t| t.result.clone());
    let error = active_tab.and_then(|t| t.last_error.clone());
    let exec_time = active_tab.and_then(|t| t.execution_time_ms);
    let current_sort = active_tab
        .and_then(|t| t.filter_state.as_ref())
        .and_then(|s| s.sort.clone());
    let has_source_table = result
        .as_ref()
        .map(|r| r.source_table.is_some())
        .unwrap_or(false);
    let can_edit = result
        .as_ref()
        .map(|r| r.source_table.is_some() && !r.primary_keys.is_empty())
        .unwrap_or(false);
    let edit_mode = active_tab.map(|t| t.edit_mode).unwrap_or(false);
    let pending_edits = active_tab
        .map(|t| t.pending_edits.clone())
        .unwrap_or_default();
    let selected_rows = SELECTED_ROWS.read().clone();
    let inserting = *INSERTING_ROW.read();
    let is_dark = *IS_DARK_MODE.read();

    // Theme-aware classes
    let header_bg = if is_dark { "bg-black" } else { "bg-gray-50" };
    let header_border = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };
    let header_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let row_alt = if is_dark { "bg-gray-950" } else { "bg-gray-50" };
    let cell_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-700"
    };
    let muted_text = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };
    let table_divider = if is_dark {
        "divide-gray-800"
    } else {
        "divide-gray-200"
    };

    rsx! {
        div {
            class: "flex flex-col h-full",

            div {
                class: "h-8 {header_bg} border-b {header_border} flex items-center px-3 justify-between",

                if let Some(error) = error {
                    span { class: "text-red-500 text-sm", "{error}" }
                } else if let Some(ref result) = result {
                    span { class: "{header_text} text-sm", "{result.rows.len()} rows" }
                } else {
                    span { class: "{muted_text} text-sm", "No results" }
                }

                div {
                    class: "flex items-center space-x-3",

                    if let Some(exec_time) = exec_time {
                        span {
                            class: "text-xs {muted_text}",
                            "{exec_time}ms"
                        }
                    }

                    // Edit mode controls
                    if can_edit {
                        {
                            let edit_btn_class = if edit_mode {
                                "text-xs px-2 py-1 rounded bg-yellow-600 text-white"
                            } else {
                                "text-xs px-2 py-1 rounded"
                            };
                            rsx! {
                                button {
                                    class: "{edit_btn_class} {header_text}",
                                    onclick: move |_| {
                                        if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
                                            tab.edit_mode = !tab.edit_mode;
                                            if !tab.edit_mode {
                                                tab.pending_edits.clear();
                                            }
                                        }
                                        *SELECTED_ROWS.write() = Default::default();
                                        *EDITING_CELL.write() = None;
                                        *INSERTING_ROW.write() = false;
                                    },
                                    if edit_mode { "Editing" } else { "Edit" }
                                }
                            }
                        }
                    }

                    // Save/Discard when there are pending edits
                    if !pending_edits.is_empty() {
                        span {
                            class: "text-xs {muted_text}",
                            "{pending_edits.len()} changes"
                        }
                        button {
                            class: "text-xs px-2 py-1 rounded bg-green-700 text-white hover:bg-green-600",
                            onclick: move |_| save_pending_edits(),
                            "Save"
                        }
                        button {
                            class: "text-xs px-2 py-1 rounded bg-red-700 text-white hover:bg-red-600",
                            onclick: move |_| discard_pending_edits(),
                            "Discard"
                        }
                    }

                    // Insert/Delete row buttons in edit mode
                    if edit_mode {
                        button {
                            class: "text-xs px-2 py-1 rounded {header_text} hover:opacity-80",
                            onclick: move |_| *INSERTING_ROW.write() = true,
                            "+ Row"
                        }
                        if !selected_rows.is_empty() {
                            button {
                                class: "text-xs px-2 py-1 rounded text-red-500 hover:text-red-400",
                                onclick: move |_| delete_selected_rows(),
                                "Delete ({selected_rows.len()})"
                            }
                        }
                    }

                    // Explain button (only when we have results)
                    if result.is_some() {
                        button {
                            class: "text-xs px-2 py-1 rounded {header_bg} {header_text} hover:opacity-80",
                            onclick: move |_| show_execution_plan(),
                            "Explain"
                        }
                    }
                }
            }

            // Filter panel (only for single-table queries)
            FilterPanel {}

            div {
                class: "flex-1 overflow-auto",

                if let Some(result) = result {
                    {
                        // Detect FK columns for link rendering
                        let fk_map = result
                            .source_table
                            .as_ref()
                            .map(|t| detect_fk_columns(t, &result.columns))
                            .unwrap_or_default();

                        let total_rows = result.rows.len();
                        rsx! {
                            table {
                                class: "w-full text-sm text-left",

                                thead {
                                    class: "{header_bg} {header_text} sticky top-0",
                                    tr {
                                        // Checkbox column in edit mode
                                        if edit_mode {
                                            th {
                                                class: "px-2 py-2 w-8 border-b {header_border}",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: selected_rows.len() == total_rows && total_rows > 0,
                                                    onchange: move |_| {
                                                        let mut sel = SELECTED_ROWS.write();
                                                        if sel.len() == total_rows {
                                                            sel.clear();
                                                        } else {
                                                            *sel = (0..total_rows).collect();
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                        for col in result.columns.clone() {
                                            {
                                                let sort_indicator = current_sort.as_ref().and_then(|s| {
                                                    if s.column == col {
                                                        Some(match s.direction {
                                                            SortDirection::Asc => "\u{25B2}",
                                                            SortDirection::Desc => "\u{25BC}",
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                });
                                                let clickable = if has_source_table {
                                                    " cursor-pointer select-none"
                                                } else {
                                                    ""
                                                };
                                                rsx! {
                                                    th {
                                                        class: "px-4 py-2 font-medium border-b {header_border}{clickable}",
                                                        onclick: {
                                                            let col = col.clone();
                                                            move |_| {
                                                                if has_source_table {
                                                                    toggle_sort(col.clone());
                                                                }
                                                            }
                                                        },
                                                        "{col}"
                                                        if let Some(indicator) = sort_indicator {
                                                            span { class: "ml-1", "{indicator}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                tbody {
                                    class: "{table_divider}",
                                    for (row_idx, row) in result.rows.iter().enumerate() {
                                        tr {
                                            class: if row_idx % 2 == 0 { "" } else { row_alt },

                                            // Checkbox in edit mode
                                            if edit_mode {
                                                td {
                                                    class: "px-2 py-2 w-8",
                                                    input {
                                                        r#type: "checkbox",
                                                        checked: selected_rows.contains(&row_idx),
                                                        onchange: move |_| {
                                                            let mut sel = SELECTED_ROWS.write();
                                                            if sel.contains(&row_idx) {
                                                                sel.remove(&row_idx);
                                                            } else {
                                                                sel.insert(row_idx);
                                                            }
                                                        },
                                                    }
                                                }
                                            }

                                            for (col_idx, cell) in row.iter().enumerate() {
                                                {
                                                    let is_null = cell == "NULL";
                                                    let has_fk = !is_null && fk_map.contains_key(&col_idx);
                                                    let col_name = result.columns.get(col_idx).cloned().unwrap_or_default();
                                                    let has_edit = pending_edits.iter().any(|e| {
                                                        e.row_idx == row_idx && e.column == col_name
                                                    });
                                                    let display_value = if has_edit {
                                                        pending_edits
                                                            .iter()
                                                            .find(|e| e.row_idx == row_idx && e.column == col_name)
                                                            .map(|e| e.new_value.clone())
                                                            .unwrap_or_else(|| cell.clone())
                                                    } else {
                                                        cell.clone()
                                                    };
                                                    let highlight_class = if has_edit {
                                                        "bg-yellow-900 bg-opacity-30 border-l-2 border-yellow-500"
                                                    } else {
                                                        ""
                                                    };
                                                    let editing_this = *EDITING_CELL.read() == Some((row_idx, col_idx));

                                                    if editing_this && edit_mode {
                                                        let original_value = cell.clone();
                                                        let col_for_commit = col_name.clone();
                                                        rsx! {
                                                            td {
                                                                class: "px-4 py-2 {cell_text} font-mono {highlight_class}",
                                                                input {
                                                                    class: "w-full bg-transparent border border-blue-500 px-1 outline-none {cell_text} font-mono text-sm",
                                                                    value: "{display_value}",
                                                                    autofocus: true,
                                                                    onblur: {
                                                                        let original_value = original_value.clone();
                                                                        let col_for_commit = col_for_commit.clone();
                                                                        move |evt: FocusEvent| {
                                                                            // Commit is handled via oninput tracking
                                                                            *EDITING_CELL.write() = None;
                                                                        }
                                                                    },
                                                                    onkeydown: {
                                                                        let original_value = original_value.clone();
                                                                        let col_for_commit = col_for_commit.clone();
                                                                        move |evt: KeyboardEvent| {
                                                                            if evt.key() == Key::Escape {
                                                                                *EDITING_CELL.write() = None;
                                                                            }
                                                                        }
                                                                    },
                                                                    onchange: {
                                                                        let original_value = original_value.clone();
                                                                        let col_for_commit = col_for_commit.clone();
                                                                        move |evt: FormEvent| {
                                                                            let new_val = evt.value();
                                                                            commit_cell_edit(
                                                                                row_idx,
                                                                                &col_for_commit,
                                                                                &original_value,
                                                                                &new_val,
                                                                            );
                                                                            *EDITING_CELL.write() = None;
                                                                        }
                                                                    },
                                                                }
                                                            }
                                                        }
                                                    } else if is_null {
                                                        rsx! {
                                                            td {
                                                                class: "px-4 py-2 {cell_text} font-mono italic opacity-50 {highlight_class}",
                                                                ondoubleclick: move |_| {
                                                                    if edit_mode {
                                                                        *EDITING_CELL.write() = Some((row_idx, col_idx));
                                                                    }
                                                                },
                                                                "NULL"
                                                            }
                                                        }
                                                    } else if has_fk && !edit_mode {
                                                        let fk = &fk_map[&col_idx];
                                                        let foreign_table = fk.foreign_table.clone();
                                                        let column_mapping = fk.column_mapping.clone();
                                                        let row_data = row.clone();
                                                        let columns = result.columns.clone();
                                                        rsx! {
                                                            td {
                                                                class: "px-4 py-2 {cell_text} font-mono {highlight_class}",
                                                                a {
                                                                    class: "underline text-blue-500 hover:text-blue-400 cursor-pointer",
                                                                    onclick: move |_| {
                                                                        navigate_fk(
                                                                            &foreign_table,
                                                                            &column_mapping,
                                                                            &row_data,
                                                                            &columns,
                                                                        );
                                                                    },
                                                                    "{display_value}"
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        rsx! {
                                                            td {
                                                                class: "px-4 py-2 {cell_text} font-mono {highlight_class}",
                                                                ondoubleclick: move |_| {
                                                                    if edit_mode {
                                                                        *EDITING_CELL.write() = Some((row_idx, col_idx));
                                                                    }
                                                                },
                                                                "{display_value}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Insert row form at the bottom
                                    if inserting && edit_mode {
                                        InsertRowForm {
                                            columns: result.columns.clone(),
                                            source_table: result.source_table.clone().unwrap_or_default(),
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
}

#[component]
fn InsertRowForm(columns: Vec<String>, source_table: String) -> Element {
    let is_dark = *IS_DARK_MODE.read();
    let cell_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-700"
    };
    let input_bg = if is_dark { "bg-gray-800" } else { "bg-white" };

    let mut values = use_signal(|| vec![String::new(); columns.len()]);

    rsx! {
        tr {
            class: "bg-green-900 bg-opacity-20",
            // Empty checkbox column
            td { class: "px-2 py-2 w-8" }
            for (idx, col) in columns.iter().enumerate() {
                td {
                    class: "px-4 py-1",
                    input {
                        class: "w-full text-xs px-1 py-1 rounded {input_bg} {cell_text} border border-green-700 font-mono",
                        placeholder: "{col}",
                        value: "{values.read()[idx]}",
                        oninput: move |evt: FormEvent| {
                            values.write()[idx] = evt.value();
                        },
                    }
                }
            }
        }
        tr {
            class: "bg-green-900 bg-opacity-20",
            td {
                class: "px-4 py-2",
                colspan: "{columns.len() + 1}",
                div {
                    class: "flex items-center space-x-2",
                    button {
                        class: "text-xs px-2 py-1 rounded bg-green-700 text-white hover:bg-green-600",
                        onclick: {
                            let source_table = source_table.clone();
                            let columns = columns.clone();
                            move |_| {
                                insert_row(&source_table, &columns, &values.read());
                                *INSERTING_ROW.write() = false;
                            }
                        },
                        "Insert"
                    }
                    button {
                        class: "text-xs px-2 py-1 rounded text-gray-400 hover:text-white",
                        onclick: move |_| *INSERTING_ROW.write() = false,
                        "Cancel"
                    }
                }
            }
        }
    }
}

fn commit_cell_edit(row_idx: usize, column: &str, old_value: &str, new_value: &str) {
    if old_value == new_value {
        return;
    }
    if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
        // Remove any existing edit for this cell
        tab.pending_edits
            .retain(|e| !(e.row_idx == row_idx && e.column == column));
        tab.pending_edits.push(CellEdit {
            row_idx,
            column: column.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
        });
    }
}

fn save_pending_edits() {
    let (table, primary_keys, edits, result_rows, result_columns) = {
        let tabs = EDITOR_TABS.read();
        let tab = match tabs.active_tab() {
            Some(t) => t,
            None => return,
        };
        let result = match &tab.result {
            Some(r) => r,
            None => return,
        };
        let table = match &result.source_table {
            Some(t) => t.clone(),
            None => return,
        };
        if result.primary_keys.is_empty() {
            return;
        }
        (
            table,
            result.primary_keys.clone(),
            tab.pending_edits.clone(),
            result.rows.clone(),
            result.columns.clone(),
        )
    };

    // Group edits by row
    let mut edits_by_row: HashMap<usize, Vec<&CellEdit>> = HashMap::new();
    for edit in &edits {
        edits_by_row.entry(edit.row_idx).or_default().push(edit);
    }

    let mut statements = Vec::new();
    for (row_idx, row_edits) in &edits_by_row {
        let row = match result_rows.get(*row_idx) {
            Some(r) => r,
            None => continue,
        };

        let set_clauses: Vec<String> = row_edits
            .iter()
            .map(|e| {
                if e.new_value == "NULL" {
                    format!("{} = NULL", e.column)
                } else {
                    format!("{} = '{}'", e.column, e.new_value.replace('\'', "''"))
                }
            })
            .collect();

        let where_clauses: Vec<String> = primary_keys
            .iter()
            .filter_map(|pk| {
                let col_idx = result_columns.iter().position(|c| c == pk)?;
                let value = row.get(col_idx)?;
                Some(format!("{} = '{}'", pk, value.replace('\'', "''")))
            })
            .collect();

        if !set_clauses.is_empty() && !where_clauses.is_empty() {
            statements.push(format!(
                "UPDATE {} SET {} WHERE {}",
                table,
                set_clauses.join(", "),
                where_clauses.join(" AND ")
            ));
        }
    }

    if !statements.is_empty() {
        send_db_request(crate::db::DbRequest::ExecuteBatch(statements));
    }

    if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
        tab.pending_edits.clear();
        tab.edit_mode = false;
    }
    *EDITING_CELL.write() = None;
}

fn discard_pending_edits() {
    if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
        tab.pending_edits.clear();
    }
    *EDITING_CELL.write() = None;
}

fn delete_selected_rows() {
    let selected = SELECTED_ROWS.read().clone();
    if selected.is_empty() {
        return;
    }

    let (table, primary_keys, result_rows, result_columns) = {
        let tabs = EDITOR_TABS.read();
        let tab = match tabs.active_tab() {
            Some(t) => t,
            None => return,
        };
        let result = match &tab.result {
            Some(r) => r,
            None => return,
        };
        let table = match &result.source_table {
            Some(t) => t.clone(),
            None => return,
        };
        if result.primary_keys.is_empty() {
            return;
        }
        (
            table,
            result.primary_keys.clone(),
            result.rows.clone(),
            result.columns.clone(),
        )
    };

    let mut statements = Vec::new();
    for row_idx in &selected {
        let row = match result_rows.get(*row_idx) {
            Some(r) => r,
            None => continue,
        };

        let where_clauses: Vec<String> = primary_keys
            .iter()
            .filter_map(|pk| {
                let col_idx = result_columns.iter().position(|c| c == pk)?;
                let value = row.get(col_idx)?;
                Some(format!("{} = '{}'", pk, value.replace('\'', "''")))
            })
            .collect();

        if !where_clauses.is_empty() {
            statements.push(format!(
                "DELETE FROM {} WHERE {}",
                table,
                where_clauses.join(" AND ")
            ));
        }
    }

    if !statements.is_empty() {
        send_db_request(crate::db::DbRequest::ExecuteBatch(statements));
    }

    *SELECTED_ROWS.write() = Default::default();
}

fn insert_row(table: &str, columns: &[String], values: &[String]) {
    let non_empty: Vec<(&String, &String)> = columns
        .iter()
        .zip(values.iter())
        .filter(|(_, v)| !v.is_empty())
        .collect();

    if non_empty.is_empty() {
        return;
    }

    let col_list: Vec<&str> = non_empty.iter().map(|(c, _)| c.as_str()).collect();
    let val_list: Vec<String> = non_empty
        .iter()
        .map(|(_, v)| {
            if v.to_uppercase() == "NULL" {
                "NULL".to_string()
            } else {
                format!("'{}'", v.replace('\'', "''"))
            }
        })
        .collect();

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        col_list.join(", "),
        val_list.join(", ")
    );

    send_db_request(crate::db::DbRequest::ExecuteMutation(sql));
}

fn show_execution_plan() {
    use crate::components::execution_plan::request_execution_plan;
    request_execution_plan();
}
