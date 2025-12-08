use eframe::egui::{self, text::LayoutJob, Color32, FontId, TextFormat};
use crate::db::SchemaInfo;

const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER",
    "TABLE", "INDEX", "VIEW", "INTO", "VALUES", "SET", "AND", "OR", "NOT", "NULL",
    "IS", "IN", "LIKE", "BETWEEN", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL",
    "ON", "AS", "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "DISTINCT",
    "UNION", "ALL", "EXISTS", "CASE", "WHEN", "THEN", "ELSE", "END", "ASC", "DESC",
    "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT", "DEFAULT", "CHECK",
    "UNIQUE", "CASCADE", "RETURNING", "WITH", "RECURSIVE", "OVER", "PARTITION",
];

const SQL_TYPES: &[&str] = &[
    "INT", "INTEGER", "BIGINT", "SMALLINT", "SERIAL", "BIGSERIAL", "TEXT", "VARCHAR",
    "CHAR", "BOOLEAN", "BOOL", "DATE", "TIME", "TIMESTAMP", "TIMESTAMPTZ", "UUID",
    "JSON", "JSONB", "NUMERIC", "DECIMAL", "REAL", "FLOAT", "DOUBLE", "BYTEA",
];

const SQL_FUNCTIONS: &[&str] = &[
    "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "NULLIF", "NOW", "CURRENT_DATE",
    "CURRENT_TIMESTAMP", "EXTRACT", "TO_CHAR", "TO_DATE", "CAST", "ARRAY_AGG",
    "STRING_AGG", "ROW_NUMBER", "RANK", "DENSE_RANK", "LAG", "LEAD", "FIRST_VALUE",
    "LAST_VALUE", "LOWER", "UPPER", "TRIM", "LENGTH", "SUBSTRING", "REPLACE", "CONCAT",
];

pub struct Editor {
    pub query: String,
    pub schema: SchemaInfo,
    autocomplete: AutocompleteState,
}

struct AutocompleteState {
    show: bool,
    suggestions: Vec<String>,
    selected: usize,
    filter: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            query: String::from("SELECT * FROM users LIMIT 10;"),
            schema: SchemaInfo::default(),
            autocomplete: AutocompleteState {
                show: false,
                suggestions: Vec::new(),
                selected: 0,
                filter: String::new(),
            },
        }
    }
}

pub struct EditorAction {
    pub execute: bool,
    pub save: bool,
}

impl Editor {
    pub fn set_schema(&mut self, schema: SchemaInfo) {
        self.schema = schema;
    }

