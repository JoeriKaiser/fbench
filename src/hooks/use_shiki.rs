use dioxus::prelude::*;
use serde_json::Value;

pub struct ShikiHighlighter;

impl ShikiHighlighter {
    pub async fn new() -> Result<Self, document::EvalError> {
        let mut eval = document::eval(
            r#"
            try {
                if (typeof window.shikiHighlighter === 'undefined') {
                    const shiki = await import('https://esm.sh/shiki@3.0.0');
                    
                    const highlighter = await shiki.createHighlighter({
                        themes: ['nord'],
                        langs: ['sql'],
                    });
                    
                    window.shikiHighlighter = highlighter;
                    window.shikiTheme = 'nord';
                }
                dioxus.send({ success: true, error: null });
            } catch (err) {
                console.error('Shiki initialization error:', err);
                dioxus.send({ success: false, error: err.toString() });
            }
        "#,
        );

        let result = eval.recv::<Value>().await?;
        if result["success"].as_bool() != Some(true) {
            let error = result["error"].as_str().unwrap_or("Unknown error");
            tracing::error!("Shiki initialization failed: {}", error);
            return Err(document::EvalError::Communication(
                "Shiki initialization failed".into(),
            ));
        }

        Ok(Self)
    }

    pub async fn highlight(&self, code: &str) -> Result<String, document::EvalError> {
        // Escape special characters for JavaScript template literal
        let escaped_code = code
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace('$', "\\$")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r");

        let mut eval = document::eval(&format!(
            r#"
            try {{
                const html = window.shikiHighlighter.codeToHtml(`{escaped_code}`, {{
                    lang: 'sql',
                    theme: window.shikiTheme,
                }});
                dioxus.send({{ success: true, html: html }});
            }} catch (err) {{
                console.error('Shiki highlight error:', err);
                dioxus.send({{ success: false, error: err.toString() }});
            }}
        "#
        ));

        let result = eval.recv::<Value>().await?;
        if result["success"].as_bool() == Some(true) {
            Ok(result["html"].as_str().unwrap_or_default().to_string())
        } else {
            let error = result["error"].as_str().unwrap_or("Unknown error");
            tracing::error!("Shiki highlight failed: {}", error);
            Err(document::EvalError::Communication(
                "Shiki highlight failed".into(),
            ))
        }
    }
}

#[derive(Clone, Copy)]
pub struct UseShiki {
    highlighter: Signal<Option<ShikiHighlighter>>,
    ready: Signal<bool>,
}

pub fn use_shiki() -> UseShiki {
    let mut highlighter = use_signal(|| None::<ShikiHighlighter>);
    let mut ready = use_signal(|| false);

    use_hook(move || {
        spawn(async move {
            match ShikiHighlighter::new().await {
                Ok(h) => {
                    highlighter.set(Some(h));
                    ready.set(true);
                    tracing::info!("Shiki highlighter initialized successfully");
                }
                Err(e) => {
                    tracing::error!("Failed to initialize Shiki: {:?}", e);
                    ready.set(false);
                }
            }
        });
    });

    UseShiki { highlighter, ready }
}

impl UseShiki {
    pub async fn highlight(&self, code: &str) -> Option<String> {
        if !*self.ready.read() {
            return None;
        }
        let highlighter = self.highlighter.read();
        if let Some(ref h) = *highlighter {
            match h.highlight(code).await {
                Ok(html) => Some(html),
                Err(e) => {
                    tracing::error!("Highlight error: {:?}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn is_ready(&self) -> bool {
        *self.ready.read()
    }
}
