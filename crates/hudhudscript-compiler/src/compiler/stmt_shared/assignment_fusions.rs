//! B3: NumMulAddAssign fusion pattern detection.
//! Detects `dest = dest * mul_expr + add_expr` for Horner-style accumulation.

use hudhudscript_ast::{BinaryOp, Expr, Literal};

/// If `value` matches `name * mul_expr + add_expr`, returns `Some((mul_expr, add_expr))`.
/// Handles commutativity: `name * X + Y` and `X * name + Y` both match.
#[doc(hidden)]
pub fn try_fma_pattern<'a>(value: &'a Expr, name: &str) -> Option<(&'a Expr, &'a Expr)> {
    // outer: Add
    if let Expr::Binary {
        op: BinaryOp::Add,
        left,
        right: add_expr,
        ..
    } = value
    {
        // inner: Mul with Identifier(name) on either side
        if let Expr::Binary {
            op: BinaryOp::Mul,
            left: mul_left,
            right: mul_right,
            ..
        } = left.as_ref()
        {
            if is_ident(mul_left, name) {
                return Some((mul_right, add_expr));
            }
            if is_ident(mul_right, name) {
                return Some((mul_left, add_expr));
            }
        }
    }
    None
}

/// If `value` matches `name - positive_int_literal`, returns `Some(imm)`.
/// Only matches when left side is `Identifier(name)` — no commutativity for subtract.
#[doc(hidden)]
pub fn try_self_sub_int<'a>(value: &'a Expr, name: &str) -> Option<i16> {
    if let Expr::Binary {
        op: BinaryOp::Sub,
        left,
        right,
        ..
    } = value
    {
        if !is_ident(left, name) {
            return None;
        }
        if let Expr::Literal(Literal::Int(i), _) = right.as_ref() {
            if *i > 0 && *i <= i16::MAX as i64 {
                return Some(*i as i16);
            }
        }
        if let Expr::Literal(Literal::Number(n, false), _) = right.as_ref() {
            let i = *n as i64;
            if i > 0 && i <= i16::MAX as i64 {
                return Some(i as i16);
            }
        }
    }
    None
}

/// If `value` matches `name + positive_int_literal` or `positive_int_literal + name`,
/// returns `Some(imm)`. Only matches when one side is Identifier(name).
pub(super) fn try_self_add_int<'a>(value: &'a Expr, name: &str) -> Option<i16> {
    let lit_to_i16 = |lit: &Literal| -> Option<i16> {
        let i = match lit {
            Literal::Int(i) => *i,
            Literal::Number(n, false) => *n as i64,
            _ => return None,
        };
        if i > 0 && i <= i16::MAX as i64 {
            Some(i as i16)
        } else {
            None
        }
    };
    if let Expr::Binary {
        op: BinaryOp::Add,
        left,
        right,
        ..
    } = value
    {
        if let Expr::Literal(lit, _) = right.as_ref() {
            if let Some(imm) = lit_to_i16(lit) {
                if is_ident(left, name) {
                    return Some(imm);
                }
            }
        }
        if let Expr::Literal(lit, _) = left.as_ref() {
            if let Some(imm) = lit_to_i16(lit) {
                if is_ident(right, name) {
                    return Some(imm);
                }
            }
        }
    }
    None
}

fn is_ident(expr: &Expr, name: &str) -> bool {
    matches!(expr, Expr::Identifier(n, _) if n == name)
}
