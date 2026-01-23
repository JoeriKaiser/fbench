use eframe::egui::{self, text::LayoutJob, Color32, FontId, TextFormat};
use crate::db::SchemaInfo;

const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER",
    "TABLE", "INDEX", "VIEW", "INTO", "VALUES", "SET", "AND", "OR", "NOT", "NULL", "IS",
    "IN", "LIKE", "BETWEEN", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL", "ON",
    "AS", "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "DISTINCT", "UNION", "ALL",
    "EXISTS", "CASE", "WHEN", "THEN", "ELSE", "END", "ASC", "DESC", "PRIMARY", "KEY",
    "FOREIGN", "REFERENCES", "CONSTRAINT", "DEFAULT", "CHECK", "UNIQUE", "CASCADE",
    "RETURNING", "WITH", "RECURSIVE", "OVER", "PARTITION",
    "SHOW", "DESCRIBE", "EXPLAIN", "USE", "DATABASE", "DATABASES", "TABLES", "COLUMNS",
    "ENGINE", "AUTO_INCREMENT", "IF", "SCHEMA", "SCHEMAS", "TRUNCATE", "RENAME",
    "PROCEDURE", "FUNCTION", "TRIGGER", "EVENT", "GRANT", "REVOKE", "COMMIT", "ROLLBACK",
    "START", "TRANSACTION", "SAVEPOINT", "LOCK", "UNLOCK", "CALL",
];

const SQL_TYPES: &[&str] = &[
    "INT", "INTEGER", "BIGINT", "SMALLINT", "TINYINT", "MEDIUMINT", "TEXT", "VARCHAR",
    "CHAR", "BOOLEAN", "BOOL", "DATE", "TIME", "DATETIME", "TIMESTAMP", "TIMESTAMPTZ",
    "YEAR", "NUMERIC", "DECIMAL", "REAL", "FLOAT", "DOUBLE",
    "SERIAL", "BIGSERIAL", "SMALLSERIAL", "UUID", "JSON", "JSONB", "BYTEA", "INET",
    "CIDR", "MACADDR", "INTERVAL", "POINT", "LINE", "POLYGON", "ARRAY", "MONEY", "BIT",
    "VARBIT", "XML", "TSQUERY", "TSVECTOR",
    "BLOB", "TINYBLOB", "MEDIUMBLOB", "LONGBLOB", "TINYTEXT", "MEDIUMTEXT", "LONGTEXT",
    "ENUM", "SET", "BINARY", "VARBINARY", "GEOMETRY", "UNSIGNED", "SIGNED", "ZEROFILL",
];

const SQL_FUNCTIONS: &[&str] = &[
    "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "NULLIF",
    "NOW", "CURRENT_DATE", "CURRENT_TIMESTAMP", "CURRENT_TIME", "EXTRACT", "DATE_FORMAT",
    "STR_TO_DATE", "DATEDIFF", "DATE_ADD", "DATE_SUB", "YEAR", "MONTH", "DAY", "HOUR",
    "MINUTE", "SECOND",
    "CONCAT", "CONCAT_WS", "SUBSTRING", "SUBSTR", "LEFT", "RIGHT", "LOWER", "UPPER",
    "TRIM", "LTRIM", "RTRIM", "LENGTH", "CHAR_LENGTH", "REPLACE", "REVERSE", "REPEAT",
    "SPACE", "LPAD", "RPAD", "INSTR", "LOCATE", "POSITION", "FIELD", "FIND_IN_SET",
    "CAST", "CONVERT", "TO_CHAR", "TO_DATE", "TO_NUMBER",
    "ROW_NUMBER", "RANK", "DENSE_RANK", "NTILE", "LAG", "LEAD", "FIRST_VALUE",
    "LAST_VALUE", "NTH_VALUE",
    "ARRAY_AGG", "STRING_AGG", "JSON_AGG", "JSONB_AGG", "JSON_BUILD_OBJECT",
    "JSONB_BUILD_OBJECT", "JSON_EXTRACT_PATH", "JSONB_EXTRACT_PATH",
    "JSON_EXTRACT", "JSON_UNQUOTE", "JSON_SET", "JSON_INSERT", "JSON_REPLACE",
    "JSON_REMOVE", "JSON_CONTAINS", "JSON_SEARCH", "JSON_KEYS", "JSON_LENGTH",
    "ABS", "CEIL", "CEILING", "FLOOR", "ROUND", "TRUNCATE", "MOD", "POW", "POWER",
    "SQRT", "EXP", "LOG", "LOG10", "LOG2", "SIN", "COS", "TAN", "ASIN", "ACOS", "ATAN",
    "ATAN2", "RAND", "RANDOM", "SIGN", "PI", "DEGREES", "RADIANS",
    "IF", "IFNULL", "NULLIF", "CASE", "COALESCE", "GREATEST", "LEAST",
    "GROUP_CONCAT", "FOUND_ROWS", "LAST_INSERT_ID", "UUID", "VERSION", "DATABASE",
    "USER", "CURRENT_USER", "CONNECTION_ID",
];

