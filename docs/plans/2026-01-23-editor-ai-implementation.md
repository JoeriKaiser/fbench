# Editor & AI Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix editor keyboard navigation bugs and add lightweight AI features (quick actions + schema suggestions).

**Architecture:** Intercept keyboard events before egui's default handling for editor fixes. Extend LlmRequest/LlmResponse enums for new AI actions. Add inline results panel and context menu to editor. Add suggestions section to schema panel.

**Tech Stack:** Rust, egui/eframe, tokio channels for async LLM calls

---

## Task 1: Fix Ctrl+Arrow Word Navigation

**Files:**
- Modify: `src/ui/editor.rs`

**Step 1: Add word boundary detection helper**

Add after line 135 (after `get_word_bounds`):

```rust
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn find_word_boundary_right(text: &str, cursor_byte: usize) -> usize {
    let bytes = text.as_bytes();
    let mut i = cursor_byte;

    // Skip current word characters
    while i < bytes.len() {
        let ch = text[i..].chars().next().unwrap();
        if !Self::is_word_char(ch) { break; }
        i += ch.len_utf8();
    }

    // Skip whitespace/punctuation
    while i < bytes.len() {
        let ch = text[i..].chars().next().unwrap();
        if Self::is_word_char(ch) { break; }
        i += ch.len_utf8();
    }

    i
}

fn find_word_boundary_left(text: &str, cursor_byte: usize) -> usize {
    if cursor_byte == 0 { return 0; }

    let mut i = cursor_byte;

    // Move back one char to start
    let before = &text[..i];
    if let Some(ch) = before.chars().last() {
        i -= ch.len_utf8();
    }

    // Skip whitespace/punctuation going backwards
    while i > 0 {
        let ch = text[..i].chars().last().unwrap();
        if Self::is_word_char(ch) { break; }
        i -= ch.len_utf8();
    }

    // Skip word characters going backwards
    while i > 0 {
        let ch = text[..i].chars().last().unwrap();
        if !Self::is_word_char(ch) { break; }
        i -= ch.len_utf8();
    }

    i
}
```

**Step 2: Add navigation key handler**

Add this method to the `Editor` impl:

```rust
fn handle_ctrl_navigation(&mut self, ui: &mut egui::Ui, text_edit_id: egui::Id) -> bool {
    let mut handled = false;

    ui.input_mut(|input| {
        if input.modifiers.ctrl && !input.modifiers.shift {
            if input.key_pressed(egui::Key::ArrowRight) {
                let cursor_b = Self::char_to_byte(&self.query, self.get_cursor_char_index());
                let new_pos = Self::find_word_boundary_right(&self.query, cursor_b);
                self.set_cursor_byte_position(new_pos);
                input.consume_key(egui::Modifiers::CTRL, egui::Key::ArrowRight);
                handled = true;
            }
            if input.key_pressed(egui::Key::ArrowLeft) {
                let cursor_b = Self::char_to_byte(&self.query, self.get_cursor_char_index());
                let new_pos = Self::find_word_boundary_left(&self.query, cursor_b);
                self.set_cursor_byte_position(new_pos);
                input.consume_key(egui::Modifiers::CTRL, egui::Key::ArrowLeft);
                handled = true;
            }
        }
    });

    handled
}

fn get_cursor_char_index(&self) -> usize {
    self.autocomplete.last_cursor_char
}

fn set_cursor_byte_position(&mut self, _byte_pos: usize) {
    // Note: egui TextEdit cursor position is managed internally
    // We'll need to track this differently - see Step 3
}
```

**Step 3: Refactor to track cursor state**

Add field to `Editor` struct around line 84:

```rust
pub struct Editor {
    pub query: String,
    schema: SchemaInfo,
    autocomplete: AutocompleteState,
    wrap: bool,
    pending_cursor_move: Option<usize>, // byte position to move cursor to
}
```

Update `Default` impl:

```rust
impl Default for Editor {
    fn default() -> Self {
        Self {
            query: String::from("SELECT * FROM users LIMIT 10;"),
            schema: SchemaInfo::default(),
            autocomplete: AutocompleteState::default(),
            wrap: false,
            pending_cursor_move: None,
        }
    }
}
```

Update `set_cursor_byte_position`:

```rust
fn set_cursor_byte_position(&mut self, byte_pos: usize) {
    self.pending_cursor_move = Some(byte_pos);
}
```

**Step 4: Apply pending cursor move in show()**

