use eframe::egui::{self, text::LayoutJob, Color32, FontId, TextFormat};
use crate::db::SchemaInfo;

// Combined SQL keywords for PostgreSQL and MySQL
const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER",
    "TABLE", "INDEX", "VIEW", "INTO", "VALUES", "SET", "AND", "OR", "NOT", "NULL", "IS",
    "IN", "LIKE", "BETWEEN", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL", "ON",
    "AS", "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "DISTINCT", "UNION", "ALL",
    "EXISTS", "CASE", "WHEN", "THEN", "ELSE", "END", "ASC", "DESC", "PRIMARY", "KEY",
    "FOREIGN", "REFERENCES", "CONSTRAINT", "DEFAULT", "CHECK", "UNIQUE", "CASCADE",
    "RETURNING", "WITH", "RECURSIVE", "OVER", "PARTITION",
    // MySQL specific
    "SHOW", "DESCRIBE", "EXPLAIN", "USE", "DATABASE", "DATABASES", "TABLES", "COLUMNS",
    "ENGINE", "AUTO_INCREMENT", "IF", "SCHEMA", "SCHEMAS", "TRUNCATE", "RENAME",
    "PROCEDURE", "FUNCTION", "TRIGGER", "EVENT", "GRANT", "REVOKE", "COMMIT", "ROLLBACK",
    "START", "TRANSACTION", "SAVEPOINT", "LOCK", "UNLOCK", "CALL",
];

const SQL_TYPES: &[&str] = &[
    // Common types
    "INT", "INTEGER", "BIGINT", "SMALLINT", "TINYINT", "MEDIUMINT", "TEXT", "VARCHAR",
    "CHAR", "BOOLEAN", "BOOL", "DATE", "TIME", "DATETIME", "TIMESTAMP", "TIMESTAMPTZ",
    "YEAR", "NUMERIC", "DECIMAL", "REAL", "FLOAT", "DOUBLE",
    // PostgreSQL specific
    "SERIAL", "BIGSERIAL", "SMALLSERIAL", "UUID", "JSON", "JSONB", "BYTEA", "INET",
    "CIDR", "MACADDR", "INTERVAL", "POINT", "LINE", "POLYGON", "ARRAY", "MONEY", "BIT",
    "VARBIT", "XML", "TSQUERY", "TSVECTOR",
    // MySQL specific
    "BLOB", "TINYBLOB", "MEDIUMBLOB", "LONGBLOB", "TINYTEXT", "MEDIUMTEXT", "LONGTEXT",
    "ENUM", "SET", "BINARY", "VARBINARY", "GEOMETRY", "UNSIGNED", "SIGNED", "ZEROFILL",
];

const SQL_FUNCTIONS: &[&str] = &[
    // Aggregate functions
    "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "NULLIF",
    // Date/time functions
    "NOW", "CURRENT_DATE", "CURRENT_TIMESTAMP", "CURRENT_TIME", "EXTRACT", "DATE_FORMAT",
    "STR_TO_DATE", "DATEDIFF", "DATE_ADD", "DATE_SUB", "YEAR", "MONTH", "DAY", "HOUR",
    "MINUTE", "SECOND",
    // String functions
    "CONCAT", "CONCAT_WS", "SUBSTRING", "SUBSTR", "LEFT", "RIGHT", "LOWER", "UPPER",
    "TRIM", "LTRIM", "RTRIM", "LENGTH", "CHAR_LENGTH", "REPLACE", "REVERSE", "REPEAT",
    "SPACE", "LPAD", "RPAD", "INSTR", "LOCATE", "POSITION", "FIELD", "FIND_IN_SET",
    // Type conversion
    "CAST", "CONVERT", "TO_CHAR", "TO_DATE", "TO_NUMBER",
    // Window functions
    "ROW_NUMBER", "RANK", "DENSE_RANK", "NTILE", "LAG", "LEAD", "FIRST_VALUE",
    "LAST_VALUE", "NTH_VALUE",
    // Array/JSON (PostgreSQL)
    "ARRAY_AGG", "STRING_AGG", "JSON_AGG", "JSONB_AGG", "JSON_BUILD_OBJECT",
    "JSONB_BUILD_OBJECT", "JSON_EXTRACT_PATH", "JSONB_EXTRACT_PATH",
    // MySQL JSON
    "JSON_EXTRACT", "JSON_UNQUOTE", "JSON_SET", "JSON_INSERT", "JSON_REPLACE",
    "JSON_REMOVE", "JSON_CONTAINS", "JSON_SEARCH", "JSON_KEYS", "JSON_LENGTH",
    // Math functions
    "ABS", "CEIL", "CEILING", "FLOOR", "ROUND", "TRUNCATE", "MOD", "POW", "POWER",
    "SQRT", "EXP", "LOG", "LOG10", "LOG2", "SIN", "COS", "TAN", "ASIN", "ACOS", "ATAN",
    "ATAN2", "RAND", "RANDOM", "SIGN", "PI", "DEGREES", "RADIANS",
    // Control flow
    "IF", "IFNULL", "NULLIF", "CASE", "COALESCE", "GREATEST", "LEAST",
    // MySQL specific
    "GROUP_CONCAT", "FOUND_ROWS", "LAST_INSERT_ID", "UUID", "VERSION", "DATABASE",
    "USER", "CURRENT_USER", "CONNECTION_ID",
];