#[derive(Clone, Copy, PartialEq)]
enum SuggestionKind { Keyword, Type, Function, Table, Column }

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
        match self { Self::Keyword => "keyword", Self::Type => "type", Self::Function => "func", Self::Table => "table", Self::Column => "column" }
    }
}

#[derive(Clone)]
struct Suggestion { display: String, insert: String, kind: SuggestionKind }

#[derive(Default)]
struct AutocompleteState {
    active: bool,
    suggestions: Vec<Suggestion>,
    selected: usize,
    word_start_b: usize,
    word_end_b: usize,
    popup_pos: egui::Pos2,
    last_cursor_char: usize,
}

pub struct Editor {
    pub query: String,
    schema: SchemaInfo,
    autocomplete: AutocompleteState,
    wrap: bool,
    pending_cursor_move: Option<usize>, // byte position to move cursor to
}

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

pub struct EditorAction {
    pub execute_sql: Option<String>,
    pub save: bool,
}

impl Editor {
    pub fn set_schema(&mut self, schema: SchemaInfo) { self.schema = schema; }

    fn char_to_byte(s: &str, idx: usize) -> usize {
        if idx == 0 { 0 } else { s.char_indices().nth(idx).map(|(i, _)| i).unwrap_or(s.len()) }
    }

    fn selection_range(cr: &egui::text::CursorRange) -> (usize, usize) {
        let (a, b) = (cr.primary.ccursor.index, cr.secondary.ccursor.index);
        (a.min(b), a.max(b))
    }

    fn cursor_pos(text: &str, cursor_b: usize) -> (usize, usize) {
        let (mut ln, mut col) = (1, 1);
        for ch in text[..cursor_b.min(text.len())].chars() {
            if ch == '\n' { ln += 1; col = 1; } else { col += 1; }
        }
        (ln, col)
    }

    fn get_word_bounds(&self, cursor_char: usize) -> (usize, usize, String) {
        let text = &self.query;
        let cursor_b = Self::char_to_byte(text, cursor_char);
        let is_delim = |c: char| c.is_whitespace() || ",();".contains(c);

        let start = text[..cursor_b].rfind(is_delim).map(|i| i + 1).unwrap_or(0);
        let end = cursor_b + text[cursor_b..].find(is_delim).unwrap_or(text.len() - cursor_b);
        (start, end, text[start..cursor_b].to_string())
    }

    fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    fn find_word_boundary_right(text: &str, cursor_byte: usize) -> usize {
        let chars: Vec<char> = text.chars().collect();
        let byte_to_char: Vec<usize> = text.char_indices().map(|(b, _)| b).collect();

        // Find which char index corresponds to cursor_byte
        let mut char_idx = byte_to_char.iter().position(|&b| b >= cursor_byte).unwrap_or(chars.len());
        if char_idx < chars.len() && byte_to_char.get(char_idx) == Some(&cursor_byte) {
            // exact match
        } else if char_idx > 0 {
            char_idx = char_idx.saturating_sub(1);
        }

        // Skip current word characters
        while char_idx < chars.len() && Self::is_word_char(chars[char_idx]) {
            char_idx += 1;
        }

        // Skip whitespace/punctuation (but not newlines for now, stop at word)
        while char_idx < chars.len() && !Self::is_word_char(chars[char_idx]) {
            char_idx += 1;
        }

        // Return byte position
        if char_idx >= chars.len() {
            text.len()
        } else {
            Self::char_to_byte(text, char_idx)
        }
    }

