//! External tests for parser declaration modules (common, agent).

use hudhudscript_parser::parse;

#[test]
fn test_parse_empty_agent_declaration() {
    let source = "agent EmptyAgent {}";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Empty agent should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_agent_with_single_field() {
    let source = "agent MinAgent { model: \"test\" }";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Minimal agent should parse: {:?}",
        result.err()
    );
}

// ── Real parser tests for declarations ──────────────────────────────────

#[test]
fn test_parse_agent_declaration_real() {
    let source = r#"
        agent Analyst {
            model: "gpt-4",
            temperature: 0.7,
            maxTokens: 2048
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Agent { name, fields, .. }) => {
            assert_eq!(name, "Analyst");
            assert_eq!(fields.len(), 3);
            assert_eq!(fields[0].0, "model");
            assert_eq!(fields[1].0, "temperature");
            assert_eq!(fields[2].0, "maxTokens");
        }
        other => panic!("Expected Agent declaration, got {:?}", other),
    }
}

/// tool_decl was removed from grammar (#489). Tools now live inside MCP server blocks.
/// This test verifies parsing of tool definitions within an MCP declaration.
#[test]
fn test_parse_tool_declaration_real() {
    let source = r#"
        mcp DataServer {
            transport: "stdio",
            command: "npx",
            tool fetchData(url: String) {
                return url;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::McpServer(decl) => {
            assert_eq!(decl.name, "DataServer");
            assert_eq!(decl.fields.len(), 2);
            assert_eq!(decl.fields[0].0, "transport");
            assert_eq!(decl.fields[1].0, "command");
            assert_eq!(decl.tools.len(), 1);
            assert_eq!(decl.tools[0].name, "fetchData");
            assert_eq!(decl.tools[0].params.len(), 1);
            assert_eq!(decl.tools[0].params[0].0, "url");
            assert_eq!(decl.tools[0].params[0].1, "String");
        }
        other => panic!("Expected McpServer with tool, got {:?}", other),
    }
}
