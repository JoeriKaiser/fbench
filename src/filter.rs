/// A single column filter condition.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnFilter {
    pub column: String,
    pub operator: FilterOperator,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Like,
    NotLike,
    IsNull,
    IsNotNull,
}

impl FilterOperator {
    pub fn sql_operator(&self) -> &str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::GreaterOrEqual => ">=",
            Self::LessOrEqual => "<=",
            Self::Like => "LIKE",
            Self::NotLike => "NOT LIKE",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
        }
    }

    pub fn needs_value(&self) -> bool {
        !matches!(self, Self::IsNull | Self::IsNotNull)
    }

    /// Return available operators for a column type.
    pub fn for_type(col_type: &str) -> Vec<Self> {
        let lower = col_type.to_lowercase();
        if lower.contains("bool") {
            vec![Self::Equal, Self::NotEqual, Self::IsNull, Self::IsNotNull]
        } else if lower.contains("int")
            || lower.contains("float")
            || lower.contains("double")
            || lower.contains("numeric")
            || lower.contains("decimal")
            || lower.contains("serial")
        {
            vec![
                Self::Equal,
                Self::NotEqual,
                Self::GreaterThan,
                Self::LessThan,
                Self::GreaterOrEqual,
                Self::LessOrEqual,
                Self::IsNull,
                Self::IsNotNull,
            ]
        } else if lower.contains("date") || lower.contains("time") || lower.contains("timestamp") {
            vec![
                Self::Equal,
                Self::NotEqual,
                Self::GreaterThan,
                Self::LessThan,
                Self::GreaterOrEqual,
                Self::LessOrEqual,
                Self::IsNull,
                Self::IsNotNull,
            ]
        } else {
            // Text/varchar/char/etc
            vec![
                Self::Equal,
                Self::NotEqual,
                Self::Like,
                Self::NotLike,
                Self::IsNull,
                Self::IsNotNull,
            ]
        }
    }

    pub fn display_label(&self) -> &str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::GreaterOrEqual => ">=",
            Self::LessOrEqual => "<=",
            Self::Like => "LIKE",
            Self::NotLike => "NOT LIKE",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
        }
    }

    pub fn all_variants() -> Vec<Self> {
        vec![
            Self::Equal,
            Self::NotEqual,
            Self::GreaterThan,
            Self::LessThan,
            Self::GreaterOrEqual,
            Self::LessOrEqual,
            Self::Like,
            Self::NotLike,
            Self::IsNull,
            Self::IsNotNull,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortColumn {
    pub column: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterState {
    pub table: String,
    pub filters: Vec<ColumnFilter>,
    pub sort: Option<SortColumn>,
    pub limit: usize,
}

impl FilterState {
    pub fn new(table: String) -> Self {
        Self {
            table,
            filters: vec![],
            sort: None,
            limit: 100,
        }
    }

    /// Generate a SELECT query from the current filter state.
    pub fn to_sql(&self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table);

        let active_filters: Vec<&ColumnFilter> = self
            .filters
            .iter()
            .filter(|f| !f.column.is_empty())
            .collect();

        if !active_filters.is_empty() {
            sql.push_str(" WHERE ");
            let clauses: Vec<String> = active_filters
                .iter()
                .map(|f| {
                    if f.operator.needs_value() {
                        format!(
                            "{} {} '{}'",
                            f.column,
                            f.operator.sql_operator(),
                            f.value.replace('\'', "''")
                        )
                    } else {
                        format!("{} {}", f.column, f.operator.sql_operator())
                    }
                })
                .collect();
            sql.push_str(&clauses.join(" AND "));
        }

        if let Some(sort) = &self.sort {
            let dir = match sort.direction {
                SortDirection::Asc => "ASC",
                SortDirection::Desc => "DESC",
            };
            sql.push_str(&format!(" ORDER BY {} {}", sort.column, dir));
        }

        sql.push_str(&format!(" LIMIT {}", self.limit));
        sql
    }
}
