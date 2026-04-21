use crate::components::*;
use crate::config::{SessionState, SessionStore};
use crate::state::*;
use dioxus::prelude::*;

const APP_STYLE: &str = r#"
    /* Ensure solid background for window */
    :root {
        --bg-color: black;
        --text-color: #d1d5db;
        --border-color: #1f2937;
        --input-bg: black;
        --button-hover: #374151;
        --scrollbar-hover: #4b5563;
        --resize-hover-bg: #374151;
    }

    html, body {
        background: var(--bg-color);
        color: var(--text-color);
        margin: 0;
        padding: 0;
        width: 100%;
        height: 100%;
    }

    /* Shiki syntax highlighting styles */
    .shiki {
        background-color: transparent !important;
        margin: 0 !important;
        padding: 0 !important;
    }
    .shiki code {
        background-color: transparent !important;
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
        font-size: 0.875rem;
        line-height: 1.5rem;
    }
    .shiki span {
        background-color: transparent !important;
    }

    /* Scrollbar styling for webkit browsers */
    ::-webkit-scrollbar {
        width: 8px;
        height: 8px;
    }
    ::-webkit-scrollbar-track {
        background: transparent;
    }
    ::-webkit-scrollbar-thumb {
        background: var(--border-color);
        border-radius: 4px;
    }
    ::-webkit-scrollbar-thumb:hover {
        background: var(--scrollbar-hover);
    }

    /* Resize handle styles */
    .resize-handle {
        cursor: ns-resize;
        user-select: none;
        touch-action: none;
    }
    .resize-handle:hover {
        background: var(--resize-hover-bg);
    }
"#;

#[component]
pub fn AppLayout() -> Element {
    use_hook(|| {
        spawn(async move {
            inject_app_style().await;
        });
    });

    // Initialize system theme detection
    use_effect(|| {
        spawn(async move {
            // Initialize lucide icons
            let _ = document::eval(r#"lucide.createIcons();"#).await;

            // Detect system theme preference
            let is_dark = detect_system_theme().await;
            *IS_DARK_MODE.write() = is_dark;
        });
    });

    // Listen for system theme changes
    use_effect(|| {
        spawn(async move {
            listen_for_theme_changes().await;
        });
    });

    let theme_class = if *IS_DARK_MODE.read() {
        "bg-black text-gray-300"
    } else {
        "bg-white text-gray-700"
    };

    let editor_height = *EDITOR_PANEL_HEIGHT.read();
    let is_resizing = *IS_RESIZING_PANELS.read();
    let is_dark = *IS_DARK_MODE.read();

    use_effect(move || {
        spawn(async move {
            apply_theme_variables(is_dark).await;
        });
    });

    // Save session state when UI changes
    use_effect(move || {
        let is_resizing = *IS_RESIZING_PANELS.read();
        if is_resizing {
            return;
        }

        let left_tab = match *LEFT_TAB.read() {
            LeftTab::Schema => "Schema",
            LeftTab::Queries => "Queries",
            LeftTab::History => "History",
        };
        let panel_height = *EDITOR_PANEL_HEIGHT.read();

        let state = SessionState {
            left_tab: left_tab.to_string(),
            sidebar_scroll_position: 0.0,
            editor_panel_height: panel_height,
        };

        let store = SessionStore::new();
        let _ = store.save(&state);
    });
    let resize_bg = if is_resizing {
        if is_dark {
            "bg-gray-700"
        } else {
            "bg-gray-300"
        }
    } else if is_dark {
        "bg-gray-900"
    } else {
        "bg-gray-100"
    };

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: "https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css",
        }

        document::Script {
            src: "https://unpkg.com/lucide@latest",
        }

        div {
            class: "h-screen w-screen flex flex-col overflow-hidden {theme_class}",
            // Global keyboard shortcut for quick switcher
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Character("p".to_string()) &&
                   e.modifiers().contains(Modifiers::CONTROL) {
                    e.prevent_default();
                    *SHOW_QUICK_SWITCHER.write() = true;
                }
            },
            // Global mouse events for resizing
            onmousemove: move |e: MouseEvent| {
                if *IS_RESIZING_PANELS.read() {
                    let coords = e.client_coordinates();
                    let new_height = coords.y - 100.0; // Offset for menubar and LLM panel
                    let min_height = 100.0;
                    let max_height = 600.0;
                    let clamped_height = new_height.clamp(min_height, max_height);
                    *EDITOR_PANEL_HEIGHT.write() = clamped_height;
                }
            },
            onmouseup: move |_| {
                *IS_RESIZING_PANELS.write() = false;
            },
            onmouseleave: move |_| {
                *IS_RESIZING_PANELS.write() = false;
            },

            MenuBar {}

            div {
                class: "flex-1 flex overflow-hidden",
                Sidebar {}
                div {
                    class: "flex-1 flex flex-col min-w-0",
                    LlmPanel {}
                    // SQL Editor with fixed height
                    div {
                        class: "flex flex-col border-b min-h-0",
                        class: if is_dark { "border-gray-800" } else { "border-gray-200" },
                        style: "height: {editor_height}px",
                        SqlEditor {}
                    }

                    // AI Results Panel (collapsible)
                    AiResultsPanel {}

                    // Resize handle
                    div {
                        class: "h-1 resize-handle flex items-center justify-center transition-colors {resize_bg}",
                        onmousedown: move |_| {
                            *IS_RESIZING_PANELS.write() = true;
                        },
                        // Visual indicator line
                        div {
                            class: "w-8 h-0.5 rounded-full",
                            class: if is_dark { "bg-gray-700" } else { "bg-gray-300" },
                        }
                    }
                    // Results table takes remaining space
                    div {
                        class: "flex-1 flex flex-col min-h-0",
                        class: if is_dark { "bg-black" } else { "bg-white" },
                        ResultsTable {}
                    }
                }
            }

            StatusBar {}
            QuickSwitcher {}
        }

        ConnectionDialog {}

        ContextMenu {}

        LlmSettingsDialog {}

        SaveQueryDialog {}

        JsonViewer {}

        ExecutionPlanDialog {}

        ImportDialog {}
    }
}

