//! Web.accept — blocking accept loop that returns parsed request objects.

use super::registry::{conn_registry, listener_registry};
use super::serve::next_conn_id;
use hudhud_http::http_server_ops::parse_http_request;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}
fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(ErrorCode::RuntimeTypeError, format!("{}: expected {}, got {}", context, expected, got))
}

/// `Web.accept(server_obj)` → request object.
///
/// Blocks until a client connects, parses the HTTP request,
/// and returns a rich request object with `conn_id` for later response.
pub fn accept(args: &[Value16]) -> HudHudResult<Value16> {
    let server = args.first().and_then(|v| v.as_object()).ok_or_else(||
        type_error("object", "", "Web.accept"))?;
    let server_id = server.get("id").and_then(|v| v.as_number()).ok_or_else(||
        runtime_error("Web.accept: server object missing 'id'".to_string()))? as u64;

    // Clone the listener from registry (Rust won't let us hold lock during accept)
    let listener = {
        let reg = listener_registry().lock().unwrap();
        let listener = reg.get(&server_id).ok_or_else(||
            runtime_error(format!("Web.accept: server {} not found", server_id)))?;
        listener.try_clone().map_err(|e|
            runtime_error(format!("Web.accept: try_clone: {}", e)))?
    };

    let (mut stream, peer_addr) = loop {
        match listener.accept() {
            Ok(conn) => break conn,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            Err(e) => return Err(runtime_error(format!("Web.accept: {}", e))),
        }
    };

    // Parse the HTTP request using shared parser (Kural 7)
    let parsed = parse_http_request(&mut stream).map_err(|e|
        runtime_error(format!("Web.accept: parse error: {}", e)))?;

    // Enrich with web-request parser
    let enriched = hudhud_web_request::parse(&parsed).map_err(|e|
        runtime_error(format!("Web.accept: request parse error: {}", e)))?;

    // Store stream in connection registry
    let conn_id = next_conn_id();
    conn_registry().lock().unwrap().insert(conn_id, stream);

    // Add conn_id and peer to the request object
    let mut req_obj: hudhudscript_bytecode::ObjMap = enriched
        .as_object()
        .cloned()
        .unwrap_or_default();
    req_obj.insert("conn_id".to_string(), Value16::number(conn_id as f64));
    req_obj.insert("peer".to_string(), Value16::string(peer_addr.to_string()));

    Ok(Value16::object(req_obj))
}
