use crate::config::QueryStore;
use crate::state::{EDITOR_TABS, IS_DARK_MODE, QUERIES_REVISION, SHOW_SAVE_QUERY_DIALOG};
use dioxus::prelude::*;

#[component]
pub fn QueriesPanel() -> Element {
    let mut query_store = use_signal(QueryStore::new);
    let _revision = *QUERIES_REVISION.read();
    let mut queries = use_resource(move || async move {
        // Read revision to trigger re-fetch when queries change
        let _ = *QUERIES_REVISION.read();
        query_store.read().load_queries()
    });
    let is_dark = *IS_DARK_MODE.read();

    // Theme-aware classes
    let header_text = "text-gray-500";
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
                    onclick: move |_| *SHOW_SAVE_QUERY_DIALOG.write() = true,
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
                                    if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
                                        tab.content = query_clone.sql.clone();
                                        tab.unsaved_changes = true;
                                    }
                                }
                            },
                            "{query.name}"
                        }

                        // Bookmark button
                        button {
                            class: "opacity-0 group-hover:opacity-100 {muted_text} hover:text-yellow-500 transition-colors mr-1",
                            class: if query.is_bookmarked { "text-yellow-500 opacity-100" },
                            onclick: {
                                let query_name = query.name.clone();
                                move |_| {
                                    let store = query_store.write();
                                    let _ = store.toggle_bookmark(&query_name);
                                    queries.restart();
                                }
                            },
                            svg {
                                class: "w-4 h-4",
                                fill: if query.is_bookmarked { "currentColor" } else { "none" },
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z",
                                }
                            }
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