In the `show` method, after the TextEdit is created (around line 481), add cursor handling:

```rust
let output = output.unwrap();

// Apply pending cursor move
if let Some(byte_pos) = self.pending_cursor_move.take() {
    let char_pos = self.query[..byte_pos.min(self.query.len())]
        .chars()
        .count();
    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
        let ccursor = egui::text::CCursor::new(char_pos);
        state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
        state.store(ui.ctx(), text_edit_id);
    }
}
```

**Step 5: Call handler early in show()**

At the start of `show()`, after creating `text_edit_id` (around line 455):

```rust
let text_edit_id = ui.make_persistent_id("sql_editor_textedit");
self.handle_ctrl_navigation(ui, text_edit_id);
```

**Step 6: Test manually**

Run: `cargo run --release`
- Type `SELECT * FROM users WHERE id = 1`
- Place cursor at start
- Press Ctrl+Right repeatedly
- Should stop at: SELECT|, *|, FROM|, users|, WHERE|, id|, =|, 1|

**Step 7: Commit**

```bash
git add src/ui/editor.rs
git commit -m "fix: Ctrl+Arrow word navigation stops at word boundaries"
```

---

## Task 2: Fix Tab Key for Autocomplete

**Files:**
- Modify: `src/ui/editor.rs`

**Step 1: Consume Tab event when autocomplete active**

Replace the autocomplete key handling block (around line 519-535) with:

```rust
if self.autocomplete.active && !self.autocomplete.suggestions.is_empty() {
    let (mut dismiss, mut apply) = (false, false);

    ui.input_mut(|input| {
        if input.key_pressed(egui::Key::Escape) {
            dismiss = true;
        }
        if input.key_pressed(egui::Key::ArrowDown) {
            self.autocomplete.selected = (self.autocomplete.selected + 1)
                .min(self.autocomplete.suggestions.len() - 1);
            // Consume to prevent textarea scrolling
            input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown);
        }
        if input.key_pressed(egui::Key::ArrowUp) {
            self.autocomplete.selected = self.autocomplete.selected.saturating_sub(1);
            input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp);
        }
        // Consume Tab to prevent focus change
        if input.key_pressed(egui::Key::Tab) {
            apply = true;
            input.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
        }
        // Enter without modifiers accepts suggestion
        if input.key_pressed(egui::Key::Enter) && !input.modifiers.ctrl && !input.modifiers.shift {
            apply = true;
            input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
        }
    });

    if dismiss {
        self.dismiss_autocomplete();
    } else if apply {
        if let Some(s) = self.autocomplete.suggestions.get(self.autocomplete.selected).cloned() {
            self.apply_suggestion(&s);
            ui.memory_mut(|m| m.request_focus(text_edit_id));
        }
    }
}
```

**Step 2: Test manually**

Run: `cargo run --release`
- Connect to a database
- Type `SEL` in the editor
- Autocomplete should appear with `SELECT`
- Press Tab
- Should insert `SELECT`, NOT move focus to another UI element

**Step 3: Commit**

```bash
git add src/ui/editor.rs
git commit -m "fix: Tab key accepts autocomplete instead of changing focus"
```

---

## Task 3: Add LLM Request/Response Types for AI Actions

**Files:**
- Modify: `src/llm/mod.rs`

**Step 1: Extend LlmRequest enum**

Replace the `LlmRequest` enum (around line 64-71):

```rust
#[derive(Debug)]
pub enum LlmRequest {
    Generate {
        prompt: String,
        schema: SchemaInfo,
        config: LlmConfig,
    },
    Explain {
        sql: String,
        config: LlmConfig,
    },
    Optimize {
        sql: String,
        schema: SchemaInfo,
        config: LlmConfig,
    },
    FixError {
        sql: String,
        error: String,
        schema: SchemaInfo,
        config: LlmConfig,
    },
    SuggestQueries {
        table: crate::db::TableInfo,
        config: LlmConfig,
    },
}
```

**Step 2: Extend LlmResponse enum**

Replace the `LlmResponse` enum (around line 73-77):

```rust
#[derive(Debug, Clone)]
pub struct QuerySuggestion {
    pub label: String,
    pub sql: String,
}

#[derive(Debug)]
pub enum LlmResponse {
    Generated(String),
    Explanation(String),
    Optimization { explanation: String, sql: Option<String> },
    ErrorFix { explanation: String, sql: Option<String> },
    QuerySuggestions(Vec<QuerySuggestion>),
    Error(String),
}
```

