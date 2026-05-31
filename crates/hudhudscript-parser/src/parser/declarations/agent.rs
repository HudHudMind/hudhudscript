//! Agent declaration parsing
//!
//! Handles parsing of agent declarations for MCP integration.
//! Issue #435: tools removed from agent — agents no longer own tools directly.
//! Issue #439: permission block support added to agent body.

use hudhudscript_ast::{AgentActionDecl, Decl, Expr, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_block, parse_expression};
use crate::pest_parser::Rule;

/// Parse an agent declaration
///
/// Syntax: `agent AgentName { field: value, ..., action name() { ... } }`
pub fn parse_agent_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected agent name", span))?
        .as_str()
        .to_string();

    let body = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected agent body", span))?;

    let (fields, actions) = parse_agent_body(body)?;

    // Store actions for later emission by the parse loop
    for action in actions {
        EXTRA_DECLARATIONS.with(|extras| {
            extras.borrow_mut().push(Stmt::Decl(Decl::AgentAction {
                agent_name: name.clone(),
                name: action.name,
                params: action.params,
                body: action.body,
                is_async: action.is_async,
                span: action.span,
            }));
        });
    }

    Ok(Stmt::Decl(Decl::Agent { name, fields, span }))
}

thread_local! {
    pub(crate) static EXTRA_DECLARATIONS: std::cell::RefCell<Vec<Stmt>> = std::cell::RefCell::new(Vec::new());
}

/// Parse agent body (field: value pairs and permission blocks)
///
/// Issue #439: permission blocks are stored as a "permission" field containing
/// an object with allow/deny/dangerous arrays.
pub fn parse_agent_body(pair: Pair<Rule>) -> ParseResult<(Vec<(String, Expr)>, Vec<AgentActionDecl>)> {
    let mut fields = Vec::new();
    let mut actions = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::agent_member => {
                // agent_member wraps either permission_block or agent_field
                for member_inner in inner_pair.into_inner() {
                    match member_inner.as_rule() {
                        Rule::permission_block => {
                            let perm_span = pair_to_span(&member_inner);
                            let perm_fields = parse_permission_block(member_inner)?;
                            fields.push((
                                "permission".to_string(),
                                Expr::Object {
                                    properties: perm_fields,
                                    span: perm_span,
                                },
                            ));
                        }
                        Rule::agent_field => {
                            let field_span = pair_to_span(&member_inner);
                            let mut field_inner = member_inner.into_inner();
                            let key = field_inner.next().unwrap().as_str().to_string();
                            if key == "tools" {
                                return Err(parse_codes::invalid_syntax(
                                    "Issue #435: 'tools' field removed from agent declarations. Use MCP server tool definitions instead.",
                                    field_span,
                                ));
                            }
                            let value = parse_expression(field_inner.next().unwrap())?;
                            fields.push((key, value));
                        }
                        Rule::agent_action_decl => {
                            actions.push(parse_action_decl(member_inner)?);
                        }
                        _ => {}
                    }
                }
            }
            // Backward compat: direct agent_field (shouldn't occur with new grammar but safe)
            Rule::agent_field => {
                let mut field_inner = inner_pair.into_inner();
                let key = field_inner.next().unwrap().as_str().to_string();
                let value = parse_expression(field_inner.next().unwrap())?;
                fields.push((key, value));
            }
            _ => {}
        }
    }

    Ok((fields, actions))
}

/// Parse a permission block: permission { allow: [...], deny: [...], dangerous: [...] }
fn parse_permission_block(pair: Pair<Rule>) -> ParseResult<Vec<(String, Expr)>> {
    let mut fields = Vec::new();

    for inner_pair in pair.into_inner() {
        if let Rule::permission_field = inner_pair.as_rule() {
            let mut field_inner = inner_pair.into_inner();
            let key = field_inner.next().unwrap().as_str().to_string();
            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(fields)
}

/// Parse an agent action declaration from action_decl rule
fn parse_action_decl(pair: Pair<Rule>) -> ParseResult<AgentActionDecl> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected action name", span))?
        .as_str()
        .to_string();

    // Parse parameters (func_param list)
    let mut params = Vec::new();
    let mut is_async = false;

    for item in inner {
        match item.as_rule() {
            Rule::func_param => {
                // Extract param name from identifier inside func_param
                for param_inner in item.into_inner() {
                    if param_inner.as_rule() == Rule::identifier {
                        params.push(param_inner.as_str().to_string());
                    }
                }
            }
            Rule::async_kw => {
                is_async = true;
            }
            Rule::block => {
                let block_stmt = parse_block(item)?;
                if let Stmt::Block { statements: body_stmts, .. } = block_stmt {
                    return Ok(AgentActionDecl {
                        name,
                        params,
                        body: body_stmts,
                        is_async,
                        span,
                    });
                }
            }
            _ => {}
        }
    }

    Err(parse_codes::invalid_syntax("Expected action body block", span))
}
