//! Web.serve — opens a TCP socket with optional SO_REUSEPORT, binds, listens.
//! Returns a server handle `{id, host, port}`.

use super::registry::listener_registry;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_LISTENER_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}
fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(ErrorCode::RuntimeTypeError, format!("{}: expected {}, got {}", context, expected, got))
}

/// `Web.serve({host, port, reuse_port})` → `{id, host, port}`
pub fn serve(args: &[Value16]) -> HudHudResult<Value16> {
    let opts = args.first().and_then(|v| v.as_object()).ok_or_else(||
        type_error("object", "", "Web.serve"))?;

    let host = opts.get("host").and_then(|v| v.as_str()).unwrap_or("127.0.0.1");
    let port = opts.get("port").and_then(|v| v.as_number()).map(|n| n as u16).unwrap_or(8080);
    let reuse_port = opts.get("reuse_port").and_then(|v| v.as_bool()).unwrap_or(false);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().map_err(|e|
        runtime_error(format!("Web.serve: invalid address {}:{}: {}", host, port, e)))?;

    let socket = if reuse_port {
        let domain = if addr.is_ipv4() { Domain::IPV4 } else { Domain::IPV6 };
        let sock = Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).map_err(|e|
            runtime_error(format!("Web.serve: socket: {}", e)))?;
        sock.set_reuse_address(true).map_err(|e|
            runtime_error(format!("Web.serve: reuse_address: {}", e)))?;
        // SO_REUSEPORT is Unix-only; Windows uses SO_REUSEADDR above.
        #[cfg(unix)]
        sock.set_reuse_port(true).map_err(|e|
            runtime_error(format!("Web.serve: reuse_port: {}", e)))?;
        sock.set_nonblocking(false).map_err(|e|
            runtime_error(format!("Web.serve: nonblocking: {}", e)))?;
        sock.bind(&addr.into()).map_err(|e|
            runtime_error(format!("Web.serve: bind {}: {}", addr, e)))?;
        sock.listen(1024).map_err(|e|
            runtime_error(format!("Web.serve: listen: {}", e)))?;
        TcpListener::from(sock)
    } else {
        TcpListener::bind(addr).map_err(|e|
            runtime_error(format!("Web.serve: bind {}: {}", addr, e)))?
    };

    let id = NEXT_LISTENER_ID.fetch_add(1, Ordering::SeqCst);
    listener_registry().lock().unwrap().insert(id, socket);

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("id".to_string(), Value16::number(id as f64));
    result.insert("host".to_string(), Value16::string(host.to_string()));
    result.insert("port".to_string(), Value16::number(port as f64));
    Ok(Value16::object(result))
}

pub(crate) fn next_conn_id() -> u64 {
    NEXT_CONN_ID.fetch_add(1, Ordering::SeqCst)
}
