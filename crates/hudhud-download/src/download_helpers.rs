use hudhudscript_bytecode::Value16;
use crate::download_ops::{runtime_error, type_error};
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

pub(crate) fn build_client() -> HudHudResult<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| runtime_error(format!("HTTP client error: {}", e)))
}

pub(crate) fn require_str<'a>(args: &'a [Value16], idx: usize, method: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            method, idx
        ))),
    }
}

pub(crate) fn serde_json_to_value16(v: &serde_json::Value) -> Value16 {
    match v {
        serde_json::Value::Null => Value16::null(),
        serde_json::Value::Bool(b) => Value16::bool_(*b),
        serde_json::Value::Number(n) => Value16::number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value16::string(s.clone()),
        serde_json::Value::Array(arr) => {
            Value16::array(arr.iter().map(serde_json_to_value16).collect())
        }
        serde_json::Value::Object(map) => Value16::object(
            map.iter()
                .map(|(k, v)| (k.clone(), serde_json_to_value16(v)))
                .collect(),
        ),
    }
}
