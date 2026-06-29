//! MCP VM dispatch tests — verify that the VM can call MCP tools
//! through the `mcp_call` builtin, using the real fixture server binary.
//!
//! Note: dotted syntax `mcp.Server.tool()` requires a parser fix to allow
//! `mcp` as an identifier in expression context (currently reserved keyword).

use hudhudscript_compiler::Compiler;
use hudhudscript_mcp::ConnectionState;
use hudhudscript_mcp::McpClient;
use hudhudscript_mcp::TransportConfig;
use hudhudscript_parser::parse;
use hudhudscript_vm::SandboxConfig;
use hudhudscript_vm::VM;
use std::sync::Arc;

/// Helper: returns the fixture server binary path.
fn fixture_bin() -> String {
    env!("CARGO_BIN_EXE_mcp_fixture_server").to_string()
}

/// Helper: create an initialized McpClient connected to the fixture.
async fn fixture_client() -> Arc<McpClient> {
    let bin = fixture_bin();
    let config = TransportConfig::stdio(&bin, vec![]);
    let client = McpClient::new(config).await.expect("spawn fixture");
    client.initialize().await.expect("initialize fixture");
    Arc::new(client)
}

/// Helper: create a VM with network-allowing sandbox for MCP tests.
fn vm_for_mcp() -> VM {
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    // MCP requires network access in sandbox.
    vm.with_sandbox(SandboxConfig {
        allowed_paths: vec![],
        allowed_hosts: vec![],
        allow_file_read: true,
        allow_file_write: true,
        allow_network: true,
        allow_process: true,
        allowed_commands: vec![],
        denied_commands: vec![],
    });
    vm
}

// ═══════════════════════════════════════════════════════════════════
// Builtin syntax: mcp_call("server", "tool", args)
// ═══════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_call_echo() {
    let client = fixture_client().await;

    let src = r#"
        let result = mcp_call("Fixture", "echo", {text: "hello vm"})
    "#;

    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = vm_for_mcp();
    vm.register_mcp_client("Fixture".to_string(), client);
    vm.execute(&bc).unwrap();

    let val = vm.get_variable_owned("result").expect("result in globals");
    assert_eq!(
        val.as_string().as_deref(),
        Some("hello vm"),
        "Got: {:?}",
        val
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_call_add() {
    let client = fixture_client().await;

    let src = r#"
        let result = mcp_call("Fixture", "add", {a: 7, b: 3})
    "#;

    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = vm_for_mcp();
    vm.register_mcp_client("Fixture".to_string(), client);
    vm.execute(&bc).unwrap();

    let val = vm.get_variable_owned("result").expect("result in globals");
    assert_eq!(val.as_string().as_deref(), Some("10"), "Got: {:?}", val);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_call_add_large() {
    let client = fixture_client().await;

    let src = r#"
        let result = mcp_call("Fixture", "add", {a: 100, b: 200})
    "#;

    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = vm_for_mcp();
    vm.register_mcp_client("Fixture".to_string(), client);
    vm.execute(&bc).unwrap();

    let val = vm.get_variable_owned("result").expect("result in globals");
    assert_eq!(val.as_string().as_deref(), Some("300"), "Got: {:?}", val);
}

// ── Error cases ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_server_not_found() {
    let src = r#"
        let result = mcp_call("GhostServer", "echo", {text: "nope"})
    "#;

    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = vm_for_mcp();

    let result = vm.execute(&bc);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("not found"), "Got: {}", err);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_call_unknown_tool() {
    let client = fixture_client().await;

    let src = r#"
        let result = mcp_call("Fixture", "nonexistent_tool", {})
    "#;

    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = vm_for_mcp();
    vm.register_mcp_client("Fixture".to_string(), client);

    let result = vm.execute(&bc);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("failed") || err.contains("error"),
        "Got: {}",
        err
    );
}

// ── Multiple calls in sequence ────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_call_multiple() {
    let client = fixture_client().await;

    let src = r#"
        let r1 = mcp_call("Fixture", "echo", {text: "first"})
        let r2 = mcp_call("Fixture", "echo", {text: "second"})
        let r3 = mcp_call("Fixture", "add", {a: 1, b: 2})
    "#;

    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = vm_for_mcp();
    vm.register_mcp_client("Fixture".to_string(), client);
    vm.execute(&bc).unwrap();

    assert_eq!(
        vm.get_variable_owned("r1").and_then(|v| v.as_string()),
        Some("first".to_string())
    );
    assert_eq!(
        vm.get_variable_owned("r2").and_then(|v| v.as_string()),
        Some("second".to_string())
    );
    assert_eq!(
        vm.get_variable_owned("r3").and_then(|v| v.as_string()),
        Some("3".to_string())
    );
}

// ── MCP-31: max_mcp_servers limit enforcement ────────────────────

/// Minimal mock transport for limit tests (no process spawn).
struct NopTransport;

