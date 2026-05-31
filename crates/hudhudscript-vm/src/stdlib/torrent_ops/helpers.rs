use hudhudscript_bytecode::shared_value::{runtime_error, type_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

pub(crate) fn default_rpc_url() -> String {
    std::env::var("TRANSMISSION_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:9091/transmission/rpc".to_string())
}

pub(crate) fn require_string(args: &[Value16], idx: usize, name: &str) -> SharedResult<String> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| type_error("string", v.type_name_str(), name)),
        None => Err(runtime_error(format!(
            "{}: argument {} required",
            name, idx
        ))),
    }
}

pub(crate) fn optional_string(args: &[Value16], idx: usize) -> Option<String> {
    args.get(idx)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

pub(crate) fn require_i64(args: &[Value16], idx: usize, name: &str) -> SharedResult<i64> {
    match args.get(idx) {
        Some(v) => v
            .as_number()
            .map(|n| n as i64)
            .ok_or_else(|| type_error("number", v.type_name_str(), name)),
        None => Err(runtime_error(format!("{}: torrent id required", name))),
    }
}

pub(crate) fn ok_message(ok: bool, msg: String) -> Value16 {
    let mut m = HashMap::new();
    m.insert("ok".to_string(), Value16::boolean(ok));
    m.insert("message".to_string(), Value16::string(msg));
    Value16::object(m)
}

pub(crate) fn status_string(status: i64) -> &'static str {
    match status {
        0 => "stopped",
        1 => "check_wait",
        2 => "checking",
        3 => "download_wait",
        4 => "downloading",
        5 => "seed_wait",
        6 => "seeding",
        _ => "unknown",
    }
}
