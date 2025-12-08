use crate::db::QueryResult;
use std::fs;

#[derive(Clone, Copy)]
pub enum ExportFormat {
    Csv,
    Json,
    Xml,
}

pub fn export_results(result: &QueryResult, format: ExportFormat) {
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

    if let Some(path) = rfd::FileDialog::new()
        .add_filter(filter_name, &[extension])
        .set_file_name(format!("export.{}", extension))
        .save_file()
    {
        if let Err(e) = fs::write(&path, content) {
            eprintln!("Failed to export: {}", e);
        }
    }
}

fn export_csv(result: &QueryResult) -> String {
    let mut output = String::new();
    
    // Header
    output.push_str(&result.columns.iter()
        .map(|c| escape_csv(c))
        .collect::<Vec<_>>()
        .join(","));
    output.push('\n');

    // Rows
    for row in &result.rows {
        output.push_str(&row.iter()
            .map(|c| escape_csv(c))
            .collect::<Vec<_>>()
            .join(","));
        output.push('\n');
    }

    output
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn export_json(result: &QueryResult) -> String {
    let rows: Vec<serde_json::Value> = result.rows.iter()
        .map(|row| {
            let obj: serde_json::Map<String, serde_json::Value> = result.columns.iter()
                .zip(row.iter())
                .map(|(col, val)| {
                    let json_val = if val == "NULL" {
                        serde_json::Value::Null
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
    let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<results>\n");

    for row in &result.rows {
        output.push_str("  <row>\n");
        for (col, val) in result.columns.iter().zip(row.iter()) {
            let escaped_val = escape_xml(val);
            output.push_str(&format!("    <{}>{}</{}>\n", col, escaped_val, col));
        }
        output.push_str("  </row>\n");
    }

    output.push_str("</results>");
    output
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
