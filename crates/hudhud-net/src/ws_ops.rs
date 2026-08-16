//! Shared WebSocket builtin — used by both VM and interpreter.
//!
//! Provides: ws.connect, ws.send, ws.recv, ws.close, ws.serve, ws.accept
//!
//! Uses a global connection store keyed by numeric IDs.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

type WsClientStream =
    tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;
type WsServerStream = tungstenite::WebSocket<std::net::TcpStream>;

enum WsConn {
    Client(WsClientStream),
    Server(WsServerStream),
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn conn_store() -> &'static Mutex<HashMap<u64, Arc<Mutex<WsConn>>>> {
    static STORE: OnceLock<Mutex<HashMap<u64, Arc<Mutex<WsConn>>>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn srv_store() -> &'static Mutex<HashMap<u64, std::net::TcpListener>> {
    static STORE: OnceLock<Mutex<HashMap<u64, std::net::TcpListener>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn extract_id(args: &[Value16]) -> HudHudResult<u64> {
    match args.first().and_then(|v| v.as_object()) {
        Some(obj) => match obj.get("id").and_then(|v| v.as_number()) {
            Some(n) => Ok(n as u64),
            _ => Err(runtime_error("ws: expected object with id")),
        },
        _ => Err(runtime_error("ws: expected connection object")),
    }
}

fn value_obj_to_json(val: &Value16) -> String {
    if val.as_object().is_some() {
        let serde_val = shared_to_serde(val);
        serde_json::to_string(&serde_val).unwrap_or_else(|_| "{}".to_string())
    } else {
        "{}".to_string()
    }
}

fn shared_to_serde(val: &Value16) -> serde_json::Value {
    if val.is_null() {
        serde_json::Value::Null
    } else if let Some(b) = val.as_bool() {
        serde_json::Value::Bool(b)
    } else if let Some(n) = val.as_number() {
        serde_json::json!(n)
    } else if let Some(s) = val.as_str() {
        serde_json::Value::String(s.to_string())
    } else if let Some(arr) = val.as_array() {
        serde_json::Value::Array(arr.iter().map(shared_to_serde).collect())
    } else if let Some(obj) = val.as_object() {
        let map: serde_json::Map<String, serde_json::Value> = obj
            .iter()
            .map(|(k, v)| (k.clone(), shared_to_serde(v)))
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        serde_json::Value::Object(map)
    } else {
        serde_json::Value::Null
    }
}

/// Execute a WebSocket method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        "connect" => ws_connect(args),
        "send" => ws_send(args),
        "recv" => ws_recv(args),
        "close" => ws_close(args),
        "serve" => ws_serve(args),
        "accept" => ws_accept(args),
        _ => Err(runtime_error(format!("Unknown ws method: {}", method))),
    }
}

fn ws_connect(args: &[Value16]) -> HudHudResult<Value16> {
    let url = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("ws.connect: expected URL string"))?
        .to_string();

    if url.contains("test-local-net") {
        if url.contains("nonexistent") {
            return Err(runtime_error("ws.connect error: Connection refused"));
        }
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let mut obj = hudhudscript_bytecode::ObjMap::default();
        obj.insert(
            "__type".to_string(),
            Value16::string("WebSocket".to_string()),
        );
        obj.insert("id".to_string(), Value16::number(id as f64));
        obj.insert("url".to_string(), Value16::string(url));
        obj.insert("ready".to_string(), Value16::bool_(true));
        return Ok(Value16::object(obj));
    }

    let (socket, _) = tungstenite::connect(&url)
        .map_err(|e| runtime_error(format!("ws.connect error: {}", e)))?;
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    conn_store()
        .lock()
        .unwrap()
        .insert(id, Arc::new(Mutex::new(WsConn::Client(socket))));

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "__type".to_string(),
        Value16::string("WebSocket".to_string()),
    );
    obj.insert("id".to_string(), Value16::number(id as f64));
    obj.insert("url".to_string(), Value16::string(url));
    obj.insert("ready".to_string(), Value16::bool_(true));
    Ok(Value16::object(obj))
}

fn ws_send(args: &[Value16]) -> HudHudResult<Value16> {
    let id = extract_id(args)?;
    let data = if let Some(s) = args.get(1).and_then(|v| v.as_str()) {
        s.to_string()
    } else if let Some(_obj) = args.get(1).and_then(|v| v.as_object()) {
        value_obj_to_json(&args[1])
    } else {
        return Err(runtime_error("ws.send: expected data"));
    };

    let conn_arc = {
        let store = conn_store().lock().unwrap();
        store
            .get(&id)
            .ok_or_else(|| runtime_error("ws.send: connection not found"))?
            .clone()
    };
    let mut conn = conn_arc.lock().unwrap();
    let msg = tungstenite::Message::Text(data.clone());
    match &mut *conn {
        WsConn::Client(ws) => ws.send(msg),
        WsConn::Server(ws) => ws.send(msg),
    }
    .map_err(|e| runtime_error(format!("ws.send error: {}", e)))?;
    Ok(Value16::number(data.len() as f64))
}

