//! Shared UDP socket builtin — used by both VM and interpreter.
//!
//! Provides: udp.bind, udp.send, udp.recv, udp.close
//!
//! Uses raw file descriptors (Unix) / raw sockets (Windows) to track sockets.

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
use std::net::UdpSocket;

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{
    AsRawSocket as AsRawFd, FromRawSocket as FromRawFd, RawSocket as RawFd,
};

// ── Cross-platform helpers ─────────────────────────────────────────────

#[cfg(unix)]
fn raw_handle<T: AsRawFd>(t: &T) -> RawFd {
    t.as_raw_fd()
}

#[cfg(windows)]
fn raw_handle<T: AsRawFd>(t: &T) -> RawFd {
    t.as_raw_socket()
}

#[cfg(unix)]
unsafe fn udp_socket_from_raw(fd: RawFd) -> UdpSocket {
    <UdpSocket as FromRawFd>::from_raw_fd(fd)
}

#[cfg(windows)]
unsafe fn udp_socket_from_raw(fd: RawFd) -> UdpSocket {
    <UdpSocket as std::os::windows::io::FromRawSocket>::from_raw_socket(fd)
}

/// Validate that a raw file descriptor is non-negative and refers to an open fd.
fn validate_fd(fd: RawFd) -> HudHudResult<RawFd> {
    if fd < 0 {
        return Err(runtime_error(format!(
            "udp: invalid file descriptor {}",
            fd
        )));
    }
    // On Unix, use fcntl(F_GETFD) to verify the fd is still open.
    #[cfg(unix)]
    {
        let ret = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if ret == -1 {
            return Err(runtime_error(format!(
                "udp: file descriptor {} is not open (EBADF)",
                fd
            )));
        }
    }
    Ok(fd)
}

fn extract_fd(args: &[Value16]) -> HudHudResult<RawFd> {
    match args.first().and_then(|v| v.as_object()) {
        Some(obj) => match obj.get("fd").and_then(|v| v.as_number()) {
            Some(n) => validate_fd(n as RawFd),
            _ => Err(runtime_error("udp: missing fd")),
        },
        _ => Err(runtime_error("udp: expected socket object")),
    }
}

/// Execute a UDP method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        "bind" => udp_bind(args),
        "send" => udp_send(args),
        "recv" => udp_recv(args),
        "close" => udp_close(args),
        _ => Err(runtime_error(format!("Unknown udp method: {}", method))),
    }
}

fn udp_bind(args: &[Value16]) -> HudHudResult<Value16> {
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

    if port > 0 && port < 1024 {
        return Err(runtime_error(format!(
            "udp.bind: binding to privileged port {} is not allowed (use ports >= 1024)",
            port
        )));
    }
    let addr = format!("{}:{}", host, port);
    let socket =
        UdpSocket::bind(&addr).map_err(|e| runtime_error(format!("udp.bind error: {}", e)))?;
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .ok();
    let fd = raw_handle(&socket);
    let local = socket
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| addr);
    std::mem::forget(socket);

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "__type".to_string(),
        Value16::string("UdpSocket".to_string()),
    );
    obj.insert("fd".to_string(), Value16::number(fd as f64));
    obj.insert("address".to_string(), Value16::string(local));
    Ok(Value16::object(obj))
}

fn udp_send(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_fd(args)?;
    let target = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("udp.send: expected target address"))?
        .to_string();
    let data = args
        .get(2)
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("udp.send: expected data string"))?
        .to_string();

    let socket = unsafe { udp_socket_from_raw(fd) };
    let result = socket.send_to(data.as_bytes(), &target);
    std::mem::forget(socket);
    let n = result.map_err(|e| runtime_error(format!("udp.send error: {}", e)))?;
    Ok(Value16::number(n as f64))
}

fn udp_recv(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_fd(args)?;
    let buf_size = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n as usize)
        .unwrap_or(4096);

    let socket = unsafe { udp_socket_from_raw(fd) };
    let mut buf = vec![0u8; buf_size];
    let result = socket.recv_from(&mut buf);
    std::mem::forget(socket);
    let (n, from) = result.map_err(|e| runtime_error(format!("udp.recv error: {}", e)))?;

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "data".to_string(),
        Value16::string(String::from_utf8_lossy(&buf[..n]).to_string()),
    );
    obj.insert("from".to_string(), Value16::string(from.to_string()));
    obj.insert("bytes".to_string(), Value16::number(n as f64));
    Ok(Value16::object(obj))
}

fn udp_close(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_fd(args)?;
    unsafe {
        drop(udp_socket_from_raw(fd));
    }
    Ok(Value16::null())
}