**Step 3: Commit**

```bash
git add src/llm/mod.rs
git commit -m "feat: add LLM request/response types for AI actions"
```

---

## Task 4: Implement LLM Handlers for New Request Types

**Files:**
- Modify: `src/llm/mod.rs`

**Step 1: Add handler dispatch in run()**

Update the `run` method (around line 136-145):

```rust
pub async fn run(mut self) {
    while let Some(request) = self.request_rx.recv().await {
        let response = match request {
            LlmRequest::Generate { prompt, schema, config } => {
                self.generate(&prompt, &schema, &config).await
            }
            LlmRequest::Explain { sql, config } => {
                self.explain(&sql, &config).await
            }
            LlmRequest::Optimize { sql, schema, config } => {
                self.optimize(&sql, &schema, &config).await
            }
            LlmRequest::FixError { sql, error, schema, config } => {
                self.fix_error(&sql, &error, &schema, &config).await
            }
            LlmRequest::SuggestQueries { table, config } => {
                self.suggest_queries(&table, &config).await
            }
        };
        let _ = self.response_tx.send(response);
    }
}
```

**Step 2: Add explain handler**

Add after the `generate` method:

```rust
async fn explain(&self, sql: &str, config: &LlmConfig) -> LlmResponse {
    let prompt = format!(
        "Explain this SQL query in plain English. Be concise (2-3 sentences).\n\n\
         Query:\n{}\n\nExplanation:",
        sql
    );

    let result = match config.provider {
        LlmProvider::Ollama => self.call_ollama(&prompt, config).await,
        LlmProvider::OpenRouter => self.call_openrouter(&prompt, config).await,
    };

    match result {
        Ok(text) => LlmResponse::Explanation(text.trim().to_string()),
        Err(e) => LlmResponse::Error(e),
    }
}
```

**Step 3: Add optimize handler**

```rust
async fn optimize(&self, sql: &str, schema: &SchemaInfo, config: &LlmConfig) -> LlmResponse {
    let schema_text = self.format_schema(schema);
    let prompt = format!(
        "Analyze this SQL query for performance improvements.\n\n\
         Schema:\n{}\n\n\
         Query:\n{}\n\n\
         Provide:\n\
         1. Brief explanation of potential issues (1-2 sentences)\n\
         2. Optimized query if applicable\n\n\
         Format your response as:\n\
         EXPLANATION: <your explanation>\n\
         SQL: <optimized query or 'NO_CHANGE' if already optimal>",
        schema_text, sql
    );

    let result = match config.provider {
        LlmProvider::Ollama => self.call_ollama(&prompt, config).await,
        LlmProvider::OpenRouter => self.call_openrouter(&prompt, config).await,
    };

    match result {
        Ok(text) => Self::parse_optimization_response(&text),
        Err(e) => LlmResponse::Error(e),
    }
}

fn parse_optimization_response(response: &str) -> LlmResponse {
    let explanation = response
        .lines()
        .find(|l| l.starts_with("EXPLANATION:"))
        .map(|l| l.trim_start_matches("EXPLANATION:").trim().to_string())
        .unwrap_or_else(|| response.trim().to_string());

    let sql = response
        .lines()
        .skip_while(|l| !l.starts_with("SQL:"))
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    let sql = if sql.is_empty() || sql == "NO_CHANGE" {
        None
    } else {
        Some(Self::extract_sql(&sql))
    };

    LlmResponse::Optimization { explanation, sql }
}
```

**Step 4: Add fix_error handler**

```rust
async fn fix_error(&self, sql: &str, error: &str, schema: &SchemaInfo, config: &LlmConfig) -> LlmResponse {
    let schema_text = self.format_schema(schema);
    let prompt = format!(
        "This SQL query failed with an error. Explain the problem and provide a fix.\n\n\
         Schema:\n{}\n\n\
         Query:\n{}\n\n\
         Error:\n{}\n\n\
         Format your response as:\n\
         EXPLANATION: <what went wrong>\n\
         SQL: <corrected query>",
        schema_text, sql, error
    );

    let result = match config.provider {
        LlmProvider::Ollama => self.call_ollama(&prompt, config).await,
        LlmProvider::OpenRouter => self.call_openrouter(&prompt, config).await,
    };

    match result {
        Ok(text) => Self::parse_fix_response(&text),
        Err(e) => LlmResponse::Error(e),
    }
}

fn parse_fix_response(response: &str) -> LlmResponse {
    let explanation = response
        .lines()
        .find(|l| l.starts_with("EXPLANATION:"))
        .map(|l| l.trim_start_matches("EXPLANATION:").trim().to_string())
        .unwrap_or_else(|| response.lines().next().unwrap_or("").to_string());

    let sql_start = response.find("SQL:");
    let sql = sql_start
        .map(|i| {
            let after = &response[i + 4..];
            Self::extract_sql(after.trim())
        })
        .filter(|s| !s.is_empty());

    LlmResponse::ErrorFix { explanation, sql }
}
```

