use std::path::Path;

#[derive(Debug, Clone)]
pub struct ImportData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug)]
pub enum ImportError {
    IoError(String),
    ParseError(String),
    EmptyFile,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::EmptyFile => write!(f, "File is empty"),
        }
    }
}

pub fn parse_file(path: &Path) -> Result<ImportData, ImportError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "csv" => parse_csv(path),
        "json" => parse_json(path),
        _ => Err(ImportError::ParseError(format!(
            "Unsupported file type: .{}",
            ext
        ))),
    }
}

fn parse_csv(path: &Path) -> Result<ImportData, ImportError> {
    let mut reader =
        csv::Reader::from_path(path).map_err(|e| ImportError::IoError(e.to_string()))?;

    let columns: Vec<String> = reader
        .headers()
        .map_err(|e| ImportError::ParseError(e.to_string()))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    if columns.is_empty() {
        return Err(ImportError::EmptyFile);
    }

    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| ImportError::ParseError(e.to_string()))?;
        let row: Vec<String> = record.iter().map(|f| f.to_string()).collect();
        rows.push(row);
    }

    if rows.is_empty() {
        return Err(ImportError::EmptyFile);
    }

    Ok(ImportData { columns, rows })
}

fn parse_json(path: &Path) -> Result<ImportData, ImportError> {
    let content = std::fs::read_to_string(path).map_err(|e| ImportError::IoError(e.to_string()))?;

    let array: Vec<serde_json::Map<String, serde_json::Value>> =
        serde_json::from_str(&content).map_err(|e| ImportError::ParseError(e.to_string()))?;

    if array.is_empty() {
        return Err(ImportError::EmptyFile);
    }

    // Collect all unique keys as columns (preserving order from first object)
    let columns: Vec<String> = array[0].keys().cloned().collect();

    let rows: Vec<Vec<String>> = array
        .iter()
        .map(|obj| {
            columns
                .iter()
                .map(|col| match obj.get(col) {
                    Some(serde_json::Value::Null) | None => "NULL".to_string(),
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(v) => v.to_string(),
                })
                .collect()
        })
        .collect();

    Ok(ImportData { columns, rows })
}

/// Validate import columns against a target table's columns.
/// Returns a list of (file_column_index, table_column_name) mappings.
pub fn auto_map_columns(
    file_columns: &[String],
    table_columns: &[crate::db::ColumnInfo],
) -> Vec<(usize, String)> {
    let mut mapping = Vec::new();
    for (idx, file_col) in file_columns.iter().enumerate() {
        let lower = file_col.to_lowercase();
        if let Some(table_col) = table_columns
            .iter()
            .find(|c| c.name.to_lowercase() == lower)
        {
            mapping.push((idx, table_col.name.clone()));
        }
    }
    mapping
}
