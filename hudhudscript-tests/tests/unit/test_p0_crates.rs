//! Tests for hudhud-exec, hudhud-plugin, hudhud-unix — error paths (no real OS ops).

use hudhud_exec::exec_ops::{kill as exec_kill, run as exec_run};
use hudhud_plugin::plugin_ops::{plugin_get, plugin_list, plugin_register, plugin_unregister};
use hudhud_unix::unix_socket_ops;
use hudhudscript_bytecode::Value16;

// ── exec ──────────────────────────────────────────────────────────

#[test]
fn test_exec_run_missing_command() {
    let r = exec_run::exec_run(&[]);
    assert!(r.is_err(), "run with no args should fail");
}

#[test]
fn test_exec_run_empty_command() {
    let r = exec_run::exec_run(&[Value16::string("".to_string())]);
    assert!(r.is_err(), "run with empty command should fail");
}

#[test]
fn test_exec_kill_missing_pid() {
    let r = exec_kill::exec_kill(&[]);
    assert!(r.is_err(), "kill with no args should fail");
}

// ── plugin ────────────────────────────────────────────────────────

#[test]
fn test_plugin_register_missing_args() {
    let r = plugin_register(&[]);
    assert!(r.is_err(), "register with no args should fail");
}

#[test]
fn test_plugin_list_returns_ok() {
    let r = plugin_list(&[]);
    assert!(r.is_ok(), "list with no args should return empty or ok");
}

#[test]
fn test_plugin_unregister_missing_name() {
    let r = plugin_unregister(&[]);
    assert!(r.is_err(), "unregister with no args should fail");
}

#[test]
fn test_plugin_get_missing_name() {
    let r = plugin_get(&[]);
    assert!(r.is_err(), "get with no args should fail");
}

// ── unix ──────────────────────────────────────────────────────────

#[test]
fn test_unix_connect_missing_path() {
    let r = unix_socket_ops::unix_connect(&[]);
    assert!(r.is_err());
}

#[test]
fn test_unix_http_missing_path() {
    let r = unix_socket_ops::unix_http(&[]);
    assert!(r.is_err());
}