**Step 5: Add suggest_queries handler**

```rust
async fn suggest_queries(&self, table: &crate::db::TableInfo, config: &LlmConfig) -> LlmResponse {
    let columns: Vec<String> = table.columns.iter()
        .map(|c| {
            let pk = if c.is_primary_key { " PK" } else { "" };
            format!("{} {}{}", c.name, c.data_type, pk)
        })
        .collect();

    let prompt = format!(
        "Suggest 3 useful SQL queries for this table. Keep labels short (3-5 words).\n\n\
         Table: {}\n\
         Columns:\n{}\n\
         Row estimate: {}\n\n\
         Format each suggestion as:\n\
         LABEL: <short description>\n\
         SQL: <query>\n\
         ---",
        table.name,
        columns.join("\n"),
        table.row_estimate
    );

    let result = match config.provider {
        LlmProvider::Ollama => self.call_ollama(&prompt, config).await,
        LlmProvider::OpenRouter => self.call_openrouter(&prompt, config).await,
    };

    match result {
        Ok(text) => Self::parse_suggestions_response(&text),
        Err(e) => LlmResponse::Error(e),
    }
}

fn parse_suggestions_response(response: &str) -> LlmResponse {
    let mut suggestions = Vec::new();
    let mut current_label = String::new();
    let mut current_sql = String::new();
    let mut in_sql = false;

    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("LABEL:") {
            if !current_label.is_empty() && !current_sql.is_empty() {
                suggestions.push(QuerySuggestion {
                    label: current_label.clone(),
                    sql: Self::extract_sql(&current_sql),
                });
            }
            current_label = trimmed.trim_start_matches("LABEL:").trim().to_string();
            current_sql.clear();
            in_sql = false;
        } else if trimmed.starts_with("SQL:") {
            current_sql = trimmed.trim_start_matches("SQL:").trim().to_string();
            in_sql = true;
        } else if trimmed == "---" {
            if !current_label.is_empty() && !current_sql.is_empty() {
                suggestions.push(QuerySuggestion {
                    label: current_label.clone(),
                    sql: Self::extract_sql(&current_sql),
                });
            }
            current_label.clear();
            current_sql.clear();
            in_sql = false;
        } else if in_sql && !trimmed.is_empty() {
            current_sql.push('\n');
            current_sql.push_str(trimmed);
        }
    }

    // Don't forget the last one
    if !current_label.is_empty() && !current_sql.is_empty() {
        suggestions.push(QuerySuggestion {
            label: current_label,
            sql: Self::extract_sql(&current_sql),
        });
    }

    suggestions.truncate(3);
    LlmResponse::QuerySuggestions(suggestions)
}
```

**Step 6: Add format_schema helper**

```rust
fn format_schema(&self, schema: &SchemaInfo) -> String {
    let mut text = String::new();
    for table in &schema.tables {
        text.push_str(&format!("Table: {}\n", table.name));
        for col in &table.columns {
            let pk = if col.is_primary_key { " PK" } else { "" };
            let null = if col.nullable { "?" } else { "" };
            text.push_str(&format!("  {} {}{}{}\n", col.name, col.data_type, null, pk));
        }
    }
    text
}
```

**Step 7: Update build_prompt to use format_schema**

Replace the `build_prompt` method body to use the new helper:

```rust
fn build_prompt(&self, user_prompt: &str, schema: &SchemaInfo) -> String {
    let schema_text = self.format_schema(schema);
    format!(
        "You are a SQL expert. Generate a SQL query based on the user's request.\n\
         Only output the raw SQL query, no explanations, no markdown.\n\n\
         Database schema:\n{}\n\
         User request: {}\n\nSQL:",
        schema_text, user_prompt
    )
}
```

**Step 8: Verify it compiles**

Run: `cargo build`

