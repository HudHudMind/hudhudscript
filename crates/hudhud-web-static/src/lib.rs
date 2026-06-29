//! HudHud Web Static — static file serving.
//!
//! Reuses `hudhud-http::guess_content_type` and `write_response` (Kural 7).

use hudhud_http::http_server_ops::{guess_content_type, write_response};
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn not_found_response() -> Value16 {
    let mut map = hudhudscript_bytecode::ObjMap::default();
    map.insert("status".to_string(), Value16::number(404.0));
    map.insert("body".to_string(), Value16::string("Not Found"));
    map.insert("content_type".to_string(), Value16::string("text/plain"));
    map.insert("headers".to_string(), Value16::object(hudhudscript_bytecode::ObjMap::default()));
    map.insert("cookies".to_string(), Value16::array(vec![]));
    Value16::object(map)
}

/// `Web.static(request, directory, rel_path?)` → serves a static file from `directory`.
///
/// If `rel_path` is provided, uses it directly (from `route_params("*")`).
/// Otherwise extracts the path from the request object.
pub fn serve_static(args: &[Value16]) -> HudHudResult<Value16> {
    let req = args.first().and_then(|v| v.as_object()).ok_or_else(|| {
        Error::new(ErrorCode::RuntimeTypeError, "Web.static: expected request object".to_string())
    })?;
    let directory = args.get(1).and_then(|v| v.as_str()).ok_or_else(|| {
        Error::new(ErrorCode::RuntimeTypeError, "Web.static: expected directory string".to_string())
    })?;

    // Use explicit rel_path if provided, otherwise extract from request path
    let relative = if let Some(rp) = args.get(2).and_then(|v| v.as_str()) {
        rp.trim_start_matches('/').to_string()
    } else {
        req.get("path").and_then(|v| v.as_str()).unwrap_or("/")
            .trim_start_matches('/').to_string()
    };

    if relative.is_empty() {
        return Ok(not_found_response());
    }

    let file_path = std::path::Path::new(directory).join(&relative);

    // Canonicalize to prevent directory traversal
    let canonical = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return Ok(not_found_response()),
    };

    // Ensure we're still under the directory
    let dir_canonical = std::path::Path::new(directory).canonicalize().unwrap_or_default();
    if !canonical.starts_with(&dir_canonical) || !canonical.is_file() {
        return Ok(not_found_response());
    }

    let content_type = guess_content_type(
        canonical.extension().and_then(|e| e.to_str()).unwrap_or(""),
    );

    match std::fs::read(&canonical) {
        Ok(data) => {
            let body = String::from_utf8_lossy(&data).to_string();
            let mut map = hudhudscript_bytecode::ObjMap::default();
            map.insert("status".to_string(), Value16::number(200.0));
            map.insert("body".to_string(), Value16::string(body));
            map.insert("content_type".to_string(), Value16::string(content_type));
            map.insert("headers".to_string(), Value16::object(hudhudscript_bytecode::ObjMap::default()));
            map.insert("cookies".to_string(), Value16::array(vec![]));
            Ok(Value16::object(map))
        }
        Err(_) => {
            let mut map = hudhudscript_bytecode::ObjMap::default();
            map.insert("status".to_string(), Value16::number(500.0));
            map.insert("body".to_string(), Value16::string("Internal Server Error"));
            map.insert("content_type".to_string(), Value16::string("text/plain"));
            map.insert("headers".to_string(), Value16::object(hudhudscript_bytecode::ObjMap::default()));
            map.insert("cookies".to_string(), Value16::array(vec![]));
            Ok(Value16::object(map))
        }
    }
}
