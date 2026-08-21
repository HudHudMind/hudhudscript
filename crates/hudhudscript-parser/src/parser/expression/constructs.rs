//! Expression construct parsing (new, spawn, view, array, object)

use super::*;

pub(super) fn parse_new_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // Skip the 'new' keyword token (new_kw_en is atomic so it appears as a child).
    // The next child is the class name identifier.
    let first = inner.next();
    let class_name = if first
        .as_ref()
        .map_or(false, |f| f.as_rule() == Rule::new_kw_en)
    {
        inner.next()
    } else {
        first
    }
    .ok_or_else(|| parse_codes::invalid_syntax("Expected class name after 'new'", span))?
    .as_str()
    .to_string();

    // Remaining children are the arguments
    let mut args = Vec::new();
    for arg_pair in inner {
        args.push(parse_expression(arg_pair)?);
    }

    Ok(Expr::New {
        class_name,
        args,
        span,
    })
}

pub(super) fn parse_spawn_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // First child is the subject name (identifier)
    let subject_name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected subject name after 'spawn'", span))?
        .as_str()
        .to_string();

    // Remaining children are the arguments
    let mut args = Vec::new();
    for arg_pair in inner {
        if arg_pair.as_rule() == Rule::expression {
            args.push(parse_expression(arg_pair)?);
        }
    }

    Ok(Expr::Spawn {
        subject_name,
        args,
        span,
    })
}

/// SOP0008: parse view expr as ViewType
pub(super) fn parse_view_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let instance = parse_expression(inner.next().unwrap())?;
    let view_name = inner
        .next()
        .map(|p| p.as_str().to_string())
        .unwrap_or_default();
    Ok(Expr::ViewAs {
        instance: Box::new(instance),
        view_name,
        span,
    })
}

pub(super) fn parse_array(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut elements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::array_element => {
                let span = pair_to_span(&inner_pair);
                let child = inner_pair.into_inner().next().ok_or_else(|| {
                    parse_codes::invalid_syntax("Expected expression in array element", span)
                })?;
                match child.as_rule() {
                    Rule::spread_expr => {
                        let spread_inner = child.into_inner().next().ok_or_else(|| {
                            parse_codes::invalid_syntax("Expected expression in spread", span)
                        })?;
                        let expr = parse_expression(spread_inner)?;
                        elements.push(Expr::Spread {
                            expr: Box::new(expr),
                            span,
                        });
                    }
                    _ => {
                        elements.push(parse_expression(child)?);
                    }
                }
            }
            _ => {
                elements.push(parse_expression(inner_pair)?);
            }
        }
    }

    Ok(Expr::Array { elements, span })
}

pub(super) fn parse_object(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut properties = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::object_entry => {
                let entry_span = pair_to_span(&inner_pair);
                let child = inner_pair.into_inner().next().ok_or_else(|| {
                    parse_codes::invalid_syntax("Expected object entry content", entry_span)
                })?;
                match child.as_rule() {
                    Rule::object_spread => {
                        let spread_inner = child.into_inner().next().ok_or_else(|| {
                            parse_codes::invalid_syntax(
                                "Expected expression in object spread",
                                entry_span,
                            )
                        })?;
                        let expr = parse_expression(spread_inner)?;
                        // Use special key "__spread__N" for spread entries
                        properties.push((
                            format!("__spread__{}", properties.len()),
                            Expr::Spread {
                                expr: Box::new(expr),
                                span,
                            },
                        ));
                    }
                    Rule::object_pair => {
                        let mut pair_inner = child.into_inner();
                        let key_pair = pair_inner.next().ok_or_else(|| {
                            parse_codes::invalid_syntax("Expected object key", span)
                        })?;
                        let key = match key_pair.as_rule() {
                            Rule::identifier => key_pair.as_str().to_string(),
                            Rule::string => {
                                let s = key_pair.as_str();
                                let raw = &s[1..s.len() - 1];
                                parse_escape_sequences(raw)
                            }
                            _ => {
                                return Err(parse_codes::invalid_syntax("Invalid object key", span))
                            }
                        };
                        let value_expr = pair_inner.next().ok_or_else(|| {
                            parse_codes::invalid_syntax(
                                "Expected object value expression",
                                entry_span,
                            )
                        })?;
                        let value = parse_expression(value_expr)?;
                        properties.push((key, value));
                    }
                    _ => {}
                }
            }
            Rule::object_pair => {
                let pair_span = pair_to_span(&inner_pair);
                let mut pair_inner = inner_pair.into_inner();
                let key_pair = pair_inner
                    .next()
                    .ok_or_else(|| parse_codes::invalid_syntax("Expected object key", pair_span))?;
                let key = match key_pair.as_rule() {
                    Rule::identifier => key_pair.as_str().to_string(),
                    Rule::string => {
                        let s = key_pair.as_str();
                        let raw = &s[1..s.len() - 1];
                        parse_escape_sequences(raw)
                    }
                    _ => return Err(parse_codes::invalid_syntax("Invalid object key", span)),
                };
                let value =
                    parse_expression(pair_inner.next().ok_or_else(|| {
                        parse_codes::invalid_syntax("Expected object value", span)
                    })?)?;
                properties.push((key, value));
            }
            _ => {}
        }
    }

    Ok(Expr::Object { properties, span })
}
