use crate::filter::{ColumnFilter, FilterOperator, FilterState, SortColumn, SortDirection};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn FilterPanel() -> Element {
    let tabs = EDITOR_TABS.read();
    let active_tab = tabs.active_tab();
    let result = active_tab.and_then(|t| t.result.as_ref());
    let is_dark = *IS_DARK_MODE.read();

    // Only show filter panel for single-table queries
    let source_table = result.and_then(|r| r.source_table.as_ref());
    if source_table.is_none() {
        return rsx! {};
    }

    let source_table = source_table.unwrap().clone();
    let columns = result.map(|r| r.columns.clone()).unwrap_or_default();
    let column_types = result.map(|r| r.column_types.clone()).unwrap_or_default();
    let filter_state = active_tab.and_then(|t| t.filter_state.clone());

    let bg = if is_dark { "bg-gray-900" } else { "bg-gray-50" };
    let border = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };
    let text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };

    rsx! {
        div {
            class: "px-3 py-2 {bg} border-b {border} space-y-1",

            // Filter rows
            if let Some(state) = &filter_state {
                for (idx, filter) in state.filters.iter().enumerate() {
                    FilterRow {
                        key: "{idx}",
                        index: idx,
                        filter: filter.clone(),
                        columns: columns.clone(),
                        column_types: column_types.clone(),
                        source_table: source_table.clone(),
                    }
                }
            }

            // Controls row
            div {
                class: "flex items-center space-x-2",
                button {
                    class: "text-xs px-2 py-1 rounded {text} hover:opacity-80",
                    onclick: {
                        let source_table = source_table.clone();
                        move |_| add_filter(&source_table)
                    },
                    "+ Add Filter"
                }

                if filter_state.is_some() {
                    button {
                        class: "text-xs px-2 py-1 rounded {text} hover:opacity-80",
                        onclick: {
                            let source_table = source_table.clone();
                            move |_| clear_filters(&source_table)
                        },
                        "Clear All"
                    }
                }
            }
        }
    }
}

#[component]
fn FilterRow(
    index: usize,
    filter: ColumnFilter,
    columns: Vec<String>,
    column_types: Vec<String>,
    source_table: String,
) -> Element {
    let is_dark = *IS_DARK_MODE.read();
    let input_bg = if is_dark { "bg-gray-800" } else { "bg-white" };
    let input_border = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-300"
    };
    let text = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };

    // Get the column type for operator filtering
    let col_type = columns
        .iter()
        .position(|c| *c == filter.column)
        .and_then(|i| column_types.get(i))
        .cloned()
        .unwrap_or_default();

    let available_operators = FilterOperator::for_type(&col_type);
    let needs_value = filter.operator.needs_value();

    rsx! {
        div {
            class: "flex items-center space-x-2",

            // Column dropdown
            select {
                class: "text-xs px-2 py-1 rounded {input_bg} {input_border} {text} border",
                value: "{filter.column}",
                onchange: {
                    let source_table = source_table.clone();
                    move |evt: FormEvent| {
                        update_filter_column(index, &evt.value(), &source_table);
                    }
                },
                option { value: "", "Column..." }
                for col in &columns {
                    option {
                        value: "{col}",
                        selected: *col == filter.column,
                        "{col}"
                    }
                }
            }

            // Operator dropdown
            select {
                class: "text-xs px-2 py-1 rounded {input_bg} {input_border} {text} border",
                value: "{filter.operator.display_label()}",
                onchange: {
                    let source_table = source_table.clone();
                    move |evt: FormEvent| {
                        update_filter_operator(index, &evt.value(), &source_table);
                    }
                },
                for op in &available_operators {
                    option {
                        value: "{op.display_label()}",
                        selected: *op == filter.operator,
                        "{op.display_label()}"
                    }
                }
            }

            // Value input (hidden for IS NULL / IS NOT NULL)
            if needs_value {
                input {
                    class: "text-xs px-2 py-1 rounded {input_bg} {input_border} {text} border w-32",
                    r#type: "text",
                    value: "{filter.value}",
                    placeholder: "Value...",
                    onchange: {
                        let source_table = source_table.clone();
                        move |evt: FormEvent| {
                            update_filter_value(index, &evt.value(), &source_table);
                        }
                    },
                }
            }

            // Remove button
            button {
                class: "text-xs px-1 py-1 text-red-500 hover:text-red-400",
                onclick: {
                    let source_table = source_table.clone();
                    move |_| remove_filter(index, &source_table)
                },
                "✕"
            }
        }
    }
}

