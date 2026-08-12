//! Integration tests for Network Batch 6 — TCP/UDP (#675), Unix (#676), WebSocket (#616)

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run(code: &str) -> Interpreter {
    let ast = parse(code).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.eval_program(&ast).expect("execution failed");
    interp
}

fn run_and_get(code: &str, var: &str) -> hudhudscript_bytecode::Value16 {
    let interp = run(code);
    interp.get_variable(var).expect("variable not found")
}

// ── TCP tests (#675) ────────────────────────────────────────────────

#[test]
fn test_tcp_module_exists() {
    let val = run_and_get(r#"var t = typeof(tcp);"#, "t");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("object".to_string())
    );
}

#[test]
fn test_tcp_listen_returns_object_with_stub_transport() {
    let code = r#"
        var server = tcp.listen("test-local-net", 0);
        var has_fd = server.fd != null;
        var has_addr = server.address != null;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("has_fd").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
    assert_eq!(
        interp.get_variable("has_addr").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
}

#[test]
#[ignore = "requires HUDHUD_REAL_NETWORK_TESTS=1 and localhost bind permission"]
fn test_tcp_listen_returns_object_real_socket() {
    if std::env::var("HUDHUD_REAL_NETWORK_TESTS").unwrap_or_default() != "1" {
        return;
    }
    let code = r#"
        var server = tcp.listen("127.0.0.1", 0);
        var has_fd = server.fd != null;
        var has_addr = server.address != null;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("has_fd").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
    assert_eq!(
        interp.get_variable("has_addr").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
}

// ── UDP tests (#675) ────────────────────────────────────────────────

#[test]
fn test_udp_module_exists() {
    let val = run_and_get(r#"var t = typeof(udp);"#, "t");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("object".to_string())
    );
}

#[test]
fn test_udp_bind_returns_object_with_stub_transport() {
    let code = r#"
        var sock = udp.bind("test-local-net", 0);
        var has_fd = sock.fd != null;
        var has_addr = sock.address != null;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("has_fd").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
    assert_eq!(
        interp.get_variable("has_addr").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
}

#[test]
#[ignore = "requires HUDHUD_REAL_NETWORK_TESTS=1 and localhost bind permission"]
fn test_udp_bind_returns_object_real_socket() {
    if std::env::var("HUDHUD_REAL_NETWORK_TESTS").unwrap_or_default() != "1" {
        return;
    }
    let code = r#"
        var sock = udp.bind("127.0.0.1", 0);
        var has_fd = sock.fd != null;
        var has_addr = sock.address != null;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("has_fd").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
    assert_eq!(
        interp.get_variable("has_addr").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
}

// ── Unix domain socket tests (#676) ─────────────────────────────────

#[test]
fn test_unix_module_exists() {
    let val = run_and_get(r#"var t = typeof(unix);"#, "t");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("object".to_string())
    );
}

#[test]
fn test_unix_connect_nonexistent_fails() {
    let code = r#"var conn = unix.connect("/tmp/__hudhud_nonexistent_sock__");"#;
    let ast = parse(code).expect("parse failed");
    let mut interp = Interpreter::new();
    let result = interp.eval_program(&ast);
    assert!(result.is_err());
}

// ── WebSocket tests (#616) ──────────────────────────────────────────

#[test]
fn test_ws_module_exists() {
    let val = run_and_get(r#"var t = typeof(ws);"#, "t");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("object".to_string())
    );
}

#[test]
fn test_ws_serve_returns_server_object_with_stub_transport() {
    let code = r#"
        var server = ws.serve("test-local-net", 0);
        var has_id = server.id != null;
        var has_addr = server.address != null;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("has_id").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
    assert_eq!(
        interp.get_variable("has_addr").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
}

#[test]
#[ignore = "requires HUDHUD_REAL_NETWORK_TESTS=1 and localhost bind permission"]
fn test_ws_serve_returns_server_object_real_socket() {
    if std::env::var("HUDHUD_REAL_NETWORK_TESTS").unwrap_or_default() != "1" {
        return;
    }
    let code = r#"
        var server = ws.serve("127.0.0.1", 0);
        var has_id = server.id != null;
        var has_addr = server.address != null;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("has_id").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
    assert_eq!(
        interp.get_variable("has_addr").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
}

#[test]
fn test_ws_connect_invalid_url_fails() {
    let code = r#"
        var result = ws.connect("ws://test-local-net:nonexistent");
    "#;
    let ast = parse(code).expect("parse failed");
    let mut interp = Interpreter::new();
    let result = interp.eval_program(&ast);
    assert!(result.is_err());
}