**Step 9: Commit**

```bash
git add src/llm/mod.rs
git commit -m "feat: implement LLM handlers for explain, optimize, fix, suggest"
```

---

## Task 5: Add Inline AI Results Panel to Editor

**Files:**
- Modify: `src/ui/editor.rs`

**Step 1: Add AI panel state struct**

Add after the `AutocompleteState` struct (around line 82):

```rust
#[derive(Default)]
struct AiPanelState {
    visible: bool,
    loading: bool,
    title: String,
    content: String,
    suggested_sql: Option<String>,
}
```

**Step 2: Add field to Editor**

Update `Editor` struct:

```rust
pub struct Editor {
    pub query: String,
    schema: SchemaInfo,
    autocomplete: AutocompleteState,
    wrap: bool,
    pending_cursor_move: Option<usize>,
    ai_panel: AiPanelState,
    last_error: Option<String>,
}
```

Update `Default`:

```rust
impl Default for Editor {
    fn default() -> Self {
        Self {
            query: String::from("SELECT * FROM users LIMIT 10;"),
            schema: SchemaInfo::default(),
            autocomplete: AutocompleteState::default(),
            wrap: false,
            pending_cursor_move: None,
            ai_panel: AiPanelState::default(),
            last_error: None,
        }
    }
}
```

**Step 3: Add method to set error**

```rust
pub fn set_last_error(&mut self, error: Option<String>) {
    self.last_error = error;
}
```

**Step 4: Add method to handle LLM responses**

```rust
pub fn handle_llm_response(&mut self, response: &crate::llm::LlmResponse) {
    use crate::llm::LlmResponse;

    self.ai_panel.loading = false;

    match response {
        LlmResponse::Explanation(text) => {
            self.ai_panel.title = "Explanation".to_string();
            self.ai_panel.content = text.clone();
            self.ai_panel.suggested_sql = None;
            self.ai_panel.visible = true;
        }
        LlmResponse::Optimization { explanation, sql } => {
            self.ai_panel.title = "Optimization".to_string();
            self.ai_panel.content = explanation.clone();
            self.ai_panel.suggested_sql = sql.clone();
            self.ai_panel.visible = true;
        }
        LlmResponse::ErrorFix { explanation, sql } => {
            self.ai_panel.title = "Error Fix".to_string();
            self.ai_panel.content = explanation.clone();
            self.ai_panel.suggested_sql = sql.clone();
            self.ai_panel.visible = true;
        }
        _ => {}
    }
}
```

**Step 5: Add AI panel rendering method**

```rust
fn show_ai_panel(&mut self, ui: &mut egui::Ui) -> bool {
    if !self.ai_panel.visible { return false; }

    let mut applied = false;
    let mut dismiss = false;

    egui::Frame::group(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(&self.ai_panel.title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").clicked() {
                        dismiss = true;
                    }
                });
            });

            ui.add_space(4.0);

            if self.ai_panel.loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Thinking...");
                });
            } else {
                ui.label(&self.ai_panel.content);

                if let Some(sql) = &self.ai_panel.suggested_sql {
                    ui.add_space(4.0);
                    egui::Frame::group(ui.style())
                        .fill(ui.visuals().code_bg_color)
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(sql).monospace().size(12.0)
                            ));
                        });
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("Copy").clicked() {
                        ui.ctx().copy_text(self.ai_panel.content.clone());
                    }
                    if let Some(_sql) = &self.ai_panel.suggested_sql {
                        if ui.button("Apply SQL").clicked() {
                            applied = true;
                        }
                    }
                    if ui.button("Dismiss").clicked() {
                        dismiss = true;
                    }
                });
            }
        });

    if dismiss {
        self.ai_panel.visible = false;
    }

    applied
}
```

**Step 6: Integrate panel into show()**

In the `show` method, after the editor frame (around line 479), before the status line:

```rust
// Show AI panel if active
if self.show_ai_panel(ui) {
    if let Some(sql) = self.ai_panel.suggested_sql.take() {
        self.query = sql;
        self.ai_panel.visible = false;
    }
}
```

**Step 7: Commit**

```bash
git add src/ui/editor.rs
git commit -m "feat: add inline AI results panel to editor"
```

---

## Task 6: Add Context Menu with AI Actions

**Files:**
- Modify: `src/ui/editor.rs`

**Step 1: Add AI action enum**

