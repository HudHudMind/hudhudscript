use hudhudscript_ast::{Decl, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::pair_to_span;
use crate::pest_parser::Rule;

/// Parse a council declaration
pub fn parse_council_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected council name", span))?
        .as_str()
        .to_string();

    let body = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected council body", span))?;

    // council_decl now uses agent_body rule — parse with agent field parser
    let (fields, _actions) = super::super::agent::parse_agent_body(body)?;

    let mut constitution_id = String::new();
    let mut members_vec = Vec::new();
    let mut rules_vec = Vec::new();

    for (key, value) in &fields {
        match key.as_str() {
            "constitution" => {
                if let hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) =
                    value
                {
                    constitution_id = s.clone();
                } else if let hudhudscript_ast::Expr::Identifier(s, _) = value {
                    constitution_id = s.clone();
                }
            }
            "members" => {
                if let hudhudscript_ast::Expr::Array { elements, .. } = value {
                    for member_expr in elements {
                        if let hudhudscript_ast::Expr::Object { properties, .. } = member_expr {
                            let mut agent_id = String::new();
                            let mut role = String::new();
                            for (mk, mv) in properties {
                                if mk == "agent" {
                                    if let hudhudscript_ast::Expr::Literal(
                                        hudhudscript_ast::Literal::String(s),
                                        _,
                                    ) = mv
                                    {
                                        agent_id = s.clone();
                                    } else if let hudhudscript_ast::Expr::Identifier(s, _) = mv {
                                        agent_id = s.clone();
                                    }
                                } else if mk == "role" {
                                    if let hudhudscript_ast::Expr::Literal(
                                        hudhudscript_ast::Literal::String(s),
                                        _,
                                    ) = mv
                                    {
                                        role = s.clone();
                                    } else if let hudhudscript_ast::Expr::Identifier(s, _) = mv {
                                        role = s.clone();
                                    }
                                }
                            }
                            members_vec.push(hudhudscript_ast::CouncilMemberDecl {
                                agent_id,
                                role,
                                span,
                            });
                        }
                    }
                }
            }
            "rules" => {
                if let hudhudscript_ast::Expr::Array { elements, .. } = value {
                    for rule_expr in elements {
                        if let hudhudscript_ast::Expr::Literal(
                            hudhudscript_ast::Literal::String(s),
                            _,
                        ) = rule_expr
                        {
                            rules_vec.push(s.clone());
                        } else if let hudhudscript_ast::Expr::Identifier(s, _) = rule_expr {
                            rules_vec.push(s.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Council {
        name,
        constitution: constitution_id,
        members: members_vec,
        rules: rules_vec,
        span,
    }))
}
