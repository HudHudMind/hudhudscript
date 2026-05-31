//! Internal helper utilities shared across email sub-modules.

use std::collections::HashMap;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

pub(super) fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub(super) fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

pub(super) fn obj_str(
    obj: &HashMap<String, Value16>,
    key: &str,
    ctx: &str,
) -> HudHudResult<String> {
    match obj.get(key) {
        Some(v) => v
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| type_error("string", v.type_name_str(), &format!("{} {}", ctx, key))),
        None => Err(runtime_error(format!(
            "Missing required field '{}' in config object ({})",
            key, ctx
        ))),
    }
}

pub(super) fn obj_str_opt(obj: &HashMap<String, Value16>, key: &str) -> Option<String> {
    obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

pub(super) fn value_to_json_string(value: &Value16) -> String {
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
    format!("\"{}\"", value.display_string().replace('"', "\\\""))
}