Add near the top of the file:

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum AiAction {
    Explain,
    Optimize,
    FixError,
}
```

**Step 2: Update EditorAction to include AI requests**

```rust
pub struct EditorAction {
    pub execute_sql: Option<String>,
    pub save: bool,
    pub ai_action: Option<(AiAction, String)>, // (action, selected_sql)
}
```

**Step 3: Add context menu method**

```rust
fn show_context_menu(&mut self, ui: &mut egui::Ui, response: &egui::Response) -> Option<(AiAction, String)> {
    let mut result = None;

    response.context_menu(|ui| {
        let selected = self.get_selected_text();
        let has_selection = selected.is_some();
        let sql = selected.unwrap_or_else(|| self.query.clone());

        ui.set_min_width(160.0);

        if ui.add_enabled(has_selection || !self.query.is_empty(),
            egui::Button::new("✦ Explain")).clicked() {
            result = Some((AiAction::Explain, sql.clone()));
            ui.close_menu();
        }

        if ui.add_enabled(has_selection || !self.query.is_empty(),
            egui::Button::new("⚡ Optimize")).clicked() {
            result = Some((AiAction::Optimize, sql.clone()));
            ui.close_menu();
        }

        let has_error = self.last_error.is_some();
        if ui.add_enabled(has_error && (has_selection || !self.query.is_empty()),
            egui::Button::new("🔧 Fix Error")).clicked() {
            result = Some((AiAction::FixError, sql.clone()));
            ui.close_menu();
        }
    });

    result
}

fn get_selected_text(&self) -> Option<String> {
    // We need to track selection state - for now return None
    // This will be connected when we have cursor range tracking
    None
}
```

**Step 4: Track selection in Editor**

Add field:

```rust
pub struct Editor {
    // ... existing fields ...
    current_selection: Option<(usize, usize)>, // (start_char, end_char)
}
```

Update Default and update `get_selected_text`:

```rust
fn get_selected_text(&self) -> Option<String> {
    let (start, end) = self.current_selection?;
    if start == end { return None; }
    let (s, e) = (start.min(end), start.max(end));
    let start_b = Self::char_to_byte(&self.query, s);
    let end_b = Self::char_to_byte(&self.query, e);
    Some(self.query[start_b..end_b].to_string())
}
```

**Step 5: Update show() to track selection and show context menu**

After creating the TextEdit output, track the selection:

```rust
// Track selection for context menu
if let Some(cr) = &output.cursor_range {
    let (a, b) = (cr.primary.ccursor.index, cr.secondary.ccursor.index);
    self.current_selection = Some((a, b));
} else {
    self.current_selection = None;
}

