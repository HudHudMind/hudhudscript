use std::sync::RwLock;
use std::time::{Duration, Instant};

use tokio::time::timeout;

use super::service::{manager, DatabaseService, TransactionEntry};
use super::{DatabaseConfig, DatabaseError, ExecuteOptions};

pub(super) fn validate_sql(sql: &str) -> Result<(), DatabaseError> {
    if sql.trim().is_empty() {
        Err(DatabaseError::InvalidArguments(
            "SQL statement is empty".into(),
        ))
    } else {
        Ok(())
    }
}

pub(super) fn bounded_options(
    config: &DatabaseConfig,
    options: &ExecuteOptions,
) -> Result<(u64, usize), DatabaseError> {
    let timeout_ms = options
        .timeout_ms
        .unwrap_or(config.query_timeout_ms)
        .min(config.query_timeout_ms);
    let max_rows = options
        .max_rows
        .unwrap_or(config.max_rows)
        .min(config.max_rows);
    if timeout_ms == 0 || max_rows == 0 {
        return Err(DatabaseError::InvalidArguments(
            "timeout_ms and max_rows must be greater than zero".into(),
        ));
    }
    Ok((timeout_ms, max_rows))
}

pub(super) async fn ensure_live(id: &str, entry: &TransactionEntry) -> Result<(), DatabaseError> {
    if Instant::now() >= entry.deadline {
        let _ = write(&manager().transactions)?.remove(id);
        let operation = async {
            let mut guard = entry.connection.lock().await;
            if let Some(connection) = guard.take() {
                connection.finish(false).await?;
            }
            Ok(())
        };
        let _ = with_timeout(entry.timeout_ms, "expired transaction rollback", operation).await;
        Err(DatabaseError::TransactionClosed(format!("{id} (expired)")))
    } else {
        Ok(())
    }
}

pub(super) async fn with_timeout<T>(
    timeout_ms: u64,
    operation: &str,
    future: impl std::future::Future<Output = Result<T, DatabaseError>>,
) -> Result<T, DatabaseError> {
    timeout(Duration::from_millis(timeout_ms), future)
        .await
        .map_err(|_| DatabaseError::Timeout(format!("{operation} exceeded {timeout_ms} ms")))?
}

pub(super) async fn rollback_on_timeout<T>(
    service: &DatabaseService,
    transaction: &str,
    result: Result<T, DatabaseError>,
) -> Result<T, DatabaseError> {
    if matches!(result, Err(DatabaseError::Timeout(_))) {
        let _ = service.finish_transaction(transaction, false).await;
    }
    result
}

pub(super) fn read<T>(
    lock: &RwLock<T>,
) -> Result<std::sync::RwLockReadGuard<'_, T>, DatabaseError> {
    lock.read()
        .map_err(|_| DatabaseError::ConnectionFailed("database registry lock was poisoned".into()))
}

pub(super) fn write<T>(
    lock: &RwLock<T>,
) -> Result<std::sync::RwLockWriteGuard<'_, T>, DatabaseError> {
    lock.write()
        .map_err(|_| DatabaseError::ConnectionFailed("database registry lock was poisoned".into()))
}
