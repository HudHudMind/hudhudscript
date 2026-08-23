use std::str::FromStr;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use futures::TryStreamExt;
use serde_json::{json, Value};
use sqlx::mysql::{MySqlArguments, MySqlConnectOptions, MySqlPoolOptions, MySqlRow};
use sqlx::query::Query;
use sqlx::{
    Column, ConnectOptions, MySql, MySqlConnection, MySqlPool, Row as _, TypeInfo, ValueRef,
};

use super::codec::{
    bytes_json, connection_error, i64_json, query_error, u64_json, unsupported_type,
};
use super::params::{BoundValue, NullKind};
use super::{DatabaseConfig, DatabaseError, QueryResult, Row};

pub(crate) async fn open(config: &DatabaseConfig) -> Result<MySqlPool, DatabaseError> {
    let options = MySqlConnectOptions::from_str(&config.connection_string)
        .map_err(|_| DatabaseError::InvalidArguments("invalid MySQL URL".into()))?
        .disable_statement_logging();
    let read_only = config.read_only;
    MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_millis(config.acquire_timeout_ms))
        .idle_timeout(Duration::from_millis(config.idle_timeout_ms))
        .max_lifetime(Duration::from_millis(config.max_lifetime_ms))
        .test_before_acquire(config.test_before_acquire)
        .after_connect(move |connection, _| {
            Box::pin(async move {
                if read_only {
                    sqlx::query("SET SESSION TRANSACTION READ ONLY")
                        .execute(&mut *connection)
                        .await?;
                }
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .map_err(connection_error)
}

pub(crate) async fn query_pool(
    pool: &MySqlPool,
    sql: &str,
    params: &[BoundValue],
    max_rows: usize,
) -> Result<QueryResult, DatabaseError> {
    collect(bind(sqlx::query(sql), params).fetch(pool), max_rows).await
}

pub(crate) async fn query_connection(
    connection: &mut MySqlConnection,
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
    pool: &MySqlPool,
    sql: &str,
    params: &[BoundValue],
) -> Result<QueryResult, DatabaseError> {
    let result = bind(sqlx::query(sql), params)
        .execute(pool)
        .await
        .map_err(query_error)?;
    let last = match result.last_insert_id() {
        0 => None,
        value => Some(u64_json(value)),
    };
    Ok(QueryResult::affected(result.rows_affected(), last))
}

pub(crate) async fn execute_connection(
    connection: &mut MySqlConnection,
    sql: &str,
    params: &[BoundValue],
) -> Result<QueryResult, DatabaseError> {
    let result = bind(sqlx::query(sql), params)
        .execute(&mut *connection)
        .await
        .map_err(query_error)?;
    let last = match result.last_insert_id() {
        0 => None,
        value => Some(u64_json(value)),
    };
    Ok(QueryResult::affected(result.rows_affected(), last))
}

fn bind<'q>(
    mut query: Query<'q, MySql, MySqlArguments>,
    params: &[BoundValue],
) -> Query<'q, MySql, MySqlArguments> {
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
            BoundValue::Decimal(v) => query.bind(v.clone()),
            BoundValue::Date(v) => query.bind(*v),
            BoundValue::Time(v) => query.bind(*v),
            BoundValue::DateTime(v) => query.bind(*v),
            BoundValue::TimestampTz(v) => query.bind(v.naive_utc()),
        };
    }
    query
}

async fn collect(
    mut stream: impl futures::TryStream<Ok = MySqlRow, Error = sqlx::Error> + Unpin,
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
            column_types = row
                .columns()
                .iter()
                .map(|c| c.type_info().name().to_string())
                .collect();
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

fn decode_row(row: &MySqlRow) -> Result<Row, DatabaseError> {
    let mut result = Row::new();
    for (index, column) in row.columns().iter().enumerate() {
        let raw = row.try_get_raw(index).map_err(query_error)?;
        let value = if raw.is_null() {
            Value::Null
        } else {
            decode_value(row, index, column.type_info().name())?
        };
        result.insert(column.name().to_string(), value);
    }
    Ok(result)
}

fn decode_value(row: &MySqlRow, index: usize, kind: &str) -> Result<Value, DatabaseError> {
    macro_rules! get {
        ($ty:ty) => {
            row.try_get::<$ty, _>(index).map_err(query_error)?
        };
    }
    Ok(match kind {
        "BOOLEAN" => json!(get!(bool)),
        "TINYINT" => i64_json(get!(i8) as i64),
        "SMALLINT" => i64_json(get!(i16) as i64),
        "INT" | "MEDIUMINT" | "YEAR" => i64_json(get!(i32) as i64),
        "BIGINT" => i64_json(get!(i64)),
        "TINYINT UNSIGNED" => json!(get!(u8)),
        "SMALLINT UNSIGNED" => json!(get!(u16)),
        "INT UNSIGNED" | "MEDIUMINT UNSIGNED" => json!(get!(u32)),
        "BIGINT UNSIGNED" => u64_json(get!(u64)),
        "FLOAT" => json!(get!(f32)),
        "DOUBLE" => json!(get!(f64)),
        "DECIMAL" => json!(get!(BigDecimal).to_string()),
        "JSON" => get!(sqlx::types::Json<Value>).0,
        "DATE" => json!(get!(NaiveDate).to_string()),
        "TIME" => json!(get!(NaiveTime).to_string()),
        "DATETIME" | "TIMESTAMP" => json!(get!(NaiveDateTime).to_string()),
        "BINARY" | "VARBINARY" | "TINYBLOB" | "BLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BIT"
        | "GEOMETRY" => bytes_json(&get!(Vec<u8>)),
        "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" | "ENUM" | "SET" => {
            json!(get!(String))
        }
        other => return Err(unsupported_type("MySQL", other)),
    })
}