    fn find_word_boundary_left(text: &str, cursor_byte: usize) -> usize {
        if cursor_byte == 0 { return 0; }

        let chars: Vec<char> = text.chars().collect();

        // Find which char index corresponds to cursor_byte
        let mut char_idx = text[..cursor_byte].chars().count();
        if char_idx == 0 { return 0; }

        char_idx -= 1; // Move back one char to start

        // Skip whitespace/punctuation going backwards
        while char_idx > 0 && !Self::is_word_char(chars[char_idx]) {
            char_idx -= 1;
        }

        // Skip word characters going backwards
        while char_idx > 0 && Self::is_word_char(chars[char_idx - 1]) {
            char_idx -= 1;
        }

        // If we're on a non-word char and there's nothing before, go to 0
        if char_idx > 0 && !Self::is_word_char(chars[char_idx]) {
            // We stopped on punctuation, keep going
        }

        Self::char_to_byte(text, char_idx)
    }

    fn handle_ctrl_navigation(&mut self, ui: &mut egui::Ui, _text_edit_id: egui::Id) {
        let cursor_char = self.autocomplete.last_cursor_char;
        let cursor_b = Self::char_to_byte(&self.query, cursor_char);

        ui.input_mut(|input| {
            if input.modifiers.ctrl && !input.modifiers.alt {
                if input.key_pressed(egui::Key::ArrowRight) {
                    let new_pos = Self::find_word_boundary_right(&self.query, cursor_b);
                    self.pending_cursor_move = Some(new_pos);
                    input.consume_key(egui::Modifiers::CTRL, egui::Key::ArrowRight);
                }
                if input.key_pressed(egui::Key::ArrowLeft) {
                    let new_pos = Self::find_word_boundary_left(&self.query, cursor_b);
                    self.pending_cursor_move = Some(new_pos);
                    input.consume_key(egui::Modifiers::CTRL, egui::Key::ArrowLeft);
                }
            }
        });
    }

