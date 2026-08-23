use std::fmt;

use serde::{Deserialize, Serialize};

use super::DatabaseBackend;
#[cfg(feature = "db")]
use super::DatabaseError;

fn default_max_connections() -> u32 {
    10
}
fn default_acquire_timeout() -> u64 {
    10_000
}
fn default_query_timeout() -> u64 {
    30_000
}
fn default_idle_timeout() -> u64 {
    600_000
}
fn default_lifetime() -> u64 {
    1_800_000
}
fn default_transaction_timeout() -> u64 {
    60_000
}
fn default_max_rows() -> usize {
    10_000
}
fn default_busy_timeout() -> u64 {
    5_000
}
fn yes() -> bool {
    true
}

/// Pool, timeout, and backend settings for one database connection handle.
#[derive(Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub backend: DatabaseBackend,
    #[serde(alias = "url")]
    pub connection_string: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default)]
    pub min_connections: u32,
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout_ms: u64,
    #[serde(default = "default_query_timeout")]
    pub query_timeout_ms: u64,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_ms: u64,
    #[serde(default = "default_lifetime")]
    pub max_lifetime_ms: u64,
    #[serde(default = "default_transaction_timeout")]
    pub transaction_timeout_ms: u64,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub sqlite_create_if_missing: bool,
    #[serde(default = "yes")]
    pub sqlite_wal: bool,
    #[serde(default = "default_busy_timeout")]
    pub sqlite_busy_timeout_ms: u64,
    #[serde(default = "yes")]
    pub test_before_acquire: bool,
    /// Require an encrypted remote connection. The URL must also select a
    /// backend TLS mode; certificate verification remains controlled by it.
    #[serde(default)]
    pub tls_required: bool,
}

impl fmt::Debug for DatabaseConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DatabaseConfig")
            .field("backend", &self.backend)
            .field("connection_string", &"<redacted>")
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("acquire_timeout_ms", &self.acquire_timeout_ms)
            .field("query_timeout_ms", &self.query_timeout_ms)
            .field("idle_timeout_ms", &self.idle_timeout_ms)
            .field("max_lifetime_ms", &self.max_lifetime_ms)
            .field("transaction_timeout_ms", &self.transaction_timeout_ms)
            .field("max_rows", &self.max_rows)
            .field("read_only", &self.read_only)
            .field("sqlite_create_if_missing", &self.sqlite_create_if_missing)
            .field("sqlite_wal", &self.sqlite_wal)
            .field("sqlite_busy_timeout_ms", &self.sqlite_busy_timeout_ms)
            .field("test_before_acquire", &self.test_before_acquire)
            .field("tls_required", &self.tls_required)
            .finish()
    }
}

impl DatabaseConfig {
    pub fn postgres(connection_string: impl Into<String>) -> Self {
        Self::new(DatabaseBackend::Postgres, connection_string.into())
    }

    pub fn mysql(connection_string: impl Into<String>) -> Self {
        Self::new(DatabaseBackend::Mysql, connection_string.into())
    }

    pub fn sqlite(path: impl Into<String>) -> Self {
        let path = path.into();
        let url = if path.starts_with("sqlite:") {
            path
        } else {
            format!("sqlite://{path}")
        };
        let mut config = Self::new(DatabaseBackend::Sqlite, url);
        config.max_connections = 1;
        config.sqlite_create_if_missing = true;
        config
    }

    fn new(backend: DatabaseBackend, connection_string: String) -> Self {
        Self {
            backend,
            connection_string,
            max_connections: default_max_connections(),
            min_connections: 0,
            acquire_timeout_ms: default_acquire_timeout(),
            query_timeout_ms: default_query_timeout(),
            idle_timeout_ms: default_idle_timeout(),
            max_lifetime_ms: default_lifetime(),
            transaction_timeout_ms: default_transaction_timeout(),
            max_rows: default_max_rows(),
            read_only: false,
            sqlite_create_if_missing: false,
            sqlite_wal: true,
            sqlite_busy_timeout_ms: default_busy_timeout(),
            test_before_acquire: true,
            tls_required: false,
        }
    }

