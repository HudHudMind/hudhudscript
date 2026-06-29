//! Exception value normalisation and construction helpers.
//!
//! Canonical type: `exception` (lowercase).
//! Turkish alias: `istisna`.

use hudhudscript_bytecode::error::CompileError;
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

/// Build an `exception` object with the canonical field set.
pub fn make_exception(code: &str, title: &str, description: &str, value: Value16) -> Value16 {
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("code".to_string(), Value16::string(code));
    obj.insert("title".to_string(), Value16::string(title));
    obj.insert("description".to_string(), Value16::string(description));
    obj.insert("value".to_string(), value);
    obj.insert("cause".to_string(), Value16::null());
    obj.insert("stack".to_string(), Value16::null());
    obj.insert("__hudhud_exception".to_string(), Value16::bool_(true));
    Value16::object(obj)
}

/// Check if a value is an exception object.
pub fn is_exception(v: &Value16) -> bool {
    v.as_object()
        .and_then(|o| o.get("__hudhud_exception"))
        .and_then(|m| m.as_bool())
        .unwrap_or(false)
}

/// Get a field from an exception object, empty string if missing.
pub fn exception_field_str(v: &Value16, field: &str) -> String {
    v.as_object()
        .and_then(|o| o.get(field))
        .and_then(|f| f.as_string())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// Normalize any thrown value into an exception object.
///
/// - Already an exception → passes through unchanged.
/// - String → wrapped as `exception { code: "E_USER_THROW", title: "exception",
///   description: <string>, value: <string> }`.
/// - Number/bool/null/object/array → wrapped similarly.
pub fn normalize_throw_value(raw: Value16) -> Value16 {
    if is_exception(&raw) {
        return raw;
    }
    let desc = value_to_description(&raw);
    make_exception("E_USER_THROW", "exception", &desc, raw)
}

/// Convert a runtime `CompileError` into an exception object for catch.
pub fn runtime_error_to_exception(err: &CompileError) -> Value16 {
    let code = err.short_code().to_string();
    let title = err.title();
    let desc = format!("{}", err);
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("code".to_string(), Value16::string(&code));
    obj.insert("title".to_string(), Value16::string(title));
    obj.insert("description".to_string(), Value16::string(&desc));
    obj.insert("value".to_string(), Value16::null());
    obj.insert("cause".to_string(), Value16::null());
    obj.insert("stack".to_string(), Value16::null());
    obj.insert("__hudhud_exception".to_string(), Value16::bool_(true));
    Value16::object(obj)
}

fn value_to_description(v: &Value16) -> String {
    if let Some(s) = v.as_string() {
        return s.to_string();
    }
    if let Some(n) = v.as_number() {
        return format!("{}", n);
    }
    if let Some(n) = v.as_int() {
        return format!("{}", n);
    }
    if v.as_bool().is_some() {
        return "bool".to_string();
    }
    if v.is_null() {
        return "null".to_string();
    }
    if v.as_array().is_some() {
        return "[array]".to_string();
    }
    if v.as_object().is_some() {
        return "{object}".to_string();
    }
    "unknown".to_string()
}
