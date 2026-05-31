use super::*;

/// Parse C-style for loop: for (init; condition; update) { body }
pub fn parse_for_c_style_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let inner = pair.into_inner();

    let mut init: Option<Box<Stmt>> = None;
    let mut condition: Option<Expr> = None;
    let mut update: Option<Box<Stmt>> = None;
    let mut body: Option<Box<Stmt>> = None;

    for item in inner {
        match item.as_rule() {
            Rule::for_c_init => {
                let init_span = pair_to_span(&item);
                let mut init_inner = item.into_inner();
                let first = init_inner.next().ok_or_else(|| {
                    parse_codes::invalid_syntax("Expected for_c_init content", init_span)
                })?;

                match first.as_rule() {
                    Rule::identifier => {
                        // let/var identifier = expression
                        let name = first.as_str().to_string();
                        let value = parse_expression(init_inner.next().ok_or_else(|| {
                            parse_codes::invalid_syntax("Expected expression", init_span)
                        })?)?;
                        init = Some(Box::new(Stmt::Let {
                            name,
                            value,
                            span: init_span,
                        }));
                    }
                    Rule::postfix_expr => {
                        // assignment: postfix_expr = expression
                        let target = parse_expression(first)?;
                        let value = parse_expression(init_inner.next().ok_or_else(|| {
                            parse_codes::invalid_syntax("Expected expression", init_span)
                        })?)?;
                        init = Some(Box::new(Stmt::Assignment {
                            target,
                            value,
                            span: init_span,
                        }));
                    }
                    _ => {}
                }
            }
            Rule::expression => {
                condition = Some(parse_expression(item)?);
            }
            Rule::for_c_update => {
                let update_span = pair_to_span(&item);
                let mut update_inner = item.into_inner();
                let first = update_inner.next().ok_or_else(|| {
                    parse_codes::invalid_syntax("Expected identifier or expression", update_span)
                })?;
                // i++/i--: desugar to i = i + 1 / i = i - 1
                if first.as_rule() == Rule::identifier {
                    let name = first.as_str().to_string();
                    let op_pair = update_inner.next().ok_or_else(|| {
                        parse_codes::invalid_syntax("Expected ++ or --", update_span)
                    })?;
                    let is_inc = op_pair.as_rule() == Rule::increment_op;
                    let imm_literal = hudhudscript_ast::Expr::Literal(
                        hudhudscript_ast::Literal::Number(1.0, false), update_span);
                    let target_expr = hudhudscript_ast::Expr::Identifier(name.clone(), update_span);
                    update = Some(Box::new(Stmt::Assignment {
                        target: target_expr,
                        value: hudhudscript_ast::Expr::Binary {
                            left: Box::new(hudhudscript_ast::Expr::Identifier(name, update_span)),
                            op: if is_inc { hudhudscript_ast::BinaryOp::Add } else { hudhudscript_ast::BinaryOp::Sub },
                            right: Box::new(imm_literal),
                            span: update_span,
                        },
                        span: update_span,
                    }));
                } else {
                    // assignment: target = value (or +=, -= etc via desugar)
                    let op_pair = update_inner.next()
                        .ok_or_else(|| parse_codes::invalid_syntax("Expected assignment operator", update_span))?;
                    let op_str = op_pair.as_str();
                    let target = parse_expression(first)?;
                    let right = parse_expression(update_inner.next().ok_or_else(|| {
                        parse_codes::invalid_syntax("Expected expression", update_span)
                    })?)?;
                    let value = if op_str == "=" {
                        right
                    } else {
                        let binop = match op_str {
                            "+=" => hudhudscript_ast::BinaryOp::Add,
                            "-=" => hudhudscript_ast::BinaryOp::Sub,
                            "*=" => hudhudscript_ast::BinaryOp::Mul,
                            "/=" => hudhudscript_ast::BinaryOp::Div,
                            "%=" => hudhudscript_ast::BinaryOp::Mod,
                            _ => return Err(parse_codes::invalid_syntax(&format!("Unknown assignment operator: {}", op_str), update_span)),
                        };
                        hudhudscript_ast::Expr::Binary {
                            left: Box::new(target.clone()),
                            op: binop,
                            right: Box::new(right),
                            span: update_span,
                        }
                    };
                    update = Some(Box::new(Stmt::Assignment { target, value, span: update_span }));
                }
            }
            Rule::block => {
                body = Some(Box::new(parse_block(item)?));
            }
            _ => {}
        }
    }

    let body = body.ok_or_else(|| parse_codes::invalid_syntax("Expected body", span))?;

    Ok(Stmt::ForCStyle {
        init,
        condition,
        update,
        body,
        span,
    })
}

