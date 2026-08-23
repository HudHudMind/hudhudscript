use base64::Engine;
use serde_json::{json, Value};

use super::DatabaseError;

pub(crate) const MAX_SAFE_JSON_INTEGER: i64 = 9_007_199_254_740_991;

pub(crate) fn i64_json(value: i64) -> Value {
    if (-MAX_SAFE_JSON_INTEGER..=MAX_SAFE_JSON_INTEGER).contains(&value) {
        json!(value)
    } else {
        json!({ "$type": "i64", "value": value.to_string() })
    }
}

pub(crate) fn u64_json(value: u64) -> Value {
    if value <= MAX_SAFE_JSON_INTEGER as u64 {
        json!(value)
    } else {
        json!({ "$type": "u64", "value": value.to_string() })
    }
}

pub(crate) fn bytes_json(value: &[u8]) -> Value {
    json!({
        "$type": "bytes",
        "base64": base64::engine::general_purpose::STANDARD.encode(value)
    })
}

pub(crate) fn query_error(error: sqlx::Error) -> DatabaseError {
    DatabaseError::QueryFailed(safe_sqlx_error(error))
}

pub(crate) fn connection_error(error: sqlx::Error) -> DatabaseError {
    DatabaseError::ConnectionFailed(safe_sqlx_error(error))
}

fn safe_sqlx_error(error: sqlx::Error) -> String {
    match error {
        sqlx::Error::Database(db) => {
            let code = db.code().map(|v| v.into_owned());
            match code {
                Some(code) => format!("database rejected operation (code {code})"),
                None => "database rejected operation".into(),
            }
        }
        sqlx::Error::Configuration(_) => "invalid database connection configuration".into(),
        sqlx::Error::Io(io) => format!("database network I/O error: {io}"),
        sqlx::Error::Tls(_) => "database TLS negotiation failed".into(),
        sqlx::Error::PoolTimedOut => "database pool acquisition timed out".into(),
        sqlx::Error::PoolClosed => "database pool is closed".into(),
        sqlx::Error::RowNotFound => "database row was not found".into(),
        sqlx::Error::ColumnNotFound(_) => "database result column was not found".into(),
        sqlx::Error::ColumnIndexOutOfBounds { .. } => {
            "database result column index was out of bounds".into()
        }
        sqlx::Error::ColumnDecode { .. } => "database result column could not be decoded".into(),
        sqlx::Error::Decode(_) => "database value could not be decoded".into(),
        _ => "database operation failed".into(),
    }
}

pub(crate) fn unsupported_type(backend: &str, name: &str) -> DatabaseError {
    DatabaseError::QueryFailed(format!(
        "{backend} column type '{name}' is not supported by dynamic decoding; cast it to text or JSON"
    ))
}
