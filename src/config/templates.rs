use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTemplate {
    pub name: String,
    pub description: String,
    pub sql: String,
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub placeholder: String,
    pub default_value: Option<String>,
}

impl QueryTemplate {
    pub fn apply(&self, values: &[(String, String)]) -> String {
        let mut sql = self.sql.clone();
        for (var_name, value) in values {
            sql = sql.replace(&format!("${{{}}}", var_name), value);
        }
        sql
    }
}

pub fn get_builtin_templates() -> Vec<QueryTemplate> {
    vec![
        QueryTemplate {
            name: "Select All".to_string(),
            description: "Basic SELECT with LIMIT".to_string(),
            sql: "SELECT * FROM ${table} LIMIT ${limit};".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "table".to_string(),
                    placeholder: "table_name".to_string(),
                    default_value: None,
                },
                TemplateVariable {
                    name: "limit".to_string(),
                    placeholder: "100".to_string(),
                    default_value: Some("100".to_string()),
                },
            ],
        },
        QueryTemplate {
            name: "Insert".to_string(),
            description: "INSERT statement".to_string(),
            sql: "INSERT INTO ${table} (${columns}) VALUES (${values});".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "table".to_string(),
                    placeholder: "table_name".to_string(),
                    default_value: None,
                },
                TemplateVariable {
                    name: "columns".to_string(),
                    placeholder: "col1, col2".to_string(),
                    default_value: None,
                },
                TemplateVariable {
                    name: "values".to_string(),
                    placeholder: "'val1', 'val2'".to_string(),
                    default_value: None,
                },
            ],
        },
        QueryTemplate {
            name: "Update".to_string(),
            description: "UPDATE with WHERE clause".to_string(),
            sql: "UPDATE ${table} SET ${column} = ${value} WHERE ${condition};".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "table".to_string(),
                    placeholder: "table_name".to_string(),
                    default_value: None,
                },
                TemplateVariable {
                    name: "column".to_string(),
                    placeholder: "column_name".to_string(),
                    default_value: None,
                },
                TemplateVariable {
                    name: "value".to_string(),
                    placeholder: "new_value".to_string(),
                    default_value: None,
                },
                TemplateVariable {
                    name: "condition".to_string(),
                    placeholder: "id = 1".to_string(),
                    default_value: None,
                },
            ],
        },
        QueryTemplate {
            name: "Count by Group".to_string(),
            description: "Count rows grouped by column".to_string(),
            sql: "SELECT ${column}, COUNT(*) as count FROM ${table} GROUP BY ${column} ORDER BY count DESC;".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "table".to_string(),
                    placeholder: "table_name".to_string(),
                    default_value: None,
                },
                TemplateVariable {
                    name: "column".to_string(),
                    placeholder: "column_name".to_string(),
                    default_value: None,
                },
            ],
        },
        QueryTemplate {
            name: "Find Duplicates".to_string(),
            description: "Find duplicate values in a column".to_string(),
            sql: "SELECT ${column}, COUNT(*) as count FROM ${table} GROUP BY ${column} HAVING COUNT(*) > 1;".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "table".to_string(),
                    placeholder: "table_name".to_string(),
                    default_value: None,
                },
                TemplateVariable {
                    name: "column".to_string(),
                    placeholder: "column_name".to_string(),
                    default_value: None,
                },
            ],
        },
    ]
}
