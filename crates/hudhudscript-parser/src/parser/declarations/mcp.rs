//! MCP server declaration parsing — Issue #437
//!
//! Handles parsing of MCP server declarations with nested tool definitions.
//!
//! Syntax:
//! ```text
//! mcp ServerName {
//!     transport: "stdio"
//!     command: "npx"
//!     tool toolName(param: type) { body }
//! }
//! ```

use hudhudscript_ast::{McpServerDecl, McpToolDef, ServerConfig, Stmt, TransportType};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_block, parse_expression};
use crate::pest_parser::Rule;

/// Parse an MCP server declaration
pub fn parse_mcp_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected MCP server name", span))?
        .as_str()
        .to_string();

    let mut fields = Vec::new();
    let mut tools = Vec::new();

    for member_pair in inner {
        if let Rule::mcp_member = member_pair.as_rule() {
            for member_inner in member_pair.into_inner() {
                match member_inner.as_rule() {
                    Rule::mcp_field => {
                        let mut field_inner = member_inner.into_inner();
                        let key = field_inner.next().unwrap().as_str().to_string();
                        let value = parse_expression(field_inner.next().unwrap())?;
                        fields.push((key, value));
                    }
                    Rule::mcp_tool_decl => {
                        let tool_def = parse_mcp_tool_decl(member_inner)?;
                        tools.push(tool_def);
                    }
                    _ => {}
                }
            }
        }
    }

    // Build a default ServerConfig (transport/command can be overridden at runtime from fields)
    let config = ServerConfig {
        transport: TransportType::Stdio,
        command: None,
        args: vec![],
        url: None,
        auth: None,
    };

    Ok(Stmt::McpServer(McpServerDecl {
        name,
        config,
        fields,
        tools,
        span,
    }))
}

/// Parse a tool definition inside an MCP server block
fn parse_mcp_tool_decl(pair: Pair<Rule>) -> ParseResult<McpToolDef> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected tool name", span))?
        .as_str()
        .to_string();

    let mut params = Vec::new();

    // Collect params and block
    let mut body_stmts = Vec::new();
    for p in inner {
        match p.as_rule() {
            Rule::mcp_tool_param => {
                let mut param_inner = p.into_inner();
                let param_name = param_inner.next().unwrap().as_str().to_string();
                let param_type = param_inner.next().unwrap().as_str().to_string();
                params.push((param_name, param_type));
            }
            Rule::block => {
                // parse_block returns Stmt::Block { statements, .. }
                let block_stmt = parse_block(p)?;
                if let Stmt::Block { statements, .. } = block_stmt {
                    body_stmts = statements;
                }
            }
            _ => {}
        }
    }

    Ok(McpToolDef {
        name,
        params,
        body: body_stmts,
        span,
    })
}