// Show context menu
if let Some((ai_action, sql)) = self.show_context_menu(ui, &output.response) {
    self.ai_panel.loading = true;
    self.ai_panel.visible = true;
    self.ai_panel.title = match ai_action {
        AiAction::Explain => "Explaining...",
        AiAction::Optimize => "Optimizing...",
        AiAction::FixError => "Fixing...",
    }.to_string();
    action.ai_action = Some((ai_action, sql));
}
```

**Step 6: Update EditorAction default**

Make sure `ai_action` field has a default:

```rust
let mut action = EditorAction {
    execute_sql: None,
    save: false,
    ai_action: None,
};
```

**Step 7: Commit**

```bash
git add src/ui/editor.rs
git commit -m "feat: add right-click context menu with AI actions"
```

---

## Task 7: Wire Up AI Actions in App

**Files:**
- Modify: `src/app.rs`
- Modify: `src/ui/editor.rs` (export AiAction)

**Step 1: Export AiAction from editor module**

In `src/ui/mod.rs`, ensure AiAction is exported:

```rust
pub use editor::{Editor, EditorAction, AiAction};
```

**Step 2: Update app.rs imports**

```rust
use crate::ui::{
    AiAction, AiPrompt, ConnectionDialog, Editor, QueriesPanel, Results, SchemaPanel,
    TableDetailPanel,
};
```

**Step 3: Handle AI actions in update()**

In the central panel section where editor action is handled (around line 251-259), add:

```rust
let action = self.editor.show(ui);
if let Some(sql) = action.execute_sql {
    if self.statusbar.connected {
        self.execute_query(&sql);
    }
}
if action.save {
    self.queries_panel.open_save_dialog();
}
if let Some((ai_action, sql)) = action.ai_action {
    let config = crate::llm::LlmConfig::load();
    let request = match ai_action {
        AiAction::Explain => crate::llm::LlmRequest::Explain {
            sql,
            config,
        },
        AiAction::Optimize => crate::llm::LlmRequest::Optimize {
            sql,
            schema: self.schema.clone(),
            config,
        },
        AiAction::FixError => crate::llm::LlmRequest::FixError {
            sql,
            error: self.results.error.clone().unwrap_or_default(),
            schema: self.schema.clone(),
            config,
        },
    };
    let _ = self.llm_tx.send(request);
}
```

**Step 4: Handle AI responses for editor**

In `poll_responses`, add handling for new response types:

```rust
while let Ok(response) = self.llm_rx.try_recv() {
    match &response {
        LlmResponse::Generated(sql) => {
            self.ai_prompt.set_generating(false);
            self.ai_prompt.take_prompt();
            self.editor.query = sql.clone();
        }
        LlmResponse::Explanation(_) |
        LlmResponse::Optimization { .. } |
        LlmResponse::ErrorFix { .. } => {
            self.editor.handle_llm_response(&response);
        }
        LlmResponse::QuerySuggestions(_) => {
            self.schema_panel.handle_llm_response(&response);
        }
        LlmResponse::Error(e) => {
            self.ai_prompt.set_error(e.clone());
            self.editor.handle_llm_response(&response);
        }
    }
}
```

**Step 5: Pass error to editor when query fails**

In the `DbResponse::Error` handler:

```rust
DbResponse::Error(e) => {
    self.results.set_error(e.clone());
    self.editor.set_last_error(Some(e));
}
```

Clear error on success:

```rust
DbResponse::QueryResult(result) => {
    self.statusbar.row_count = Some(result.rows.len());
    self.statusbar.exec_time_ms = Some(result.execution_time_ms);
    self.results.set_result(result);
    self.editor.set_last_error(None);
}
```

**Step 6: Verify it compiles**

Run: `cargo build`

**Step 7: Commit**

```bash
git add src/ui/editor.rs src/ui/mod.rs src/app.rs
git commit -m "feat: wire up AI actions from editor to LLM worker"
```

---

## Task 8: Add Schema-Aware Suggestions Panel

**Files:**
- Modify: `src/ui/schema.rs`

**Step 1: Add suggestion state**

Add to `SchemaPanel` struct:

```rust
use crate::llm::{LlmResponse, QuerySuggestion};

#[derive(Default)]
pub struct SchemaPanel {
    pub schema: SchemaInfo,
    expanded_tables: std::collections::HashSet<String>,
    tables_expanded: bool,
    views_expanded: bool,
    pub selection: SchemaSelection,
    search_query: String,
    // New fields
    suggestions: Vec<QuerySuggestion>,
    suggestions_loading: bool,
    suggestions_table: Option<String>,
}
```

**Step 2: Add action for requesting suggestions**

Update `SchemaPanelAction`:

```rust
#[derive(Default)]
pub struct SchemaPanelAction {
    pub select_table_data: Option<String>,
    pub view_table_structure: Option<String>,
    pub request_suggestions: Option<crate::db::TableInfo>,
}
```

**Step 3: Add LLM response handler**

```rust
pub fn handle_llm_response(&mut self, response: &LlmResponse) {
    match response {
        LlmResponse::QuerySuggestions(suggestions) => {
            self.suggestions = suggestions.clone();
            self.suggestions_loading = false;
        }
        LlmResponse::Error(_) => {
            self.suggestions_loading = false;
            // Keep old suggestions or show fallback
        }
        _ => {}
    }
}
```

**Step 4: Add suggestions rendering method**

```rust
fn show_suggestions(&mut self, ui: &mut egui::Ui, action: &mut SchemaPanelAction) {
    let selected_table = match &self.selection {
        SchemaSelection::Table(name) => Some(name.clone()),
        _ => None,
    };

    // Request suggestions when table changes
    if let Some(table_name) = &selected_table {
        if self.suggestions_table.as_ref() != Some(table_name) {
            self.suggestions_table = Some(table_name.clone());
            self.suggestions_loading = true;
            self.suggestions.clear();

            if let Some(table) = self.schema.tables.iter().find(|t| &t.name == table_name) {
                action.request_suggestions = Some(table.clone());
            }
        }
    }

    if selected_table.is_none() {
        return;
    }

    ui.add_space(8.0);
    ui.separator();

    ui.horizontal(|ui| {
        ui.strong("Suggested Queries");
        if self.suggestions_loading {
            ui.spinner();
        } else if ui.small_button("↻").on_hover_text("Refresh suggestions").clicked() {
            if let Some(table_name) = &self.suggestions_table {
                if let Some(table) = self.schema.tables.iter().find(|t| &t.name == table_name) {
                    self.suggestions_loading = true;
                    action.request_suggestions = Some(table.clone());
                }
            }
        }
    });

    ui.add_space(4.0);

    if self.suggestions.is_empty() && !self.suggestions_loading {
        ui.colored_label(Color32::GRAY, "No suggestions available");
    } else {
        for suggestion in &self.suggestions {
            let response = ui.selectable_label(false, format!("▸ {}", suggestion.label));
            if response.clicked() {
                action.select_table_data = None; // Don't also select data
                // Signal to set query - we'll handle this via a new action field
            }
            response.on_hover_text(&suggestion.sql);
        }
    }
}
```

**Step 5: Add action field for suggestion selection**

```rust
#[derive(Default)]
pub struct SchemaPanelAction {
    pub select_table_data: Option<String>,
    pub view_table_structure: Option<String>,
    pub request_suggestions: Option<crate::db::TableInfo>,
    pub apply_suggestion: Option<String>, // SQL to insert
}
```

Update the suggestion click handler:

```rust
for suggestion in &self.suggestions {
    let response = ui.selectable_label(false, format!("▸ {}", suggestion.label));
    if response.clicked() {
        action.apply_suggestion = Some(suggestion.sql.clone());
    }
    response.on_hover_text(&suggestion.sql);
}
```

**Step 6: Call show_suggestions in show()**

At the end of the `show` method, before returning action:

```rust
self.show_suggestions(ui, &mut action);

