use serde::{Deserialize, Serialize};

use super::DatabaseBackend;

/// Configuration for a database connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// The database backend to use
    pub backend: DatabaseBackend,
    /// Connection string (e.g. `postgres://user:pass@host/db` or `/path/to/db.sqlite`)
    pub connection_string: String,
    /// Maximum number of pooled connections (default: 5)
    pub max_connections: Option<u32>,
}

impl DatabaseConfig {
    /// Create a PostgreSQL configuration
    pub fn postgres(connection_string: impl Into<String>) -> Self {
        Self {
            backend: DatabaseBackend::Postgres,
            connection_string: connection_string.into(),
            max_connections: Some(5),
        }
    }

    /// Create a SQLite configuration
    pub fn sqlite(path: impl Into<String>) -> Self {
        Self {
            backend: DatabaseBackend::Sqlite,
            connection_string: path.into(),
            max_connections: Some(1),
        }
    }
}
