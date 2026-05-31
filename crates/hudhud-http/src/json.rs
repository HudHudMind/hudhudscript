//! JSON helpers (no builtins dependency).

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonMethodId {
    Parse,
    Stringify,
}

impl std::str::FromStr for JsonMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parse" => Ok(Self::Parse),
            "stringify" => Ok(Self::Stringify),
            _ => Err(runtime_error(format!("Unknown JSON method: {}", s))),
        }
    }
}

impl JsonMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Parse => parse(args),
            Self::Stringify => stringify(args),
        }
    }
}

pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

pub fn parse(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("JSON.parse() requires a string argument"))?;
    let parsed: serde_json::Value =
        serde_json::from_str(s).map_err(|e| runtime_error(format!("JSON.parse error: {}", e)))?;
    Ok(serde_to_value(&parsed))
}

pub fn stringify(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("JSON.stringify() requires an argument"))?;
    let json_str = value_to_json_string(val);
    Ok(Value16::string(json_str))
}

/// Convert a serde_json::Value to a Value16.
pub fn serde_to_value(v: &serde_json::Value) -> Value16 {
    match v {
        serde_json::Value::Null => Value16::null(),
        serde_json::Value::Bool(b) => Value16::bool_(*b),
        serde_json::Value::Number(n) => Value16::number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value16::string(s.clone()),
        serde_json::Value::Array(arr) => {
            Value16::array(arr.iter().map(|v| serde_to_value(v)).collect())
        }
        serde_json::Value::Object(obj) => {
            let map: HashMap<String, Value16> = obj
                .iter()
                .map(|(k, v)| (k.clone(), serde_to_value(v)))
                .collect();
            Value16::object(map)
        }
    }
}

/// Convert a Value16 to a JSON string.
pub fn value_to_json_string(value: &Value16) -> String {
    if value.is_null() {
        return "null".to_string();
    }
    if let Some(b) = value.as_bool() {
        return b.to_string();
    }
    if let Some(n) = value.as_number() {
        return format_number(n);
    }
    if let Some(i) = value.as_int() {
        return format_number(i as f64);
    }
    if let Some(s) = value.as_str() {
        return format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    }
    if let Some(arr) = value.as_array() {
        let items: Vec<String> = arr.iter().map(|v| value_to_json_string(v)).collect();
        return format!("[{}]", items.join(","));
    }
    if let Some(obj) = value.as_object() {
        let mut pairs: Vec<String> = obj
            .iter()
            .filter(|(k, _)| !k.starts_with("__"))
            .map(|(k, v)| format!("\"{}\":{}", k, value_to_json_string(v)))
            .collect();
        pairs.sort();
        return format!("{{{}}}", pairs.join(","));
    }
    // Fallback for non-basic types (Function, Promise, Class, etc.)
    format!("\"{}\"", value.display_string().replace('"', "\\\""))
}
