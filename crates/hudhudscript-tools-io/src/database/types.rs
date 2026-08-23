use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    Postgres,
    Mysql,
    Sqlite,
}

impl std::fmt::Display for DatabaseBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
            Self::Sqlite => "sqlite",
        })
    }
}

pub type Row = std::collections::HashMap<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub rows: Vec<Row>,
    pub rows_affected: u64,
    pub columns: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub column_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_insert_id: Option<serde_json::Value>,
    #[serde(default)]
    pub truncated: bool,
}

impl QueryResult {
    pub fn affected(rows_affected: u64, last_insert_id: Option<serde_json::Value>) -> Self {
        Self {
            rows: Vec::new(),
            rows_affected,
            columns: Vec::new(),
            column_types: Vec::new(),
            last_insert_id,
            truncated: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    #[serde(default)]
    pub primary_key: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatus {
    pub backend: DatabaseBackend,
    pub size: u32,
    pub idle: usize,
    pub closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecuteOptions {
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub max_rows: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionOptions {
    #[serde(default)]
    pub isolation: Option<String>,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnection {
    pub handle: String,
    pub backend: DatabaseBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTransaction {
    pub transaction: String,
    pub backend: DatabaseBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub sql: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub applied: Vec<i64>,
    pub skipped: Vec<i64>,
}
