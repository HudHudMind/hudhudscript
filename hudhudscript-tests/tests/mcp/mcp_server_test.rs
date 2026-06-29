//! MCP Server integration tests — verify VM-level server creation and tool dispatch.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

/// Helper: returns the fixture server binary path.
fn fixture_bin() -> String {
    env!("CARGO_BIN_EXE_mcp_fixture_server").to_string()
}

/// Smoke test: create an MCP server with the fixture binary.
/// Verifies that the VM spawns the client without panicking.
#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_server_create_smoke() {
    let bin = fixture_bin();
    let src = format!(
        r#"
        mcp TestServer {{
            transport: "stdio"
            command: "{}"
        }}
    "#,
        bin
    );
    let ast = parse(&src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = VM::permissive();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bc).unwrap();
}

#[test]
fn test_mcp_syntax_accepted() {
    let src = r#"
        mcp MyServer {
            transport: "sse"
            url: "http://localhost:8080"
            tool greet(name: string) { return "Hello " + name }
        }
    "#;
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    c.compile(&ast).unwrap();
}

/// Verify that MCP server without required fields gives a proper error.
#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_server_stdio_missing_command() {
    let src = r#"
        mcp BadServer {
            transport: "stdio"
        }
    "#;
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = VM::permissive();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    let result = vm.execute(&bc);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("requires 'command'"), "Got: {}", err);
}

/// MCP-40: Process capability must be explicitly granted for stdio servers.
#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_server_process_denied_by_default() {
    let bin = fixture_bin();
    let src = format!(
        r#"
        mcp DeniedServer {{
            transport: "stdio"
            command: "{}"
        }}
    "#,
        bin
    );
    let ast = parse(&src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    // Default sandbox has allow_process: false → should fail.
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    let result = vm.execute(&bc);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("process execution denied"), "Got: {}", err);
}
