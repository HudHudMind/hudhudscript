use super::*;

pub(super) fn parse_unary_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);

    // Check the full text first to see if it starts with await
    let full_text = pair.as_str().trim_start();
    let is_await = full_text.starts_with("await ")
        || full_text.starts_with("bekle ")
        || full_text.starts_with("matsu ");

    // Recursively parse nested expressions
    let mut inner = pair.into_inner();
    if let Some(first) = inner.next() {
        let first_str = first.as_str();

        if first.as_rule() == Rule::recall_expr {
            return parse_recall_expr(first);
        }

        if first.as_rule() == Rule::perform_expr {
            let mut pinner = first.into_inner();
            let action = Box::new(parse_expression(pinner.next().unwrap())?);
            return Ok(Expr::Perform { action, span });
        }

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
        match first.as_rule() {
            Rule::recall_expr => parse_recall_expr(first),
            Rule::postfix_expr => parse_postfix_expr(first),
            _ => parse_expression(first),
        }
    } else {
        Err(parse_codes::invalid_syntax("Empty expression", span))
    }
}

pub(super) fn parse_recall_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let query = Box::new(parse_expression(inner.next().ok_or_else(|| {
        parse_codes::invalid_syntax("Expected query expression in recall", span)
    })?)?);

    let store_name = inner.next().map(|p| p.as_str().to_string());

    Ok(Expr::Recall {
        query,
        store_name,
        span,
    })
}

pub(super) fn parse_postfix_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    if pair.as_rule() != Rule::postfix_expr {
        return parse_expression(pair);
    }
    let mut inner = pair.into_inner();

    // Parse the primary expression first
    let mut expr = parse_expression(
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

    if pair.as_rule() != Rule::primary {
        return parse_expression(pair);
    }

    let mut inner = pair.into_inner();
    if let Some(first) = inner.next() {
        parse_expression(first)
    } else {
        Err(parse_codes::invalid_syntax("Empty primary", span))
    }
}
