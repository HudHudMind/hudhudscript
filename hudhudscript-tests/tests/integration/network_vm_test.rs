//! VM integration tests for Network Batch 6 — TCP/UDP (#675), Unix (#676), WebSocket (#616)

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn vm_run(code: &str) -> VM {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execution failed");
    vm
}

// ── TCP tests (#675) ────────────────────────────────────────────────

#[test]
fn test_vm_tcp_listen_returns_object_with_stub_transport() {
    let code = r#"
        var server = tcp.listen("test-local-net", 0);
        var has_fd = server.fd != null;
        var has_addr = server.address != null;
    "#;
    let vm = vm_run(code);
    assert!(
        vm.get_variable("has_fd")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
    assert!(
        vm.get_variable("has_addr")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
}

#[test]
#[ignore = "requires HUDHUD_REAL_NETWORK_TESTS=1 and localhost bind permission"]
fn test_vm_tcp_listen_returns_object_real_socket() {
    if std::env::var("HUDHUD_REAL_NETWORK_TESTS").unwrap_or_default() != "1" {
        return;
    }
    let code = r#"
        var server = tcp.listen("127.0.0.1", 0);
        var has_fd = server.fd != null;
        var has_addr = server.address != null;
    "#;
    let vm = vm_run(code);
    assert!(
        vm.get_variable("has_fd")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
    assert!(
        vm.get_variable("has_addr")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
}

// ── UDP tests (#675) ────────────────────────────────────────────────

#[test]
fn test_vm_udp_bind_returns_object_with_stub_transport() {
    let code = r#"
        var sock = udp.bind("test-local-net", 0);
        var has_fd = sock.fd != null;
        var has_addr = sock.address != null;
    "#;
    let vm = vm_run(code);
    assert!(
        vm.get_variable("has_fd")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
    assert!(
        vm.get_variable("has_addr")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
}

#[test]
#[ignore = "requires HUDHUD_REAL_NETWORK_TESTS=1 and localhost bind permission"]
fn test_vm_udp_bind_returns_object_real_socket() {
    if std::env::var("HUDHUD_REAL_NETWORK_TESTS").unwrap_or_default() != "1" {
        return;
    }
    let code = r#"
        var sock = udp.bind("127.0.0.1", 0);
        var has_fd = sock.fd != null;
        var has_addr = sock.address != null;
    "#;
    let vm = vm_run(code);
    assert!(
        vm.get_variable("has_fd")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
    assert!(
        vm.get_variable("has_addr")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
}

// ── Unix domain socket tests (#676) ─────────────────────────────────

#[test]
fn test_vm_unix_connect_nonexistent_fails() {
    let code = r#"var conn = unix.connect("/tmp/__hudhud_nonexistent_sock__");"#;
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_err());
}

// ── WebSocket tests (#616) ──────────────────────────────────────────

#[test]
fn test_vm_ws_serve_returns_server_with_stub_transport() {
    let code = r#"
        var server = ws.serve("test-local-net", 0);
        var has_id = server.id != null;
        var has_addr = server.address != null;
    "#;
    let vm = vm_run(code);
    assert!(
        vm.get_variable("has_id")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
    assert!(
        vm.get_variable("has_addr")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
}

#[test]
#[ignore = "requires HUDHUD_REAL_NETWORK_TESTS=1 and localhost bind permission"]
fn test_vm_ws_serve_returns_server_real_socket() {
    if std::env::var("HUDHUD_REAL_NETWORK_TESTS").unwrap_or_default() != "1" {
        return;
    }
    let code = r#"
        var server = ws.serve("127.0.0.1", 0);
        var has_id = server.id != null;
        var has_addr = server.address != null;
    "#;
    let vm = vm_run(code);
    assert!(
        vm.get_variable("has_id")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
    assert!(
        vm.get_variable("has_addr")
            .and_then(|v: &hudhudscript_bytecode::Value16| v.as_bool())
            == Some(true)
    );
}

#[test]
fn test_vm_ws_connect_invalid_fails() {
    let code = r#"var conn = ws.connect("ws://127.0.0.1:1");"#;
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_err());
}
