use crate::db::QueryResult;
use std::fs;

#[derive(Clone, Copy, Debug)]
pub enum ExportFormat {
    Csv,
    Json,
    Xml,
}

pub fn export_results(result: &QueryResult, format: ExportFormat) {
    tracing::info!("Starting export with format {:?}", format);

    let (extension, content) = match format {
        ExportFormat::Csv => ("csv", export_csv(result)),
        ExportFormat::Json => ("json", export_json(result)),
        ExportFormat::Xml => ("xml", export_xml(result)),
    };

    let filter_name = match format {
        ExportFormat::Csv => "CSV files",
        ExportFormat::Json => "JSON files",
        ExportFormat::Xml => "XML files",
    };

    tracing::info!("Opening file dialog for {} export", extension);

    if let Some(path) = rfd::FileDialog::new()
        .add_filter(filter_name, &[extension])
        .set_file_name(format!("export.{}", extension))
        .save_file()
    {
        tracing::info!("Selected path: {:?}", path);
        if let Err(e) = fs::write(&path, content) {
            tracing::error!("Failed to export: {}", e);
        } else {
            tracing::info!("Export successful");
        }
    } else {
        tracing::info!("File dialog cancelled");
    }
}

fn export_csv(result: &QueryResult) -> String {
    let mut output = String::with_capacity(result.rows.len() * 100);

    output.push_str(
        &result
            .columns
            .iter()
            .map(|c| escape_csv(c))
            .collect::<Vec<_>>()
            .join(","),
    );
    output.push('\n');

    for row in &result.rows {
        output.push_str(
            &row.iter()
                .map(|c| escape_csv(c))
                .collect::<Vec<_>>()
                .join(","),
        );
        output.push('\n');
    }

    output
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn export_json(result: &QueryResult) -> String {
    let rows: Vec<serde_json::Value> = result
        .rows
        .iter()
        .map(|row| {
            let obj: serde_json::Map<String, serde_json::Value> = result
                .columns
                .iter()
                .zip(row.iter())
                .map(|(col, val)| {
                    let json_val = if val == "NULL" {
                        serde_json::Value::Null
                    } else if let Ok(n) = val.parse::<i64>() {
                        serde_json::Value::Number(n.into())
                    } else if let Ok(n) = val.parse::<f64>() {
                        serde_json::Number::from_f64(n)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::String(val.clone()))
                    } else if val == "true" {
                        serde_json::Value::Bool(true)
                    } else if val == "false" {
                        serde_json::Value::Bool(false)
                    } else {
                        serde_json::Value::String(val.clone())
                    };
                    (col.clone(), json_val)
                })
                .collect();
            serde_json::Value::Object(obj)
        })
        .collect();

    serde_json::to_string_pretty(&rows).unwrap_or_default()
}

fn export_xml(result: &QueryResult) -> String {
    let mut output = String::with_capacity(result.rows.len() * 200);
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<results>\n");

    for row in &result.rows {
        output.push_str("  <row>\n");
        for (col, val) in result.columns.iter().zip(row.iter()) {
            let safe_col = sanitize_xml_tag(col);
            let escaped_val = escape_xml(val);
            output.push_str(&format!(
                "    <{}>{}</{}>\n",
                safe_col, escaped_val, safe_col
            ));
        }
        output.push_str("  </row>\n");
    }

    output.push_str("</results>");
    output
}

fn sanitize_xml_tag(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for (i, c) in s.chars().enumerate() {
        if i == 0 && c.is_ascii_digit() {
            result.push('_');
        }
        if c.is_alphanumeric() || c == '_' || c == '-' {
            result.push(c);
        } else {
            result.push('_');
        }
    }
    if result.is_empty() {
        result.push_str("column");
    }
    result
}

fn escape_xml(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}
