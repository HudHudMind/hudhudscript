use sqlx::{MySql, Postgres, Sqlite};

use super::params::BoundValue;
use super::{
    mysql, postgres, sqlite, DatabaseBackend, DatabaseConfig, DatabaseError, PoolStatus,
    QueryResult, TransactionOptions,
};

#[derive(Clone)]
pub(crate) enum ManagedPool {
    Postgres(sqlx::PgPool),
    Mysql(sqlx::MySqlPool),
    Sqlite(sqlx::SqlitePool),
}

pub(crate) enum ManagedTransaction {
    Postgres(sqlx::Transaction<'static, Postgres>),
    Mysql(sqlx::Transaction<'static, MySql>),
    Sqlite(sqlx::Transaction<'static, Sqlite>),
}

impl ManagedPool {
    pub(crate) async fn open(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        match config.backend {
            DatabaseBackend::Postgres => postgres::open(config).await.map(Self::Postgres),
            DatabaseBackend::Mysql => mysql::open(config).await.map(Self::Mysql),
            DatabaseBackend::Sqlite => sqlite::open(config).await.map(Self::Sqlite),
        }
    }

    pub(crate) fn backend(&self) -> DatabaseBackend {
        match self {
            Self::Postgres(_) => DatabaseBackend::Postgres,
            Self::Mysql(_) => DatabaseBackend::Mysql,
            Self::Sqlite(_) => DatabaseBackend::Sqlite,
        }
    }

    pub(crate) fn status(&self) -> PoolStatus {
        match self {
            Self::Postgres(pool) => status(pool, DatabaseBackend::Postgres),
            Self::Mysql(pool) => status(pool, DatabaseBackend::Mysql),
            Self::Sqlite(pool) => status(pool, DatabaseBackend::Sqlite),
        }
    }

    pub(crate) async fn close(&self) {
        match self {
            Self::Postgres(pool) => pool.close().await,
            Self::Mysql(pool) => pool.close().await,
            Self::Sqlite(pool) => pool.close().await,
        }
    }

    pub(crate) async fn begin(
        &self,
        options: &TransactionOptions,
    ) -> Result<ManagedTransaction, DatabaseError> {
        use super::codec::connection_error;
        match self {
            Self::Postgres(pool) => {
                let sql = begin_sql(DatabaseBackend::Postgres, options)?;
                pool.begin_with(sql).await.map(ManagedTransaction::Postgres)
            }
            Self::Mysql(pool) => begin_mysql(pool, options)
                .await
                .map(ManagedTransaction::Mysql),
            Self::Sqlite(pool) => {
                let sql = begin_sql(DatabaseBackend::Sqlite, options)?;
                pool.begin_with(sql).await.map(ManagedTransaction::Sqlite)
            }
        }
        .map_err(connection_error)
    }

    pub(crate) async fn query(
        &self,
        sql: &str,
        params: &[BoundValue],
        max_rows: usize,
    ) -> Result<QueryResult, DatabaseError> {
        match self {
            Self::Postgres(pool) => postgres::query_pool(pool, sql, params, max_rows).await,
            Self::Mysql(pool) => mysql::query_pool(pool, sql, params, max_rows).await,
            Self::Sqlite(pool) => sqlite::query_pool(pool, sql, params, max_rows).await,
        }
    }

    pub(crate) async fn execute(
        &self,
        sql: &str,
        params: &[BoundValue],
    ) -> Result<QueryResult, DatabaseError> {
        match self {
            Self::Postgres(pool) => postgres::execute_pool(pool, sql, params).await,
            Self::Mysql(pool) => mysql::execute_pool(pool, sql, params).await,
            Self::Sqlite(pool) => sqlite::execute_pool(pool, sql, params).await,
        }
    }
}

impl ManagedTransaction {
    pub(crate) async fn query(
        &mut self,
        sql: &str,
        params: &[BoundValue],
        max_rows: usize,
    ) -> Result<QueryResult, DatabaseError> {
        match self {
            Self::Postgres(connection) => {
                postgres::query_connection(&mut **connection, sql, params, max_rows).await
            }
            Self::Mysql(connection) => {
                mysql::query_connection(&mut **connection, sql, params, max_rows).await
            }
            Self::Sqlite(connection) => {
                sqlite::query_connection(&mut **connection, sql, params, max_rows).await
            }
        }
    }

    pub(crate) async fn execute(
        &mut self,
        sql: &str,
        params: &[BoundValue],
    ) -> Result<QueryResult, DatabaseError> {
        match self {
            Self::Postgres(connection) => {
                postgres::execute_connection(&mut **connection, sql, params).await
            }
            Self::Mysql(connection) => {
                mysql::execute_connection(&mut **connection, sql, params).await
            }
            Self::Sqlite(connection) => {
                sqlite::execute_connection(&mut **connection, sql, params).await
            }
        }
    }

    pub(crate) async fn finish(self, commit: bool) -> Result<(), DatabaseError> {
        match self {
            Self::Postgres(transaction) => finish(transaction, commit).await,
            Self::Mysql(transaction) => finish(transaction, commit).await,
            Self::Sqlite(transaction) => finish(transaction, commit).await,
        }
    }

    pub(crate) async fn execute_batch(&mut self, sql: &str) -> Result<(), DatabaseError> {
        use sqlx::Executor;
        match self {
            Self::Postgres(connection) => (&mut **connection)
                .execute(sqlx::raw_sql(sql))
                .await
                .map(|_| ()),
            Self::Mysql(connection) => (&mut **connection)
                .execute(sqlx::raw_sql(sql))
                .await
                .map(|_| ()),
            Self::Sqlite(connection) => (&mut **connection)
                .execute(sqlx::raw_sql(sql))
                .await
                .map(|_| ()),
        }
        .map_err(super::codec::query_error)
    }
}

async fn finish<DB: sqlx::Database>(
    transaction: sqlx::Transaction<'static, DB>,
    commit: bool,
) -> Result<(), DatabaseError> {
    let result = if commit {
        transaction.commit().await
    } else {
        transaction.rollback().await
    };
    result.map_err(super::codec::query_error)
}

async fn begin_mysql(
    pool: &sqlx::MySqlPool,
    options: &TransactionOptions,
) -> Result<sqlx::Transaction<'static, MySql>, sqlx::Error> {
    use std::borrow::Cow;
    let mut connection = pool.acquire().await?;
    if let Some(isolation) = mysql_isolation(options.isolation.as_deref())
        .map_err(|error| sqlx::Error::Protocol(error.to_string()))?
    {
        sqlx::query(&format!("SET TRANSACTION ISOLATION LEVEL {isolation}"))
            .execute(&mut *connection)
            .await?;
    }
    let statement = if options.read_only {
        "START TRANSACTION READ ONLY"
    } else {
        "START TRANSACTION"
    };
    sqlx::Transaction::begin(connection, Some(Cow::Borrowed(statement))).await
}

fn status<DB: sqlx::Database>(pool: &sqlx::Pool<DB>, backend: DatabaseBackend) -> PoolStatus {
    PoolStatus {
        backend,
        size: pool.size(),
        idle: pool.num_idle(),
        closed: pool.is_closed(),
    }
}

fn begin_sql(
    backend: DatabaseBackend,
    options: &TransactionOptions,
) -> Result<String, DatabaseError> {
    match backend {
        DatabaseBackend::Postgres => {
            let isolation = pg_isolation(options.isolation.as_deref())?;
            Ok(format!(
                "BEGIN ISOLATION LEVEL {isolation} {}",
                if options.read_only {
                    "READ ONLY"
                } else {
                    "READ WRITE"
                }
            ))
        }
        DatabaseBackend::Mysql => Ok("START TRANSACTION".into()),
        DatabaseBackend::Sqlite => Ok(match options
            .isolation
            .as_deref()
            .unwrap_or("deferred")
            .to_ascii_lowercase()
            .as_str()
        {
            "deferred" => "BEGIN DEFERRED",
            "immediate" => "BEGIN IMMEDIATE",
            "exclusive" => "BEGIN EXCLUSIVE",
            other => {
                return Err(DatabaseError::InvalidArguments(format!(
                    "invalid SQLite transaction mode '{other}'"
                )))
            }
        }
        .into()),
    }
}

fn pg_isolation(value: Option<&str>) -> Result<&'static str, DatabaseError> {
    match value
        .unwrap_or("read_committed")
        .to_ascii_lowercase()
        .replace(['-', ' '], "_")
        .as_str()
    {
        "read_uncommitted" => Ok("READ UNCOMMITTED"),
        "read_committed" => Ok("READ COMMITTED"),
        "repeatable_read" => Ok("REPEATABLE READ"),
        "serializable" => Ok("SERIALIZABLE"),
        other => Err(DatabaseError::InvalidArguments(format!(
            "invalid transaction isolation '{other}'"
        ))),
    }
}

fn mysql_isolation(value: Option<&str>) -> Result<Option<&'static str>, DatabaseError> {
    value.map(|value| pg_isolation(Some(value))).transpose()
}
