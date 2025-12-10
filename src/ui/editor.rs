use eframe::egui::{self, text::LayoutJob, Color32, FontId, TextFormat};
use crate::db::SchemaInfo;

// Combined SQL keywords for PostgreSQL and MySQL
const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER",
    "TABLE", "INDEX", "VIEW", "INTO", "VALUES", "SET", "AND", "OR", "NOT", "NULL",
    "IS", "IN", "LIKE", "BETWEEN", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL",
    "ON", "AS", "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "DISTINCT",
    "UNION", "ALL", "EXISTS", "CASE", "WHEN", "THEN", "ELSE", "END", "ASC", "DESC",
    "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT", "DEFAULT", "CHECK",
    "UNIQUE", "CASCADE", "RETURNING", "WITH", "RECURSIVE", "OVER", "PARTITION",
    // MySQL specific
    "SHOW", "DESCRIBE", "EXPLAIN", "USE", "DATABASE", "DATABASES", "TABLES", "COLUMNS",
    "ENGINE", "AUTO_INCREMENT", "IF", "SCHEMA", "SCHEMAS", "TRUNCATE", "RENAME",
    "PROCEDURE", "FUNCTION", "TRIGGER", "EVENT", "GRANT", "REVOKE", "COMMIT", "ROLLBACK",
    "START", "TRANSACTION", "SAVEPOINT", "LOCK", "UNLOCK", "CALL",
];

const SQL_TYPES: &[&str] = &[
    // Common types
    "INT", "INTEGER", "BIGINT", "SMALLINT", "TINYINT", "MEDIUMINT",
    "TEXT", "VARCHAR", "CHAR", "BOOLEAN", "BOOL",
    "DATE", "TIME", "DATETIME", "TIMESTAMP", "TIMESTAMPTZ", "YEAR",
    "NUMERIC", "DECIMAL", "REAL", "FLOAT", "DOUBLE",
    // PostgreSQL specific
    "SERIAL", "BIGSERIAL", "SMALLSERIAL", "UUID", "JSON", "JSONB", "BYTEA",
    "INET", "CIDR", "MACADDR", "INTERVAL", "POINT", "LINE", "POLYGON",
    "ARRAY", "MONEY", "BIT", "VARBIT", "XML", "TSQUERY", "TSVECTOR",
    // MySQL specific
    "BLOB", "TINYBLOB", "MEDIUMBLOB", "LONGBLOB",
    "TINYTEXT", "MEDIUMTEXT", "LONGTEXT",
    "ENUM", "SET", "BINARY", "VARBINARY", "GEOMETRY",
    "UNSIGNED", "SIGNED", "ZEROFILL",
];

const SQL_FUNCTIONS: &[&str] = &[
    // Aggregate functions
    "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "NULLIF",
    // Date/time functions
    "NOW", "CURRENT_DATE", "CURRENT_TIMESTAMP", "CURRENT_TIME",
    "EXTRACT", "DATE_FORMAT", "STR_TO_DATE", "DATEDIFF", "DATE_ADD", "DATE_SUB",
    "YEAR", "MONTH", "DAY", "HOUR", "MINUTE", "SECOND",
    // String functions
    "CONCAT", "CONCAT_WS", "SUBSTRING", "SUBSTR", "LEFT", "RIGHT",
    "LOWER", "UPPER", "TRIM", "LTRIM", "RTRIM", "LENGTH", "CHAR_LENGTH",
    "REPLACE", "REVERSE", "REPEAT", "SPACE", "LPAD", "RPAD",
    "INSTR", "LOCATE", "POSITION", "FIELD", "FIND_IN_SET",
    // Type conversion
    "CAST", "CONVERT", "TO_CHAR", "TO_DATE", "TO_NUMBER",
    // Window functions
    "ROW_NUMBER", "RANK", "DENSE_RANK", "NTILE",
    "LAG", "LEAD", "FIRST_VALUE", "LAST_VALUE", "NTH_VALUE",
    // Array/JSON (PostgreSQL)
    "ARRAY_AGG", "STRING_AGG", "JSON_AGG", "JSONB_AGG",
    "JSON_BUILD_OBJECT", "JSONB_BUILD_OBJECT",
    "JSON_EXTRACT_PATH", "JSONB_EXTRACT_PATH",
    // MySQL JSON
    "JSON_EXTRACT", "JSON_UNQUOTE", "JSON_SET", "JSON_INSERT", "JSON_REPLACE",
    "JSON_REMOVE", "JSON_CONTAINS", "JSON_SEARCH", "JSON_KEYS", "JSON_LENGTH",
    // Math functions
    "ABS", "CEIL", "CEILING", "FLOOR", "ROUND", "TRUNCATE",
    "MOD", "POW", "POWER", "SQRT", "EXP", "LOG", "LOG10", "LOG2",
    "SIN", "COS", "TAN", "ASIN", "ACOS", "ATAN", "ATAN2",
    "RAND", "RANDOM", "SIGN", "PI", "DEGREES", "RADIANS",
    // Control flow
    "IF", "IFNULL", "NULLIF", "CASE", "COALESCE", "GREATEST", "LEAST",
    // MySQL specific
    "GROUP_CONCAT", "FOUND_ROWS", "LAST_INSERT_ID", "UUID", "VERSION",
    "DATABASE", "USER", "CURRENT_USER", "CONNECTION_ID",
];

