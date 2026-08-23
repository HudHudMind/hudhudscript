use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use uuid::Uuid;

use super::params::parse_params;
use super::service::{manager, DatabaseService, TransactionEntry, MAX_OPEN_TRANSACTIONS};
use super::service_support::{
    ensure_live, read, rollback_on_timeout, validate_sql, with_timeout, write,
};
use super::{DatabaseError, DatabaseTransaction, ExecuteOptions, QueryResult, TransactionOptions};

impl DatabaseService {
    pub async fn begin(
        &self,
        handle: &str,
        mut options: TransactionOptions,
    ) -> Result<DatabaseTransaction, DatabaseError> {
        let entry = self.pool(handle)?;
        if read(&manager().transactions)?.len() >= MAX_OPEN_TRANSACTIONS {
            return Err(DatabaseError::PoolLimit(format!(
                "at most {MAX_OPEN_TRANSACTIONS} transactions may be open"
            )));
        }
        if entry.config.read_only {
            options.read_only = true;
        }
        let timeout_ms = options
            .timeout_ms
            .unwrap_or(entry.config.transaction_timeout_ms)
            .min(entry.config.transaction_timeout_ms);
        if timeout_ms == 0 {
            return Err(DatabaseError::InvalidArguments(
                "transaction timeout must be greater than zero".into(),
            ));
        }
        let connection = with_timeout(
            entry.config.acquire_timeout_ms,
            "begin transaction",
            entry.pool.begin(&options),
        )
        .await?;
        let transaction = Uuid::new_v4().to_string();
        let backend = entry.pool.backend();
        let state = TransactionEntry {
            connection_handle: handle.to_string(),
            deadline: Instant::now() + Duration::from_millis(timeout_ms),
            timeout_ms: entry.config.query_timeout_ms,
            max_rows: entry.config.max_rows,
            read_only: options.read_only,
            connection: Mutex::new(Some(connection)),
        };
        let inserted = {
            let mut transactions = write(&manager().transactions)?;
            if transactions.len() >= MAX_OPEN_TRANSACTIONS {
                false
            } else {
                transactions.insert(transaction.clone(), Arc::new(state));
                true
            }
        };
        if !inserted {
            return Err(DatabaseError::PoolLimit(format!(
                "at most {MAX_OPEN_TRANSACTIONS} transactions may be open"
            )));
        }
        schedule_expiration(transaction.clone(), timeout_ms);
        Ok(DatabaseTransaction {
            transaction,
            backend,
        })
    }

    pub async fn transaction_query(
        &self,
        transaction: &str,
        sql: &str,
        params: &[serde_json::Value],
        options: ExecuteOptions,
    ) -> Result<QueryResult, DatabaseError> {
        validate_sql(sql)?;
        let entry = self.transaction(transaction)?;
        ensure_live(transaction, &entry).await?;
        let params = parse_params(params)?;
        let (timeout_ms, max_rows) = bounded_transaction_options(&entry, &options)?;
        let operation = async {
            let mut guard = entry.connection.lock().await;
            let connection = guard
                .as_mut()
                .ok_or_else(|| DatabaseError::TransactionClosed(transaction.into()))?;
            connection.query(sql, &params, max_rows).await
        };
        let result = with_timeout(timeout_ms, "transaction query", operation).await;
        rollback_on_timeout(self, transaction, result).await
    }

    pub async fn transaction_execute(
        &self,
        transaction: &str,
        sql: &str,
        params: &[serde_json::Value],
        options: ExecuteOptions,
    ) -> Result<QueryResult, DatabaseError> {
        validate_sql(sql)?;
        let entry = self.transaction(transaction)?;
        ensure_live(transaction, &entry).await?;
        if entry.read_only {
            return Err(DatabaseError::QueryFailed(
                "transaction is read-only".into(),
            ));
        }
        let params = parse_params(params)?;
        let (timeout_ms, _) = bounded_transaction_options(&entry, &options)?;
        let operation = async {
            let mut guard = entry.connection.lock().await;
            let connection = guard
                .as_mut()
                .ok_or_else(|| DatabaseError::TransactionClosed(transaction.into()))?;
            connection.execute(sql, &params).await
        };
        let result = with_timeout(timeout_ms, "transaction execute", operation).await;
        rollback_on_timeout(self, transaction, result).await
    }

