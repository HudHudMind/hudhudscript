//! Shared Date/Time builtin — used by both VM and interpreter.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use chrono::{Datelike, NaiveDateTime, TimeZone, Timelike, Utc};
use std::collections::HashMap;

/// Zero-cost enum identifier for every Date method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateMethodId {
    Now,
    ToMillis,
    Parse,
    Format,
    FromMillis,
    FromTimestamp,
    Parts,
    Diff,
    Add,
    Iso,
}

impl std::str::FromStr for DateMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "now" | "timestamp" => Ok(Self::Now),
            "to_millis" => Ok(Self::ToMillis),
            "parse" => Ok(Self::Parse),
            "format" => Ok(Self::Format),
            "from_millis" => Ok(Self::FromMillis),
            "from_timestamp" => Ok(Self::FromTimestamp),
            "parts" => Ok(Self::Parts),
            "diff" => Ok(Self::Diff),
            "add" => Ok(Self::Add),
            "iso" => Ok(Self::Iso),
            _ => Err(runtime_error(format!("Unknown Date method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for Date operations.
pub fn dispatch(method: DateMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        DateMethodId::Now => Ok(Value16::number(Utc::now().timestamp() as f64)),

        DateMethodId::ToMillis => Ok(Value16::int(Utc::now().timestamp_millis())),

        DateMethodId::Parse => {
            let input = args
                .first()
                .and_then(|v| v.as_str())
                .ok_or_else(|| runtime_error("Date.parse: expected string"))?
                .to_string();

            let fmt = args.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());

            let ts = if let Some(fmt) = fmt {
                NaiveDateTime::parse_from_str(&input, &fmt)
                    .map(|dt| dt.and_utc().timestamp() as f64)
                    .map_err(|e| runtime_error(format!("Date.parse error: {}", e)))?
            } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&input) {
                dt.timestamp() as f64
            } else if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(&input) {
                dt.timestamp() as f64
            } else if let Ok(dt) = NaiveDateTime::parse_from_str(&input, "%Y-%m-%d %H:%M:%S") {
                dt.and_utc().timestamp() as f64
            } else if let Ok(dt) = NaiveDateTime::parse_from_str(&input, "%Y-%m-%dT%H:%M:%S") {
                dt.and_utc().timestamp() as f64
            } else if let Ok(date) = chrono::NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
                date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as f64
            } else {
                return Err(runtime_error(format!(
                    "Date.parse: cannot parse '{}'",
                    input
                )));
            };
            Ok(Value16::number(ts))
        }

        DateMethodId::Format => {
            let ts = args
                .first()
                .and_then(|v| v.as_number())
                .ok_or_else(|| runtime_error("Date.format: expected number"))?
                as i64;

            let fmt = args
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("%Y-%m-%d %H:%M:%S");

            let dt = Utc
                .timestamp_opt(ts, 0)
                .single()
                .ok_or_else(|| runtime_error(format!("Date.format: invalid timestamp {}", ts)))?;
            Ok(Value16::string(dt.format(fmt).to_string()))
        }

        DateMethodId::FromMillis => from_millis(args),

        DateMethodId::FromTimestamp => {
            let ts = match args.first() {
                Some(v) => v
                    .as_number()
                    .ok_or_else(|| runtime_error("Date: expected number"))?
                    as i64,
                None => return Err(runtime_error("Date: expected number")),
            };
            build_parts_object(ts)
        }

        DateMethodId::Parts => {
            let ts = match args.first() {
                Some(v) => v
                    .as_number()
                    .ok_or_else(|| runtime_error("Date: expected number"))?
                    as i64,
                None => Utc::now().timestamp(),
            };
            build_parts_object(ts)
        }

        DateMethodId::Diff => {
            let ts1 = args
                .first()
                .and_then(|v| v.as_number())
                .ok_or_else(|| runtime_error("Date.diff: expected number"))?;
            let ts2 = args
                .get(1)
                .and_then(|v| v.as_number())
                .ok_or_else(|| runtime_error("Date.diff: expected second number"))?;
            let diff = ts1 - ts2;
            let unit = args.get(2).and_then(|v| v.as_str()).unwrap_or("seconds");
            let result = match unit {
                "seconds" | "s" => diff,
                "minutes" | "m" => diff / 60.0,
                "hours" | "h" => diff / 3600.0,
                "days" | "d" => diff / 86400.0,
                "millis" | "ms" => diff * 1000.0,
                _ => return Err(runtime_error(format!("Date.diff: unknown unit '{}'", unit))),
            };
            Ok(Value16::number(result))
        }

        DateMethodId::Add => {
            let ts = args
                .first()
                .and_then(|v| v.as_number())
                .ok_or_else(|| runtime_error("Date.add: expected number"))?;
            let amount = args
                .get(1)
                .and_then(|v| v.as_number())
                .ok_or_else(|| runtime_error("Date.add: expected amount"))?;
            let unit = args.get(2).and_then(|v| v.as_str()).unwrap_or("seconds");
            let delta = match unit {
                "seconds" | "s" => amount,
                "minutes" | "m" => amount * 60.0,
                "hours" | "h" => amount * 3600.0,
                "days" | "d" => amount * 86400.0,
                "millis" | "ms" => amount / 1000.0,
                _ => return Err(runtime_error(format!("Date.add: unknown unit '{}'", unit))),
            };
            Ok(Value16::number(ts + delta))
        }

        DateMethodId::Iso => {
            let ts = match args.first() {
                Some(v) => v
                    .as_number()
                    .ok_or_else(|| runtime_error("Date.iso: expected number"))?
                    as i64,
                None => Utc::now().timestamp(),
            };
            let dt = Utc
                .timestamp_opt(ts, 0)
                .single()
                .ok_or_else(|| runtime_error(format!("Date.iso: invalid timestamp {}", ts)))?;
            Ok(Value16::string(dt.to_rfc3339()))
        }
    }
}

