//! TCP / UDP builtin integration tests (Kural 7 follow-up).
//!
//! Previously imported `tcp_connect`, `tcp_listen`, `tcp_close`,
//! `udp_bind`, `udp_send`, `udp_recv`, `udp_close` from
//! `hudhudscript_builtins::builtins::tcp_udp` and called them directly.
//! Those helpers were a second copy of the exact operations already
//! implemented in `hudhudscript_shared_builtins::tcp_ops::call_tcp_method`
//! and `hudhudscript_shared_builtins::udp_ops::call_udp_method`; after the
//! `hudhudscript-builtins` crate was deleted, the module-factory tests
//! (`test_tcp_module_has_all_methods` / `test_udp_module_has_all_methods`)
//! had no shared counterpart (they probed the `Value::NativeFunction`
//! layout of `create_tcp_module()` / `create_udp_module()`), so they are
//! removed and the remaining network-roundtrip tests run through the
//! shared dispatcher — the same path both runtimes hit at runtime.

use hudhudscript_bytecode::Value16;
use std::net::TcpListener;
use std::os::unix::io::RawFd;

#[test]
fn test_tcp_listen_and_connect() {
    // Listen on a random port through the shared dispatcher — the same
    // function path both runtimes reach from `tcp.listen(...)`.
    let listener = hudhudscript_shared_builtins::tcp_ops::dispatch(
        "listen",
        &[
            Value16::string("127.0.0.1".to_string()),
            Value16::number(0.0),
        ],
    )
    .unwrap();

    let addr = if let Some(obj) = listener.as_object() {
        match obj.get("address").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            _ => panic!("No address"),
        }
    } else {
        panic!("Expected object");
    };

    let port: u16 = addr.split(':').last().unwrap().parse().unwrap();

    let conn = hudhudscript_shared_builtins::tcp_ops::dispatch(
        "connect",
        &[
            Value16::string("127.0.0.1".to_string()),
            Value16::number(port as f64),
        ],
    )
    .unwrap();

    assert!(conn.as_object().is_some());

    hudhudscript_shared_builtins::tcp_ops::dispatch("close", &[conn]).unwrap();
    // Close listener by reconstructing from fd.
    if let Some(obj) = listener.as_object() {
        let fd = match obj.get("fd").and_then(|v| v.as_number()) {
            Some(n) => n as RawFd,
            _ => panic!("No fd"),
        };
        unsafe {
            drop(<TcpListener as std::os::unix::io::FromRawFd>::from_raw_fd(
                fd,
            ));
        }
    }
}

#[test]
fn test_udp_bind_and_send_recv() {
    let sock1 = hudhudscript_shared_builtins::udp_ops::dispatch(
        "bind",
        &[
            Value16::string("127.0.0.1".to_string()),
            Value16::number(0.0),
        ],
    )
    .unwrap();
    let sock2 = hudhudscript_shared_builtins::udp_ops::dispatch(
        "bind",
        &[
            Value16::string("127.0.0.1".to_string()),
            Value16::number(0.0),
        ],
    )
    .unwrap();

    let addr2 = if let Some(obj) = sock2.as_object() {
        match obj.get("address").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            _ => panic!("No address"),
        }
    } else {
        panic!("Expected object");
    };

    // Send sock1 → sock2 through the shared dispatcher.
    let sent = hudhudscript_shared_builtins::udp_ops::dispatch(
        "send",
        &[
            sock1.clone(),
            Value16::string(addr2),
            Value16::string("hello udp".to_string()),
        ],
    )
    .unwrap();
    assert_eq!(sent, Value16::number(9.0));

    let msg = hudhudscript_shared_builtins::udp_ops::dispatch("recv", &[sock2.clone()]).unwrap();
    if let Some(obj) = &msg.as_object() {
        assert_eq!(
            obj.get("data"),
            Some(&Value16::string("hello udp".to_string()))
        );
        assert_eq!(obj.get("bytes"), Some(&Value16::number(9.0)));
    } else {
        panic!("Expected object");
    }

    hudhudscript_shared_builtins::udp_ops::dispatch("close", &[sock1]).unwrap();
    hudhudscript_shared_builtins::udp_ops::dispatch("close", &[sock2]).unwrap();
}
