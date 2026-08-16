use super::*;

pub fn parse_assignment_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let target = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected assignment target", span))?,
    )?;

    let op_str = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected assignment operator", span))?
        .as_str()
        .to_string();

    let right = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", span))?,
    )?;

    // Desugar compound assignment: i += j → i = i + j
    if op_str == "=" {
        return Ok(Stmt::Assignment {
            target,
            value: right,
            span,
        });
    }
    let binop = match op_str.as_str() {
        "+=" => hudhudscript_ast::BinaryOp::Add,
        "-=" => hudhudscript_ast::BinaryOp::Sub,
        "*=" => hudhudscript_ast::BinaryOp::Mul,
        "/=" => hudhudscript_ast::BinaryOp::Div,
        "%=" => hudhudscript_ast::BinaryOp::Mod,
        _ => {
            return Err(parse_codes::invalid_syntax(
                &format!("Unknown assignment operator: {}", op_str),
                span,
            ))
        }
    };
    let value = hudhudscript_ast::Expr::Binary {
        left: Box::new(target.clone()),
        op: binop,
        right: Box::new(right),
        span,
    };
    Ok(Stmt::Assignment {
        target,
        value,
        span,
    })
}

pub fn parse_return_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let value = if let Some(expr_pair) = inner.next() {
        Some(parse_expression(expr_pair)?)
    } else {
        None
    };

    Ok(Stmt::Return { value, span })
}

pub fn parse_break_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    Ok(Stmt::Break { span })
}

pub fn parse_continue_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    Ok(Stmt::Continue { span })
}

pub fn parse_if_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // Parse main if condition and block
    let condition = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected condition", span))?,
    )?;

    let then_branch = parse_block_or_stmt(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected then branch", span))?,
    )?;

    // Parse else if and else branches
    let mut else_branch: Option<Box<Stmt>> = None;
    let mut else_if_chain: Vec<(hudhudscript_ast::Expr, Stmt)> = Vec::new();

    // Collect all remaining pairs
    let remaining: Vec<_> = inner.collect();
    let mut i = 0;

    while i < remaining.len() {
        let pair = &remaining[i];

        // Check if this is an else if condition
        if pair.as_rule() == Rule::expression {
            // This is an else if condition
            let else_if_condition = parse_expression(pair.clone())?;
            i += 1;

            if i < remaining.len() {
                let else_if_body = parse_block_or_stmt(remaining[i].clone())?;
                else_if_chain.push((else_if_condition, else_if_body));
                i += 1;
            }
        } else {
            // This is the final else body (block or single statement)
            else_branch = Some(Box::new(parse_block_or_stmt(pair.clone())?));
            i += 1;
        }
    }

    // Build the else if chain from bottom to top
    let mut current_else = else_branch;

    for (else_if_cond, else_if_block) in else_if_chain.into_iter().rev() {
        current_else = Some(Box::new(Stmt::If {
            condition: else_if_cond,
            then_branch: Box::new(else_if_block),
            else_branch: current_else,
            span,
        }));
    }

    Ok(Stmt::If {
        condition,
        then_branch: Box::new(then_branch),
        else_branch: current_else,
        span,
    })
}

pub fn parse_while_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let condition = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected condition", span))?,
    )?;

    let body = parse_block(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected body", span))?,
    )?;

    Ok(Stmt::While {
        condition,
        body: Box::new(body),
        span,
    })
}

pub fn parse_for_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // The grammar now directly contains the for loop parts
    // We need to check if we have for_init (C-style) or identifier (for-in)
    let first = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected for loop content", span))?;

    match first.as_rule() {
        //         Rule::for_init => {
        //             // C-style for loop
        //             parse_c_style_from_parts(first, &mut inner, span)
        //         }
        Rule::identifier => {
            // For-in loop
            let variable = first.as_str().to_string();

            let iterable = parse_expression(
                inner
                    .next()
                    .ok_or_else(|| parse_codes::invalid_syntax("Expected iterable", span))?,
            )?;

            let body = parse_block(
                inner
                    .next()
                    .ok_or_else(|| parse_codes::invalid_syntax("Expected body", span))?,
            )?;

            Ok(Stmt::For {
                variable,
                iterable,
                body: Box::new(body),
                span,
            })
        }
        Rule::postfix_expr => {
            // herbir loop: herbir (array içinde item)
            // First element is the iterable (postfix_expr)
            let iterable = parse_expression(first)?;

            // Next should be identifier (the loop variable)
            let variable_pair = inner
                .next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected loop variable", span))?;
            let variable = variable_pair.as_str().to_string();

            let body = parse_block(
                inner
                    .next()
                    .ok_or_else(|| parse_codes::invalid_syntax("Expected body", span))?,
            )?;

            Ok(Stmt::For {
                variable,
                iterable,
                body: Box::new(body),
                span,
            })
        }
        Rule::expression => {
            //             // C-style for loop with no init, starts with condition
            //             parse_c_style_from_condition(first, &mut inner, span)
            //         }
            //         Rule::for_update => {
            //             // C-style for loop with no init and no condition, starts with update
            //             parse_c_style_from_update(first, &mut inner, span)
            //         }
            //         Rule::block => {
            // C-style for loop with no init, no condition, no update - just body
            Ok(Stmt::ForCStyle {
                init: None,
                condition: None,
                update: None,
                body: Box::new(parse_block(first)?),
                span,
            })
        }
        _ => Err(parse_codes::invalid_syntax("Unknown for loop type", span)),
    }
}

/// Parse range-based for loop: for(start, stop) or for(start, stop, step)
/// Turkish: döngü(0, 100) / döngü(0, 100, -1)
/// Default step is +1 if start < stop, -1 if start > stop
pub fn parse_for_range_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let start = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected start expression", span))?,
    )?;

    let stop = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected stop expression", span))?,
    )?;

    let step = if let Some(step_pair) = inner.next() {
        // Could be the step expression or the block
        if step_pair.as_rule() == Rule::block {
            // No step provided, this is the body
            let body = parse_block(step_pair)?;
            return Ok(Stmt::ForRange {
                start,
                stop,
                step: None,
                body: Box::new(body),
                span,
            });
        } else {
            Some(parse_expression(step_pair)?)
        }
    } else {
        None
    };

    let body = parse_block(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected body block", span))?,
    )?;

    Ok(Stmt::ForRange {
        start,
        stop,
        step,
        body: Box::new(body),
        span,
    })
}
