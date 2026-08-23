use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Instant;

use super::metadata;
use super::migrations;
use super::params::parse_params;
use super::pool::{ManagedPool, ManagedTransaction};
use super::service_support::{bounded_options, read, validate_sql, with_timeout, write};
use super::{
    DatabaseBackend, DatabaseConfig, DatabaseConnection, DatabaseError, ExecuteOptions, Migration,
    MigrationReport, PoolStatus, QueryResult,
};
use tokio::sync::Mutex;
use uuid::Uuid;

const MAX_OPEN_POOLS: usize = 128;
pub(super) const MAX_OPEN_TRANSACTIONS: usize = 1_024;

pub(super) struct PoolEntry {
    pub(super) pool: ManagedPool,
    pub(super) config: DatabaseConfig,
}

pub(super) struct TransactionEntry {
    pub(super) connection_handle: String,
    pub(super) deadline: Instant,
    pub(super) timeout_ms: u64,
    pub(super) max_rows: usize,
    pub(super) read_only: bool,
    pub(super) connection: Mutex<Option<ManagedTransaction>>,
}

pub(super) struct Manager {
    pools: RwLock<HashMap<String, Arc<PoolEntry>>>,
    pub(super) transactions: RwLock<HashMap<String, Arc<TransactionEntry>>>,
}

pub(super) fn manager() -> &'static Manager {
    static MANAGER: OnceLock<Manager> = OnceLock::new();
    MANAGER.get_or_init(|| Manager {
        pools: RwLock::new(HashMap::new()),
        transactions: RwLock::new(HashMap::new()),
    })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DatabaseService;

impl DatabaseService {
    pub async fn open(&self, config: DatabaseConfig) -> Result<DatabaseConnection, DatabaseError> {
        config.validate()?;
        if read(&manager().pools)?.len() >= MAX_OPEN_POOLS {
            return Err(DatabaseError::PoolLimit(format!(
                "at most {MAX_OPEN_POOLS} pools may be open"
            )));
        }
        let pool = ManagedPool::open(&config).await?;
        let backend = pool.backend();
        let handle = Uuid::new_v4().to_string();
        let entry = Arc::new(PoolEntry { pool, config });
        let at_capacity = {
            let mut pools = write(&manager().pools)?;
            if pools.len() >= MAX_OPEN_POOLS {
                true
            } else {
                let replaced = pools.insert(handle.clone(), Arc::clone(&entry));
                debug_assert!(replaced.is_none());
                false
            }
        };
        if at_capacity {
            entry.pool.close().await;
            return Err(DatabaseError::PoolLimit(format!(
                "at most {MAX_OPEN_POOLS} pools may be open"
            )));
        }
        Ok(DatabaseConnection { handle, backend })
    }

    pub fn backend(&self, handle: &str) -> Result<DatabaseBackend, DatabaseError> {
        Ok(self.pool(handle)?.pool.backend())
    }

    pub fn status(&self, handle: &str) -> Result<PoolStatus, DatabaseError> {
        Ok(self.pool(handle)?.pool.status())
    }

    pub async fn health(&self, handle: &str) -> Result<PoolStatus, DatabaseError> {
        let entry = self.pool(handle)?;
        let sql = match entry.pool.backend() {
            DatabaseBackend::Postgres | DatabaseBackend::Mysql | DatabaseBackend::Sqlite => {
                "SELECT 1 AS health"
            }
        };
        with_timeout(
            entry.config.query_timeout_ms,
            "health check",
            entry.pool.query(sql, &[], 1),
        )
        .await?;
        Ok(entry.pool.status())
    }

    pub async fn query(
        &self,
        handle: &str,
        sql: &str,
        params: &[serde_json::Value],
        options: ExecuteOptions,
    ) -> Result<QueryResult, DatabaseError> {
        validate_sql(sql)?;
        let entry = self.pool(handle)?;
        let params = parse_params(params)?;
        let (timeout_ms, max_rows) = bounded_options(&entry.config, &options)?;
        with_timeout(
            timeout_ms,
            "query",
            entry.pool.query(sql, &params, max_rows),
        )
        .await
    }

    pub async fn execute(
        &self,
        handle: &str,
        sql: &str,
        params: &[serde_json::Value],
        options: ExecuteOptions,
    ) -> Result<QueryResult, DatabaseError> {
        validate_sql(sql)?;
        let entry = self.pool(handle)?;
        if entry.config.read_only {
            return Err(DatabaseError::QueryFailed("connection is read-only".into()));
        }
        let params = parse_params(params)?;
        let (timeout_ms, _) = bounded_options(&entry.config, &options)?;
        with_timeout(timeout_ms, "execute", entry.pool.execute(sql, &params)).await
    }

    pub async fn close(&self, handle: &str) -> Result<(), DatabaseError> {
        let entry = write(&manager().pools)?
            .remove(handle)
            .ok_or_else(|| DatabaseError::HandleNotFound(handle.into()))?;
        let transaction_ids: Vec<String> = read(&manager().transactions)?
            .iter()
            .filter(|(_, tx)| tx.connection_handle == handle)
            .map(|(id, _)| id.clone())
            .collect();
        for id in transaction_ids {
            let _ = self.finish_transaction(&id, false).await;
        }
        entry.pool.close().await;
        Ok(())
    }

    pub async fn list_tables(
        &self,
        handle: &str,
        schema: Option<&str>,
    ) -> Result<Vec<String>, DatabaseError> {
        metadata::list_tables(self, handle, schema).await
    }

    pub(crate) async fn query_metadata(
        &self,
        handle: &str,
        sql: &str,
        params: &[serde_json::Value],
    ) -> Result<QueryResult, DatabaseError> {
        let entry = self.pool(handle)?;
        let params = parse_params(params)?;
        let result = with_timeout(
            entry.config.query_timeout_ms,
            "metadata query",
            entry.pool.query(sql, &params, 100_000),
        )
        .await?;
        if result.truncated {
            return Err(DatabaseError::PoolLimit(
                "metadata query exceeded 100000 rows".into(),
            ));
        }
        Ok(result)
    }

    pub async fn describe_table(
        &self,
        handle: &str,
        table: &str,
        schema: Option<&str>,
    ) -> Result<QueryResult, DatabaseError> {
        metadata::describe_table(self, handle, table, schema).await
    }

    pub async fn migrate(
        &self,
        handle: &str,
        migrations: Vec<Migration>,
    ) -> Result<MigrationReport, DatabaseError> {
        let entry = self.pool(handle)?;
        if entry.config.read_only {
            return Err(DatabaseError::QueryFailed("connection is read-only".into()));
        }
        migrations::migrate(self, handle, migrations).await
    }

    pub(super) fn pool(&self, handle: &str) -> Result<Arc<PoolEntry>, DatabaseError> {
        read(&manager().pools)?
            .get(handle)
            .cloned()
            .ok_or_else(|| DatabaseError::HandleNotFound(handle.into()))
    }

    pub(super) fn transaction(
        &self,
        transaction: &str,
    ) -> Result<Arc<TransactionEntry>, DatabaseError> {
        read(&manager().transactions)?
            .get(transaction)
            .cloned()
            .ok_or_else(|| DatabaseError::TransactionClosed(transaction.into()))
    }
}
