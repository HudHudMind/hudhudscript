use super::*;

pub(super) fn parse_unary_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);

    // Check the full text first to see if it starts with await or perform
    let full_text = pair.as_str().trim_start();
    let is_await = full_text.starts_with("await ")
        || full_text.starts_with("bekle ")
        || full_text.starts_with("matsu ");
    let is_perform = full_text.starts_with("perform ");

    // Recursively parse nested expressions
    let mut inner = pair.into_inner();
    if let Some(first) = inner.next() {
        let first_str = first.as_str();

        // If we detected await from the full text, this first element is the inner unary_expr
        // which may itself contain postfix operations (e.g. `await fetchData()` where
        // `fetchData()` is a unary_expr → postfix_expr → primary + call_op).
        // We must recurse into parse_unary_expr (not parse_postfix_expr) so that the
        // inner pair is unwrapped correctly and call/member/index suffixes are preserved.
        if is_await {
            let expr = parse_unary_expr(first)?;
            return Ok(Expr::Await {
                expr: Box::new(expr),
                span,
            });
        }

        if is_perform {
            let expr = parse_unary_expr(first)?;
            return Ok(Expr::Perform {
                action: Box::new(expr),
                span,
            });
        }

        // Check if this is a unary operator (!, -, or +)
        if first_str == "!" || first_str == "-" || first_str == "+" {
            let op = if first_str == "!" {
                UnaryOp::Not
            } else if first_str == "-" {
                UnaryOp::Neg
            } else {
                UnaryOp::Plus
            };

            if let Some(expr_pair) = inner.next() {
                let expr = parse_unary_expr(expr_pair)?; // Recursive for chained unary ops
                return Ok(Expr::Unary {
                    op,
                    expr: Box::new(expr),
                    span,
                });
            }
        }

        // Otherwise, just parse as postfix expression
        parse_postfix_expr(first)
    } else {
        Err(parse_codes::invalid_syntax("Empty expression", span))
    }
}

pub(super) fn parse_postfix_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // Parse the primary expression first
    let mut expr = parse_primary(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected primary expression", span))?,
    )?;

    // Process postfix operations (call, member, index)
    for postfix_pair in inner {
        match postfix_pair.as_rule() {
            Rule::postfix_op => {
                // Unwrap the postfix_op to get the actual operation
                let op_pair = postfix_pair.into_inner().next().ok_or_else(|| {
                    parse_codes::invalid_syntax("Expected postfix operation", span)
                })?;

                match op_pair.as_rule() {
                    Rule::call_op => {
                        // Function call: foo()
                        let mut args = Vec::new();
                        for arg_pair in op_pair.into_inner() {
                            match arg_pair.as_rule() {
                                Rule::call_arg => {
                                    let child = arg_pair.into_inner().next().ok_or_else(|| {
                                        parse_codes::invalid_syntax(
                                            "Expected expression in call argument",
                                            span,
                                        )
                                    })?;
                                    match child.as_rule() {
                                        Rule::spread_expr => {
                                            let spread_inner =
                                                child.into_inner().next().ok_or_else(|| {
                                                    parse_codes::invalid_syntax(
                                                        "Expected expression after spread operator",
                                                        span,
                                                    )
                                                })?;
                                            let inner_expr = parse_expression(spread_inner)?;
                                            args.push(Expr::Spread {
                                                expr: Box::new(inner_expr),
                                                span,
                                            });
                                        }
                                        _ => {
                                            args.push(parse_expression(child)?);
                                        }
                                    }
                                }
                                _ => {
                                    args.push(parse_expression(arg_pair)?);
                                }
                            }
                        }
                        expr = Expr::Call {
                            callee: Box::new(expr),
                            args,
                            span,
                        };
                    }
                    Rule::optional_member_op => {
                        // Optional chaining: foo?.bar
                        let property = op_pair
                            .into_inner()
                            .next()
                            .ok_or_else(|| {
                                parse_codes::invalid_syntax("Expected property name", span)
                            })?
                            .as_str()
                            .to_string();
                        expr = Expr::OptionalMember {
                            object: Box::new(expr),
                            property,
                            span,
                        };
                    }
                    Rule::member_op => {
                        // Member access: foo.bar
                        let property = op_pair
                            .into_inner()
                            .next()
                            .ok_or_else(|| {
                                parse_codes::invalid_syntax("Expected property name", span)
                            })?
                            .as_str()
                            .to_string();
                        expr = Expr::Member {
                            object: Box::new(expr),
                            property,
                            span,
                        };
                    }
                    Rule::index_op => {
                        // Index access: foo[0]
                        let index =
                            parse_expression(op_pair.into_inner().next().ok_or_else(|| {
                                parse_codes::invalid_syntax("Expected index expression", span)
                            })?)?;
                        expr = Expr::Index {
                            object: Box::new(expr),
                            index: Box::new(index),
                            span,
                        };
                    }
                    Rule::increment_op => {
                        expr = Expr::Unary {
                            op: UnaryOp::PostIncrement,
                            expr: Box::new(expr),
                            span,
                        };
                    }
                    Rule::decrement_op => {
                        expr = Expr::Unary {
                            op: UnaryOp::PostDecrement,
                            expr: Box::new(expr),
                            span,
                        };
                    }
                    _ => {
                        return Err(parse_codes::invalid_syntax(
                            "Unknown postfix operation",
                            span,
                        ))
                    }
                }
            }
            _ => {
                return Err(parse_codes::invalid_syntax(
                    "Expected postfix operation",
                    span,
                ))
            }
        }
    }

    Ok(expr)
}

pub(super) fn parse_primary(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);

    // Primary can contain: number, string, boolean, null, identifier, array, object, arrow_function, template_string, or (expression)
    // We need to check if this is a parenthesized expression or a direct value
    let mut inner = pair.into_inner();
    if let Some(first) = inner.next() {
        // If the first child is an expression, it means we have "(" ~ expression ~ ")"
        // Otherwise, it's a direct value that should be parsed by parse_expression routing
        match first.as_rule() {
            Rule::expression => {
                // Parenthesized expression: (expr)
                parse_expression(first)
            }
            _ => {
                // Direct value: number, string, identifier, etc.
                // These are already handled by parse_expression routing
                parse_expression(first)
            }
        }
    } else {
        Err(parse_codes::invalid_syntax("Empty primary", span))
    }
}

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
                                // Parse escape sequences properly
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
                        // Parse escape sequences properly
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