pub struct Editor {
    pub query: String,
    schema: SchemaInfo,
    autocomplete: AutocompleteState,
    wrap: bool,
}

struct AutocompleteState {
    active: bool,
    suggestions: Vec<Suggestion>,
    selected: usize,
    word_start_b: usize,
    word_end_b: usize,
    popup_pos: egui::Pos2,
    last_cursor_char: usize,
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
            word_start_b: 0,
            word_end_b: 0,
            popup_pos: egui::Pos2::ZERO,
            last_cursor_char: 0,
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            query: String::from("SELECT * FROM users LIMIT 10;"),
            schema: SchemaInfo::default(),
            autocomplete: AutocompleteState::default(),
            wrap: false,
        }
    }
}

pub struct EditorAction {
    pub execute_sql: Option<String>,
    pub save: bool,
}

impl Editor {
    pub fn set_schema(&mut self, schema: SchemaInfo) {
        self.schema = schema;
    }

    fn char_to_byte(s: &str, char_idx: usize) -> usize {
        if char_idx == 0 {
            return 0;
        }
        s.char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(s.len())
    }

    fn selection_char_range(
        cursor_range: &egui::text::CursorRange,
    ) -> (usize, usize) {
        let a = cursor_range.primary.ccursor.index;
        let b = cursor_range.secondary.ccursor.index;
        (a.min(b), a.max(b))
    }

