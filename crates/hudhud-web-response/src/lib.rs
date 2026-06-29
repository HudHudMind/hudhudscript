//! HudHud Web Response — response builders (Flask response equivalent).
//!
//! Each function produces a normalized response Value16 object:
//! `{status, body, content_type, headers:{}, cookies:[]}`

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

/// Build a normalized response object.
fn make_response(status: u16, body: &str, content_type: &str) -> Value16 {
    let mut headers: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
    let mut cookies: Vec<Value16> = Vec::new();
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("status".to_string(), Value16::number(status as f64));
    obj.insert("body".to_string(), Value16::string(body.to_string()));
    obj.insert(
        "content_type".to_string(),
        Value16::string(content_type.to_string()),
    );
    obj.insert("headers".to_string(), Value16::object(headers));
    obj.insert("cookies".to_string(), Value16::array(cookies));
    Value16::object(obj)
}

// ── Public API ─────────────────────────────────────────────────────────

/// `Web.html(body, status?)` → HTML response.
/// - `body`: string HTML content
/// - `status` (optional): HTTP status code, default 200
pub fn html(args: &[Value16]) -> HudHudResult<Value16> {
    let body = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| type_error("string", "", "Web.html"))?;
    let status = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n as u16)
        .unwrap_or(200);
    Ok(make_response(status, body, "text/html; charset=utf-8"))
}

/// `Web.json(value, status?)` → JSON response.
/// - `value`: any Value16 (object, array, number, string)
/// - `status` (optional): HTTP status code, default 200
pub fn json(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("Web.json: expected value argument"))?;
    let json_str = hudhud_http::json::value_to_json_string(val);
    let status = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n as u16)
        .unwrap_or(200);
    Ok(make_response(status, &json_str, "application/json"))
}

/// `Web.redirect(location, status?)` → redirect response.
/// - `location`: URL to redirect to
/// - `status` (optional): default 302
pub fn redirect(args: &[Value16]) -> HudHudResult<Value16> {
    let location = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| type_error("string", "", "Web.redirect"))?;
    let status = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n as u16)
        .unwrap_or(302);

    let mut resp = make_response(status, "", "text/plain");
    if let Some(obj) = resp.as_object_mut() {
        if let Some(headers_val) = obj.get_mut("headers") {
            if let Some(headers) = headers_val.as_object_mut() {
                headers.insert(
                    "Location".to_string(),
                    Value16::string(location.to_string()),
                );
            }
        }
    }
    Ok(resp)
}

/// `Web.set_cookie(resp, name, value, opts?)` → set cookie on response.
///
/// Returns the modified response object with a `Set-Cookie` header added.
/// `opts` may contain: `path`, `httponly`, `secure`, `max_age`, `samesite`.
pub fn set_cookie(args: &[Value16]) -> HudHudResult<Value16> {
    let resp = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| type_error("object", "", "Web.set_cookie:arg0"))?;

    let name = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| type_error("string", "", "Web.set_cookie:arg1"))?;

    let value = args
        .get(2)
        .and_then(|v| v.as_str())
        .ok_or_else(|| type_error("string", "", "Web.set_cookie:arg2"))?;

    let opts = args.get(3).and_then(|v| v.as_object());

    // Build Set-Cookie header value
    let mut cookie_str = format!("{}={}", name, value);

    if let Some(opts) = opts {
        if let Some(path) = opts.get("path").and_then(|v| v.as_str()) {
            cookie_str.push_str(&format!("; Path={}", path));
        }
        if let Some(max_age) = opts.get("max_age").and_then(|v| v.as_number()) {
            cookie_str.push_str(&format!("; Max-Age={}", max_age as i64));
        }
        if let Some(samesite) = opts.get("samesite").and_then(|v| v.as_str()) {
            cookie_str.push_str(&format!("; SameSite={}", samesite));
        }
        if opts
            .get("httponly")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            cookie_str.push_str("; HttpOnly");
        }
        if opts
            .get("secure")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            cookie_str.push_str("; Secure");
        }
    }

    // Clone the response and add cookie
    let mut new_cookies: Vec<Value16> = Vec::new();
    if let Some(existing) = resp.get("cookies").and_then(|v| v.as_array()) {
        for c in existing {
            new_cookies.push(c.clone());
        }
    }
    new_cookies.push(Value16::string(cookie_str));

    let mut new_obj: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
    for (k, v) in resp.iter() {
        if k != "cookies" {
            new_obj.insert(k.clone(), v.clone());
        }
    }
    new_obj.insert("cookies".to_string(), Value16::array(new_cookies));

    Ok(Value16::object(new_obj))
}

// ── Unit tests ─────────────────────────────────────────────────────────

