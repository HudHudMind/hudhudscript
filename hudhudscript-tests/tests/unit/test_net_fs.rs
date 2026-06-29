//! Tests for hudhud-net and hudhud-fs — dispatch error paths.
//! Uses argument validation and unknown method paths (no real I/O needed).

use hudhud_fs::path as fs_path;
use hudhud_net::tcp_ops;
use hudhudscript_bytecode::Value16;

// ── tcp ───────────────────────────────────────────────────────────

#[test]
fn test_tcp_connect_missing_args() {
    let r = tcp_ops::dispatch("connect", &[]);
    assert!(r.is_err(), "connect with no args should fail");
}

#[test]
fn test_tcp_connect_missing_port() {
    let r = tcp_ops::dispatch("connect", &[Value16::string("localhost".to_string())]);
    assert!(r.is_err(), "connect without port should fail");
}

#[test]
fn test_tcp_unknown_method() {
    let r = tcp_ops::dispatch("nonexistent", &[]);
    assert!(r.is_err());
    let err = format!("{}", r.unwrap_err());
    assert!(
        err.contains("Unknown"),
        "should say Unknown tcp method, got: {err}"
    );
}

#[test]
fn test_tcp_listen_missing_port() {
    let r = tcp_ops::dispatch("listen", &[]);
    assert!(r.is_err(), "listen with no args should fail");
}

#[test]
fn test_tcp_close_missing_handle() {
    let r = tcp_ops::dispatch("close", &[]);
    assert!(r.is_err(), "close with no args should fail");
}

// ── fs path ───────────────────────────────────────────────────────

#[test]
fn test_path_join_no_args() {
    let r = fs_path::dispatch("join", &[]);
    assert!(r.is_ok(), "join with no args should succeed (empty path)");
    assert_eq!(r.unwrap().as_string().unwrap(), "");
}

#[test]
fn test_path_unknown_method() {
    let r = fs_path::dispatch("nonexistent", &[]);
    assert!(r.is_err());
    let err = format!("{}", r.unwrap_err());
    assert!(
        err.contains("Unknown") || err.contains("unknown"),
        "got: {err}"
    );
}

#[test]
fn test_path_dirname_missing_arg() {
    let r = fs_path::dispatch("dirname", &[]);
    assert!(r.is_err(), "dirname with no args should fail");
}
