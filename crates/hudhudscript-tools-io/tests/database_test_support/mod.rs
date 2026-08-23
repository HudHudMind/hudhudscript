#![allow(dead_code)]

use hudhudscript_tools_io::database::{
    DatabaseConfig, DatabaseConnection, DatabaseError, DatabaseService,
};
use serde::Deserialize;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiveBackend {
    Postgres,
    Mysql,
}

impl LiveBackend {
    pub fn parameter(self, index: usize) -> String {
        match self {
            Self::Postgres => format!("${index}"),
            Self::Mysql => "?".into(),
        }
    }

    pub fn sleep_query(self) -> &'static str {
        match self {
            Self::Postgres => "SELECT pg_sleep(0.2)",
            Self::Mysql => "SELECT SLEEP(0.2) AS slept",
        }
    }
}

#[derive(Deserialize)]
struct ProjectConfig {
    database_tests: DatabaseTestReference,
}

#[derive(Deserialize)]
struct DatabaseTestReference {
    secrets_file: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalConfig {
    database_tests: DatabaseTestSecrets,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DatabaseTestSecrets {
    postgres_url: String,
    mysql_url: String,
}

pub fn live_database_config(backend: LiveBackend) -> DatabaseConfig {
    let secrets = load_database_secrets();
    let mut config = match backend {
        LiveBackend::Postgres => DatabaseConfig::postgres(secrets.postgres_url),
        LiveBackend::Mysql => DatabaseConfig::mysql(secrets.mysql_url),
    };
    config.tls_required = true;
    config.max_connections = 2;
    config.acquire_timeout_ms = 5_000;
    config.query_timeout_ms = 5_000;
    config
}

pub fn invalid_password_config(backend: LiveBackend) -> (DatabaseConfig, String) {
    let mut config = live_database_config(backend);
    config.max_connections = 1;
    config.acquire_timeout_ms = 5_000;
    let mut parsed = url::Url::parse(&config.connection_string)
        .unwrap_or_else(|_| panic!("invalid URL in local HudHud database config"));
    let original = parsed
        .password()
        .unwrap_or_else(|| panic!("database test URL has no password"))
        .to_string();
    parsed
        .set_password(Some("hudhud-intentionally-invalid"))
        .unwrap_or_else(|_| panic!("cannot construct invalid credential test URL"));
    config.connection_string = parsed.into();
    (config, original)
}

pub fn unique_suffix() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn unique_migration_version() -> i64 {
    let bytes = uuid::Uuid::new_v4().into_bytes();
    let value = i64::from_be_bytes(bytes[..8].try_into().expect("eight UUID bytes"));
    value.checked_abs().unwrap_or(i64::MAX).max(1)
}

pub async fn live_test_guard() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

pub async fn open_live_database(
    config: DatabaseConfig,
) -> Result<DatabaseConnection, DatabaseError> {
    let mut last_error = None;
    for attempt in 1..=5 {
        match DatabaseService.open(config.clone()).await {
            Ok(connection) => return Ok(connection),
            Err(error) if transient_connection_error(&error) && attempt < 5 => {
                last_error = Some(error);
                let delay = 250_u64 * (1_u64 << (attempt - 1));
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error.expect("a failed live database connection attempt"))
}

fn transient_connection_error(error: &DatabaseError) -> bool {
    match error {
        DatabaseError::ConnectionFailed(message) => {
            message.contains("network I/O") || message.contains("pool acquisition timed out")
        }
        _ => false,
    }
}

fn load_database_secrets() -> DatabaseTestSecrets {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project_source = std::fs::read_to_string(workspace.join("hudhud.toml"))
        .unwrap_or_else(|_| panic!("cannot read the project hudhud.toml"));
    let project: ProjectConfig = toml::from_str(&project_source)
        .unwrap_or_else(|_| panic!("invalid database_tests section in hudhud.toml"));
    let relative = Path::new(&project.database_tests.secrets_file);
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        panic!("database_tests.secrets_file must be a relative path without traversal");
    }
    let secrets_path = workspace.join(relative);
    require_private_permissions(&secrets_path);
    let secrets_source = std::fs::read_to_string(secrets_path)
        .unwrap_or_else(|_| panic!("cannot read the local HudHud database config"));
    let config: LocalConfig = toml::from_str(&secrets_source)
        .unwrap_or_else(|_| panic!("invalid local HudHud database config"));
    if config.database_tests.postgres_url.trim().is_empty()
        || config.database_tests.mysql_url.trim().is_empty()
    {
        panic!("database test URLs cannot be empty");
    }
    config.database_tests
}

#[cfg(unix)]
fn require_private_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path)
        .unwrap_or_else(|_| panic!("cannot inspect the local HudHud database config"));
    if !metadata.is_file() || metadata.permissions().mode() & 0o077 != 0 {
        panic!("local HudHud database config must be a 0600 regular file");
    }
}

#[cfg(not(unix))]
fn require_private_permissions(path: &Path) {
    if !path.is_file() {
        panic!("local HudHud database config is not a file");
    }
}
