use hudhudscript_ast::{Decl, Expr, Literal, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

use super::{normalize_execution, normalize_governance};

/// Parse a protocol declaration (formerly strategy)
pub fn parse_protocol_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected strategy name", span))?
        .as_str()
        .to_string();

    let body = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected strategy body", span))?;

    let mut execution: Option<String> = None;
    let mut governance: Option<String> = None;
    let mut timeout: Option<f64> = None;
    let mut session: Vec<(String, Expr)> = Vec::new();

    for field_pair in body.into_inner() {
        match field_pair.as_rule() {
            Rule::strategy_session_field => {
                parse_session_hooks(field_pair, &mut session)?;
            }
            Rule::strategy_field => {
                let mut field_inner = field_pair.into_inner();
                let first = match field_inner.next() {
                    Some(p) => p,
                    None => continue,
                };

                if first.as_rule() == Rule::strategy_session_field {
                    parse_session_hooks(first, &mut session)?;
                    continue;
                }

                let key = first.as_str().to_string();
                let value_pair = match field_inner.next() {
                    Some(p) => p,
                    None => continue,
                };
                let value = parse_expression(value_pair)?;

                match key.as_str() {
                    "execution" | "yürütme" => {
                        if let Expr::Identifier(s, _) = &value {
                            execution = Some(normalize_execution(s));
                        } else if let Expr::Literal(Literal::String(s), _) = &value {
                            execution = Some(normalize_execution(s));
                        }
                    }
                    "governance" | "yönetim" => {
                        if let Expr::Identifier(s, _) = &value {
                            governance = Some(normalize_governance(s));
                        } else if let Expr::Literal(Literal::String(s), _) = &value {
                            governance = Some(normalize_governance(s));
                        }
                    }
                    "timeout" | "zamanAsimi" => {
                        if let Expr::Literal(Literal::Number(n, _), _) = &value {
                            timeout = Some(*n);
                        }
                    }
                    "voting" | "oylama" => {}
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Protocol {
        name,
        execution,
        governance,
        timeout,
        session,
        span,
    }))
}

fn parse_session_hooks(pair: Pair<Rule>, session: &mut Vec<(String, Expr)>) -> ParseResult<()> {
    for hook_pair in pair.into_inner() {
        if let Rule::session_hook = hook_pair.as_rule() {
            let mut hook_inner = hook_pair.into_inner();
            let hook_name = match hook_inner.next() {
                Some(p) => p.as_str().to_string(),
                None => continue,
            };
            let hook_val_pair = match hook_inner.next() {
                Some(p) => p,
                None => continue,
            };
            let hook_expr = parse_expression(hook_val_pair)?;
            let canonical = match hook_name.as_str() {
                "başlangıçta" => "onStart",
                "üyeKonuşmadan" => "onMemberStart",
                "üyeTamamladığında" => "onMemberComplete",
                "oylamada" => "onVote",
                "tamamlandığında" => "onComplete",
                "hatada" => "onError",
                other => other,
            };
            session.push((canonical.to_string(), hook_expr));
        }
    }
    Ok(())
}