// fn parse_c_style_from_parts(init_pair: Pair<Rule>, inner: &mut pest::iterators::Pairs<Rule>, span: hudhudscript_ast::Span) -> ParseResult<Stmt> {
// Parse init - check if it's a declaration or assignment
//     let init_span = pair_to_span(&init_pair);
//     let mut init_inner = init_pair.into_inner();
//
//     let first_item = init_inner.next().ok_or_else(|| {
//         parse_codes::invalid_syntax("Expected for_init content", init_span.clone())
//     })?;
//
//     let init = match first_item.as_rule() {
//         Rule::for_init_decl => {
// Variable declaration: var/let identifier = expression
//             let mut decl_inner = first_item.into_inner();
//             let name_pair = decl_inner.next().ok_or_else(|| {
//                 parse_codes::invalid_syntax("Expected identifier", init_span.clone())
//             })?;
//             let name = name_pair.as_str().to_string();
//             let name_span = pair_to_span(&name_pair);
//
//             let value = parse_expression(decl_inner.next().ok_or_else(|| {
//                 parse_codes::invalid_syntax("Expected expression", init_span.clone())
//             })?)?;
//
//             Some(Box::new(Stmt::Let {
//                 name,
//                 value,
//                 span: init_span.clone(),
//             }))
//         }
//         Rule::for_init_assign => {
// Assignment: identifier = expression
//             let mut assign_inner = first_item.into_inner();
//             let name_pair = assign_inner.next().ok_or_else(|| {
//                 parse_codes::invalid_syntax("Expected identifier", init_span.clone())
//             })?;
//             let name = name_pair.as_str().to_string();
//             let name_span = pair_to_span(&name_pair);
//
//             let value = parse_expression(assign_inner.next().ok_or_else(|| {
//                 parse_codes::invalid_syntax("Expected expression", init_span.clone())
//             })?)?;
//
//             Some(Box::new(Stmt::Assignment {
//                 target: hudhudscript_ast::Expr::Identifier(name, name_span),
//                 value,
//                 span: init_span.clone(),
//             }))
//         }
//         _ => None,
//     };
//
// Parse condition, update, and body
//     let mut condition = None;
//     let mut update = None;
//     let mut body = None;
//
//     for item in inner {
//         match item.as_rule() {
//             Rule::expression => {
//                 condition = Some(parse_expression(item)?);
//             }
//             Rule::for_update => {
//                 let update_span = pair_to_span(&item);
//                 let mut update_inner = item.into_inner();
//
//                 let name_pair = update_inner
//                     .next()
//                     .ok_or_else(|| parse_codes::invalid_syntax("Expected identifier", update_span.clone()))?;
//                 let name = name_pair.as_str().to_string();
//                 let name_span = pair_to_span(&name_pair);
//
//                 let value = parse_expression(
//                     update_inner
//                         .next()
//                         .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", update_span.clone()))?,
//                 )?;
//
//                 update = Some(Box::new(Stmt::Assignment {
//                     target: hudhudscript_ast::Expr::Identifier(name, name_span),
//                     value,
//                     span: update_span,
//                 }));
//             }
//             Rule::block => {
//                 body = Some(Box::new(parse_block(item)?));
//             }
//             _ => {}
//         }
//     }
//
//     let body = body.ok_or_else(|| parse_codes::invalid_syntax("Expected body", span.clone()))?;
//
//     Ok(Stmt::ForCStyle {
//         init,
//         condition,
//         update,
//         body,
//         span,
//     })
// }

// fn parse_c_style_from_condition(cond_pair: Pair<Rule>, inner: &mut pest::iterators::Pairs<Rule>, span: hudhudscript_ast::Span) -> ParseResult<Stmt> {
//     let condition = Some(parse_expression(cond_pair)?);
//
//     let mut update = None;
//     let mut body = None;
//
//     for item in inner {
//         match item.as_rule() {
//             Rule::for_update => {
//                 let update_span = pair_to_span(&item);
//                 let mut update_inner = item.into_inner();
//
//                 let name_pair = update_inner
//                     .next()
//                     .ok_or_else(|| parse_codes::invalid_syntax("Expected identifier", update_span.clone()))?;
//                 let name = name_pair.as_str().to_string();
//                 let name_span = pair_to_span(&name_pair);
//
//                 let value = parse_expression(
//                     update_inner
//                         .next()
//                         .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", update_span.clone()))?,
//                 )?;
//
//                 update = Some(Box::new(Stmt::Assignment {
//                     target: hudhudscript_ast::Expr::Identifier(name, name_span),
//                     value,
//                     span: update_span,
//                 }));
//             }
//             Rule::block => {
//                 body = Some(Box::new(parse_block(item)?));
//             }
//             _ => {}
//         }
//     }
//
//     let body = body.ok_or_else(|| parse_codes::invalid_syntax("Expected body", span.clone()))?;
//
//     Ok(Stmt::ForCStyle {
//         init: None,
//         condition,
//         update,
//         body,
//         span,
//     })
// }

// fn parse_c_style_from_update(update_pair: Pair<Rule>, inner: &mut pest::iterators::Pairs<Rule>, span: hudhudscript_ast::Span) -> ParseResult<Stmt> {
//     let update_span = pair_to_span(&update_pair);
//     let mut update_inner = update_pair.into_inner();
//
//     let name_pair = update_inner
//         .next()
//         .ok_or_else(|| parse_codes::invalid_syntax("Expected identifier", update_span.clone()))?;
//     let name = name_pair.as_str().to_string();
//     let name_span = pair_to_span(&name_pair);
//
//     let value = parse_expression(
//         update_inner
//             .next()
//             .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", update_span.clone()))?,
//     )?;
//
//     let update = Some(Box::new(Stmt::Assignment {
//         target: hudhudscript_ast::Expr::Identifier(name, name_span),
//         value,
//         span: update_span,
//     }));
//
//     let body = inner
//         .next()
//         .ok_or_else(|| parse_codes::invalid_syntax("Expected body", span.clone()))?;
//
//     Ok(Stmt::ForCStyle {
//         init: None,
//         condition: None,
//         update,
//         body: Box::new(parse_block(body)?),
//         span,
//     })
// }
