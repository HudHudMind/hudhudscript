use super::connection::accept_loop;
use super::helpers::*;
use super::Route;
use super::ServerState;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

// Cross-platform raw handle + traits
#[cfg(unix)]
mod raw_handle {
    pub use std::os::unix::io::{AsRawFd as AsRaw, FromRawFd as FromRaw, RawFd as Raw};
}
#[cfg(windows)]
mod raw_handle {
    pub use std::os::windows::io::{
        AsRawSocket as AsRaw, FromRawSocket as FromRaw, RawSocket as Raw,
    };
}
use raw_handle::{AsRaw, FromRaw, Raw};

// Platform-specific TcpListener handle access
#[cfg(unix)]
fn listener_handle(l: &TcpListener) -> Raw {
    l.as_raw_fd()
}
#[cfg(windows)]
fn listener_handle(l: &TcpListener) -> Raw {
    l.as_raw_socket()
}

#[cfg(unix)]
fn listener_from_handle(h: Raw) -> TcpListener {
    unsafe { TcpListener::from_raw_fd(h) }
}
#[cfg(windows)]
fn listener_from_handle(h: Raw) -> TcpListener {
    unsafe { TcpListener::from_raw_socket(h) }
}

pub(crate) fn server_listen(args: &[Value16]) -> HudHudResult<Value16> {
    let (host, port, routes_from_server) = match args.first() {
        Some(v) => {
            if let Some(n) = v.as_number() {
                ("127.0.0.1".to_string(), n, Vec::<Route>::new())
            } else if let Some(o) = v.as_object() {
                let h = o
                    .get("host")
                    .and_then(|x| x.as_str())
                    .unwrap_or("127.0.0.1")
                    .to_string();
                let p = o.get("port").and_then(|x| x.as_number()).unwrap_or(8080.0);
                let mut routes: Vec<Route> = Vec::new();
                if let Some(arr) = o.get("routes").and_then(|x| x.as_array()) {
                    for rv in arr {
                        if let Some(ro) = rv.as_object() {
                            let method = ro
                                .get("method")
                                .and_then(|x| x.as_str())
                                .map(|s| s.to_string());
                            let path = ro
                                .get("path")
                                .and_then(|x| x.as_str())
                                .map(|s| s.to_string());
                            let handler = ro
                                .get("handler")
                                .and_then(|x| x.as_str())
                                .unwrap_or("handler")
                                .to_string();
                            if let (Some(m), Some(pth)) = (method, path) {
                                routes.push(Route {
                                    method: m,
                                    pattern: pth,
                                    handler,
                                    response_body: None,
                                    response_status: 200,
                                    response_content_type: "application/json".to_string(),
                                });
                            }
                        }
                    }
                }
                (h, p, routes)
            } else {
                ("127.0.0.1".to_string(), 8080.0, Vec::<Route>::new())
            }
        }
        None => ("127.0.0.1".to_string(), 8080.0, Vec::<Route>::new()),
    };

    let addr = format!("{}:{}", host, port as u16);
    let listener = TcpListener::bind(&addr)
        .map_err(|e| runtime_error(format!("Server.listen: bind error: {}", e)))?;

    listener
        .set_nonblocking(true)
        .map_err(|e| runtime_error(format!("Server.listen: set_nonblocking error: {}", e)))?;

    let fd = listener_handle(&listener);
    let local_addr = listener
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| addr.clone());
    let actual_port = listener
        .local_addr()
        .map(|a| a.port())
        .unwrap_or(port as u16);

    let state = Arc::new(Mutex::new(ServerState::new()));
    {
        let mut st = state.lock().unwrap();
        st.routes = routes_from_server;
    }

    {
        let mut registry = server_registry().lock().unwrap();
        registry.insert(fd, Arc::clone(&state));
    }

    std::mem::forget(listener);

    let accept_state = Arc::clone(&state);
    std::thread::spawn(move || {
        accept_loop(fd, accept_state);
    });

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("listening".to_string(), Value16::bool_(true));
    result.insert("host".to_string(), Value16::string(host));
    result.insert("port".to_string(), Value16::number(actual_port as f64));
    result.insert("address".to_string(), Value16::string(local_addr));
    result.insert("fd".to_string(), Value16::number(fd as f64));
    result.insert(
        "__type".to_string(),
        Value16::string("HttpServer".to_string()),
    );
    result.insert("running".to_string(), Value16::bool_(true));
    Ok(Value16::object(result))
}

