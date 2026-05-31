//! Shared type-conversion builtins — used by both VM and interpreter.

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

/// `len(value)` — returns the length of a string, array, or object.
pub fn shared_len(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("len() requires 1 argument"))?;
    if let Some(s) = val.as_str() {
        Ok(Value16::number(s.chars().count() as f64))
    } else if let Some(arr) = val.as_array() {
        Ok(Value16::number(arr.len() as f64))
    } else if let Some(obj) = val.as_object() {
        Ok(Value16::number(obj.len() as f64))
    } else if let Some(items) = val.as_set() {
        Ok(Value16::number(items.len() as f64))
    } else if let Some(pairs) = val.as_map_pairs() {
        Ok(Value16::number(pairs.len() as f64))
    } else {
        Err(type_error(
            "string, array, object, set, or map",
            val.type_name_str(),
            "len()",
        ))
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

/// `typeof(value)` — returns the type name as a string.
pub fn shared_type_of(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("typeof() requires 1 argument"))?;
    Ok(Value16::string(val.type_name_str().to_string()))
}

/// `toString(value)` — converts a value to its string representation.
pub fn shared_to_string(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("toString() requires 1 argument"))?;
    if let Some(s) = val.as_str() {
        Ok(Value16::string(s.to_string()))
    } else if let Some(n) = val.as_number() {
        Ok(Value16::string(format_number(n)))
    } else if let Some(b) = val.as_bool() {
        Ok(Value16::string(
            if b { "true" } else { "false" }.to_string(),
        ))
    } else if val.is_null() {
        Ok(Value16::string("null".to_string()))
    } else {
        // Array, Object, and other types — use display_string
        Ok(Value16::string(val.display_string()))
    }
}

/// `toNumber(value)` — converts a value to a number.
pub fn shared_to_number(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("toNumber() requires 1 argument"))?;
    if let Some(n) = val.as_number() {
        Ok(Value16::number(n))
    } else if let Some(s) = val.as_str() {
        let n = s
            .parse::<f64>()
            .map_err(|_| runtime_error(format!("Cannot convert '{}' to number", s)))?;
        Ok(Value16::number(n))
    } else if let Some(b) = val.as_bool() {
        Ok(Value16::number(if b { 1.0 } else { 0.0 }))
    } else if val.is_null() {
        Ok(Value16::number(0.0))
    } else {
        Err(runtime_error(format!(
            "Cannot convert {} to number",
            val.type_name_str()
        )))
    }
}

/// `toBoolean(value)` — converts a value to a boolean.
///
/// Falsy: 0, NaN, "", null. Everything else is truthy.
pub fn shared_to_boolean(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("toBoolean() requires 1 argument"))?;
    let b = if val.is_null() {
        false
    } else if let Some(b) = val.as_bool() {
        b
    } else if let Some(n) = val.as_number() {
        n != 0.0 && !n.is_nan()
    } else if let Some(s) = val.as_str() {
        !s.is_empty()
    } else {
        // Arrays, Objects, Functions, etc. are truthy
        true
    };
    Ok(Value16::bool_(b))
}

/// `keys(object)` — returns an array of key strings (sorted).
pub fn shared_keys(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("keys() requires 1 argument"))?;
    if let Some(obj) = val.as_object() {
        let mut ks: Vec<String> = obj.keys().cloned().collect();
        ks.sort();
        let items: Vec<Value16> = ks.into_iter().map(Value16::string).collect();
        Ok(Value16::array(items))
    } else {
        Err(runtime_error("keys() requires an object"))
    }
}

/// `values(object)` — returns an array of values (sorted by key).
pub fn shared_values(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("values() requires 1 argument"))?;
    if let Some(obj) = val.as_object() {
        let mut pairs: Vec<(&String, &Value16)> = obj.iter().collect();
        pairs.sort_by_key(|(k, _)| k.as_str().to_string());
        let items: Vec<Value16> = pairs.into_iter().map(|(_, v)| v.clone()).collect();
        Ok(Value16::array(items))
    } else {
        Err(runtime_error("values() requires an object"))
    }
}
