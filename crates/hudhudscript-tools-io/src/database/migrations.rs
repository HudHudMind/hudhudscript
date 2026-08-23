use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::{
    DatabaseBackend, DatabaseError, DatabaseService, ExecuteOptions, Migration, MigrationReport,
    TransactionOptions,
};

pub(crate) async fn migrate(
    service: &DatabaseService,
    handle: &str,
    mut migrations: Vec<Migration>,
) -> Result<MigrationReport, DatabaseError> {
    validate(&mut migrations)?;
    let backend = service.backend(handle)?;
    let transaction = service
        .begin(
            handle,
            TransactionOptions {
                isolation: Some(
                    match backend {
                        DatabaseBackend::Postgres => "serializable",
                        DatabaseBackend::Mysql => "repeatable_read",
                        DatabaseBackend::Sqlite => "immediate",
                    }
                    .into(),
                ),
                read_only: false,
                timeout_ms: None,
            },
        )
        .await?;
    let transaction_id = transaction.transaction;
    let result = run_locked(service, &transaction_id, backend, &migrations).await;
    match result {
        Ok(report) => {
            if backend == DatabaseBackend::Mysql {
                release_mysql_lock(service, &transaction_id).await;
            }
            service.commit(&transaction_id).await?;
            Ok(report)
        }
        Err(error) => {
            if backend == DatabaseBackend::Mysql {
                release_mysql_lock(service, &transaction_id).await;
            }
            let _ = service.rollback(&transaction_id).await;
            Err(error)
        }
    }
}

async fn run_locked(
    service: &DatabaseService,
    transaction: &str,
    backend: DatabaseBackend,
    migrations: &[Migration],
) -> Result<MigrationReport, DatabaseError> {
    acquire_lock(service, transaction, backend).await?;
    service
        .transaction_execute_batch(transaction, migration_table_sql(backend))
        .await?;
    let applied = service
        .transaction_query_all(
            transaction,
            "SELECT version, checksum FROM __hudhud_migrations ORDER BY version",
            &[],
        )
        .await?;
    let existing = existing_checksums(applied.rows)?;
    let mut report = MigrationReport {
        applied: Vec::new(),
        skipped: Vec::new(),
    };
    for migration in migrations {
        let checksum = checksum(&migration.sql);
        if let Some(previous) = existing.get(&migration.version) {
            if previous != &checksum {
                return Err(DatabaseError::QueryFailed(format!(
                    "migration {} checksum changed after it was applied",
                    migration.version
                )));
            }
            report.skipped.push(migration.version);
            continue;
        }
        service
            .transaction_execute_batch(transaction, &migration.sql)
            .await?;
        let (insert, params) = insert_statement(backend, migration, &checksum);
        service
            .transaction_execute(transaction, insert, &params, ExecuteOptions::default())
            .await?;
        report.applied.push(migration.version);
    }
    Ok(report)
}