action
```

**Step 7: Commit**

```bash
git add src/ui/schema.rs
git commit -m "feat: add schema-aware AI suggestions panel"
```

---

## Task 9: Wire Up Schema Suggestions in App

**Files:**
- Modify: `src/app.rs`

**Step 1: Handle suggestion actions**

In the schema panel section of `update()`, handle the new actions:

```rust
LeftTab::Schema => {
    let action = self.schema_panel.show(ui);
    if let Some(t) = action.select_table_data {
        self.select_table_data(&t);
    }
    if let Some(t) = action.view_table_structure {
        self.view_table_structure(&t);
    }
    if let Some(table) = action.request_suggestions {
        let config = crate::llm::LlmConfig::load();
        let _ = self.llm_tx.send(crate::llm::LlmRequest::SuggestQueries {
            table,
            config,
        });
    }
    if let Some(sql) = action.apply_suggestion {
        self.editor.query = sql;
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Test manually**

Run: `cargo run --release`
- Connect to a database
- Click on a table in the schema panel
- Should see "Suggested Queries" section appear
- Should show loading spinner, then suggestions
- Click a suggestion to insert it into editor

**Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: wire up schema suggestions to LLM worker"
```

---

## Task 10: Final Testing and Polish

**Step 1: Test all features**

Run: `cargo run --release`

Test checklist:
- [ ] Ctrl+Right stops at word boundaries
- [ ] Ctrl+Left stops at word boundaries
- [ ] Tab accepts autocomplete suggestion
- [ ] Right-click shows context menu with Explain/Optimize/Fix Error
- [ ] Clicking Explain shows inline panel with explanation
- [ ] Clicking Optimize shows optimization suggestions
- [ ] After query error, Fix Error is enabled and provides fix
- [ ] Selecting a table shows suggested queries
- [ ] Clicking a suggestion inserts SQL into editor

**Step 2: Fix any issues found**

Address any bugs discovered during testing.

**Step 3: Final commit**

```bash
git add -A
git commit -m "chore: polish and fix any remaining issues"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Fix Ctrl+Arrow navigation | `src/ui/editor.rs` |
| 2 | Fix Tab for autocomplete | `src/ui/editor.rs` |
| 3 | Add LLM types | `src/llm/mod.rs` |
| 4 | Implement LLM handlers | `src/llm/mod.rs` |
| 5 | Add AI results panel | `src/ui/editor.rs` |
| 6 | Add context menu | `src/ui/editor.rs` |
| 7 | Wire up AI actions | `src/app.rs` |
| 8 | Add suggestions panel | `src/ui/schema.rs` |
| 9 | Wire up suggestions | `src/app.rs` |
| 10 | Final testing | All files |