fn add_filter(source_table: &str) {
    let mut tabs = EDITOR_TABS.write();
    if let Some(tab) = tabs.active_tab_mut() {
        let state = tab
            .filter_state
            .get_or_insert_with(|| FilterState::new(source_table.to_string()));
        state.filters.push(ColumnFilter {
            column: String::new(),
            operator: FilterOperator::Equal,
            value: String::new(),
        });
    }
}

fn clear_filters(source_table: &str) {
    {
        let mut tabs = EDITOR_TABS.write();
        if let Some(tab) = tabs.active_tab_mut() {
            tab.filter_state = None;
        }
    }
    // Re-execute a simple select
    let sql = format!("SELECT * FROM {} LIMIT 100", source_table);
    execute_filter_sql(&sql);
}

fn update_filter_column(index: usize, column: &str, source_table: &str) {
    {
        let mut tabs = EDITOR_TABS.write();
        if let Some(tab) = tabs.active_tab_mut() {
            if let Some(state) = &mut tab.filter_state {
                if let Some(filter) = state.filters.get_mut(index) {
                    filter.column = column.to_string();
                }
            }
        }
    }
    apply_filters(source_table);
}

fn update_filter_operator(index: usize, label: &str, source_table: &str) {
    {
        let mut tabs = EDITOR_TABS.write();
        if let Some(tab) = tabs.active_tab_mut() {
            if let Some(state) = &mut tab.filter_state {
                if let Some(filter) = state.filters.get_mut(index) {
                    filter.operator = operator_from_label(label);
                }
            }
        }
    }
    apply_filters(source_table);
}

fn update_filter_value(index: usize, value: &str, source_table: &str) {
    {
        let mut tabs = EDITOR_TABS.write();
        if let Some(tab) = tabs.active_tab_mut() {
            if let Some(state) = &mut tab.filter_state {
                if let Some(filter) = state.filters.get_mut(index) {
                    filter.value = value.to_string();
                }
            }
        }
    }
    apply_filters(source_table);
}

fn remove_filter(index: usize, source_table: &str) {
    {
        let mut tabs = EDITOR_TABS.write();
        if let Some(tab) = tabs.active_tab_mut() {
            if let Some(state) = &mut tab.filter_state {
                if index < state.filters.len() {
                    state.filters.remove(index);
                }
            }
        }
    }
    apply_filters(source_table);
}

fn apply_filters(source_table: &str) {
    let sql = {
        let tabs = EDITOR_TABS.read();
        let tab = match tabs.active_tab() {
            Some(t) => t,
            None => return,
        };
        match &tab.filter_state {
            Some(state) => state.to_sql(),
            None => format!("SELECT * FROM {} LIMIT 100", source_table),
        }
    };
    execute_filter_sql(&sql);
}

fn execute_filter_sql(sql: &str) {
    {
        let mut tabs = EDITOR_TABS.write();
        if let Some(tab) = tabs.active_tab_mut() {
            tab.content = sql.to_string();
        }
    }
    send_db_request(crate::db::DbRequest::Execute(sql.to_string()));
}

pub fn toggle_sort(column: String) {
    let (source_table, current_sort) = {
        let tabs = EDITOR_TABS.read();
        let tab = match tabs.active_tab() {
            Some(t) => t,
            None => return,
        };
        let source_table = tab.result.as_ref().and_then(|r| r.source_table.clone());
        let current_sort = tab.filter_state.as_ref().and_then(|s| s.sort.clone());
        (source_table, current_sort)
    };

    let source_table = match source_table {
        Some(t) => t,
        None => return,
    };

    let new_sort = match current_sort {
        Some(sort) if sort.column == column => match sort.direction {
            SortDirection::Asc => Some(SortColumn {
                column: column.clone(),
                direction: SortDirection::Desc,
            }),
            SortDirection::Desc => None,
        },
        _ => Some(SortColumn {
            column: column.clone(),
            direction: SortDirection::Asc,
        }),
    };

    {
        let mut tabs = EDITOR_TABS.write();
        if let Some(tab) = tabs.active_tab_mut() {
            let state = tab
                .filter_state
                .get_or_insert_with(|| FilterState::new(source_table.clone()));
            state.sort = new_sort;
        }
    }
    apply_filters(&source_table);
}

fn operator_from_label(label: &str) -> FilterOperator {
    match label {
        "=" => FilterOperator::Equal,
        "!=" => FilterOperator::NotEqual,
        ">" => FilterOperator::GreaterThan,
        "<" => FilterOperator::LessThan,
        ">=" => FilterOperator::GreaterOrEqual,
        "<=" => FilterOperator::LessOrEqual,
        "LIKE" => FilterOperator::Like,
        "NOT LIKE" => FilterOperator::NotLike,
        "IS NULL" => FilterOperator::IsNull,
        "IS NOT NULL" => FilterOperator::IsNotNull,
        _ => FilterOperator::Equal,
    }
}