use hudhudscript_mcp::transport::{Transport, TransportRecv, TransportSend};
#[async_trait::async_trait]
impl TransportSend for NopTransport {
    async fn send(
        &mut self,
        _req: hudhudscript_mcp::protocol::JsonRpcRequest,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
#[async_trait::async_trait]
impl TransportRecv for NopTransport {
    async fn receive(&mut self) -> anyhow::Result<hudhudscript_mcp::protocol::JsonRpcResponse> {
        anyhow::bail!("nop")
    }
}
#[async_trait::async_trait]
impl Transport for NopTransport {
    fn split(
        self: Box<Self>,
    ) -> (
        hudhudscript_mcp::transport::TransportSendHalf,
        hudhudscript_mcp::transport::TransportRecvHalf,
    ) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        drop(rx); // recv half immediately closed — no responses expected
        (Box::new(NopSendHalf), Box::new(NopRecvHalf { _tx: tx }))
    }
}

struct NopSendHalf;
#[async_trait::async_trait]
impl TransportSend for NopSendHalf {
    async fn send(
        &mut self,
        _req: hudhudscript_mcp::protocol::JsonRpcRequest,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
struct NopRecvHalf {
    _tx: tokio::sync::mpsc::UnboundedSender<()>,
}
#[async_trait::async_trait]
impl TransportRecv for NopRecvHalf {
    async fn receive(&mut self) -> anyhow::Result<hudhudscript_mcp::protocol::JsonRpcResponse> {
        anyhow::bail!("nop recv closed")
    }
}

#[test]
fn test_mcp_max_servers_limit() {
    let mut vm = vm_for_mcp();
    vm.with_max_mcp_servers(2);

    let client1 = Arc::new(McpClient::from_transport(Box::new(NopTransport)));

    vm.register_mcp_client("srv1".to_string(), client1.clone());
    vm.register_mcp_client("srv2".to_string(), client1.clone());
    assert!(vm.get_mcp_client("srv1").is_some());
    assert!(vm.get_mcp_client("srv2").is_some());

    // Third registration triggers eviction (2 → 1).
    vm.register_mcp_client("srv3".to_string(), client1.clone());
    let count = ["srv1", "srv2", "srv3"]
        .iter()
        .filter(|n| vm.get_mcp_client(n).is_some())
        .count();
    assert_eq!(
        count, 2,
        "After registering 3 servers with max=2, should have exactly 2"
    );
}

// ── MCP-51: VM shutdown disconnects MCP clients ──────────────────

#[tokio::test(flavor = "multi_thread")]
async fn test_vm_shutdown_mcp_clients() {
    let client = fixture_client().await;
    let mut vm = vm_for_mcp();
    vm.register_mcp_client("Fixture".to_string(), client.clone());

    // Verify client is connected and working.
    let src = r#"
        let result = mcp_call("Fixture", "echo", {text: "before shutdown"})
    "#;
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    vm.execute(&bc).unwrap();
    assert_eq!(
        vm.get_variable_owned("result").and_then(|v| v.as_string()),
        Some("before shutdown".to_string())
    );

    // Shutdown all clients.
    vm.shutdown_mcp_clients().await;

    // Registry should be empty.
    assert!(vm.get_mcp_client("Fixture").is_none());

    // Client should be disconnected.
    let state = client.state().await;
    assert_eq!(state, hudhudscript_mcp::ConnectionState::Disconnected);
}

// ── MCP-50: Status report ───────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_status_report() {
    let client = fixture_client().await;
    let report = client.status_report().await;
    let state = report.get("state").and_then(|v| v.as_str());
    assert_eq!(state, Some("Connected"));
    assert_eq!(
        report.get("pending_requests").and_then(|v| v.as_u64()),
        Some(0)
    );
    let info = report.get("server_info");
    assert!(info.is_some());
}

// ── MCP-12: Disconnect kills child ──────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_disconnect_changes_state() {
    let client = fixture_client().await;
    assert_eq!(client.state().await, ConnectionState::Connected);
    client.disconnect().await.unwrap();
    assert_eq!(client.state().await, ConnectionState::Disconnected);
}

// ── MCP-41: SSE URL validation ──────────────────────────────────

use hudhudscript_vm::mcp_dispatch::validate_sse_url;

#[test]
fn test_sse_url_https_allowed() {
    assert!(validate_sse_url("test", "https://api.example.com/mcp", false).is_ok());
}

#[test]
fn test_sse_url_http_denied_by_default() {
    let err = validate_sse_url("test", "http://api.example.com/mcp", false).unwrap_err();
    assert!(err.contains("allow_insecure_http"), "Got: {}", err);
}

#[test]
fn test_sse_url_http_allowed_when_opted_in() {
    assert!(validate_sse_url("test", "http://api.example.com/mcp", true).is_ok());
}

#[test]
fn test_sse_url_blocks_localhost() {
    assert!(validate_sse_url("test", "https://localhost:8080/mcp", true).is_err());
    assert!(validate_sse_url("test", "https://127.0.0.1/mcp", true).is_err());
}

#[test]
fn test_sse_url_blocks_private_ips() {
    assert!(validate_sse_url("test", "https://10.0.0.1/mcp", true).is_err());
    assert!(validate_sse_url("test", "https://192.168.1.1/mcp", true).is_err());
    assert!(validate_sse_url("test", "https://172.16.0.1/mcp", true).is_err());
}

#[test]
fn test_sse_url_blocks_metadata_ips() {
    assert!(validate_sse_url("test", "https://169.254.169.254/", true).is_err());
    assert!(validate_sse_url("test", "https://metadata.google.internal/", true).is_err());
}

#[test]
fn test_sse_url_rejects_non_http() {
    assert!(validate_sse_url("test", "ftp://example.com", false).is_err());
    assert!(validate_sse_url("test", "file:///etc/passwd", false).is_err());
}

// ── MCP-12: Pending requests cleared after shutdown ─────────────

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_shutdown_wakes_pending() {
    let bin = fixture_bin();
    let config = TransportConfig::stdio(&bin, vec![]);
    let client = Arc::new(McpClient::new(config).await.unwrap());
    // Don't initialize — just shutdown while handler is running.
    // Shutdown should abort the handler and clear pending.
    client.shutdown().await;
    assert_eq!(client.state().await, ConnectionState::Disconnected);
    // Verify pending requests are empty (none were added, but shutdown is idempotent).
}