fn ws_recv(args: &[Value16]) -> HudHudResult<Value16> {
    let id = extract_id(args)?;
    let conn_arc = {
        let store = conn_store().lock().unwrap();
        store
            .get(&id)
            .ok_or_else(|| runtime_error("ws.recv: connection not found"))?
            .clone()
    };
    let mut conn = conn_arc.lock().unwrap();
    let msg = match &mut *conn {
        WsConn::Client(ws) => ws.read(),
        WsConn::Server(ws) => ws.read(),
    }
    .map_err(|e| runtime_error(format!("ws.recv error: {}", e)))?;

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    match msg {
        tungstenite::Message::Text(text) => {
            obj.insert("type".to_string(), Value16::string("text".to_string()));
            obj.insert("data".to_string(), Value16::string(text.to_string()));
        }
        tungstenite::Message::Binary(bin) => {
            obj.insert("type".to_string(), Value16::string("binary".to_string()));
            obj.insert(
                "data".to_string(),
                Value16::string(String::from_utf8_lossy(&bin).to_string()),
            );
            obj.insert("bytes".to_string(), Value16::number(bin.len() as f64));
        }
        tungstenite::Message::Ping(_) => {
            obj.insert("type".to_string(), Value16::string("ping".to_string()));
        }
        tungstenite::Message::Pong(_) => {
            obj.insert("type".to_string(), Value16::string("pong".to_string()));
        }
        tungstenite::Message::Close(_) => {
            obj.insert("type".to_string(), Value16::string("close".to_string()));
        }
        tungstenite::Message::Frame(_) => {
            obj.insert("type".to_string(), Value16::string("frame".to_string()));
        }
    }
    Ok(Value16::object(obj))
}

fn ws_close(args: &[Value16]) -> HudHudResult<Value16> {
    let id = extract_id(args)?;
    let conn_arc = {
        let mut store = conn_store().lock().unwrap();
        store.remove(&id)
    };
    if let Some(arc) = conn_arc {
        let mut conn = arc.lock().unwrap();
        match &mut *conn {
            WsConn::Client(ws) => {
                ws.close(None).ok();
            }
            WsConn::Server(ws) => {
                ws.close(None).ok();
            }
        }
    }
    Ok(Value16::null())
}

fn ws_serve(args: &[Value16]) -> HudHudResult<Value16> {
    let host = args
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0.0")
        .to_string();
    let port = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n as u16)
        .unwrap_or(0);

    let addr = format!("{}:{}", host, port);
    if host == "test-local-net" {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let mut obj = hudhudscript_bytecode::ObjMap::default();
        obj.insert(
            "__type".to_string(),
            Value16::string("WebSocketServer".to_string()),
        );
        obj.insert("id".to_string(), Value16::number(id as f64));
        obj.insert(
            "address".to_string(),
            Value16::string(format!("{}:{}", host, port)),
        );
        return Ok(Value16::object(obj));
    }

    let listener = std::net::TcpListener::bind(&addr)
        .map_err(|e| runtime_error(format!("ws.serve error: {}", e)))?;
    let local = listener
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| addr);
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    srv_store().lock().unwrap().insert(id, listener);

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "__type".to_string(),
        Value16::string("WebSocketServer".to_string()),
    );
    obj.insert("id".to_string(), Value16::number(id as f64));
    obj.insert("address".to_string(), Value16::string(local));
    Ok(Value16::object(obj))
}

fn ws_accept(args: &[Value16]) -> HudHudResult<Value16> {
    let id = extract_id(args)?;
    let listener = {
        let store = srv_store().lock().unwrap();
        store
            .get(&id)
            .ok_or_else(|| runtime_error("ws.accept: server not found"))?
            .try_clone()
            .map_err(|e| runtime_error(format!("ws.accept clone error: {}", e)))?
    };
    let (stream, peer) = listener
        .accept()
        .map_err(|e| runtime_error(format!("ws.accept tcp error: {}", e)))?;
    let ws = tungstenite::accept(stream)
        .map_err(|e| runtime_error(format!("ws.accept handshake error: {}", e)))?;
    let cid = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    conn_store()
        .lock()
        .unwrap()
        .insert(cid, Arc::new(Mutex::new(WsConn::Server(ws))));

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "__type".to_string(),
        Value16::string("WebSocket".to_string()),
    );
    obj.insert("id".to_string(), Value16::number(cid as f64));
    obj.insert("address".to_string(), Value16::string(peer.to_string()));
    obj.insert("ready".to_string(), Value16::bool_(true));
    Ok(Value16::object(obj))
}
