//! Shared TCP socket builtin — used by both VM and interpreter.
//!
//! Provides: tcp.connect, tcp.listen, tcp.accept, tcp.write, tcp.read, tcp.close
//!
//! Uses raw file descriptors (Unix) / raw sockets (Windows) to track connections.

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
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

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
unsafe fn tcp_stream_from_raw(fd: RawFd) -> TcpStream {
    <TcpStream as FromRawFd>::from_raw_fd(fd)
}

#[cfg(windows)]
unsafe fn tcp_stream_from_raw(fd: RawFd) -> TcpStream {
    <TcpStream as std::os::windows::io::FromRawSocket>::from_raw_socket(fd)
}

#[cfg(unix)]
unsafe fn tcp_listener_from_raw(fd: RawFd) -> TcpListener {
    <TcpListener as FromRawFd>::from_raw_fd(fd)
}

#[cfg(windows)]
unsafe fn tcp_listener_from_raw(fd: RawFd) -> TcpListener {
    <TcpListener as std::os::windows::io::FromRawSocket>::from_raw_socket(fd)
}

/// Validate that a raw file descriptor is non-negative and refers to an open fd.
fn validate_fd(fd: RawFd) -> HudHudResult<RawFd> {
    if fd < 0 {
        return Err(runtime_error(format!(
            "tcp: invalid file descriptor {}",
            fd
        )));
    }
    // On Unix, use fcntl(F_GETFD) to verify the fd is still open.
    #[cfg(unix)]
    {
        let ret = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if ret == -1 {
            return Err(runtime_error(format!(
                "tcp: file descriptor {} is not open (EBADF)",
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
            _ => Err(runtime_error("tcp: missing fd")),
        },
        _ => Err(runtime_error("tcp: expected connection object")),
    }
}

/// Execute a TCP method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        "connect" => tcp_connect(args),
        "listen" => tcp_listen(args),
        "accept" => tcp_accept(args),
        "write" => tcp_write(args),
        "read" => tcp_read(args),
        "close" => tcp_close(args),
        _ => Err(runtime_error(format!("Unknown tcp method: {}", method))),
    }
}

fn tcp_connect(args: &[Value16]) -> HudHudResult<Value16> {
    let host = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("tcp.connect: expected host string"))?
        .to_string();
    let port = args
        .get(1)
        .and_then(|v| v.as_number())
        .ok_or_else(|| runtime_error("tcp.connect: expected port number"))? as u16;

    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(&addr)
        .map_err(|e| runtime_error(format!("tcp.connect error: {}", e)))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .ok();
    let fd = raw_handle(&stream);
    std::mem::forget(stream);

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "__type".to_string(),
        Value16::string("TcpStream".to_string()),
    );
    obj.insert("fd".to_string(), Value16::number(fd as f64));
    obj.insert("address".to_string(), Value16::string(addr));
    Ok(Value16::object(obj))
}

fn tcp_listen(args: &[Value16]) -> HudHudResult<Value16> {
    let host = args
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0.0")
        .to_string();
    let port = args
        .get(1)
        .and_then(|v| v.as_number())
        .ok_or_else(|| runtime_error("tcp.listen: expected port number"))? as u16;

    // #994: Reject privileged ports (<1024) unless port 0 (OS-assigned)
    if port > 0 && port < 1024 {
        return Err(runtime_error(format!(
            "tcp.listen: binding to privileged port {} is not allowed (use ports >= 1024)",
            port
        )));
    }
    let addr = format!("{}:{}", host, port);
    let listener =
        TcpListener::bind(&addr).map_err(|e| runtime_error(format!("tcp.listen error: {}", e)))?;
    let fd = raw_handle(&listener);
    let local = listener
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| addr);
    std::mem::forget(listener);

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "__type".to_string(),
        Value16::string("TcpListener".to_string()),
    );
    obj.insert("fd".to_string(), Value16::number(fd as f64));
    obj.insert("address".to_string(), Value16::string(local));
    Ok(Value16::object(obj))
}

fn tcp_accept(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_fd(args)?;
    let listener = unsafe { tcp_listener_from_raw(fd) };
    let result = listener.accept();
    std::mem::forget(listener);
    let (stream, peer) = result.map_err(|e| runtime_error(format!("tcp.accept error: {}", e)))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .ok();
    let sfd = raw_handle(&stream);
    std::mem::forget(stream);

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "__type".to_string(),
        Value16::string("TcpStream".to_string()),
    );
    obj.insert("fd".to_string(), Value16::number(sfd as f64));
    obj.insert("address".to_string(), Value16::string(peer.to_string()));
    Ok(Value16::object(obj))
}

fn tcp_write(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_fd(args)?;
    let data = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("tcp.write: expected data string"))?
        .as_bytes()
        .to_vec();
    let mut stream = unsafe { tcp_stream_from_raw(fd) };
    let result = stream.write_all(&data);
    std::mem::forget(stream);
    result.map_err(|e| runtime_error(format!("tcp.write error: {}", e)))?;
    Ok(Value16::number(data.len() as f64))
}

fn tcp_read(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_fd(args)?;
    let buf_size = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n as usize)
        .unwrap_or(4096);
    let mut stream = unsafe { tcp_stream_from_raw(fd) };
    let mut buf = vec![0u8; buf_size];
    let result = stream.read(&mut buf);
    std::mem::forget(stream);
    let n = result.map_err(|e| runtime_error(format!("tcp.read error: {}", e)))?;
    Ok(Value16::string(
        String::from_utf8_lossy(&buf[..n]).to_string(),
    ))
}

fn tcp_close(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_fd(args)?;
    unsafe {
        drop(tcp_stream_from_raw(fd));
    }
    Ok(Value16::null())
}
