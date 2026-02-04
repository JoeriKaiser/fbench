use crate::config::get_builtin_templates;
use crate::state::*;
use dioxus::prelude::*;

pub static SELECTED_TEMPLATE_INDEX: GlobalSignal<usize> = Signal::global(|| 0);

#[component]
pub fn TemplateSelector() -> Element {
    let templates = get_builtin_templates();
    let is_dark = *IS_DARK_MODE.read();
    let selected_index = *SELECTED_TEMPLATE_INDEX.read();

    let select_class = if is_dark {
        "bg-black border-gray-800 text-white focus:border-white"
    } else {
        "bg-white border-gray-300 text-gray-900 focus:border-blue-500"
    };

    rsx! {
        div {
            class: "flex items-center space-x-2",

            select {
                class: "px-3 py-1.5 text-sm rounded border {select_class} focus:outline-none appearance-none",
                value: "{selected_index}",
                onchange: move |e| {
                    if let Ok(index) = e.value().parse::<usize>() {
                        *SELECTED_TEMPLATE_INDEX.write() = index;
                        if let Some(template) = templates.get(index) {
                            let values: Vec<(String, String)> = template.variables.iter()
                                .map(|v| (v.name.clone(), v.default_value.clone().unwrap_or_default()))
                                .collect();
                            let sql = template.apply(&values);
                            if let Some(tab) = EDITOR_TABS.write().active_tab_mut() {
                                tab.content = sql;
                                tab.unsaved_changes = true;
                            }
                        }
                    }
                },
                option {
                    class: if is_dark { "bg-black text-white" } else { "bg-white text-gray-900" },
                    value: "",
                    disabled: true,
                    selected: selected_index == 0,
                    "Select template..."
                }
                for (index, template) in templates.iter().enumerate() {
                    option {
                        class: if is_dark { "bg-black text-white" } else { "bg-white text-gray-900" },
                        value: "{index}",
                        selected: selected_index == index,
                        "{template.name}"
                    }
                }
            }
        }
    }
}