    fn get_suggestions(&self, word: &str) -> Vec<Suggestion> {
        if word.is_empty() {
            return Vec::new();
        }

        let (upper, lower) = (word.to_ascii_uppercase(), word.to_ascii_lowercase());
        let mut suggestions = Vec::new();

        let categories = [
            (SQL_KEYWORDS, SuggestionKind::Keyword),
            (SQL_TYPES, SuggestionKind::Type),
            (SQL_FUNCTIONS, SuggestionKind::Function),
        ];

        for (arr, kind) in categories {
            for &item in arr {
                if item.starts_with(&upper) {
                    let insert = if matches!(kind, SuggestionKind::Function) {
                        format!("{}()", item)
                    } else {
                        item.to_string()
                    };
                    suggestions.push(Suggestion {
                        display: if matches!(kind, SuggestionKind::Function) {
                            format!("{}()", item)
                        } else {
                            item.to_string()
                        },
                        insert,
                        kind,
                    });
                }
            }
        }

        // Handle table and column suggestions
        if let Some((table_prefix, col_prefix)) = word.split_once('.') {
            // Dot notation: suggest columns from specific table
            let table_upper = table_prefix.to_ascii_uppercase();
            let col_upper = col_prefix.to_ascii_uppercase();
            for table in &self.schema.tables {
                if table.name.to_ascii_uppercase() == table_upper {
                    for col in &table.columns {
                        if col.name.to_ascii_uppercase().starts_with(&col_upper) {
                            suggestions.push(Suggestion {
                                display: format!("{}", col.name),
                                insert: col.name.clone(),
                                kind: SuggestionKind::Column,
                            });
                        }
                    }
                    break; // Assuming unique table names
                }
            }
        } else {
            // No dot: suggest tables and columns as before
            for table in &self.schema.tables {
                if table.name.to_ascii_lowercase().starts_with(&lower) {
                    suggestions.push(Suggestion {
                        display: table.name.clone(),
                        insert: table.name.clone(),
                        kind: SuggestionKind::Table,
                    });
                }
                for col in &table.columns {
                    if col.name.to_ascii_lowercase().starts_with(&lower) {
                        suggestions.push(Suggestion {
                            display: format!("{} ({})", col.name, table.name),
                            insert: col.name.clone(),
                            kind: SuggestionKind::Column,
                        });
                    }
                }
            }
        }

        suggestions.sort_by(|a, b| {
            let a_exact = a.insert.to_ascii_uppercase().starts_with(&upper);
            let b_exact = b.insert.to_ascii_uppercase().starts_with(&upper);
            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    let order = |k: SuggestionKind| {
                        match k {
                            SuggestionKind::Table => 0,
                            SuggestionKind::Column => 1,
                            SuggestionKind::Keyword => 2,
                            SuggestionKind::Function => 3,
                            SuggestionKind::Type => 4,
                        }
                    };
                    order(a.kind).cmp(&order(b.kind)).then(a.display.cmp(&b.display))
                }
            }
        });
        suggestions.truncate(12);
        suggestions
    }

    fn highlight_sql(text: &str) -> LayoutJob {
        let mut job = LayoutJob::default();
        let font = FontId::monospace(14.0);
        let colors = (
            Color32::from_rgb(212, 212, 212),  // 0: default
            Color32::from_rgb(86, 156, 214),   // 1: keyword
            Color32::from_rgb(78, 201, 176),   // 2: type
            Color32::from_rgb(220, 220, 170),  // 3: function
            Color32::from_rgb(206, 145, 120),  // 4: string
            Color32::from_rgb(181, 206, 168),  // 5: number
            Color32::from_rgb(106, 153, 85),   // 6: comment
            Color32::from_rgb(156, 220, 254),  // 7: ident
        );

        let bytes = text.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            let start = i; // Define start here so it is available inside the match
            let color = match bytes[i] {
                b'-' if i + 1 < bytes.len() && bytes[i + 1] == b'-' => {
                    while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
                    colors.6
                }
                b'#' => { while i < bytes.len() && bytes[i] != b'\n' { i += 1; } colors.6 }
                b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                    i += 2;
                    while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') { i += 1; }
                    if i + 1 < bytes.len() { i += 2; }
                    colors.6
                }
                b'\'' => {
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == b'\'' {
                            if i + 1 < bytes.len() && bytes[i + 1] == b'\'' { i += 2; continue; }
                            break;
                        }
                        i += 1;
                    }
                    if i < bytes.len() { i += 1; }
                    colors.4
                }
                b'"' | b'`' => {
                    let quote = bytes[i];
                    i += 1;
                    while i < bytes.len() && bytes[i] != quote { i += 1; }
                    if i < bytes.len() { i += 1; }
                    colors.7
                }
                b if b.is_ascii_digit() => {
                    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') { i += 1; }
                    colors.5
                }
                b if b.is_ascii_alphabetic() || b == b'_' => {
                    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') { i += 1; }
                    let word = &text[start..i];
                    let upper = word.to_ascii_uppercase();
                    if SQL_KEYWORDS.contains(&upper.as_str()) { colors.1 }
                    else if SQL_TYPES.contains(&upper.as_str()) { colors.2 }
                    else if SQL_FUNCTIONS.contains(&upper.as_str()) { colors.3 }
                    else { colors.0 }
                }
                _ => {
                    let ch = text[i..].chars().next().unwrap();
                    i += ch.len_utf8();
                    colors.0
                }
            };
            job.append(&text[start..i], 0.0, TextFormat::simple(font.clone(), color));
        }
        job
    }

    fn apply_suggestion(&mut self, s: &Suggestion) {
        let (start, end) = (self.autocomplete.word_start_b.min(self.query.len()), 
                           self.autocomplete.word_end_b.min(self.query.len()));
        self.query = format!("{}{}{}", &self.query[..start], s.insert, &self.query[end.max(start)..]);
        self.autocomplete.active = false;
    }

    fn dismiss_autocomplete(&mut self) {
        self.autocomplete.active = false;
        self.autocomplete.suggestions.clear();
        self.autocomplete.selected = 0;
    }

    fn selected_text(&self, cr: Option<&egui::text::CursorRange>) -> Option<String> {
        let cr = cr?;
        let (a, b) = Self::selection_range(cr);
        if a == b { return None; }
        let (a_b, b_b) = (Self::char_to_byte(&self.query, a), Self::char_to_byte(&self.query, b));
        Some(self.query[a_b.min(b_b)..a_b.max(b_b)].to_string())
    }

    fn statement_at_cursor(&self, cursor_char: usize) -> Option<String> {
        let text = self.query.as_str();
        if text.trim().is_empty() { return None; }

        let cursor_b = Self::char_to_byte(text, cursor_char);
        let bytes = text.as_bytes();
        let mut ranges = Vec::new();
        let mut i = 0;
        let mut stmt_start = 0;
        let (mut in_sq, mut in_dq, mut in_bt, mut in_lc, mut in_bc) = (false, false, false, false, false);

        while i < bytes.len() {
            let b = bytes[i];
            if in_lc { if b == b'\n' { in_lc = false; } i += 1; continue; }
            if in_bc { if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' { in_bc = false; i += 2; continue; } i += 1; continue; }
            if in_sq { if b == b'\'' { if i + 1 < bytes.len() && bytes[i + 1] == b'\'' { i += 2; continue; } in_sq = false; } i += 1; continue; }
            if in_dq { if b == b'"' { in_dq = false; } i += 1; continue; }
            if in_bt { if b == b'`' { in_bt = false; } i += 1; continue; }

            if (i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'-') || bytes[i] == b'#' { in_lc = true; i += 1; continue; }
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' { in_bc = true; i += 2; continue; }
            if b == b'\'' { in_sq = true; i += 1; continue; }
            if b == b'"' { in_dq = true; i += 1; continue; }
            if b == b'`' { in_bt = true; i += 1; continue; }
            if b == b';' { ranges.push((stmt_start, i + 1)); stmt_start = i + 1; }
            i += 1;
        }
        if stmt_start < text.len() { ranges.push((stmt_start, text.len())); }

        let (mut s, mut e) = ranges.into_iter().find(|&(s, e)| cursor_b >= s && cursor_b <= e).unwrap_or((0, text.len()));
        while s < e && bytes[s].is_ascii_whitespace() { s += 1; }
        while e > s && bytes[e - 1].is_ascii_whitespace() { e -= 1; }
        
        let stmt = text.get(s..e)?.trim();
        (!stmt.is_empty()).then(|| stmt.to_string())
    }

    fn apply_to_lines<F: FnMut(&str) -> String>(&mut self, cr: Option<&egui::text::CursorRange>, mut f: F) -> bool {
        let Some(cr) = cr else { return false };
        let (a, b) = Self::selection_range(cr);
        let (a_b, b_b) = (Self::char_to_byte(&self.query, a), Self::char_to_byte(&self.query, b));
        let (sel_start, sel_end) = (a_b.min(b_b), a_b.max(b_b));

        let line_start = self.query[..sel_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = self.query[sel_end..].find('\n').map(|i| sel_end + i + 1).unwrap_or(self.query.len());

        let region = self.query[line_start..line_end].to_string();
        let out: String = region.split_inclusive('\n').map(&mut f).collect();
        self.query.replace_range(line_start..line_end, &out);
        true
    }

    fn indent(&mut self, cr: Option<&egui::text::CursorRange>) -> bool {
        self.apply_to_lines(cr, |l| if l == "\n" { l.to_string() } else { format!("  {}", l) })
    }

    fn outdent(&mut self, cr: Option<&egui::text::CursorRange>) -> bool {
        self.apply_to_lines(cr, |line| {
            let (body, nl) = line.strip_suffix('\n').map(|s| (s, "\n")).unwrap_or((line, ""));
            let trimmed = body.strip_prefix("  ").or_else(|| body.strip_prefix('\t')).or_else(|| body.strip_prefix(' ')).unwrap_or(body);
            format!("{}{}", trimmed, nl)
        })
    }

    fn toggle_comment(&mut self, cr: Option<&egui::text::CursorRange>) -> bool {
        let Some(cr) = cr else { return false };
        let (a, b) = Self::selection_range(cr);
        let (a_b, b_b) = (Self::char_to_byte(&self.query, a), Self::char_to_byte(&self.query, b));
        let (sel_start, sel_end) = (a_b.min(b_b), a_b.max(b_b));

        let line_start = self.query[..sel_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = self.query[sel_end..].find('\n').map(|i| sel_end + i + 1).unwrap_or(self.query.len());
        let region = self.query[line_start..line_end].to_string();

        let all_commented = region.lines().filter(|l| !l.trim().is_empty()).all(|l| l.trim_start().starts_with("--"));

        let out: String = region.split_inclusive('\n').map(|line| {
            let (body, nl) = line.strip_suffix('\n').map(|s| (s, "\n")).unwrap_or((line, ""));
            if body.trim().is_empty() { return format!("{}{}", body, nl); }

            let ws_len = body.chars().take_while(|c| c.is_whitespace()).count();
            let ws_b = Self::char_to_byte(body, ws_len);
            let (ws, rest) = body.split_at(ws_b);

            if all_commented {
                let r = rest.strip_prefix("--").map(|r| r.strip_prefix(' ').unwrap_or(r)).unwrap_or(rest);
                format!("{}{}{}", ws, r, nl)
            } else {
                format!("{}-- {}{}", ws, rest, nl)
            }
        }).collect();

        self.query.replace_range(line_start..line_end, &out);
        true
    }

    fn gutter(ui: &mut egui::Ui, lines: usize, font_size: f32) {
        let digits = lines.to_string().len().max(2);
        let gutter_w = ui.fonts(|f| f.layout_no_wrap("0".repeat(digits + 1), FontId::monospace(font_size), ui.visuals().weak_text_color()).size().x).ceil() + 6.0;

        let s: String = (1..=lines).map(|i| format!("{:>width$}\n", i, width = digits)).collect();
        ui.allocate_ui_with_layout(egui::vec2(gutter_w, ui.available_height()), egui::Layout::top_down(egui::Align::RIGHT), |ui| {
            ui.add(egui::Label::new(egui::RichText::new(s).monospace().color(ui.visuals().weak_text_color())).selectable(false));
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> EditorAction {
        let mut action = EditorAction { execute_sql: None, save: false };
        let (mut want_run_all, mut want_run_stmt, mut want_save, mut want_indent, mut want_outdent, mut want_comment) = (false, false, false, false, false, false);

        ui.horizontal(|ui| {
            if ui.button("Run (Ctrl+Enter)").clicked() { want_run_all = true; }
            if ui.button("Run stmt (Ctrl+Shift+Enter)").clicked() { want_run_stmt = true; }
            if ui.button("Save (Ctrl+S)").clicked() { want_save = true; }
            ui.separator();
            ui.toggle_value(&mut self.wrap, "Wrap");
            ui.separator();
            if ui.button("Indent (Ctrl+])").clicked() { want_indent = true; }
            if ui.button("Outdent (Ctrl+[)").clicked() { want_outdent = true; }
            if ui.button("Comment (Ctrl+/)").clicked() { want_comment = true; }
        });
        ui.add_space(6.0);

        let text_edit_id = ui.make_persistent_id("sql_editor_textedit");
        self.handle_ctrl_navigation(ui, text_edit_id);
        let wrap = self.wrap;
        let mut layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let mut job = Self::highlight_sql(text);
            job.wrap.max_width = if wrap { wrap_width } else { f32::INFINITY };
            ui.fonts(|f| f.layout_job(job))
        };

        let lines = self.query.bytes().filter(|&b| b == b'\n').count() + 1;
        let mut output: Option<egui::text_edit::TextEditOutput> = None;

        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::both().id_salt("sql_editor_scroll").auto_shrink([false, false]).show(ui, |ui| {
                ui.horizontal(|ui| {
                    Self::gutter(ui, lines, 14.0);
                    output = Some(egui::TextEdit::multiline(&mut self.query)
                        .id(text_edit_id)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .hint_text("Write SQL…")
                        .layouter(&mut layouter)
                        .show(ui));
                });
            });
        });

        let output = output.unwrap();
        let response_rect = output.response.rect;

        // Apply pending cursor move from Ctrl+Arrow navigation
        if let Some(byte_pos) = self.pending_cursor_move.take() {
            let char_pos = self.query[..byte_pos.min(self.query.len())].chars().count();
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                let ccursor = egui::text::CCursor::new(char_pos);
                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                state.store(ui.ctx(), text_edit_id);
                ui.ctx().request_repaint();
            }
        }

        let (cursor_char, cursor_b, ln, col) = output.cursor_range.as_ref().map(|cr| {
            let c = cr.primary.ccursor.index;
            let b = Self::char_to_byte(&self.query, c);
            let (ln, col) = Self::cursor_pos(&self.query, b);
            (c, b, ln, col)
        }).unwrap_or((0, 0, 1, 1));

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.colored_label(ui.visuals().weak_text_color(), format!("Ln {}, Col {}", ln, col));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.colored_label(ui.visuals().weak_text_color(), "Autocomplete: ↑↓ Tab/Enter • Esc dismiss");
            });
        });

        if let Some(cr) = output.cursor_range.clone() {
            let moved_away = (cursor_char as i32 - self.autocomplete.last_cursor_char as i32).abs() > 1;

            if output.response.changed() {
                let (start_b, end_b, word) = self.get_word_bounds(cursor_char);
                if word.len() >= 2 {
                    let suggestions = self.get_suggestions(&word);
                    if !suggestions.is_empty() {
                        self.autocomplete = AutocompleteState {
                            active: true, suggestions, selected: 0,
                            word_start_b: start_b, word_end_b: end_b,
                            popup_pos: response_rect.min + output.galley.pos_from_cursor(&output.galley.from_ccursor(cr.primary.ccursor)).min.to_vec2() + egui::vec2(0.0, 20.0),
                            last_cursor_char: cursor_char,
                        };
                    } else { self.dismiss_autocomplete(); }
                } else { self.dismiss_autocomplete(); }
            } else if moved_away && self.autocomplete.active { self.dismiss_autocomplete(); }
            self.autocomplete.last_cursor_char = cursor_char;
        }

        if self.autocomplete.active && !self.autocomplete.suggestions.is_empty() {
            let (mut dismiss, mut apply) = (false, false);
            ui.input(|i| {
                if i.key_pressed(egui::Key::Escape) { dismiss = true; }
                if i.key_pressed(egui::Key::ArrowDown) { self.autocomplete.selected = (self.autocomplete.selected + 1).min(self.autocomplete.suggestions.len() - 1); }
                if i.key_pressed(egui::Key::ArrowUp) { self.autocomplete.selected = self.autocomplete.selected.saturating_sub(1); }
                if i.key_pressed(egui::Key::Tab) || (i.key_pressed(egui::Key::Enter) && !i.modifiers.ctrl && !i.modifiers.shift) { apply = true; }
            });

            if dismiss { self.dismiss_autocomplete(); }
            else if apply {
                if let Some(s) = self.autocomplete.suggestions.get(self.autocomplete.selected).cloned() {
                    self.apply_suggestion(&s);
                    ui.memory_mut(|m| m.request_focus(text_edit_id));
                }
            }
        }

        if self.autocomplete.active && !self.autocomplete.suggestions.is_empty() {
            let mut clicked: Option<Suggestion> = None;
            egui::Area::new(ui.make_persistent_id("sql_autocomplete"))
                .order(egui::Order::Foreground)
                .fixed_pos(self.autocomplete.popup_pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).inner_margin(4.0).show(ui, |ui| {
                        ui.set_min_width(260.0);
                        ui.set_max_width(420.0);
                        for (idx, s) in self.autocomplete.suggestions.iter().enumerate() {
                            let bg = if idx == self.autocomplete.selected { ui.visuals().selection.bg_fill } else { Color32::TRANSPARENT };
                            let r = egui::Frame::new().fill(bg).inner_margin(egui::Margin::symmetric(6, 3)).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(s.kind.label()).small().color(s.kind.color()));
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new(&s.display).monospace());
                                });
                            }).response;
                            if r.clicked() { clicked = Some(s.clone()); }
                        }
                    });
                });

            let clicked_any = clicked.is_some();
            if let Some(s) = clicked {
                self.apply_suggestion(&s);
                ui.memory_mut(|m| m.request_focus(text_edit_id));
            }
            if ui.input(|i| i.pointer.any_click()) && !clicked_any {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    let popup_rect = egui::Rect::from_min_size(self.autocomplete.popup_pos, egui::vec2(420.0, 260.0));
                    if !popup_rect.contains(pos) && !response_rect.contains(pos) { self.dismiss_autocomplete(); }
                }
            }
        }

        let cr = output.cursor_range.as_ref();
        ui.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Enter) { want_run_all = true; }
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Enter) { want_run_stmt = true; }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) { want_save = true; }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Slash) { want_comment = true; }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::CloseBracket) { want_indent = true; }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::OpenBracket) { want_outdent = true; }
        });

        if want_save { action.save = true; }
        if want_run_all { action.execute_sql = Some(self.query.clone()); self.dismiss_autocomplete(); }
        if want_run_stmt { action.execute_sql = self.selected_text(cr).or_else(|| self.statement_at_cursor(cursor_char)); self.dismiss_autocomplete(); }
        if want_comment && self.toggle_comment(cr) { ui.memory_mut(|m| m.request_focus(text_edit_id)); }
        if want_indent && self.indent(cr) { ui.memory_mut(|m| m.request_focus(text_edit_id)); }
        if want_outdent && self.outdent(cr) { ui.memory_mut(|m| m.request_focus(text_edit_id)); }
        if cursor_b > self.query.len() { self.dismiss_autocomplete(); }

        action
    }
}
