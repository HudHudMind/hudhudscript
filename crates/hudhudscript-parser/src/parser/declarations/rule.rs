//! Rule declaration parsing
//!
//! Handles parsing of rule declarations with conditions and actions.

use hudhudscript_ast::{ActionDecl, ConditionDecl, Decl, Expr, Literal, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

/// Parse a rule declaration
pub fn parse_rule_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected rule name", span))?
        .as_str()
        .to_string();

    let body = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected rule body", span))?;

    let fields = parse_rule_body(body)?;

    // Extract fields
    let mut conditions = Vec::new();
    let mut actions = Vec::new();
    let mut priority = 0;

    for (key, value) in fields {
        match key.as_str() {
            "conditions" => {
                if let Expr::Array {
                    elements: cond_exprs,
                    ..
                } = value
                {
                    for cond_expr in cond_exprs {
                        if let Expr::Object {
                            properties: cond_fields,
                            span: cond_span,
                        } = cond_expr
                        {
                            conditions.push(parse_condition_from_fields(cond_fields, cond_span)?);
                        }
                    }
                }
            }
            "actions" => {
                if let Expr::Array {
                    elements: action_exprs,
                    ..
                } = value
                {
                    for action_expr in action_exprs {
                        if let Expr::Object {
                            properties: action_fields,
                            span: action_span,
                        } = action_expr
                        {
                            actions.push(parse_action_from_fields(action_fields, action_span)?);
                        }
                    }
                }
            }
            "priority" => {
                if let Expr::Literal(lit, _) = value {
                    let n = match lit {
                        Literal::Number(n, _) => n as u32,
                        Literal::Int(i) => i as u32,
                        Literal::BigInt(s) => s.parse::<u32>().unwrap_or(0),
                        _ => 0,
                    };
                    priority = n;
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Rule {
        name,
        conditions,
        actions,
        priority,
        span,
    }))
}

fn parse_rule_body(pair: Pair<Rule>) -> ParseResult<Vec<(String, Expr)>> {
    let mut fields = Vec::new();

    for inner_pair in pair.into_inner() {
        if let Rule::rule_field = inner_pair.as_rule() {
            let mut field_inner = inner_pair.into_inner();
            let key = field_inner.next().unwrap().as_str().to_string();
            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(fields)
}

fn parse_condition_from_fields(
    fields: Vec<(String, Expr)>,
    span: hudhudscript_ast::Span,
) -> ParseResult<ConditionDecl> {
    let mut condition_type = String::new();
    let mut field = String::new();
    let mut value = Expr::Literal(Literal::Null, span);

    for (key, val) in fields {
        match key.as_str() {
            "type" => {
                if let Expr::Literal(Literal::String(s), _) = val {
                    condition_type = s;
                }
            }
            "field" => {
                if let Expr::Literal(Literal::String(s), _) = val {
                    field = s;
                }
            }
            "value" => {
                value = val;
            }
            _ => {}
        }
    }

    Ok(ConditionDecl {
        condition_type,
        field,
        value,
        span,
    })
}

fn parse_action_from_fields(
    fields: Vec<(String, Expr)>,
    span: hudhudscript_ast::Span,
) -> ParseResult<ActionDecl> {
    let mut action_type = String::new();
    let mut params = Vec::new();

    for (key, val) in fields {
        match key.as_str() {
            "type" => {
                if let Expr::Literal(Literal::String(s), _) = val {
                    action_type = s;
                }
            }
            _ => {
                params.push((key, val));
            }
        }
    }

    Ok(ActionDecl {
        action_type,
        params,
        span,
    })
}
