//! G7: MCP Server integration test — verify server creation and tool listing.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

#[test]
fn test_mcp_server_create_smoke() {
    // Verify MCP server ops don't crash
    let src = r#"
        mcp TestServer {
            transport: "stdio"
            command: "echo"
            args: ["hello"]
        }
    "#;
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = VM::new();
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