    fn cursor_ln_col(text: &str, cursor_b: usize) -> (usize, usize) {
        let mut ln = 1usize;
        let mut col = 1usize;
        for ch in text[..cursor_b.min(text.len())].chars() {
            if ch == '\n' {
                ln += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (ln, col)
    }

    fn count_lines(text: &str) -> usize {
        text.bytes().filter(|&b| b == b'\n').count() + 1
    }

    fn get_word_bounds_b(
        &self,
        cursor_char: usize,
    ) -> (usize, usize, String) {
        let text = &self.query;
        let cursor_b = Self::char_to_byte(text, cursor_char);

        let before = &text[..cursor_b];
        let start = before
            .rfind(|c: char| {
                c.is_whitespace()
                    || c == ','
                    || c == '('
                    || c == ')'
                    || c == ';'
                    || c == '.'
            })
            .map(|i| i + 1)
            .unwrap_or(0);

        let after = &text[cursor_b..];
        let end = cursor_b
            + after
                .find(|c: char| {
                    c.is_whitespace()
                        || c == ','
                        || c == '('
                        || c == ')'
                        || c == ';'
                        || c == '.'
                })
                .unwrap_or(after.len());

        let word = text[start..cursor_b].to_string();
        (start, end, word)
    }

    fn get_suggestions(&self, word: &str) -> Vec<Suggestion> {
        if word.is_empty() {
            return Vec::new();
        }

        let word_upper = word.to_ascii_uppercase();
        let word_lower = word.to_ascii_lowercase();
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
            if table.name.to_ascii_lowercase().starts_with(&word_lower) {
                suggestions.push(Suggestion {
                    display: table.name.clone(),
                    insert: table.name.clone(),
                    kind: SuggestionKind::Table,
                });
            }
        }

        for table in &self.schema.tables {
            for col in &table.columns {
                if col.name.to_ascii_lowercase().starts_with(&word_lower) {
                    suggestions.push(Suggestion {
                        display: format!("{} ({})", col.name, table.name),
                        insert: col.name.clone(),
                        kind: SuggestionKind::Column,
                    });
                }
            }
        }

        suggestions.sort_by(|a, b| {
            let a_exact = a.insert.to_ascii_uppercase().starts_with(&word_upper);
            let b_exact = b.insert.to_ascii_uppercase().starts_with(&word_upper);
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
                    kind_order(a.kind)
                        .cmp(&kind_order(b.kind))
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
        let ident_color = Color32::from_rgb(156, 220, 254);

        let font_id = FontId::monospace(14.0);

        let bytes = text.as_bytes();
        let mut i = 0usize;

        while i < bytes.len() {
            // Line comments: -- ... or # ...
            if (i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'-')
                || bytes[i] == b'#'
            {
                let start = i;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                job.append(
                    &text[start..i],
                    0.0,
                    TextFormat::simple(font_id.clone(), comment_color),
                );
                continue;
            }

            // Block comments: /* ... */
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*'
            {
                let start = i;
                i += 2;
                while i + 1 < bytes.len()
                    && !(bytes[i] == b'*' && bytes[i + 1] == b'/')
                {
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    i += 2;
                }
                job.append(
                    &text[start..i],
                    0.0,
                    TextFormat::simple(font_id.clone(), comment_color),
                );
                continue;
            }

            // Single-quoted strings: '...'
            if bytes[i] == b'\'' {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\'' {
                        if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                            i += 2; // escaped ''
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                job.append(
                    &text[start..i],
                    0.0,
                    TextFormat::simple(font_id.clone(), string_color),
                );
                continue;
            }

            // Double-quoted identifiers (PostgreSQL): "..."
            if bytes[i] == b'"' {
                let start = i;
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                job.append(
                    &text[start..i],
                    0.0,
                    TextFormat::simple(font_id.clone(), ident_color),
                );
                continue;
            }

            // Backtick identifiers (MySQL): `...`
            if bytes[i] == b'`' {
                let start = i;
                i += 1;
                while i < bytes.len() && bytes[i] != b'`' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                job.append(
                    &text[start..i],
                    0.0,
                    TextFormat::simple(font_id.clone(), ident_color),
                );
                continue;
            }

            // Numbers
            if bytes[i].is_ascii_digit() {
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_digit() || bytes[i] == b'.')
                {
                    i += 1;
                }
                job.append(
                    &text[start..i],
                    0.0,
                    TextFormat::simple(font_id.clone(), number_color),
                );
                continue;
            }

            // Words (ASCII): keywords/types/functions/idents
            if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                {
                    i += 1;
                }
                let word = &text[start..i];
                let upper = word.to_ascii_uppercase();

                let color = if SQL_KEYWORDS.contains(&upper.as_str()) {
                    keyword_color
                } else if SQL_TYPES.contains(&upper.as_str()) {
                    type_color
                } else if SQL_FUNCTIONS.contains(&upper.as_str()) {
                    function_color
                } else {
                    default_color
                };

                job.append(
                    word,
                    0.0,
                    TextFormat::simple(font_id.clone(), color),
                );
                continue;
            }

            // Other (keep UTF-8 correctness)
            let ch = text[i..].chars().next().unwrap();
            let len = ch.len_utf8();
            job.append(
                &text[i..(i + len).min(text.len())],
                0.0,
                TextFormat::simple(font_id.clone(), default_color),
            );
            i += len;
        }

        job
    }

    fn apply_suggestion(&mut self, suggestion: &Suggestion) {
        let start = self.autocomplete.word_start_b.min(self.query.len());
        let end = self.autocomplete.word_end_b.min(self.query.len()).max(start);

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

    fn selected_text(
        &self,
        cursor_range: Option<&egui::text::CursorRange>,
    ) -> Option<String> {
        let cr = cursor_range?;
        let (a_char, b_char) = Self::selection_char_range(cr);
        if a_char == b_char {
            return None;
        }
        let a_b = Self::char_to_byte(&self.query, a_char);
        let b_b = Self::char_to_byte(&self.query, b_char);
        Some(self.query[a_b.min(b_b)..a_b.max(b_b)].to_string())
    }

    fn statement_at_cursor(
        &self,
        cursor_char: usize,
    ) -> Option<String> {
        let text = self.query.as_str();
        if text.trim().is_empty() {
            return None;
        }

        let cursor_b = Self::char_to_byte(text, cursor_char);
        let mut ranges = Vec::<(usize, usize)>::new();

        let bytes = text.as_bytes();
        let mut i = 0usize;
        let mut stmt_start = 0usize;

        let mut in_single = false;
        let mut in_double = false;
        let mut in_backtick = false;
        let mut in_line_comment = false;
        let mut in_block_comment = false;

        while i < bytes.len() {
            let b = bytes[i];

            if in_line_comment {
                if b == b'\n' {
                    in_line_comment = false;
                }
                i += 1;
                continue;
            }

            if in_block_comment {
                if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    in_block_comment = false;
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }

            if in_single {
                if b == b'\'' {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                        i += 2;
                        continue;
                    }
                    in_single = false;
                }
                i += 1;
                continue;
            }

            if in_double {
                if b == b'"' {
                    in_double = false;
                }
                i += 1;
                continue;
            }

            if in_backtick {
                if b == b'`' {
                    in_backtick = false;
                }
                i += 1;
                continue;
            }

            // Enter comments
            if (i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'-')
                || bytes[i] == b'#'
            {
                in_line_comment = true;
                i += 1;
                continue;
            }
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                in_block_comment = true;
                i += 2;
                continue;
            }

            // Enter quotes
            if b == b'\'' {
                in_single = true;
                i += 1;
                continue;
            }
            if b == b'"' {
                in_double = true;
                i += 1;
                continue;
            }
            if b == b'`' {
                in_backtick = true;
                i += 1;
                continue;
            }

            // Statement boundary
            if b == b';' {
                let end = (i + 1).min(bytes.len());
                ranges.push((stmt_start, end));
                stmt_start = end;
                i += 1;
                continue;
            }

            i += 1;
        }

        if stmt_start < text.len() {
            ranges.push((stmt_start, text.len()));
        }

        let mut chosen = None;
        for (s, e) in ranges {
            if cursor_b >= s && cursor_b <= e {
                chosen = Some((s, e));
                break;
            }
        }
        let (mut s, mut e) = chosen.unwrap_or((0, text.len()));

        while s < e && text.as_bytes()[s].is_ascii_whitespace() {
            s += 1;
        }
        while e > s && text.as_bytes()[e - 1].is_ascii_whitespace() {
            e -= 1;
        }

        let stmt = text.get(s..e)?.trim();
        if stmt.is_empty() {
            None
        } else {
            Some(stmt.to_string())
        }
    }

    fn apply_to_selected_lines<F: FnMut(&str) -> String>(
        &mut self,
        cursor_range: Option<&egui::text::CursorRange>,
        mut f: F,
    ) -> bool {
        let cr = match cursor_range {
            Some(cr) => cr,
            None => return false,
        };

        let (a_char, b_char) = Self::selection_char_range(cr);
        let a_b = Self::char_to_byte(&self.query, a_char);
        let b_b = Self::char_to_byte(&self.query, b_char);
        let sel_start = a_b.min(b_b);
        let sel_end = a_b.max(b_b);

        let line_start = self.query[..sel_start]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let line_end = self.query[sel_end..]
            .find('\n')
            .map(|i| sel_end + i + 1)
            .unwrap_or(self.query.len());

        let region = self.query[line_start..line_end].to_string();
        let mut out = String::with_capacity(region.len() + 16);

        for line in region.split_inclusive('\n') {
            out.push_str(&f(line));
        }

        self.query.replace_range(line_start..line_end, &out);
        true
    }

    fn indent_selection(&mut self, cursor_range: Option<&egui::text::CursorRange>) -> bool {
        const IND: &str = "  ";
        self.apply_to_selected_lines(cursor_range, |line| {
            if line == "\n" {
                line.to_string()
            } else {
                format!("{IND}{line}")
            }
        })
    }

    fn outdent_selection(&mut self, cursor_range: Option<&egui::text::CursorRange>) -> bool {
        self.apply_to_selected_lines(cursor_range, |line| {
            let (body, nl) = if let Some(stripped) = line.strip_suffix('\n') {
                (stripped, "\n")
            } else {
                (line, "")
            };

            let trimmed = if body.starts_with("  ") {
                &body[2..]
            } else if body.starts_with('\t') {
                &body[1..]
            } else if body.starts_with(' ') {
                &body[1..]
            } else {
                body
            };

            format!("{trimmed}{nl}")
        })
    }

    fn toggle_comment_selection(
        &mut self,
        cursor_range: Option<&egui::text::CursorRange>,
    ) -> bool {
        let cr = match cursor_range {
            Some(cr) => cr,
            None => return false,
        };

        let (a_char, b_char) = Self::selection_char_range(cr);
        let a_b = Self::char_to_byte(&self.query, a_char);
        let b_b = Self::char_to_byte(&self.query, b_char);
        let sel_start = a_b.min(b_b);
        let sel_end = a_b.max(b_b);

        let line_start = self.query[..sel_start]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let line_end = self.query[sel_end..]
            .find('\n')
            .map(|i| sel_end + i + 1)
            .unwrap_or(self.query.len());

        let region = self.query[line_start..line_end].to_string();

        let mut all_commented = true;
        for raw in region.lines() {
            if raw.trim().is_empty() {
                continue;
            }
            let ws = raw
                .chars()
                .take_while(|c| c.is_whitespace())
                .count();
            let rest = &raw[Self::char_to_byte(raw, ws)..];
            if !rest.starts_with("--") {
                all_commented = false;
                break;
            }
        }

        let mut out = String::with_capacity(region.len() + 16);
        for line in region.split_inclusive('\n') {
            let (body, nl) = if let Some(stripped) = line.strip_suffix('\n') {
                (stripped, "\n")
            } else {
                (line, "")
            };

            if body.trim().is_empty() {
                out.push_str(body);
                out.push_str(nl);
                continue;
            }

            let ws_chars = body
                .chars()
                .take_while(|c| c.is_whitespace())
                .count();
            let ws_b = Self::char_to_byte(body, ws_chars);
            let (ws, rest) = body.split_at(ws_b);

            if all_commented {
                let mut rest2 = rest;
                if let Some(r) = rest2.strip_prefix("--") {
                    rest2 = r;
                    if let Some(r2) = rest2.strip_prefix(' ') {
                        rest2 = r2;
                    }
                }
                out.push_str(ws);
                out.push_str(rest2);
                out.push_str(nl);
            } else {
                out.push_str(ws);
                out.push_str("-- ");
                out.push_str(rest);
                out.push_str(nl);
            }
        }

        self.query.replace_range(line_start..line_end, &out);
        true
    }

    fn gutter(ui: &mut egui::Ui, lines: usize, font_size: f32) {
        let digits = lines.to_string().len().max(2);
        let gutter_w = ui
            .fonts(|f| {
                f.layout_no_wrap(
                    "0".repeat(digits + 1),
                    FontId::monospace(font_size),
                    ui.visuals().weak_text_color(),
                )
                .size()
                .x
            })
            .ceil()
            + 6.0;

        let mut s = String::with_capacity(lines * (digits + 1));
        for i in 1..=lines {
            use std::fmt::Write;
            let _ = writeln!(&mut s, "{:>width$}", i, width = digits);
        }

        let text = egui::RichText::new(s)
            .monospace()
            .color(ui.visuals().weak_text_color());

        ui.allocate_ui_with_layout(
            egui::vec2(gutter_w, ui.available_height()),
            egui::Layout::top_down(egui::Align::RIGHT),
            |ui| {
                ui.add(egui::Label::new(text).selectable(false));
            },
        );
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> EditorAction {
        let mut action = EditorAction {
            execute_sql: None,
            save: false,
        };

        let mut want_run_all = false;
        let mut want_run_stmt = false;
        let mut want_save = false;
        let mut want_indent = false;
        let mut want_outdent = false;
        let mut want_comment = false;

        ui.horizontal(|ui| {
            if ui.button("Run (Ctrl+Enter)").clicked() {
                want_run_all = true;
            }
            if ui.button("Run stmt (Ctrl+Shift+Enter)").clicked() {
                want_run_stmt = true;
            }
            if ui.button("Save (Ctrl+S)").clicked() {
                want_save = true;
            }

            ui.separator();

            ui.toggle_value(&mut self.wrap, "Wrap");
            ui.separator();

            if ui.button("Indent (Ctrl+])").clicked() {
                want_indent = true;
            }
            if ui.button("Outdent (Ctrl+[)").clicked() {
                want_outdent = true;
            }
            if ui.button("Comment (Ctrl+/)").clicked() {
                want_comment = true;
            }
        });

        ui.add_space(6.0);

        let text_edit_id = ui.make_persistent_id("sql_editor_textedit");

        let wrap_on = self.wrap;
        let mut layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let mut job = Self::highlight_sql(text);
            if wrap_on {
                job.wrap.max_width = wrap_width;
            } else {
                job.wrap.max_width = f32::INFINITY;
            }
            ui.fonts(|f| f.layout_job(job))
        };

        let lines = Self::count_lines(&self.query);

        let mut output: Option<egui::text_edit::TextEditOutput> = None;

        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt("sql_editor_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        Self::gutter(ui, lines, 14.0);

                        let out = egui::TextEdit::multiline(&mut self.query)
                            .id(text_edit_id)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .hint_text("Write SQL…")
                            .layouter(&mut layouter)
                            .show(ui);

                        output = Some(out);
                    });
                });
        });

        let output = output.expect("TextEditOutput should exist");
        let response = &output.response;
        let response_rect = response.rect;

        // Cursor info (Ln/Col)
        let (cursor_char, cursor_b, ln, col) = if let Some(cr) = &output.cursor_range {
            let c = cr.primary.ccursor.index;
            let b = Self::char_to_byte(&self.query, c);
            let (ln, col) = Self::cursor_ln_col(&self.query, b);
            (c, b, ln, col)
        } else {
            (0, 0, 1, 1)
        };

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.colored_label(
                ui.visuals().weak_text_color(),
                format!("Ln {}, Col {}", ln, col),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.colored_label(
                    ui.visuals().weak_text_color(),
                    "Autocomplete: ↑↓ Tab/Enter • Esc dismiss",
                );
            });
        });

        // Autocomplete update
        if let Some(cursor_range) = output.cursor_range.clone() {
            let cursor_moved_away =
                (cursor_char as i32 - self.autocomplete.last_cursor_char as i32).abs() > 1;

            if response.changed() {
                let (start_b, end_b, word) = self.get_word_bounds_b(cursor_char);

                if word.len() >= 2 {
                    let suggestions = self.get_suggestions(&word);
                    if !suggestions.is_empty() {
                        self.autocomplete.active = true;
                        self.autocomplete.suggestions = suggestions;
                        self.autocomplete.selected = 0;
                        self.autocomplete.word_start_b = start_b;
                        self.autocomplete.word_end_b = end_b;

                        let ccursor = cursor_range.primary.ccursor;
                        let cursor_rect = output
                            .galley
                            .pos_from_cursor(&output.galley.from_ccursor(ccursor));
                        self.autocomplete.popup_pos = response_rect.min
                            + cursor_rect.min.to_vec2()
                            + egui::vec2(0.0, 20.0);
                    } else {
                        self.dismiss_autocomplete();
                    }
                } else {
                    self.dismiss_autocomplete();
                }
            } else if cursor_moved_away && self.autocomplete.active {
                self.dismiss_autocomplete();
            }

            self.autocomplete.last_cursor_char = cursor_char;
        }

        // Autocomplete keyboard
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

                let accept_enter = i.key_pressed(egui::Key::Enter)
                    && !i.modifiers.ctrl
                    && !i.modifiers.shift
                    && !i.modifiers.alt;
                let accept_tab = i.key_pressed(egui::Key::Tab);

                if accept_tab || accept_enter {
                    should_apply = true;
                }
            });

            if should_dismiss {
                self.dismiss_autocomplete();
            } else if should_apply {
                if let Some(s) = self
                    .autocomplete
                    .suggestions
                    .get(self.autocomplete.selected)
                    .cloned()
                {
                    self.apply_suggestion(&s);
                    ui.memory_mut(|m| m.request_focus(text_edit_id));
                }
            }
        }

        // Autocomplete popup
        if self.autocomplete.active && !self.autocomplete.suggestions.is_empty() {
            let popup_id = ui.make_persistent_id("sql_autocomplete");
            let mut clicked: Option<Suggestion> = None;

            egui::Area::new(popup_id)
                .order(egui::Order::Foreground)
                .fixed_pos(self.autocomplete.popup_pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            ui.set_min_width(260.0);
                            ui.set_max_width(420.0);

                            for (idx, suggestion) in
                                self.autocomplete.suggestions.iter().enumerate()
                            {
                                let is_selected = idx == self.autocomplete.selected;
                                let bg = if is_selected {
                                    ui.visuals().selection.bg_fill
                                } else {
                                    Color32::TRANSPARENT
                                };

                                let r = egui::Frame::new()
                                    .fill(bg)
                                    .inner_margin(egui::Margin::symmetric(6, 3))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(suggestion.kind.label())
                                                    .small()
                                                    .color(suggestion.kind.color()),
                                            );
                                            ui.add_space(8.0);
                                            ui.label(
                                                egui::RichText::new(&suggestion.display)
                                                    .monospace(),
                                            );
                                        });
                                    })
                                    .response;

                                if r.clicked() {
                                    clicked = Some(suggestion.clone());
                                }
                            }
                        });
                });

            let clicked_any = clicked.is_some();
            if let Some(s) = clicked {
                self.apply_suggestion(&s);
                ui.memory_mut(|m| m.request_focus(text_edit_id));
            }

            if ui.input(|i| i.pointer.any_click()) && !clicked_any {
                let pos = ui.input(|i| i.pointer.interact_pos());
                if let Some(pos) = pos {
                    let popup_rect = egui::Rect::from_min_size(
                        self.autocomplete.popup_pos,
                        egui::vec2(420.0, 260.0),
                    );
                    if !popup_rect.contains(pos) && !response_rect.contains(pos) {
                        self.dismiss_autocomplete();
                    }
                }
            }
        }

        // Shortcuts + toolbar actions that need cursor/selection
        let ctrl_enter = ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter));
        let ctrl_shift_enter = ui.input(|i| {
            i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Enter)
        });
        let ctrl_s = ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S));
        let ctrl_slash =
            ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Slash));
        let ctrl_rbracket =
            ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::CloseBracket));
        let ctrl_lbracket =
            ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::OpenBracket));

        if ctrl_s || want_save {
            action.save = true;
        }

        if ctrl_enter || want_run_all {
            action.execute_sql = Some(self.query.clone());
            self.dismiss_autocomplete();
        }

        if ctrl_shift_enter || want_run_stmt {
            let sel = self.selected_text(output.cursor_range.as_ref());
            let stmt = sel.or_else(|| self.statement_at_cursor(cursor_char));
            action.execute_sql = stmt;
            self.dismiss_autocomplete();
        }

        let cr = output.cursor_range.as_ref();
        if ctrl_slash || want_comment {
            if self.toggle_comment_selection(cr) {
                ui.memory_mut(|m| m.request_focus(text_edit_id));
            }
        }
        if ctrl_rbracket || want_indent {
            if self.indent_selection(cr) {
                ui.memory_mut(|m| m.request_focus(text_edit_id));
            }
        }
        if ctrl_lbracket || want_outdent {
            if self.outdent_selection(cr) {
                ui.memory_mut(|m| m.request_focus(text_edit_id));
            }
        }

        // Keep autocomplete state sane if cursor jumps far (e.g. due to edits)
        if cursor_b > self.query.len() {
            self.dismiss_autocomplete();
        }

        action
    }
}
