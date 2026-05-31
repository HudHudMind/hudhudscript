//! Shared Unix-domain-socket builtin — single source of truth (Kural 7).
//!
//! Unix-only. On non-Unix platforms all functions return "not supported"
//! errors.

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

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Connect,
    Write,
    Read,
    Close,
    Http,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "connect" => Ok(Self::Connect),
            "write" => Ok(Self::Write),
            "read" => Ok(Self::Read),
            "close" => Ok(Self::Close),
            "http" => Ok(Self::Http),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Connect => unix_connect(args),
        ScriptMethodId::Write => unix_write(args),
        ScriptMethodId::Read => unix_read(args),
        ScriptMethodId::Close => unix_close(args),
        ScriptMethodId::Http => unix_http(args),
    }
}

/// Main entry point (kept for backward compat).

#[cfg(not(unix))]
fn not_supported(name: &str) -> HudHudResult<Value16> {
    Err(runtime_error(format!(
        "{} is not supported on this platform (Unix sockets are not available)",
        name
    )))
}

#[cfg(not(unix))]
pub fn unix_connect(_args: &[Value16]) -> HudHudResult<Value16> {
    not_supported("unix.connect")
}
#[cfg(not(unix))]
pub fn unix_write(_args: &[Value16]) -> HudHudResult<Value16> {
    not_supported("unix.write")
}
#[cfg(not(unix))]
pub fn unix_read(_args: &[Value16]) -> HudHudResult<Value16> {
    not_supported("unix.read")
}
#[cfg(not(unix))]
pub fn unix_close(_args: &[Value16]) -> HudHudResult<Value16> {
    not_supported("unix.close")
}
#[cfg(not(unix))]
pub fn unix_http(_args: &[Value16]) -> HudHudResult<Value16> {
    not_supported("unix.http")
}

#[cfg(unix)]
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(unix)]
pub fn unix_connect(args: &[Value16]) -> HudHudResult<Value16> {
    let path = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("unix.connect: expected socket path string"))?
        .to_string();

    let stream = UnixStream::connect(&path)
        .map_err(|e| runtime_error(format!("unix.connect error: {}", e)))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .ok();

    let fd = stream.as_raw_fd();
    std::mem::forget(stream);

    let mut obj = HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value16::string("UnixStream".to_string()),
    );
    obj.insert("fd".to_string(), Value16::number(fd as f64));
    obj.insert("path".to_string(), Value16::string(path));
    Ok(Value16::object(obj))
}

#[cfg(unix)]
pub fn unix_write(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_unix_fd(args, "unix.write")?;
    let data = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("unix.write: expected data string as second argument"))?
        .as_bytes()
        .to_vec();

    let mut stream = unsafe { UnixStream::from_raw_fd(fd) };
    let result = stream.write_all(&data);
    std::mem::forget(stream);

    result.map_err(|e| runtime_error(format!("unix.write error: {}", e)))?;
    Ok(Value16::number(data.len() as f64))
}

#[cfg(unix)]
pub fn unix_read(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_unix_fd(args, "unix.read")?;
    let buf_size = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n as usize)
        .unwrap_or(4096);

    let mut stream = unsafe { UnixStream::from_raw_fd(fd) };
    let mut buf = vec![0u8; buf_size];
    let result = stream.read(&mut buf);
    std::mem::forget(stream);

    let n = result.map_err(|e| runtime_error(format!("unix.read error: {}", e)))?;
    Ok(Value16::string(
        String::from_utf8_lossy(&buf[..n]).to_string(),
    ))
}

#[cfg(unix)]
pub fn unix_close(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_unix_fd(args, "unix.close")?;
    unsafe {
        drop(UnixStream::from_raw_fd(fd));
    }
    Ok(Value16::null())
}

