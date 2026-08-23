use std::str::FromStr;

use base64::Engine;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value;

use super::DatabaseError;

#[derive(Debug, Clone)]
pub(crate) enum NullKind {
    Text,
    I64,
    F64,
    Bool,
    Bytes,
    Json,
}

#[derive(Debug, Clone)]
pub(crate) enum BoundValue {
    Null(NullKind),
    Bool(bool),
    I64(i64),
    F64(f64),
    Text(String),
    Bytes(Vec<u8>),
    Json(Value),
    Uuid(uuid::Uuid),
    Decimal(BigDecimal),
    Date(NaiveDate),
    Time(NaiveTime),
    DateTime(NaiveDateTime),
    TimestampTz(DateTime<Utc>),
}

pub(crate) fn parse_params(values: &[Value]) -> Result<Vec<BoundValue>, DatabaseError> {
    values.iter().map(parse_param).collect()
}

fn parse_param(value: &Value) -> Result<BoundValue, DatabaseError> {
    match value {
        Value::Null => Ok(BoundValue::Null(NullKind::Text)),
        Value::Bool(v) => Ok(BoundValue::Bool(*v)),
        Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                Ok(BoundValue::I64(i))
            } else if let Some(u) = v.as_u64() {
                i64::try_from(u)
                    .map(BoundValue::I64)
                    .map_err(|_| invalid("unsigned integer exceeds SQL i64 range"))
            } else {
                v.as_f64()
                    .map(BoundValue::F64)
                    .ok_or_else(|| invalid("invalid number"))
            }
        }
        Value::String(v) => Ok(BoundValue::Text(v.clone())),
        Value::Array(_) => Ok(BoundValue::Json(value.clone())),
        Value::Object(map) => {
            let Some(kind) = map.get("$type").and_then(Value::as_str) else {
                return Ok(BoundValue::Json(value.clone()));
            };
            let inner = map.get("value").unwrap_or(&Value::Null);
            parse_typed(kind, inner, map.get("as").and_then(Value::as_str))
        }
    }
}

fn parse_typed(
    kind: &str,
    value: &Value,
    null_as: Option<&str>,
) -> Result<BoundValue, DatabaseError> {
    let text = || {
        value
            .as_str()
            .ok_or_else(|| invalid(format!("typed {kind} parameter requires a string value")))
    };
    match kind.to_ascii_lowercase().as_str() {
        "null" => Ok(BoundValue::Null(match null_as.unwrap_or("text") {
            "text" | "string" => NullKind::Text,
            "i64" | "int" | "integer" => NullKind::I64,
            "f64" | "float" | "number" => NullKind::F64,
            "bool" | "boolean" => NullKind::Bool,
            "bytes" | "blob" => NullKind::Bytes,
            "json" => NullKind::Json,
            other => return Err(invalid(format!("unsupported null type '{other}'"))),
        })),
        "text" | "string" => Ok(BoundValue::Text(text()?.to_string())),
        "i64" | "int" | "integer" => value
            .as_i64()
            .map(BoundValue::I64)
            .ok_or_else(|| invalid("typed i64 parameter requires an integer value")),
        "f64" | "float" | "number" => value
            .as_f64()
            .map(BoundValue::F64)
            .ok_or_else(|| invalid("typed f64 parameter requires a number value")),
        "bool" | "boolean" => value
            .as_bool()
            .map(BoundValue::Bool)
            .ok_or_else(|| invalid("typed bool parameter requires a boolean value")),
        "json" => Ok(BoundValue::Json(value.clone())),
        "uuid" => uuid::Uuid::parse_str(text()?)
            .map(BoundValue::Uuid)
            .map_err(|e| invalid(e.to_string())),
        "decimal" => BigDecimal::from_str(text()?)
            .map(BoundValue::Decimal)
            .map_err(|e| invalid(e.to_string())),
        "bytes" | "blob" => base64::engine::general_purpose::STANDARD
            .decode(text()?)
            .map(BoundValue::Bytes)
            .map_err(|e| invalid(format!("invalid base64: {e}"))),
        "date" => NaiveDate::parse_from_str(text()?, "%Y-%m-%d")
            .map(BoundValue::Date)
            .map_err(|e| invalid(e.to_string())),
        "time" => NaiveTime::parse_from_str(text()?, "%H:%M:%S%.f")
            .map(BoundValue::Time)
            .map_err(|e| invalid(e.to_string())),
        "datetime" | "timestamp" => parse_datetime(text()?).map(BoundValue::DateTime),
        "timestamptz" => DateTime::parse_from_rfc3339(text()?)
            .map(|v| BoundValue::TimestampTz(v.with_timezone(&Utc)))
            .map_err(|e| invalid(e.to_string())),
        other => Err(invalid(format!("unsupported parameter type '{other}'"))),
    }
}

fn parse_datetime(value: &str) -> Result<NaiveDateTime, DatabaseError> {
    DateTime::parse_from_rfc3339(value)
        .map(|v| v.naive_utc())
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f"))
        .map_err(|e| invalid(e.to_string()))
}

fn invalid(message: impl Into<String>) -> DatabaseError {
    DatabaseError::InvalidArguments(message.into())
}