    fn get_word_at_cursor(&self, cursor_pos: usize) -> (usize, String) {
        let text = &self.query;
        let before_cursor = &text[..cursor_pos.min(text.len())];
        
        let start = before_cursor
            .rfind(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        let word = before_cursor[start..].to_string();
        (start, word)
    }

    fn get_suggestions(&self, word: &str) -> Vec<String> {
        if word.is_empty() {
            return Vec::new();
        }

        let word_upper = word.to_uppercase();
        let word_lower = word.to_lowercase();
        let mut suggestions = Vec::new();

        for kw in SQL_KEYWORDS.iter().chain(SQL_TYPES).chain(SQL_FUNCTIONS) {
            if kw.starts_with(&word_upper) {
                suggestions.push(kw.to_string());
            }
        }

        for table in &self.schema.tables {
            if table.to_lowercase().starts_with(&word_lower) {
                suggestions.push(table.clone());
            }
        }

        for col in &self.schema.columns {
            if col.column_name.to_lowercase().starts_with(&word_lower) {
                suggestions.push(format!("{} ({})", col.column_name, col.table_name));
            }
        }

        suggestions.truncate(10);
        suggestions
    }

    fn highlight_sql(text: &str) -> LayoutJob {
        let mut job = LayoutJob::default();
        
        let default_color = Color32::from_rgb(212, 212, 212);
        let keyword_color = Color32::from_rgb(86, 156, 214);
        let type_color = Color32::from_rgb(78, 201, 176);
        let function_color = Color32::from_rgb(220, 220, 170);
        let string_color = Color32::from_rgb(206, 145, 120);
        let number_color = Color32::from_rgb(181, 206, 168);
        let comment_color = Color32::from_rgb(106, 153, 85);

        let font_id = FontId::monospace(14.0);
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '-' {
                let start = i;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, TextFormat::simple(font_id.clone(), comment_color));
                continue;
            }

            if chars[i] == '\'' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '\'' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, TextFormat::simple(font_id.clone(), string_color));
                continue;
            }

            if chars[i].is_ascii_digit() {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, TextFormat::simple(font_id.clone(), number_color));
                continue;
            }

            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let upper = word.to_uppercase();

                let color = if SQL_KEYWORDS.contains(&upper.as_str()) {
                    keyword_color
                } else if SQL_TYPES.contains(&upper.as_str()) {
                    type_color
                } else if SQL_FUNCTIONS.contains(&upper.as_str()) {
                    function_color
                } else {
                    default_color
                };

                job.append(&word, 0.0, TextFormat::simple(font_id.clone(), color));
                continue;
            }

            let s: String = chars[i..i + 1].iter().collect();
            job.append(&s, 0.0, TextFormat::simple(font_id.clone(), default_color));
            i += 1;
        }

        job
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> EditorAction {
        let mut action = EditorAction {
            execute: false,
            save: false,
        };

        let mut layouter = |ui: &egui::Ui, text: &str, _wrap_width: f32| {
            let layout_job = Self::highlight_sql(text);
            ui.fonts(|f| f.layout_job(layout_job))
        };

        let output = egui::TextEdit::multiline(&mut self.query)
            .font(egui::TextStyle::Monospace)
            .desired_rows(8)
            .desired_width(f32::INFINITY)
            .layouter(&mut layouter)
            .show(ui);

        let response = output.response;

        if response.changed() {
            if let Some(cursor_range) = output.cursor_range {
                let pos = cursor_range.primary.ccursor.index;
                let (_, word) = self.get_word_at_cursor(pos);
                
                if word.len() >= 2 {
                    self.autocomplete.suggestions = self.get_suggestions(&word);
                    self.autocomplete.show = !self.autocomplete.suggestions.is_empty();
                    self.autocomplete.selected = 0;
                    self.autocomplete.filter = word;
                } else {
                    self.autocomplete.show = false;
                }
            }
        }

        if self.autocomplete.show && !self.autocomplete.suggestions.is_empty() {
            let popup_id = ui.make_persistent_id("autocomplete_popup");
            
            // Clone suggestions to avoid borrow issues
            let suggestions: Vec<String> = self.autocomplete.suggestions.clone();
            let selected_idx = self.autocomplete.selected;
            
            let mut clicked_suggestion = None;
            
            egui::Area::new(popup_id)
                .order(egui::Order::Foreground)
                .fixed_pos(response.rect.left_bottom() + egui::vec2(20.0, 0.0))
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        for (idx, suggestion) in suggestions.iter().enumerate() {
                            let selected = idx == selected_idx;
                            if ui.selectable_label(selected, suggestion).clicked() {
                                clicked_suggestion = Some(suggestion.clone());
                            }
                        }
                    });
                });

            if let Some(suggestion) = clicked_suggestion {
                self.apply_suggestion(&suggestion);
                self.autocomplete.show = false;
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                self.autocomplete.selected = 
                    (self.autocomplete.selected + 1).min(self.autocomplete.suggestions.len() - 1);
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                self.autocomplete.selected = self.autocomplete.selected.saturating_sub(1);
            }
            if ui.input(|i| i.key_pressed(egui::Key::Tab) || i.key_pressed(egui::Key::Enter)) {
                let suggestion = self.autocomplete.suggestions
                    .get(self.autocomplete.selected)
                    .cloned();
                if let Some(suggestion) = suggestion {
                    self.apply_suggestion(&suggestion);
                    self.autocomplete.show = false;
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.autocomplete.show = false;
            }
        }

        ui.horizontal(|ui| {
            if ui.button("▶ Run (Ctrl+Enter)").clicked() {
                action.execute = true;
            }
            if ui.button("💾 Save").clicked() {
                action.save = true;
            }
        });

        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter)) {
            action.execute = true;
        }
        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            action.save = true;
        }

        action
    }

    fn apply_suggestion(&mut self, suggestion: &str) {
        let text = suggestion.split(" (").next().unwrap_or(suggestion);
        
        let cursor_pos = self.query.len();
        let (start, word) = self.get_word_at_cursor(cursor_pos);
        
        self.query = format!(
            "{}{}{}",
            &self.query[..start],
            text,
            &self.query[start + word.len()..]
        );
    }
}
