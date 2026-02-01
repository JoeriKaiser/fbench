use crate::components::*;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn AppLayout() -> Element {
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
            let _ = listen_for_theme_changes().await;
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
    let resize_bg = if is_resizing {
        if is_dark {
            "bg-gray-700"
        } else {
            "bg-gray-300"
        }
    } else {
        if is_dark {
            "bg-gray-900"
        } else {
            "bg-gray-100"
        }
    };

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: "https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css",
        }

        document::Script {
            src: "https://unpkg.com/lucide@latest",
        }

        document::Style {{
            let is_dark = *IS_DARK_MODE.read();
            let bg_color = if is_dark { "black" } else { "white" };
            let text_color = if is_dark { "#d1d5db" } else { "#374151" };
            let border_color = if is_dark { "#1f2937" } else { "#e5e7eb" };
            let input_bg = if is_dark { "black" } else { "white" };
            let button_hover = if is_dark { "#374151" } else { "#f3f4f6" };
            let scrollbar_hover = if is_dark { "#4b5563" } else { "#d1d5db" };
            let resize_hover_bg = if is_dark { "#374151" } else { "#d1d5db" };

            format!(r#"
            /* Ensure solid background for window */
            html, body {{
                background: {bg_color};
                margin: 0;
                padding: 0;
                width: 100%;
                height: 100%;
            }}
            
            /* Shiki syntax highlighting styles */
            .shiki {{
                background-color: transparent !important;
                margin: 0 !important;
                padding: 0 !important;
            }}
            .shiki code {{
                background-color: transparent !important;
                font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
                font-size: 0.875rem;
                line-height: 1.5rem;
            }}
            /* Ensure highlighted text is visible on dark background */
            .shiki span {{
                background-color: transparent !important;
            }}
            
            /* Theme-aware variables */
            :root {{
                --bg-color: {bg_color};
                --text-color: {text_color};
                --border-color: {border_color};
                --input-bg: {input_bg};
                --button-hover: {button_hover};
            }}
            
            /* Scrollbar styling for webkit browsers */
            ::-webkit-scrollbar {{
                width: 8px;
                height: 8px;
            }}
            ::-webkit-scrollbar-track {{
                background: transparent;
            }}
            ::-webkit-scrollbar-thumb {{
                background: {border_color};
                border-radius: 4px;
            }}
            ::-webkit-scrollbar-thumb:hover {{
                background: {scrollbar_hover};
            }}
            
            /* Resize handle styles */
            .resize-handle {{
                cursor: ns-resize;
                user-select: none;
                touch-action: none;
            }}
            .resize-handle:hover {{
                background: {resize_hover_bg};
            }}
            "#)
        }}

        div {
            class: "h-screen w-screen flex flex-col overflow-hidden {theme_class}",
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
        }

        ConnectionDialog {}

        ContextMenu {}

        LlmSettingsDialog {}
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
    // Set up listener for theme changes
    let _ = document::eval(
        r#"
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        mediaQuery.addEventListener('change', (e) => {
            // Dispatch a custom event that Dioxus can listen to
            window.dispatchEvent(new CustomEvent('themechange', { detail: { isDark: e.matches } }));
        });
    "#,
    )
    .await;
}
