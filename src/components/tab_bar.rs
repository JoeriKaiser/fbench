use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn TabBar() -> Element {
    let tabs_state = EDITOR_TABS.read();
    let is_dark = *IS_DARK_MODE.read();

    let bg_class = if is_dark {
        "bg-gray-900"
    } else {
        "bg-gray-100"
    };
    let active_bg = if is_dark { "bg-black" } else { "bg-white" };
    let inactive_bg = if is_dark {
        "bg-gray-800"
    } else {
        "bg-gray-200"
    };
    let border_color = if is_dark {
        "border-gray-700"
    } else {
        "border-gray-300"
    };
    let text_color = if is_dark {
        "text-gray-300"
    } else {
        "text-gray-700"
    };
    let muted_color = if is_dark {
        "text-gray-500"
    } else {
        "text-gray-400"
    };

    rsx! {
        div {
            class: "flex items-center {bg_class} border-b {border_color} overflow-x-auto",

            // Tab list
            div {
                class: "flex items-center flex-1",

                for tab in &tabs_state.tabs {
                    {
                        let is_active = tabs_state.active_tab_id.as_ref() == Some(&tab.id);
                        let tab_bg = if is_active { active_bg } else { inactive_bg };
                        let tab_id = tab.id.clone();
                        let close_id = tab.id.clone();
                        let has_changes = tab.unsaved_changes;

                        rsx! {
                            div {
                                class: "flex items-center px-3 py-2 cursor-pointer border-r {border_color} {tab_bg} hover:opacity-90 transition-opacity min-w-[120px] max-w-[200px]",
                                class: if is_active { "border-t-2 border-t-blue-500" } else { "" },
                                onclick: move |_| {
                                    EDITOR_TABS.write().set_active(&tab_id);
                                },

                                // Tab title
                                span {
                                    class: "text-sm truncate flex-1 {text_color}",
                                    "{tab.title}"
                                    if has_changes {
                                        span { class: "{muted_color} ml-1", "●" }
                                    }
                                }

                                // Close button (only show on hover or if not last tab)
                                if tabs_state.tabs.len() > 1 {
                                    button {
                                        class: "ml-2 p-0.5 rounded hover:bg-gray-600/20 {muted_color}",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            EDITOR_TABS.write().close_tab(&close_id);
                                        },
                                        svg {
                                            class: "w-3 h-3",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "M6 18L18 6M6 6l12 12",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Add new tab button
            button {
                class: "p-2 {muted_color} hover:{text_color} transition-colors",
                onclick: move |_| {
                    let mut tabs = EDITOR_TABS.write();
                    let count = tabs.tabs.len() + 1;
                    tabs.add_tab(format!("Query {}", count));
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
                        d: "M12 4v16m8-8H4",
                    }
                }
            }
        }
    }
}