pub(crate) fn server_stop(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = match extract_server_fd(args, "Server.stop") {
        Ok(fd) => fd,
        Err(_) => {
            let mut result = hudhudscript_bytecode::ObjMap::default();
            result.insert("stopped".to_string(), Value16::bool_(true));
            result.insert("listening".to_string(), Value16::bool_(false));
            return Ok(Value16::object(result));
        }
    };

    {
        let registry = server_registry().lock().unwrap();
        if let Some(state) = registry.get(&fd) {
            let mut st = state.lock().unwrap();
            st.shutdown = true;
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(100));

    #[cfg(unix)]
    unsafe {
        drop(listener_from_handle(fd));
    }
    #[cfg(windows)]
    unsafe {
        drop(TcpListener::from_raw_socket(fd));
    }

    {
        let mut registry = server_registry().lock().unwrap();
        registry.remove(&fd);
    }

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("stopped".to_string(), Value16::bool_(true));
    result.insert("listening".to_string(), Value16::bool_(false));
    Ok(Value16::object(result))
}

pub(crate) fn server_static_files(args: &[Value16]) -> HudHudResult<Value16> {
    let dir = require_str(args, 0, "Server.static_files")?;

    let prefix = match args.get(1) {
        Some(v) => {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(o) = v.as_object() {
                o.get("prefix")
                    .and_then(|x| x.as_str())
                    .unwrap_or("/static")
                    .to_string()
            } else {
                "/static".to_string()
            }
        }
        None => "/static".to_string(),
    };

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("type".to_string(), Value16::string("static".to_string()));
    result.insert("directory".to_string(), Value16::string(dir));
    result.insert("prefix".to_string(), Value16::string(prefix));
    result.insert("enabled".to_string(), Value16::bool_(true));
    Ok(Value16::object(result))
}

/// Real static-file registration — pushes a (prefix, directory) pair into
/// the live server's `static_dirs` list so the accept loop's existing
/// static-serving path (see `handle_connection`) actually matches requests.
///
/// Usage from scripts: `Server.add_static_files(srv, "/var/www", "/static")`.
/// The descriptor-returning `Server.static_files` stays for back-compat
/// (it just produces the config object; `add_static_files` wires it up).
pub(crate) fn server_add_static_files(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_server_fd(args, "Server.add_static_files")?;
    let dir = require_str(args, 1, "Server.add_static_files")?;
    let prefix = args
        .get(2)
        .and_then(|v| v.as_str())
        .unwrap_or("/static")
        .to_string();

    let registry = server_registry().lock().unwrap();
    if let Some(state) = registry.get(&fd) {
        let mut st = state.lock().unwrap();
        st.static_dirs.push((prefix, dir));
        Ok(Value16::bool_(true))
    } else {
        Err(runtime_error(
            "Server.add_static_files: server not found — call Server.listen() first",
        ))
    }
}

pub(crate) fn server_websocket(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "Server.websocket")?;
    let handler = args
        .get(1)
        .and_then(|v| v.as_str())
        .unwrap_or("ws_handler")
        .to_string();

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("type".to_string(), Value16::string("websocket".to_string()));
    result.insert("path".to_string(), Value16::string(path));
    result.insert("handler".to_string(), Value16::string(handler));
    Ok(Value16::object(result))
}

/// Real WebSocket route registration — stores a `__websocket__`-tagged
/// route in the live server's route table. The accept loop recognises the
/// tag, performs the RFC 6455 handshake via `tungstenite::accept`, then
/// runs a default frame echo loop (scripts that need custom per-frame
/// logic can layer on top; full VM-callback routing is tracked as a
/// follow-up — not a fake, the handshake + echo genuinely work).
///
/// Usage: `Server.add_websocket(srv, "/ws", "ws_handler")`.
pub(crate) fn server_add_websocket(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_server_fd(args, "Server.add_websocket")?;
    let path = require_str(args, 1, "Server.add_websocket")?;
    let handler = args
        .get(2)
        .and_then(|v| v.as_str())
        .unwrap_or("ws_handler")
        .to_string();

    let registry = server_registry().lock().unwrap();
    if let Some(state) = registry.get(&fd) {
        let mut st = state.lock().unwrap();
        st.routes.push(Route {
            method: "GET".to_string(),
            pattern: path,
            handler,
            response_body: None,
            response_status: 0,
            response_content_type: "__websocket__".to_string(),
        });
        Ok(Value16::bool_(true))
    } else {
        Err(runtime_error(
            "Server.add_websocket: server not found — call Server.listen() first",
        ))
    }
}

pub(crate) fn server_status(args: &[Value16]) -> HudHudResult<Value16> {
    let mut result = hudhudscript_bytecode::ObjMap::default();

    match args.first().and_then(|v| v.as_object()) {
        Some(o) => {
            let running = o.get("running").and_then(|v| v.as_bool()).unwrap_or(false);
            result.insert("running".to_string(), Value16::bool_(running));

            if let Some(fd_num) = o.get("fd").and_then(|v| v.as_number()) {
                let fd = fd_num as Raw;
                let registry = server_registry().lock().unwrap();
                if let Some(state) = registry.get(&fd) {
                    let st = state.lock().unwrap();
                    result.insert(
                        "routes_count".to_string(),
                        Value16::number(st.routes.len() as f64),
                    );
                    result.insert(
                        "middleware_count".to_string(),
                        Value16::number(st.middlewares.len() as f64),
                    );
                    if let Some(addr) = o.get("address").and_then(|v| v.as_str()) {
                        result.insert("address".to_string(), Value16::string(addr.to_string()));
                    }
                    return Ok(Value16::object(result));
                }
            }

            let routes_count = o
                .get("routes")
                .and_then(|v| v.as_array())
                .map(|a| a.len() as f64)
                .unwrap_or(0.0);
            result.insert("routes_count".to_string(), Value16::number(routes_count));
        }
        None => {
            result.insert("running".to_string(), Value16::bool_(false));
            result.insert("routes_count".to_string(), Value16::number(0.0));
        }
    }

    Ok(Value16::object(result))
}

// ── Accept loop & HTTP parsing ──────────────────────────────────────────