    #[cfg(feature = "db")]
    pub(crate) fn validate(&self) -> Result<(), DatabaseError> {
        if self.connection_string.trim().is_empty() {
            return Err(DatabaseError::InvalidArguments(
                "database URL is empty".into(),
            ));
        }
        if self.max_connections == 0 || self.max_connections > 1_024 {
            return Err(DatabaseError::InvalidArguments(
                "max_connections must be between 1 and 1024".into(),
            ));
        }
        if self.min_connections > self.max_connections {
            return Err(DatabaseError::InvalidArguments(
                "min_connections cannot exceed max_connections".into(),
            ));
        }
        if self.acquire_timeout_ms == 0
            || self.query_timeout_ms == 0
            || self.transaction_timeout_ms == 0
        {
            return Err(DatabaseError::InvalidArguments(
                "database timeouts must be greater than zero".into(),
            ));
        }
        if self.max_rows == 0 || self.max_rows > 1_000_000 {
            return Err(DatabaseError::InvalidArguments(
                "max_rows must be between 1 and 1000000".into(),
            ));
        }
        let scheme_matches = match self.backend {
            DatabaseBackend::Postgres => {
                self.connection_string.starts_with("postgres://")
                    || self.connection_string.starts_with("postgresql://")
            }
            DatabaseBackend::Mysql => self.connection_string.starts_with("mysql://"),
            DatabaseBackend::Sqlite => self.connection_string.starts_with("sqlite:"),
        };
        if !scheme_matches {
            return Err(DatabaseError::InvalidArguments(format!(
                "URL scheme does not match {} backend",
                self.backend
            )));
        }
        if self.tls_required {
            self.validate_tls()?;
        }
        if self.backend == DatabaseBackend::Sqlite
            && self.connection_string.contains(":memory:")
            && self.max_connections != 1
        {
            return Err(DatabaseError::InvalidArguments(
                "in-memory SQLite requires max_connections = 1".into(),
            ));
        }
        Ok(())
    }

    #[cfg(feature = "db")]
    fn validate_tls(&self) -> Result<(), DatabaseError> {
        if self.backend == DatabaseBackend::Sqlite {
            return Err(DatabaseError::InvalidArguments(
                "tls_required is not applicable to SQLite".into(),
            ));
        }
        let url = url::Url::parse(&self.connection_string)
            .map_err(|_| DatabaseError::InvalidArguments("invalid database URL".into()))?;
        let requested = url.query_pairs().any(|(key, value)| {
            let key = key.to_ascii_lowercase().replace('_', "-");
            let value = value.to_ascii_lowercase().replace('_', "-");
            match self.backend {
                DatabaseBackend::Postgres => {
                    key == "sslmode"
                        && matches!(value.as_str(), "require" | "verify-ca" | "verify-full")
                }
                DatabaseBackend::Mysql => {
                    key == "ssl-mode"
                        && matches!(value.as_str(), "required" | "verify-ca" | "verify-identity")
                }
                DatabaseBackend::Sqlite => false,
            }
        });
        if requested {
            Ok(())
        } else {
            Err(DatabaseError::InvalidArguments(format!(
                "tls_required needs an explicit TLS mode in the {} URL",
                self.backend
            )))
        }
    }
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::*;

    #[test]
    fn tls_required_rejects_plain_remote_url() {
        let mut config = DatabaseConfig::postgres("postgres://user:pass@db.example/app");
        config.tls_required = true;
        assert!(config.validate().is_err());
        config.connection_string.push_str("?sslmode=verify-full");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn backend_and_url_scheme_must_match() {
        let config = DatabaseConfig::mysql("postgres://localhost/app");
        assert!(config.validate().is_err());
    }

    #[test]
    fn debug_output_redacts_credentials() {
        let config = DatabaseConfig::postgres("postgres://admin:secret@localhost/app");
        let output = format!("{config:?}");
        assert!(output.contains("<redacted>"));
        assert!(!output.contains("secret"));
        assert!(!output.contains("admin"));
    }
}
