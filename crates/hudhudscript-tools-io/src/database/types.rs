use serde::{Deserialize, Serialize};

/// Database backend type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    /// PostgreSQL via connection string
    Postgres,
    /// MySQL via connection string
    Mysql,
    /// SQLite via file path
    Sqlite,
}

impl std::fmt::Display for DatabaseBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseBackend::Postgres => write!(f, "postgres"),
            DatabaseBackend::Mysql => write!(f, "mysql"),
            DatabaseBackend::Sqlite => write!(f, "sqlite"),
        }
    }
}

/// Result row: a list of key-value pairs representing one row from the query
pub type Row = std::collections::HashMap<String, serde_json::Value>;

/// Result of executing a SQL query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Rows returned (empty for non-SELECT statements)
    pub rows: Vec<Row>,
    /// Number of rows affected (for INSERT / UPDATE / DELETE)
    pub rows_affected: u64,
    /// Column names in result order
    pub columns: Vec<String>,
}

impl QueryResult {
    /// Create a result for a non-row-returning statement
    pub fn affected(rows_affected: u64) -> Self {
        Self {
            rows: Vec::new(),
            rows_affected,
            columns: Vec::new(),
        }
    }
}

/// Information about a database column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// SQL data type
    pub data_type: String,
    /// Whether the column allows NULL values
    pub nullable: bool,
}