/// Backward-compatible string dispatch.

/// Build a date-parts object from a Unix timestamp in **seconds**.
fn build_parts_object(ts: i64) -> HudHudResult<Value16> {
    let dt = Utc
        .timestamp_opt(ts, 0)
        .single()
        .ok_or_else(|| runtime_error(format!("Date: invalid timestamp {}", ts)))?;
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("year".to_string(), Value16::number(dt.year() as f64));
    obj.insert("month".to_string(), Value16::number(dt.month() as f64));
    obj.insert("day".to_string(), Value16::number(dt.day() as f64));
    obj.insert("hour".to_string(), Value16::number(dt.hour() as f64));
    obj.insert("minute".to_string(), Value16::number(dt.minute() as f64));
    obj.insert("second".to_string(), Value16::number(dt.second() as f64));
    obj.insert(
        "weekday".to_string(),
        Value16::number(dt.weekday().num_days_from_monday() as f64),
    );
    obj.insert(
        "timestamp".to_string(),
        Value16::number(dt.timestamp() as f64),
    );
    obj.insert("iso".to_string(), Value16::string(dt.to_rfc3339()));
    Ok(Value16::object(obj))
}

/// `Date.from_millis(ms)` — build a date-parts object from a Unix timestamp
/// expressed in **milliseconds**.
///
/// Mirrors the interpreter-era `builtins::datetime::date_from_millis`
/// semantics byte-for-byte: integer millis are split into seconds + nanos
/// with rounding toward zero, formatted through `chrono::Utc`, and returned
/// as an object with fields `{year, month, day, hour, minute, second,
/// weekday, timestamp, iso}`. `weekday` uses `num_days_from_monday`
/// (Monday = 0). `timestamp` is in **seconds** so it round-trips with
/// `Date.now` / `Date.from_timestamp`, and `iso` is RFC 3339.
pub fn from_millis(args: &[Value16]) -> HudHudResult<Value16> {
    let first = args.first();
    let ms = first
        .and_then(|v| v.as_number().or_else(|| v.as_int().map(|i| i as f64)))
        .ok_or_else(|| {
            runtime_error(format!(
                "Date.from_millis: expected number (got {})",
                first.map(|v| v.type_name_str()).unwrap_or("missing")
            ))
        })? as i64;

    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    let dt = Utc
        .timestamp_opt(secs, nsecs)
        .single()
        .ok_or_else(|| runtime_error(format!("Date.from_millis: invalid millis {}", ms)))?;

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("year".to_string(), Value16::number(dt.year() as f64));
    obj.insert("month".to_string(), Value16::number(dt.month() as f64));
    obj.insert("day".to_string(), Value16::number(dt.day() as f64));
    obj.insert("hour".to_string(), Value16::number(dt.hour() as f64));
    obj.insert("minute".to_string(), Value16::number(dt.minute() as f64));
    obj.insert("second".to_string(), Value16::number(dt.second() as f64));
    obj.insert(
        "weekday".to_string(),
        Value16::number(dt.weekday().num_days_from_monday() as f64),
    );
    obj.insert(
        "timestamp".to_string(),
        Value16::number(dt.timestamp() as f64),
    );
    obj.insert("iso".to_string(), Value16::string(dt.to_rfc3339()));
    Ok(Value16::object(obj))
}
