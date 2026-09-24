//! Array combinator desugaring for HIR lowering.
//! Transforms .map, .filter, and .reduce into native loops and array operations.

use std::sync::atomic::{AtomicUsize, Ordering};
use hudhudscript_ast::{ArrowFunctionBody, BinaryOp, Expr, Literal, Span, Stmt};

static COMBINATOR_ID: AtomicUsize = AtomicUsize::new(1);

/// Desugars a `let target = arr.map(...) / filter(...) / reduce(...)` statement
/// or `target = arr.map(...) / ...` assignment into native loop statements.
pub(crate) fn desugar_array_combinator(
    target_name: &str,
    value: &Expr,
    is_let: bool,
    span: Span,
) -> Option<Vec<Stmt>> {
    let Expr::Call { callee, args, .. } = value else {
        return None;
    };
    let Expr::Member { object, property, .. } = callee.as_ref() else {
        return None;
    };

    match property.as_str() {
        "map" => desugar_map(target_name, object, args, is_let, span),
        "filter" => desugar_filter(target_name, object, args, is_let, span),
        "reduce" => desugar_reduce(target_name, object, args, is_let, span),
        _ => None,
    }
}

fn desugar_map(
    target_name: &str,
    object: &Expr,
    args: &[Expr],
    is_let: bool,
    span: Span,
) -> Option<Vec<Stmt>> {
    if args.is_empty() {
        return None;
    }
    let (param_x, pre_stmts, mapped_expr) = extract_single_param_cb(&args[0], span)?;
    let id = COMBINATOR_ID.fetch_add(1, Ordering::Relaxed);
    let arr_name = format!("__map_arr_{target_name}_{id}");
    let idx_name = format!("__map_idx_{target_name}_{id}");

    let mut stmts = Vec::new();

    if is_let {
        stmts.push(Stmt::Let {
            name: target_name.to_string(),
            value: Expr::Array { elements: Vec::new(), span },
            span,
        });
    } else {
        stmts.push(Stmt::Assignment {
            target: Expr::Identifier(target_name.to_string(), span),
            value: Expr::Array { elements: Vec::new(), span },
            span,
        });
    }

    stmts.push(Stmt::Let {
        name: arr_name.clone(),
        value: object.clone(),
        span,
    });
    stmts.push(Stmt::Let {
        name: idx_name.clone(),
        value: Expr::Literal(Literal::Int(0), span),
        span,
    });

    let cond = Expr::Binary {
        left: Box::new(Expr::Identifier(idx_name.clone(), span)),
        op: BinaryOp::Lt,
        right: Box::new(Expr::Member {
            object: Box::new(Expr::Identifier(arr_name.clone(), span)),
            property: "length".to_string(),
            span,
        }),
        span,
    };

    let mut loop_body = Vec::new();
    loop_body.push(Stmt::Let {
        name: param_x,
        value: Expr::Index {
            object: Box::new(Expr::Identifier(arr_name.clone(), span)),
            index: Box::new(Expr::Identifier(idx_name.clone(), span)),
            span,
        },
        span,
    });
    loop_body.extend(pre_stmts);
    loop_body.push(Stmt::Expr(Expr::Call {
        callee: Box::new(Expr::Member {
            object: Box::new(Expr::Identifier(target_name.to_string(), span)),
            property: "push".to_string(),
            span,
        }),
        args: vec![mapped_expr],
        span,
    }));
    loop_body.push(Stmt::Assignment {
        target: Expr::Identifier(idx_name.clone(), span),
        value: Expr::Binary {
            left: Box::new(Expr::Identifier(idx_name, span)),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(1), span)),
            span,
        },
        span,
    });

    stmts.push(Stmt::While {
        condition: cond,
        body: Box::new(Stmt::Block { statements: loop_body, span }),
        span,
    });

    Some(stmts)
}

