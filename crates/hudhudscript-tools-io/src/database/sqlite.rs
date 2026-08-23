use std::str::FromStr;
use std::time::Duration;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use futures::TryStreamExt;
use serde_json::{json, Value};
use sqlx::query::Query;
use sqlx::sqlite::{
    SqliteArguments, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow,
    SqliteSynchronous,
};
use sqlx::{
    Column, ConnectOptions, Row as _, Sqlite, SqliteConnection, SqlitePool, TypeInfo, ValueRef,
};

use super::codec::{bytes_json, connection_error, i64_json, query_error, unsupported_type};
use super::params::{BoundValue, NullKind};
use super::{DatabaseConfig, DatabaseError, QueryResult, Row};

pub(crate) async fn open(config: &DatabaseConfig) -> Result<SqlitePool, DatabaseError> {
    let mut options = SqliteConnectOptions::from_str(&config.connection_string)
        .map_err(|_| DatabaseError::InvalidArguments("invalid SQLite URL".into()))?
        .foreign_keys(true)
        .read_only(config.read_only)
        .create_if_missing(config.sqlite_create_if_missing)
        .busy_timeout(Duration::from_millis(config.sqlite_busy_timeout_ms))
        .disable_statement_logging();
    if config.read_only {
        options = options.pragma("query_only", "ON");
    }
    if config.sqlite_wal && !config.connection_string.contains(":memory:") {
        options = options
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
    }
    SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_millis(config.acquire_timeout_ms))
        .idle_timeout(Duration::from_millis(config.idle_timeout_ms))
        .max_lifetime(Duration::from_millis(config.max_lifetime_ms))
        .test_before_acquire(config.test_before_acquire)
        .connect_with(options)
        .await
        .map_err(connection_error)
}

pub(crate) async fn query_pool(
    pool: &SqlitePool,
    sql: &str,
    params: &[BoundValue],
    max_rows: usize,
) -> Result<QueryResult, DatabaseError> {
    collect(bind(sqlx::query(sql), params).fetch(pool), max_rows).await
}

pub(crate) async fn query_connection(
    connection: &mut SqliteConnection,
    sql: &str,
    params: &[BoundValue],
    max_rows: usize,
) -> Result<QueryResult, DatabaseError> {
    collect(
        bind(sqlx::query(sql), params).fetch(&mut *connection),
        max_rows,
    )
    .await
}

pub(crate) async fn execute_pool(
    pool: &SqlitePool,
    sql: &str,
    params: &[BoundValue],
) -> Result<QueryResult, DatabaseError> {
    let result = bind(sqlx::query(sql), params)
        .execute(pool)
        .await
        .map_err(query_error)?;
    let last = match result.last_insert_rowid() {
        0 => None,
        value => Some(i64_json(value)),
    };
    Ok(QueryResult::affected(result.rows_affected(), last))
}

pub(crate) async fn execute_connection(
    connection: &mut SqliteConnection,
    sql: &str,
    params: &[BoundValue],
) -> Result<QueryResult, DatabaseError> {
    let result = bind(sqlx::query(sql), params)
        .execute(&mut *connection)
        .await
        .map_err(query_error)?;
    let last = match result.last_insert_rowid() {
        0 => None,
        value => Some(i64_json(value)),
    };
    Ok(QueryResult::affected(result.rows_affected(), last))
}

fn bind<'q>(
    mut query: Query<'q, Sqlite, SqliteArguments<'q>>,
    params: &[BoundValue],
) -> Query<'q, Sqlite, SqliteArguments<'q>> {
    for value in params {
        query = match value {
            BoundValue::Null(NullKind::Text) => query.bind(Option::<String>::None),
            BoundValue::Null(NullKind::I64) => query.bind(Option::<i64>::None),
            BoundValue::Null(NullKind::F64) => query.bind(Option::<f64>::None),
            BoundValue::Null(NullKind::Bool) => query.bind(Option::<bool>::None),
            BoundValue::Null(NullKind::Bytes) => query.bind(Option::<Vec<u8>>::None),
            BoundValue::Null(NullKind::Json) => {
                query.bind(Option::<sqlx::types::Json<Value>>::None)
            }
            BoundValue::Bool(v) => query.bind(*v),
            BoundValue::I64(v) => query.bind(*v),
            BoundValue::F64(v) => query.bind(*v),
            BoundValue::Text(v) => query.bind(v.clone()),
            BoundValue::Bytes(v) => query.bind(v.clone()),
            BoundValue::Json(v) => query.bind(sqlx::types::Json(v.clone())),
            BoundValue::Uuid(v) => query.bind(*v),
            BoundValue::Decimal(v) => query.bind(v.to_string()),
            BoundValue::Date(v) => query.bind(*v),
            BoundValue::Time(v) => query.bind(*v),
            BoundValue::DateTime(v) => query.bind(*v),
            BoundValue::TimestampTz(v) => query.bind(v.to_rfc3339()),
        };
    }
    query
}

async fn collect(
    mut stream: impl futures::TryStream<Ok = SqliteRow, Error = sqlx::Error> + Unpin,
    max_rows: usize,
) -> Result<QueryResult, DatabaseError> {
    let mut rows = Vec::new();
    let mut columns = Vec::new();
    let mut column_types = Vec::new();
    while rows.len() < max_rows {
        let Some(row) = stream.try_next().await.map_err(query_error)? else {
            break;
        };
        if columns.is_empty() {
            columns = row.columns().iter().map(|c| c.name().to_string()).collect();
            column_types = (0..row.columns().len())
                .map(|index| {
                    row.try_get_raw(index)
                        .map(|value| value.type_info().name().to_string())
                        .map_err(query_error)
                })
                .collect::<Result<Vec<_>, _>>()?;
        }
        rows.push(decode_row(&row)?);
    }
    let truncated = stream.try_next().await.map_err(query_error)?.is_some();
    Ok(QueryResult {
        rows,
        rows_affected: 0,
        columns,
        column_types,
        last_insert_id: None,
        truncated,
    })
}

fn decode_row(row: &SqliteRow) -> Result<Row, DatabaseError> {
    let mut result = Row::new();
    for (index, column) in row.columns().iter().enumerate() {
        let raw = row.try_get_raw(index).map_err(query_error)?;
        let value = if raw.is_null() {
            Value::Null
        } else {
            decode_value(row, index, raw.type_info().name())?
        };
        result.insert(column.name().to_string(), value);
    }
    Ok(result)
}

fn decode_value(row: &SqliteRow, index: usize, kind: &str) -> Result<Value, DatabaseError> {
    macro_rules! get {
        ($ty:ty) => {
            row.try_get::<$ty, _>(index).map_err(query_error)?
        };
    }
    Ok(match kind {
        "NULL" => Value::Null,
        "BOOLEAN" => json!(get!(bool)),
        "INTEGER" => i64_json(get!(i64)),
        "REAL" | "NUMERIC" => json!(get!(f64)),
        "TEXT" => json!(get!(String)),
        "BLOB" => bytes_json(&get!(Vec<u8>)),
        "DATE" => json!(get!(NaiveDate).to_string()),
        "TIME" => json!(get!(NaiveTime).to_string()),
        "DATETIME" => json!(get!(NaiveDateTime).to_string()),
        other => return Err(unsupported_type("SQLite", other)),
    })
}
