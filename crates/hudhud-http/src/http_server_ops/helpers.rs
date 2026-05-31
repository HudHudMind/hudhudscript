use super::{Route, ServerState};
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Cross-platform raw handle: Unix uses RawHandle, Windows uses RawSocket
#[cfg(unix)]
use std::os::unix::io::RawFd as RawHandle;
#[cfg(windows)]
use std::os::windows::io::RawSocket as RawHandle;

pub(crate) fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub(crate) fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

pub(crate) fn server_registry() -> &'static Mutex<HashMap<RawHandle, Arc<Mutex<ServerState>>>> {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<Mutex<HashMap<RawHandle, Arc<Mutex<ServerState>>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

// ── Value helpers ───────────────────────────────────────────────────────

pub(crate) fn require_str(args: &[Value16], idx: usize, name: &str) -> HudHudResult<String> {
    args.get(idx)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| runtime_error(format!("{}: expected string at index {}", name, idx)))
}

pub(crate) fn build_route_obj(method: &str, path: &str, handler: &str) -> Value16 {
    let mut route = HashMap::new();
    route.insert("method".to_string(), Value16::string(method.to_string()));
    route.insert("path".to_string(), Value16::string(path.to_string()));
    route.insert("handler".to_string(), Value16::string(handler.to_string()));
    Value16::object(route)
}

pub(crate) fn extract_server_fd(args: &[Value16], callee: &str) -> HudHudResult<RawHandle> {
    match args.first() {
        Some(v) => {
            if let Some(obj) = v.as_object() {
                obj.get("fd")
                    .and_then(|x| x.as_number())
                    .map(|n| n as RawHandle)
                    .ok_or_else(|| type_error("HttpServer", "object without fd", callee))
            } else {
                Err(type_error("HttpServer", v.type_name_str(), callee))
            }
        }
        None => Err(runtime_error(format!(
            "{}: expected server object as first argument",
            callee
        ))),
    }
}

// ── Public (generic) entry points ───────────────────────────────────────

pub(crate) fn guess_content_type(ext: &str) -> &'static str {
    match ext {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => "application/octet-stream",
    }
}

pub(crate) fn find_matching_route<'a>(
    routes: &'a [Route],
    method: &str,
    path: &str,
) -> Option<&'a Route> {
    for route in routes {
        if route.method != method && route.method != "*" {
            continue;
        }
        if route_matches(&route.pattern, path) {
            return Some(route);
        }
    }
    None
}

pub(crate) fn route_matches(pattern: &str, path: &str) -> bool {
    let pattern_segments: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let path_segments: Vec<&str> = path.trim_matches('/').split('/').collect();

    if let Some(last) = pattern_segments.last() {
        if *last == "*" {
            let prefix = &pattern_segments[..pattern_segments.len() - 1];
            if path_segments.len() < prefix.len() {
                return false;
            }
            return prefix
                .iter()
                .zip(path_segments.iter())
                .all(|(p, s)| p.starts_with(':') || *p == *s);
        }
    }

    if pattern_segments.len() != path_segments.len() {
        return false;
    }

    pattern_segments
        .iter()
        .zip(path_segments.iter())
        .all(|(p, s)| p.starts_with(':') || *p == *s)
}

pub(crate) fn extract_path_params(pattern: &str, path: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let pattern_segments: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let path_segments: Vec<&str> = path.trim_matches('/').split('/').collect();

    for (p, s) in pattern_segments.iter().zip(path_segments.iter()) {
        if let Some(name) = p.strip_prefix(':') {
            params.insert(name.to_string(), s.to_string());
        }
    }
    params
}
