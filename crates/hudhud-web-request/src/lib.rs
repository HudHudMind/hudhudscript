//! HudHud Web Request — full HTTP request parse (Flask `request` equivalent).
//!
//! Takes a `ParsedRequest` from `hudhud-http` and enriches it into a
//! HudHudScript object with parsed query args, form body, JSON body,
//! multipart files, and cookies.

pub mod cookies;
pub mod json_body;
pub mod multipart;
pub mod query;

use hudhud_http::http_server_ops::ParsedRequest;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

/// Parse a `ParsedRequest` into a full HudHudScript request object.
///
/// Returns an object with fields:
/// - `method` (str): HTTP method
/// - `path` (str): URL path
/// - `query` (str): raw query string
/// - `args` (obj): parsed query parameters
/// - `headers` (obj): request headers (lowercase keys)
/// - `body` (str): raw request body
/// - `form` (obj): urlencoded form fields (from body)
/// - `json` (any): parsed JSON body (or null)
/// - `files` (obj): multipart file uploads
/// - `cookies` (obj): parsed cookies
pub fn parse(parsed: &ParsedRequest) -> HudHudResult<Value16> {
    let mut obj = hudhudscript_bytecode::ObjMap::default();

    // Basic fields
    obj.insert("method".to_string(), Value16::string(parsed.method.clone()));
    obj.insert("path".to_string(), Value16::string(parsed.path.clone()));
    obj.insert("query".to_string(), Value16::string(parsed.query.clone()));
    obj.insert("body".to_string(), Value16::string(parsed.body.clone()));

    // Query args
    obj.insert("args".to_string(), query::parse_query_string(&parsed.query));

    // Headers (convert to Value16 object)
    let headers: hudhudscript_bytecode::ObjMap = parsed
        .headers
        .iter()
        .map(|(k, v)| (k.clone(), Value16::string(v.clone())))
        .collect();
    obj.insert("headers".to_string(), Value16::object(headers.clone()));

    // Content-Type determination
    let content_type = parsed
        .headers
        .get("content-type")
        .cloned()
        .unwrap_or_default();

    // Form body (urlencoded)
    if content_type.contains("application/x-www-form-urlencoded") {
        obj.insert("form".to_string(), query::parse_query_string(&parsed.body));
    } else {
        obj.insert(
            "form".to_string(),
            Value16::object(hudhudscript_bytecode::ObjMap::default()),
        );
    }

    // JSON body
    if content_type.contains("application/json") {
        obj.insert(
            "json".to_string(),
            json_body::parse_json_body(&parsed.body)?,
        );
    } else {
        obj.insert("json".to_string(), Value16::null());
    }

    // Multipart files
    if content_type.contains("multipart/form-data") {
        let boundary = extract_boundary(&content_type);
        obj.insert(
            "files".to_string(),
            multipart::parse_multipart(parsed.body.as_bytes(), &boundary),
        );
    } else {
        obj.insert(
            "files".to_string(),
            Value16::object(hudhudscript_bytecode::ObjMap::default()),
        );
    }

    // Cookies
    let cookie_header = parsed.headers.get("cookie").cloned().unwrap_or_default();
    obj.insert(
        "cookies".to_string(),
        cookies::parse_cookies(&cookie_header),
    );

    Ok(Value16::object(obj))
}

/// Extract boundary value from `multipart/form-data; boundary=...` content-type.
fn extract_boundary(content_type: &str) -> String {
    for part in content_type.split(';') {
        let trimmed = part.trim();
        if trimmed.starts_with("boundary=") {
            return trimmed["boundary=".len()..].trim_matches('"').to_string();
        }
    }
    String::new()
}

/// Public entry point — matches the signature expected by the umbrella crate.
pub fn parse_request(_args: &[Value16]) -> HudHudResult<Value16> {
    // This function is called directly from the VM with a ParsedRequest
    // already serialized; the concrete parsing is provided by Web.accept.
    Err(runtime_error(
        "Web request: use Web.accept to get a parsed request",
    ))
}

// ── Unit tests ────────────────────────────────────────────────────────
