use crate::config::QueryStore;
use crate::state::{EDITOR_CONTENT, IS_DARK_MODE};
use dioxus::prelude::*;

#[component]
pub fn QueriesPanel() -> Element {
    let mut query_store = use_signal(QueryStore::new);
    let mut queries = use_resource(move || async move { query_store.read().load_queries() });
    let mut new_query_name = use_signal(String::new);
    let mut show_save_dialog = use_signal(|| false);
    let is_dark = *IS_DARK_MODE.read();

    // Theme-aware classes
    let header_text = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-500"
    };
    let button_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let button_hover = if is_dark {
        "hover:text-white"
    } else {
        "hover:text-gray-900"
    };
    let dialog_bg = if is_dark {
        "bg-gray-900"
    } else {
        "bg-gray-100"
    };
    let dialog_border = if is_dark {
        "border-gray-800"
    } else {
        "border-gray-200"
    };
    let input_bg = if is_dark { "bg-black" } else { "bg-white" };
    let input_text = if is_dark {
        "text-white"
    } else {
        "text-gray-900"
    };
    let input_placeholder = if is_dark {
        "placeholder-gray-600"
    } else {
        "placeholder-gray-400"
    };
    let input_border = if is_dark {
        "focus:ring-white"
    } else {
        "focus:ring-blue-500"
    };
    let item_text = if is_dark {
        "text-gray-400"
    } else {
        "text-gray-600"
    };
    let item_hover = if is_dark {
        "hover:bg-gray-900 hover:text-white"
    } else {
        "hover:bg-gray-100 hover:text-gray-900"
    };
    let delete_hover = if is_dark {
        "hover:text-white"
    } else {
        "hover:text-red-600"
    };
    let muted_text = if is_dark {
        "text-gray-600"
    } else {
        "text-gray-400"
    };

    rsx! {
        div {
            class: "space-y-2",

            div {
                class: "flex items-center justify-between mb-3",
                h3 {
                    class: "text-xs font-semibold {header_text} uppercase tracking-wider",
                    "Saved Queries"
                }

                button {
                    class: "text-xs {button_text} {button_hover} flex items-center space-x-1 transition-colors",
                    onclick: move |_| show_save_dialog.set(true),
                    svg {
                        class: "w-3.5 h-3.5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M12 4v16m8-8H4",
                        }
                    }
                    span { "Save Current" }
                }
            }

            if show_save_dialog() {
                div {
                    class: "rounded p-2 space-y-2 mb-3 border {dialog_bg} {dialog_border}",
                    input {
                        class: "w-full px-2 py-1 rounded text-sm focus:outline-none focus:ring-1 {input_bg} {input_text} {input_placeholder} {input_border}",
                        placeholder: "Query name...",
                        value: "{new_query_name}",
                        oninput: move |e| new_query_name.set(e.value().clone()),
                    }
                    div {
                        class: "flex space-x-2",
                        button {
                            class: "flex-1 px-2 py-1 text-xs rounded transition-colors",
                            class: if is_dark { "bg-white hover:bg-gray-200 text-black" } else { "bg-blue-600 hover:bg-blue-500 text-white" },
                            onclick: move |_| {
                                let name = new_query_name.read().clone();
                                if !name.is_empty() {
                                    let sql = EDITOR_CONTENT.read().clone();
                                    let store = query_store.write();
                                    let mut qs = store.load_queries();
                                    qs.push(crate::config::SavedQuery { name, sql });
                                    let _ = store.save_queries(&qs);
                                    new_query_name.set(String::new());
                                    show_save_dialog.set(false);
                                    queries.restart();
                                }
                            },
                            "Save"
                        }
                        button {
                            class: "flex-1 px-2 py-1 text-xs rounded transition-colors",
                            class: if is_dark { "bg-gray-800 hover:bg-gray-700 text-white" } else { "bg-gray-200 hover:bg-gray-300 text-gray-700" },
                            onclick: move |_| {
                                new_query_name.set(String::new());
                                show_save_dialog.set(false);
                            },
                            "Cancel"
                        }
                    }
                }
            }

            if let Some(queries_list) = queries.read().as_ref() {
            if queries_list.is_empty() {
                div {
                    class: "{muted_text} text-sm text-center py-8",
                    "No saved queries"
                }
            } else {
                for query in queries_list.iter() {
                    div {
                        class: "group flex items-center justify-between px-2 py-2 rounded {item_hover} transition-colors",
                        key: "{query.name}",

                        button {
                            class: "flex-1 text-left text-sm {item_text} truncate",
                            onclick: {
                                let query_clone = query.clone();
                                move |_| {
                                    *EDITOR_CONTENT.write() = query_clone.sql.clone();
                                }
                            },
                            "{query.name}"
                        }

                        button {
                            class: "opacity-0 group-hover:opacity-100 {muted_text} {delete_hover} transition-colors",
                            onclick: {
                                let query_name = query.name.clone();
                                move |_| {
                                    let store = query_store.write();
                                    let mut qs = store.load_queries();
                                    qs.retain(|q| q.name != query_name);
                                    let _ = store.save_queries(&qs);
                                    queries.restart();
                                }
                            },
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
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