fn desugar_filter(
    target_name: &str,
    object: &Expr,
    args: &[Expr],
    is_let: bool,
    span: Span,
) -> Option<Vec<Stmt>> {
    if args.is_empty() {
        return None;
    }
    let (param_x, pre_stmts, cond_expr) = extract_single_param_cb(&args[0], span)?;
    let id = COMBINATOR_ID.fetch_add(1, Ordering::Relaxed);
    let arr_name = format!("__filt_arr_{target_name}_{id}");
    let idx_name = format!("__filt_idx_{target_name}_{id}");

    let mut stmts = Vec::new();

    if is_let {
        stmts.push(Stmt::Let {
            name: target_name.to_string(),
            value: Expr::Array { elements: Vec::new(), span },
            span,
        });
    } else {
        stmts.push(Stmt::Assignment {
            target: Expr::Identifier(target_name.to_string(), span),
            value: Expr::Array { elements: Vec::new(), span },
            span,
        });
    }

    stmts.push(Stmt::Let {
        name: arr_name.clone(),
        value: object.clone(),
        span,
    });
    stmts.push(Stmt::Let {
        name: idx_name.clone(),
        value: Expr::Literal(Literal::Int(0), span),
        span,
    });

    let cond = Expr::Binary {
        left: Box::new(Expr::Identifier(idx_name.clone(), span)),
        op: BinaryOp::Lt,
        right: Box::new(Expr::Member {
            object: Box::new(Expr::Identifier(arr_name.clone(), span)),
            property: "length".to_string(),
            span,
        }),
        span,
    };

    let mut loop_body = Vec::new();
    loop_body.push(Stmt::Let {
        name: param_x.clone(),
        value: Expr::Index {
            object: Box::new(Expr::Identifier(arr_name.clone(), span)),
            index: Box::new(Expr::Identifier(idx_name.clone(), span)),
            span,
        },
        span,
    });
    loop_body.extend(pre_stmts);
    loop_body.push(Stmt::If {
        condition: cond_expr,
        then_branch: Box::new(Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Identifier(target_name.to_string(), span)),
                property: "push".to_string(),
                span,
            }),
            args: vec![Expr::Identifier(param_x, span)],
            span,
        })),
        else_branch: None,
        span,
    });
    loop_body.push(Stmt::Assignment {
        target: Expr::Identifier(idx_name.clone(), span),
        value: Expr::Binary {
            left: Box::new(Expr::Identifier(idx_name, span)),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(1), span)),
            span,
        },
        span,
    });

    stmts.push(Stmt::While {
        condition: cond,
        body: Box::new(Stmt::Block { statements: loop_body, span }),
        span,
    });

    Some(stmts)
}