pub struct Editor {
    pub query: String,
    schema: SchemaInfo,
    autocomplete: AutocompleteState,
}

struct AutocompleteState {
    active: bool,
    suggestions: Vec<Suggestion>,
    selected: usize,
    word_start: usize,
    word_end: usize,
    popup_pos: egui::Pos2,
    last_cursor_pos: usize,
}

#[derive(Clone)]
struct Suggestion {
    display: String,
    insert: String,
    kind: SuggestionKind,
}

#[derive(Clone, Copy, PartialEq)]
enum SuggestionKind {
    Keyword,
    Type,
    Function,
    Table,
    Column,
}

impl SuggestionKind {
    fn color(self) -> Color32 {
        match self {
            Self::Keyword => Color32::from_rgb(86, 156, 214),
            Self::Type => Color32::from_rgb(78, 201, 176),
            Self::Function => Color32::from_rgb(220, 220, 170),
            Self::Table => Color32::from_rgb(156, 220, 254),
            Self::Column => Color32::from_rgb(212, 212, 212),
        }
    }
    
    fn label(self) -> &'static str {
        match self {
            Self::Keyword => "keyword",
            Self::Type => "type",
            Self::Function => "func",
            Self::Table => "table",
            Self::Column => "column",
        }
    }
}

impl Default for AutocompleteState {
    fn default() -> Self {
        Self {
            active: false,
            suggestions: Vec::new(),
            selected: 0,
            word_start: 0,
            word_end: 0,
            popup_pos: egui::Pos2::ZERO,
            last_cursor_pos: 0,
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            query: String::from("SELECT * FROM users LIMIT 10;"),
            schema: SchemaInfo::default(),
            autocomplete: AutocompleteState::default(),
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

    fn get_word_bounds(&self, cursor_pos: usize) -> (usize, usize, String) {
        let text = &self.query;
        let cursor = cursor_pos.min(text.len());
        
        let before_cursor = &text[..cursor];
        let start = before_cursor
            .rfind(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')' || c == ';' || c == '.')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        let after_cursor = &text[cursor..];
        let end = cursor + after_cursor
            .find(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')' || c == ';' || c == '.')
            .unwrap_or(after_cursor.len());
        
        let word = text[start..cursor].to_string();
        (start, end, word)
    }

    fn get_suggestions(&self, word: &str) -> Vec<Suggestion> {
        if word.is_empty() {
            return Vec::new();
        }

        let word_upper = word.to_uppercase();
        let word_lower = word.to_lowercase();
        let mut suggestions = Vec::new();

        for kw in SQL_KEYWORDS {
            if kw.starts_with(&word_upper) {
                suggestions.push(Suggestion {
                    display: kw.to_string(),
                    insert: kw.to_string(),
                    kind: SuggestionKind::Keyword,
                });
            }
        }

        for t in SQL_TYPES {
            if t.starts_with(&word_upper) {
                suggestions.push(Suggestion {
                    display: t.to_string(),
                    insert: t.to_string(),
                    kind: SuggestionKind::Type,
                });
            }
        }

        for f in SQL_FUNCTIONS {
            if f.starts_with(&word_upper) {
                suggestions.push(Suggestion {
                    display: format!("{}()", f),
                    insert: format!("{}()", f),
                    kind: SuggestionKind::Function,
                });
            }
        }

        for table in &self.schema.tables {
            if table.name.to_lowercase().starts_with(&word_lower) {
                suggestions.push(Suggestion {
                    display: table.name.clone(),
                    insert: table.name.clone(),
                    kind: SuggestionKind::Table,
                });
            }
        }

        for table in &self.schema.tables {
            for col in &table.columns {
                if col.name.to_lowercase().starts_with(&word_lower) {
                    suggestions.push(Suggestion {
                        display: format!("{} ({})", col.name, table.name),
                        insert: col.name.clone(),
                        kind: SuggestionKind::Column,
                    });
                }
            }
        }

        suggestions.sort_by(|a, b| {
            let a_exact = a.insert.to_uppercase().starts_with(&word_upper);
            let b_exact = b.insert.to_uppercase().starts_with(&word_upper);
            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    let kind_order = |k: SuggestionKind| match k {
                        SuggestionKind::Table => 0,
                        SuggestionKind::Column => 1,
                        SuggestionKind::Keyword => 2,
                        SuggestionKind::Function => 3,
                        SuggestionKind::Type => 4,
                    };
                    kind_order(a.kind).cmp(&kind_order(b.kind))
                        .then(a.display.cmp(&b.display))
                }
            }
        });

        suggestions.truncate(12);
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
        let backtick_color = Color32::from_rgb(156, 220, 254);

        let font_id = FontId::monospace(14.0);
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();

        while i < chars.len() {
            // Line comments (-- or #)
            if (i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '-') 
                || chars[i] == '#' {
                let start = i;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, TextFormat::simple(font_id.clone(), comment_color));
                continue;
            }

            // Block comments /* */
            if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
                let start = i;
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                if i + 1 < chars.len() {
                    i += 2;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, TextFormat::simple(font_id.clone(), comment_color));
                continue;
            }

