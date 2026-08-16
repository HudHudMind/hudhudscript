//! Web.respond — writes HTTP response to the connection and closes it.

use super::registry::conn_registry;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::io::Write;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}
fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

/// `Web.respond(req_or_conn, response_obj)` → sends response and closes connection.
pub fn respond(args: &[Value16]) -> HudHudResult<Value16> {
    let req = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| type_error("object", "", "Web.respond"))?;
    let resp = args
        .get(1)
        .and_then(|v| v.as_object())
        .ok_or_else(|| type_error("object", "", "Web.respond:response"))?;

    let conn_id = req
        .get("conn_id")
        .and_then(|v| v.as_number())
        .ok_or_else(|| runtime_error("Web.respond: request missing conn_id".to_string()))?
        as u64;

    let status = resp
        .get("status")
        .and_then(|v| v.as_number())
        .unwrap_or(200.0) as u16;
    let body = resp.get("body").and_then(|v| v.as_str()).unwrap_or("");
    let content_type = resp
        .get("content_type")
        .and_then(|v| v.as_str())
        .unwrap_or("text/html; charset=utf-8");

    // Collect additional headers from response
    let mut extra_headers: Vec<(String, String)> = Vec::new();
    if let Some(headers) = resp.get("headers").and_then(|v| v.as_object()) {
        for (k, v) in headers {
            if let Some(vs) = v.as_str() {
                extra_headers.push((k.to_string(), vs.to_string()));
            }
        }
    }
    // Collect Set-Cookie headers
    let mut cookies: Vec<String> = Vec::new();
    if let Some(cookie_arr) = resp.get("cookies").and_then(|v| v.as_array()) {
        for c in cookie_arr {
            if let Some(cs) = c.as_str() {
                cookies.push(cs.to_string());
            }
        }
    }

    // Get the stream and write response
    let mut stream = {
        let mut reg = conn_registry().lock().unwrap();
        reg.remove(&conn_id).ok_or_else(|| {
            runtime_error(format!("Web.respond: connection {} not found", conn_id))
        })?
    };

    write_response_with_cookies(
        &mut stream,
        status,
        content_type,
        body,
        &extra_headers,
        &cookies,
    );

    let _ = stream.flush();
    Ok(Value16::null())
}

/// Write HTTP response with custom headers and Set-Cookie headers.
fn write_response_with_cookies(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
    extra_headers: &[(String, String)],
    cookies: &[String],
) {
    let reason = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    };

    let mut header_block = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n",
        status,
        reason,
        content_type,
        body.len()
    );

    for (k, v) in extra_headers {
        header_block.push_str(&format!("{}: {}\r\n", k, v));
    }
    for cookie in cookies {
        header_block.push_str(&format!("Set-Cookie: {}\r\n", cookie));
    }

    header_block.push_str("Connection: close\r\nServer: hudhud-web\r\n\r\n");

    let _ = stream.write_all(header_block.as_bytes());
    let _ = stream.write_all(body.as_bytes());
}
