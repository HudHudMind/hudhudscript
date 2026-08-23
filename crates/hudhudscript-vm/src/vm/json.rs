//! JSON helpers — serde_to_value / value_to_json_string only.
//! call_json_method moved to hudhud-http crate.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

/// Convert a serde_json::Value to a SharedValue.
pub fn serde_to_value(v: &serde_json::Value) -> Value16 {
    match v {
        serde_json::Value::Null => Value16::null(),
        serde_json::Value::Bool(b) => Value16::boolean(*b),
        serde_json::Value::Number(n) => Value16::number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value16::string(s.clone()),
        serde_json::Value::Array(arr) => {
            Value16::array(arr.iter().map(|v| serde_to_value(v)).collect())
        }
        serde_json::Value::Object(obj) => {
            let m: hudhudscript_bytecode::ObjMap = obj
                .iter()
                .map(|(k, v)| (k.clone(), serde_to_value(v)))
                .collect();
            Value16::object(m)
        }
    }
}

/// Convert a SharedValue to a JSON string.
pub fn value_to_json_string(value: &Value16) -> String {
    hudhud_http::json::value_to_json_string(value)
}