            // Single-quoted strings
            if chars[i] == '\'' {
                let start = i;
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\'' {
                        if i + 1 < chars.len() && chars[i + 1] == '\'' {
                            i += 2; // escaped quote
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, TextFormat::simple(font_id.clone(), string_color));
                continue;
            }

            // Double-quoted identifiers (PostgreSQL)
            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, TextFormat::simple(font_id.clone(), backtick_color));
                continue;
            }

            // Backtick-quoted identifiers (MySQL)
            if chars[i] == '`' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '`' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, TextFormat::simple(font_id.clone(), backtick_color));
                continue;
            }

            // Numbers
            if chars[i].is_ascii_digit() {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, TextFormat::simple(font_id.clone(), number_color));
                continue;
            }

            // Words (keywords, types, functions, identifiers)
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

            // Other characters
            let s: String = chars[i..i + 1].iter().collect();
            job.append(&s, 0.0, TextFormat::simple(font_id.clone(), default_color));
            i += 1;
        }

        job
    }

    fn apply_suggestion(&mut self, suggestion: &Suggestion) {
        let start = self.autocomplete.word_start;
        let end = self.autocomplete.word_end;
        
        let before = &self.query[..start];
        let after = &self.query[end..];
        
        self.query = format!("{}{}{}", before, suggestion.insert, after);
        self.autocomplete.active = false;
    }

    fn dismiss_autocomplete(&mut self) {
        self.autocomplete.active = false;
        self.autocomplete.suggestions.clear();
        self.autocomplete.selected = 0;
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

        let response = &output.response;
        let text_edit_id = response.id;
        let response_rect = response.rect;

        if let Some(cursor_range) = output.cursor_range {
            let cursor_pos = cursor_range.primary.ccursor.index;
            
            let cursor_moved_away = (cursor_pos as i32 - self.autocomplete.last_cursor_pos as i32).abs() > 1;
            
            if response.changed() {
                let (start, end, word) = self.get_word_bounds(cursor_pos);
                
                if word.len() >= 2 {
                    let suggestions = self.get_suggestions(&word);
                    if !suggestions.is_empty() {
                        self.autocomplete.active = true;
                        self.autocomplete.suggestions = suggestions;
                        self.autocomplete.selected = 0;
                        self.autocomplete.word_start = start;
                        self.autocomplete.word_end = end;
                        
                        let ccursor = cursor_range.primary.ccursor;
                        let cursor_rect = output.galley.pos_from_cursor(&output.galley.from_ccursor(ccursor));
                        self.autocomplete.popup_pos = response_rect.min + cursor_rect.min.to_vec2() + egui::vec2(0.0, 20.0);
                    } else {
                        self.dismiss_autocomplete();
                    }
                } else {
                    self.dismiss_autocomplete();
                }
            } else if cursor_moved_away && self.autocomplete.active {
                self.dismiss_autocomplete();
            }
            
            self.autocomplete.last_cursor_pos = cursor_pos;
        }

        if self.autocomplete.active && !self.autocomplete.suggestions.is_empty() {
            let mut should_dismiss = false;
            let mut should_apply = false;

            ui.input(|i| {
                if i.key_pressed(egui::Key::Escape) {
                    should_dismiss = true;
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    self.autocomplete.selected = (self.autocomplete.selected + 1)
                        .min(self.autocomplete.suggestions.len().saturating_sub(1));
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    self.autocomplete.selected = self.autocomplete.selected.saturating_sub(1);
                }
                if i.key_pressed(egui::Key::Tab) || i.key_pressed(egui::Key::Enter) {
                    if i.key_pressed(egui::Key::Tab) || !i.modifiers.ctrl {
                        should_apply = true;
                    }
                }
            });

            if should_dismiss {
                self.dismiss_autocomplete();
            } else if should_apply {
                if let Some(suggestion) = self.autocomplete.suggestions.get(self.autocomplete.selected).cloned() {
                    self.apply_suggestion(&suggestion);
                    ui.memory_mut(|m| m.request_focus(text_edit_id));
                }
            }
        }

        if self.autocomplete.active && !self.autocomplete.suggestions.is_empty() {
            let popup_id = ui.make_persistent_id("sql_autocomplete");
            let mut clicked_suggestion: Option<Suggestion> = None;
            
            egui::Area::new(popup_id)
                .order(egui::Order::Foreground)
                .fixed_pos(self.autocomplete.popup_pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            ui.set_min_width(250.0);
                            ui.set_max_width(400.0);
                            
                            for (idx, suggestion) in self.autocomplete.suggestions.iter().enumerate() {
                                let is_selected = idx == self.autocomplete.selected;
                                
                                let response = ui.horizontal(|ui| {
                                    let bg_color = if is_selected {
                                        ui.visuals().selection.bg_fill
                                    } else {
                                        Color32::TRANSPARENT
                                    };
                                    
                                    egui::Frame::new()
                                        .fill(bg_color)
                                        .inner_margin(egui::Margin::symmetric(4, 2))
                                        .show(ui, |ui| {
                                            ui.set_min_width(240.0);
                                            
                                            let kind_text = egui::RichText::new(suggestion.kind.label())
                                                .small()
                                                .color(suggestion.kind.color());
                                            ui.label(kind_text);
                                            
                                            ui.add_space(8.0);
                                            
                                            let text_color = if is_selected {
                                                ui.visuals().strong_text_color()
                                            } else {
                                                ui.visuals().text_color()
                                            };
                                            ui.label(egui::RichText::new(&suggestion.display).color(text_color).monospace());
                                        });
                                });
                                
                                if response.response.clicked() {
                                    clicked_suggestion = Some(suggestion.clone());
                                }
                            }
                            
                            ui.add_space(4.0);
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 12.0;
                                ui.colored_label(Color32::GRAY, "↑↓ navigate");
                                ui.colored_label(Color32::GRAY, "Tab/Enter accept");
                                ui.colored_label(Color32::GRAY, "Esc dismiss");
                            });
                        });
                });
            
            let suggestion_was_clicked = clicked_suggestion.is_some();
            if let Some(suggestion) = clicked_suggestion {
                self.apply_suggestion(&suggestion);
                ui.memory_mut(|m| m.request_focus(text_edit_id));
            }
            
            if ui.input(|i| i.pointer.any_click()) && !suggestion_was_clicked {
                let pointer_pos = ui.input(|i| i.pointer.interact_pos());
                if let Some(pos) = pointer_pos {
                    let popup_rect = egui::Rect::from_min_size(
                        self.autocomplete.popup_pos,
                        egui::vec2(400.0, 300.0)
                    );
                    if !popup_rect.contains(pos) && !response_rect.contains(pos) {
                        self.dismiss_autocomplete();
                    }
                }
            }
        }

        ui.add_space(4.0);
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
            self.dismiss_autocomplete();
        }
        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            action.save = true;
        }

        action
    }
}