async fn detect_system_theme() -> bool {
    // Use JavaScript to detect system theme preference
    let result = document::eval(
        r#"
        window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
    "#,
    )
    .await;

    match result {
        Ok(val) => val.as_bool().unwrap_or(true),
        Err(_) => true, // Default to dark if detection fails
    }
}

async fn listen_for_theme_changes() {
    let mut eval = document::eval(
        r#"
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        dioxus.send(mediaQuery.matches);
        mediaQuery.addEventListener('change', (e) => {
            dioxus.send(e.matches);
        });
    "#,
    );

    while let Ok(is_dark) = eval.recv::<bool>().await {
        *IS_DARK_MODE.write() = is_dark;
    }
}

async fn apply_theme_variables(is_dark: bool) {
    let (
        bg_color,
        text_color,
        border_color,
        input_bg,
        button_hover,
        scrollbar_hover,
        resize_hover_bg,
    ) = if is_dark {
        (
            "black", "#d1d5db", "#1f2937", "black", "#374151", "#4b5563", "#374151",
        )
    } else {
        (
            "white", "#374151", "#e5e7eb", "white", "#f3f4f6", "#d1d5db", "#d1d5db",
        )
    };

    let _ = document::eval(&format!(
        r#"
        const root = document.documentElement;
        root.style.setProperty('--bg-color', '{bg_color}');
        root.style.setProperty('--text-color', '{text_color}');
        root.style.setProperty('--border-color', '{border_color}');
        root.style.setProperty('--input-bg', '{input_bg}');
        root.style.setProperty('--button-hover', '{button_hover}');
        root.style.setProperty('--scrollbar-hover', '{scrollbar_hover}');
        root.style.setProperty('--resize-hover-bg', '{resize_hover_bg}');
        "#
    ))
    .await;
}

async fn inject_app_style() {
    let css = serde_json::to_string(APP_STYLE).unwrap_or_else(|_| "\"\"".to_string());
    let _ = document::eval(&format!(
        r#"
        const styleId = 'fbench-app-style';
        let style = document.getElementById(styleId);
        if (!style) {{
            style = document.createElement('style');
            style.id = styleId;
            document.head.appendChild(style);
        }}
        style.textContent = {css};
        "#
    ))
    .await;
}
