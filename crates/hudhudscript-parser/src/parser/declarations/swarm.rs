//! Swarm declaration parsing

use hudhudscript_ast::{Decl, Expr, Literal, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

/// Parse a swarm declaration
pub fn parse_swarm_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected swarm name", span))?
        .as_str()
        .to_string();

    let body = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected swarm body", span))?;

    let fields = parse_swarm_body(body)?;

    // Extract fields
    let mut agents = Vec::new();
    let mut strategy = String::from("parallel");

    for (key, value) in fields {
        match key.as_str() {
            "agents" => {
                if let Expr::Array {
                    elements: agent_exprs,
                    ..
                } = value
                {
                    for agent_expr in agent_exprs {
                        if let Expr::Literal(Literal::String(s), _) = &agent_expr {
                            agents.push(s.clone());
                        } else if let Expr::Identifier(s, _) = &agent_expr {
                            agents.push(s.clone());
                        }
                    }
                }
            }
            "strategy" => {
                if let Expr::Identifier(s, _) = value {
                    strategy = s;
                } else if let Expr::Literal(Literal::String(s), _) = value {
                    strategy = s;
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Swarm {
        name,
        agents,
        strategy,
        span,
    }))
}

fn parse_swarm_body(pair: Pair<Rule>) -> ParseResult<Vec<(String, Expr)>> {
    let mut fields = Vec::new();

    for inner_pair in pair.into_inner() {
        if let Rule::swarm_field = inner_pair.as_rule() {
            let mut field_inner = inner_pair.into_inner();
            let key = field_inner.next().unwrap().as_str().to_string();
            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(fields)
}
