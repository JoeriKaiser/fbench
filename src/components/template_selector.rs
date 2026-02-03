use crate::config::get_builtin_templates;
use crate::state::*;
use dioxus::prelude::*;

pub static SELECTED_TEMPLATE_INDEX: GlobalSignal<usize> = Signal::global(|| 0);

#[component]
pub fn TemplateSelector() -> Element {
    let templates = get_builtin_templates();
    let is_dark = *IS_DARK_MODE.read();
    let selected_index = *SELECTED_TEMPLATE_INDEX.read();

    let select_bg = if is_dark {
        "bg-gray-800"
    } else {
        "bg-gray-100"
    };
    let select_border = if is_dark {
        "border-gray-600"
    } else {
        "border-gray-300"
    };
    let select_text = if is_dark {
        "text-gray-200"
    } else {
        "text-gray-700"
    };

    rsx! {
        div {
            class: "flex items-center space-x-2",

            select {
                class: "px-3 py-1.5 text-sm rounded border {select_bg} {select_border} {select_text} focus:outline-none focus:ring-2 focus:ring-blue-500",
                value: "{selected_index}",
                onchange: move |e| {
                    if let Ok(index) = e.value().parse::<usize>() {
                        *SELECTED_TEMPLATE_INDEX.write() = index;
                        if let Some(template) = templates.get(index) {
                            let values: Vec<(String, String)> = template.variables.iter()
                                .map(|v| (v.name.clone(), v.default_value.clone().unwrap_or_default()))
                                .collect();
                            let sql = template.apply(&values);
                            *EDITOR_CONTENT.write() = sql;
                        }
                    }
                },
                option {
                    value: "0",
                    disabled: true,
                    "Select template..."
                }
                for (index, template) in templates.iter().enumerate() {
                    option {
                        value: "{index}",
                        "{template.name}"
                    }
                }
            }
        }
    }
}