    pub async fn commit(&self, transaction: &str) -> Result<(), DatabaseError> {
        self.finish_transaction(transaction, true).await
    }

    pub async fn rollback(&self, transaction: &str) -> Result<(), DatabaseError> {
        self.finish_transaction(transaction, false).await
    }

    pub(crate) async fn transaction_execute_batch(
        &self,
        transaction: &str,
        sql: &str,
    ) -> Result<(), DatabaseError> {
        validate_sql(sql)?;
        let entry = self.transaction(transaction)?;
        ensure_live(transaction, &entry).await?;
        if entry.read_only {
            return Err(DatabaseError::QueryFailed(
                "transaction is read-only".into(),
            ));
        }
        let operation = async {
            let mut guard = entry.connection.lock().await;
            let connection = guard
                .as_mut()
                .ok_or_else(|| DatabaseError::TransactionClosed(transaction.into()))?;
            connection.execute_batch(sql).await
        };
        let result = with_timeout(entry.timeout_ms, "transaction batch", operation).await;
        rollback_on_timeout(self, transaction, result).await
    }

    pub(crate) async fn transaction_query_all(
        &self,
        transaction: &str,
        sql: &str,
        params: &[serde_json::Value],
    ) -> Result<QueryResult, DatabaseError> {
        let entry = self.transaction(transaction)?;
        ensure_live(transaction, &entry).await?;
        let params = parse_params(params)?;
        let operation = async {
            let mut guard = entry.connection.lock().await;
            let connection = guard
                .as_mut()
                .ok_or_else(|| DatabaseError::TransactionClosed(transaction.into()))?;
            connection.query(sql, &params, 1_000_000).await
        };
        let result = with_timeout(entry.timeout_ms, "internal transaction query", operation).await;
        let result = rollback_on_timeout(self, transaction, result).await?;
        if result.truncated {
            return Err(DatabaseError::PoolLimit(
                "internal query exceeded 1000000 rows".into(),
            ));
        }
        Ok(result)
    }

    pub(super) async fn finish_transaction(
        &self,
        transaction: &str,
        commit: bool,
    ) -> Result<(), DatabaseError> {
        let entry = write(&manager().transactions)?
            .remove(transaction)
            .ok_or_else(|| DatabaseError::TransactionClosed(transaction.into()))?;
        let operation = async {
            let mut guard = entry.connection.lock().await;
            let connection = guard
                .take()
                .ok_or_else(|| DatabaseError::TransactionClosed(transaction.into()))?;
            connection.finish(commit).await
        };
        with_timeout(
            entry.timeout_ms,
            if commit { "commit" } else { "rollback" },
            operation,
        )
        .await
    }
}

fn bounded_transaction_options(
    entry: &TransactionEntry,
    options: &ExecuteOptions,
) -> Result<(u64, usize), DatabaseError> {
    let timeout_ms = options
        .timeout_ms
        .unwrap_or(entry.timeout_ms)
        .min(entry.timeout_ms);
    let max_rows = options
        .max_rows
        .unwrap_or(entry.max_rows)
        .min(entry.max_rows);
    if timeout_ms == 0 || max_rows == 0 {
        return Err(DatabaseError::InvalidArguments(
            "timeout_ms and max_rows must be greater than zero".into(),
        ));
    }
    Ok((timeout_ms, max_rows))
}

fn schedule_expiration(transaction: String, timeout_ms: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
        let _ = DatabaseService
            .finish_transaction(&transaction, false)
            .await;
    });
}
