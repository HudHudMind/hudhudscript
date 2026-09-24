//! HIR desugaring: döngü yapılarını çekirdek HIR'e indirger.
//! for-in ve C-style for → let + while kombinasyonu.

use crate::hir::{HirExpr, HirStmt};
use crate::hir_lower::{lower_block_stmt, lower_expr, lower_stmt, HirLowerError};
use crate::types::Type;
use hudhudscript_ast::{Expr, Stmt};

/// for (x in arr) { body } →
///   let __for_arr_x = arr; let __for_idx_x = 0;
///   while (__for_idx_x < __for_arr_x.length) {
///     let x = __for_arr_x[__for_idx_x]; body;
///     __for_idx_x = __for_idx_x + 1
///   }
pub(crate) fn desugar_for_in(
    variable: &str,
    iterable: &Expr,
    body: &Stmt,
) -> Result<Vec<HirStmt>, HirLowerError> {
    let arr_val = lower_expr(iterable)?;
    let body_stmts = lower_block_stmt(body)?;
    let arr_name = format!("__for_arr_{variable}");
    let idx_name = format!("__for_idx_{variable}");

    let while_cond = HirExpr::Binary {
        op: crate::hir::HirBinOp::Lt,
        lhs: Box::new(HirExpr::Local { name: idx_name.clone(), ty: Type::Number }),
        rhs: Box::new(HirExpr::ArrayMethod {
            array: Box::new(HirExpr::Local { name: arr_name.clone(), ty: Type::Any }),
            method: "length".to_string(),
            args: Vec::new(),
            ty: Type::Number,
        }),
        ty: Type::Boolean,
    };

    let mut while_body = vec![
        HirStmt::Let {
            name: variable.to_string(),
            ty: Type::Number,
            value: HirExpr::ArrayIndex {
                array: Box::new(HirExpr::Local { name: arr_name.clone(), ty: Type::Any }),
                index: Box::new(HirExpr::Local { name: idx_name.clone(), ty: Type::Number }),
                ty: Type::Number,
            },
        },
    ];
    while_body.extend(body_stmts);
    while_body.push(HirStmt::Assign {
        name: idx_name.clone(),
        value: HirExpr::Binary {
            op: crate::hir::HirBinOp::Add,
            lhs: Box::new(HirExpr::Local { name: idx_name.clone(), ty: Type::Number }),
            rhs: Box::new(HirExpr::IntLit(1)),
            ty: Type::Number,
        },
    });

    Ok(vec![
        HirStmt::Let { name: arr_name, ty: Type::Any, value: arr_val },
        HirStmt::Let { name: idx_name, ty: Type::Number, value: HirExpr::IntLit(0) },
        HirStmt::While { cond: while_cond, body: while_body },
    ])
}

/// for (init; cond; update) { body } → init; while (cond) { body; update }
pub(crate) fn desugar_for_c_style(
    init: &Option<Box<Stmt>>,
    condition: &Option<Expr>,
    update: &Option<Box<Stmt>>,
    body: &Stmt,
) -> Result<Vec<HirStmt>, HirLowerError> {
    let mut result = Vec::new();
    if let Some(init_stmt) = init {
        result.extend(lower_stmt(init_stmt)?);
    }
    let (cond, body_stmts) = match condition {
        Some(c) => (lower_expr(c)?, lower_block_stmt(body)?),
        None => (HirExpr::BoolLit(true), lower_block_stmt(body)?),
    };
    let mut while_body = body_stmts;
    if let Some(upd) = update {
        while_body.extend(lower_stmt(upd)?);
    }
    result.push(HirStmt::While { cond, body: while_body });
    Ok(result)
}