async fn acquire_lock(
    service: &DatabaseService,
    transaction: &str,
    backend: DatabaseBackend,
) -> Result<(), DatabaseError> {
    match backend {
        DatabaseBackend::Postgres => {
            service
                .transaction_execute(
                    transaction,
                    "SELECT pg_advisory_xact_lock(3108198902464099)",
                    &[],
                    ExecuteOptions::default(),
                )
                .await?;
        }
        DatabaseBackend::Mysql => {
            let result = service
                .transaction_query(
                    transaction,
                    "SELECT GET_LOCK('hudhudscript_migrations', 10) AS locked",
                    &[],
                    ExecuteOptions::default(),
                )
                .await?;
            let locked = result
                .rows
                .first()
                .and_then(|row| row.get("locked"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            if locked != 1 {
                return Err(DatabaseError::Timeout(
                    "could not acquire MySQL migration lock".into(),
                ));
            }
        }
        DatabaseBackend::Sqlite => {}
    }
    Ok(())
}

async fn release_mysql_lock(service: &DatabaseService, transaction: &str) {
    let _ = service
        .transaction_execute(
            transaction,
            "SELECT RELEASE_LOCK('hudhudscript_migrations')",
            &[],
            ExecuteOptions::default(),
        )
        .await;
}

fn migration_table_sql(backend: DatabaseBackend) -> &'static str {
    match backend {
        DatabaseBackend::Postgres => "CREATE TABLE IF NOT EXISTS __hudhud_migrations (version BIGINT PRIMARY KEY, name TEXT NOT NULL, checksum CHAR(64) NOT NULL, applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
        DatabaseBackend::Mysql => "CREATE TABLE IF NOT EXISTS __hudhud_migrations (version BIGINT PRIMARY KEY, name VARCHAR(255) NOT NULL, checksum CHAR(64) NOT NULL, applied_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6)) ENGINE=InnoDB",
        DatabaseBackend::Sqlite => "CREATE TABLE IF NOT EXISTS __hudhud_migrations (version INTEGER PRIMARY KEY, name TEXT NOT NULL, checksum TEXT NOT NULL, applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)",
    }
}

fn insert_statement(
    backend: DatabaseBackend,
    migration: &Migration,
    checksum: &str,
) -> (&'static str, Vec<Value>) {
    let sql = match backend {
        DatabaseBackend::Postgres => {
            "INSERT INTO __hudhud_migrations (version, name, checksum) VALUES ($1, $2, $3)"
        }
        DatabaseBackend::Mysql | DatabaseBackend::Sqlite => {
            "INSERT INTO __hudhud_migrations (version, name, checksum) VALUES (?, ?, ?)"
        }
    };
    (
        sql,
        vec![
            json!(migration.version),
            json!(migration.name),
            json!(checksum),
        ],
    )
}

fn existing_checksums(rows: Vec<super::Row>) -> Result<HashMap<i64, String>, DatabaseError> {
    rows.into_iter()
        .map(|row| {
            let version = row.get("version").and_then(decoded_i64).ok_or_else(|| {
                DatabaseError::QueryFailed("invalid migration version in metadata table".into())
            })?;
            let checksum = row.get("checksum").and_then(Value::as_str).ok_or_else(|| {
                DatabaseError::QueryFailed("invalid migration checksum in metadata table".into())
            })?;
            Ok((version, checksum.to_string()))
        })
        .collect()
}

fn decoded_i64(value: &Value) -> Option<i64> {
    value.as_i64().or_else(|| {
        let object = value.as_object()?;
        if object.get("$type")?.as_str()? != "i64" {
            return None;
        }
        object.get("value")?.as_str()?.parse().ok()
    })
}

fn validate(migrations: &mut [Migration]) -> Result<(), DatabaseError> {
    migrations.sort_by_key(|migration| migration.version);
    let mut versions = HashSet::new();
    for migration in migrations {
        if migration.version <= 0
            || migration.name.trim().is_empty()
            || migration.sql.trim().is_empty()
        {
            return Err(DatabaseError::InvalidArguments(
                "migration version must be positive and name/sql cannot be empty".into(),
            ));
        }
        if migration.name.len() > 255 {
            return Err(DatabaseError::InvalidArguments(
                "migration name cannot exceed 255 characters".into(),
            ));
        }
        if !versions.insert(migration.version) {
            return Err(DatabaseError::InvalidArguments(format!(
                "duplicate migration version {}",
                migration.version
            )));
        }
    }
    Ok(())
}

fn checksum(sql: &str) -> String {
    Sha256::digest(sql.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_versions_accept_plain_and_lossless_i64_json() {
        assert_eq!(decoded_i64(&json!(42)), Some(42));
        assert_eq!(
            decoded_i64(&json!({
                "$type": "i64",
                "value": "9007199254740992"
            })),
            Some(9_007_199_254_740_992)
        );
    }

    #[test]
    fn migration_versions_reject_other_tagged_values() {
        assert_eq!(decoded_i64(&json!({"$type": "u64", "value": "42"})), None);
        assert_eq!(
            decoded_i64(&json!({"$type": "i64", "value": "invalid"})),
            None
        );
    }
}