fn desugar_reduce(
    target_name: &str,
    object: &Expr,
    args: &[Expr],
    is_let: bool,
    span: Span,
) -> Option<Vec<Stmt>> {
    if args.len() < 2 {
        return None;
    }
    let init_val = &args[1];
    let (acc_name, elem_name, pre_stmts, step_expr) = extract_two_param_cb(&args[0], span)?;
    let id = COMBINATOR_ID.fetch_add(1, Ordering::Relaxed);
    let arr_name = format!("__red_arr_{target_name}_{id}");
    let idx_name = format!("__red_idx_{target_name}_{id}");

    let mut stmts = Vec::new();

    if is_let {
        stmts.push(Stmt::Let {
            name: target_name.to_string(),
            value: init_val.clone(),
            span,
        });
    } else {
        stmts.push(Stmt::Assignment {
            target: Expr::Identifier(target_name.to_string(), span),
            value: init_val.clone(),
            span,
        });
    }

    stmts.push(Stmt::Let {
        name: arr_name.clone(),
        value: object.clone(),
        span,
    });
    stmts.push(Stmt::Let {
        name: idx_name.clone(),
        value: Expr::Literal(Literal::Int(0), span),
        span,
    });

    let cond = Expr::Binary {
        left: Box::new(Expr::Identifier(idx_name.clone(), span)),
        op: BinaryOp::Lt,
        right: Box::new(Expr::Member {
            object: Box::new(Expr::Identifier(arr_name.clone(), span)),
            property: "length".to_string(),
            span,
        }),
        span,
    };

    let mut loop_body = Vec::new();
    loop_body.push(Stmt::Let {
        name: acc_name,
        value: Expr::Identifier(target_name.to_string(), span),
        span,
    });
    loop_body.push(Stmt::Let {
        name: elem_name,
        value: Expr::Index {
            object: Box::new(Expr::Identifier(arr_name.clone(), span)),
            index: Box::new(Expr::Identifier(idx_name.clone(), span)),
            span,
        },
        span,
    });
    loop_body.extend(pre_stmts);
    loop_body.push(Stmt::Assignment {
        target: Expr::Identifier(target_name.to_string(), span),
        value: step_expr,
        span,
    });
    loop_body.push(Stmt::Assignment {
        target: Expr::Identifier(idx_name.clone(), span),
        value: Expr::Binary {
            left: Box::new(Expr::Identifier(idx_name, span)),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(1), span)),
            span,
        },
        span,
    });

    stmts.push(Stmt::While {
        condition: cond,
        body: Box::new(Stmt::Block { statements: loop_body, span }),
        span,
    });

    Some(stmts)
}

fn extract_single_param_cb(cb: &Expr, _span: Span) -> Option<(String, Vec<Stmt>, Expr)> {
    match cb {
        Expr::ArrowFunction { params, body, .. } => {
            let param = params.first().cloned().unwrap_or_else(|| "__x".to_string());
            match body {
                ArrowFunctionBody::Expression(expr) => Some((param, Vec::new(), *expr.clone())),
                ArrowFunctionBody::Block(stmts) => {
                    if let Some(Stmt::Return { value: Some(ret_val), .. }) = stmts.last() {
                        let pre = stmts[..stmts.len() - 1].to_vec();
                        Some((param, pre, ret_val.clone()))
                    } else {
                        None
                    }
                }
            }
        }
        Expr::Identifier(fn_name, id_span) => {
            let param = "__x".to_string();
            let call = Expr::Call {
                callee: Box::new(Expr::Identifier(fn_name.clone(), *id_span)),
                args: vec![Expr::Identifier(param.clone(), *id_span)],
                span: *id_span,
            };
            Some((param, Vec::new(), call))
        }
        _ => None,
    }
}

fn extract_two_param_cb(cb: &Expr, _span: Span) -> Option<(String, String, Vec<Stmt>, Expr)> {
    match cb {
        Expr::ArrowFunction { params, body, .. } => {
            let p1 = params.first().cloned().unwrap_or_else(|| "__acc".to_string());
            let p2 = params.get(1).cloned().unwrap_or_else(|| "__elem".to_string());
            match body {
                ArrowFunctionBody::Expression(expr) => Some((p1, p2, Vec::new(), *expr.clone())),
                ArrowFunctionBody::Block(stmts) => {
                    if let Some(Stmt::Return { value: Some(ret_val), .. }) = stmts.last() {
                        let pre = stmts[..stmts.len() - 1].to_vec();
                        Some((p1, p2, pre, ret_val.clone()))
                    } else {
                        None
                    }
                }
            }
        }
        Expr::Identifier(fn_name, id_span) => {
            let p1 = "__acc".to_string();
            let p2 = "__elem".to_string();
            let call = Expr::Call {
                callee: Box::new(Expr::Identifier(fn_name.clone(), *id_span)),
                args: vec![
                    Expr::Identifier(p1.clone(), *id_span),
                    Expr::Identifier(p2.clone(), *id_span),
                ],
                span: *id_span,
            };
            Some((p1, p2, Vec::new(), call))
        }
        _ => None,
    }
}