#[cfg(unix)]
pub fn unix_http(args: &[Value16]) -> HudHudResult<Value16> {
    let path = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("unix.http: expected socket path as first argument"))?
        .to_string();
    let method = args
        .get(1)
        .and_then(|v| v.as_str())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "GET".to_string());
    let uri_path = args
        .get(2)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/".to_string());
    let body = args.get(3).and_then(|v| {
        if let Some(s) = v.as_str() {
            Some(s.to_string())
        } else if v.as_object().is_some() {
            Some(value_to_json_string(v))
        } else {
            None
        }
    });

    let mut stream = UnixStream::connect(&path)
        .map_err(|e| runtime_error(format!("unix.http connect error: {}", e)))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .ok();

    let content = body.as_deref().unwrap_or("");
    let request = if content.is_empty() {
        format!(
            "{} {} HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            method, uri_path
        )
    } else {
        format!(
            "{} {} HTTP/1.0\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            method, uri_path, content.len(), content
        )
    };

    stream
        .write_all(request.as_bytes())
        .map_err(|e| runtime_error(format!("unix.http write error: {}", e)))?;
    stream.shutdown(std::net::Shutdown::Write).ok();

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| runtime_error(format!("unix.http read error: {}", e)))?;

    let mut result: HashMap<String, Value16> = HashMap::new();

    if let Some(header_end) = response.find("\r\n\r\n") {
        let header_part = &response[..header_end];
        let body_part = &response[header_end + 4..];

        if let Some(first_line) = header_part.lines().next() {
            let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let status: f64 = parts[1].parse().unwrap_or(0.0);
                result.insert("status".to_string(), Value16::number(status));
                result.insert(
                    "ok".to_string(),
                    Value16::bool_((200.0..300.0).contains(&status)),
                );
            }
        }

        let mut headers: HashMap<String, Value16> = HashMap::new();
        for line in header_part.lines().skip(1) {
            if let Some((k, v)) = line.split_once(": ") {
                headers.insert(k.to_lowercase(), Value16::string(v.to_string()));
            }
        }
        result.insert("headers".to_string(), Value16::object(headers));
        result.insert("body".to_string(), Value16::string(body_part.to_string()));

        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(body_part) {
            result.insert("json".to_string(), serde_json_to_value16(&json_val));
        } else {
            result.insert("json".to_string(), Value16::null());
        }
    } else {
        result.insert("status".to_string(), Value16::number(0.0));
        result.insert("ok".to_string(), Value16::bool_(false));
        result.insert("body".to_string(), Value16::string(response));
        result.insert("headers".to_string(), Value16::object(HashMap::new()));
        result.insert("json".to_string(), Value16::null());
    }

    Ok(Value16::object(result))
}

#[cfg(unix)]
fn extract_unix_fd(args: &[Value16], callee: &str) -> HudHudResult<RawFd> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error(format!("{}: expected connection object", callee)))?;
    let obj = val
        .as_object()
        .ok_or_else(|| type_error("UnixStream", val.type_name_str(), callee))?;
    obj.get("fd")
        .and_then(|v| v.as_number())
        .map(|n| n as RawFd)
        .ok_or_else(|| type_error("UnixStream", "object without fd", callee))
}

fn serde_json_to_value16(v: &serde_json::Value) -> Value16 {
    match v {
        serde_json::Value::Null => Value16::null(),
        serde_json::Value::Bool(b) => Value16::bool_(*b),
        serde_json::Value::Number(n) => Value16::number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value16::string(s.clone()),
        serde_json::Value::Array(arr) => {
            Value16::array(arr.iter().map(serde_json_to_value16).collect())
        }
        serde_json::Value::Object(map) => Value16::object(
            map.iter()
                .map(|(k, v)| (k.clone(), serde_json_to_value16(v)))
                .collect(),
        ),
    }
}

fn value_to_json_string(value: &Value16) -> String {
    if value.is_null() {
        return "null".to_string();
    }
    if let Some(b) = value.as_bool() {
        return b.to_string();
    }
    if let Some(n) = value.as_number() {
        return format_number(n);
    }
    if let Some(i) = value.as_int() {
        return format_number(i as f64);
    }
    if let Some(s) = value.as_str() {
        return format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    }
    if let Some(arr) = value.as_array() {
        let items: Vec<String> = arr.iter().map(|v| value_to_json_string(v)).collect();
        return format!("[{}]", items.join(","));
    }
    if let Some(obj) = value.as_object() {
        let mut pairs: Vec<String> = obj
            .iter()
            .filter(|(k, _)| !k.starts_with("__"))
            .map(|(k, v)| format!("\"{}\":{}", k, value_to_json_string(v)))
            .collect();
        pairs.sort();
        return format!("{{{}}}", pairs.join(","));
    }
    format!("\"{}\"", value.display_string().replace('"', "\\\""))
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}
