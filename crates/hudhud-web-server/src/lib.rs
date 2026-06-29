//! HudHud Web Server — HTTP server primitives.
//!
//! Provides `Web.serve`, `Web.accept`, `Web.respond`, `Web.route_match`,
//! and `Web.route_params`. Uses socket2 for SO_REUSEPORT support.
//! Reuses `hudhud-http` for parsing and content negotiation (Kural 7).

mod accept;
mod registry;
mod respond;
mod serve;

use hudhud_http::http_server_ops::{extract_path_params, route_matches};
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

// ── Public API ─────────────────────────────────────────────────────────

/// `Web.serve({host, port, reuse_port})` → `{id, host, port}`
pub fn serve(args: &[Value16]) -> HudHudResult<Value16> {
    serve::serve(args)
}

/// `Web.accept(server_obj)` → request object with `conn_id`
pub fn accept(args: &[Value16]) -> HudHudResult<Value16> {
    accept::accept(args)
}

/// `Web.respond(req, response_obj)` → sends response, closes connection
pub fn respond(args: &[Value16]) -> HudHudResult<Value16> {
    respond::respond(args)
}

/// `Web.route_match(pattern, path)` → bool
pub fn route_match(args: &[Value16]) -> HudHudResult<Value16> {
    let pattern = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Web.route_match: expected pattern string".to_string()))?;
    let path = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Web.route_match: expected path string".to_string()))?;
    Ok(Value16::bool_(route_matches(pattern, path)))
}

/// `Web.route_params(pattern, path)` → `{param: value, ...}`
pub fn route_params(args: &[Value16]) -> HudHudResult<Value16> {
    let pattern = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Web.route_params: expected pattern string".to_string()))?;
    let path = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Web.route_params: expected path string".to_string()))?;

    let params = extract_path_params(pattern, path);
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    for (k, v) in params {
        obj.insert(k, Value16::string(v));
    }
    Ok(Value16::object(obj))
}
