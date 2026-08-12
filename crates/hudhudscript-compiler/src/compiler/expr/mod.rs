use super::*;

use crate::bytecode::{Bytecode, FunctionChunk, FunctionData, Instruction, SymId, Value16};
use crate::error::{compile_codes, CompileResult, SourcePosition};
use crate::compiler::regalloc::RegAlloc;
use crate::compiler::expr::compile_reg::compile_expr_to_reg;
use hudhudscript_ast::{BinaryOp, Decl, Expr, Literal, Span, Stmt, UnaryOp};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// This enum is deliberately conservative: only literal-rooted arithmetic
/// trees produce `Number`; anything that could be a string, a user-defined
/// value, or type-coerced is `Unknown`.  `Unknown` falls through to the
/// existing generic emit path (Kural 7c — single code path, no runtime
/// fallback — the Num* arms panic on non-number operands, so over-inferring
/// would be a correctness bug, not a perf regression).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExprType {
    /// Proven `Value16::number(f64)` — fractional, non-finite, or any
    /// numeric literal that didn't take the integer pool path.
    Number,
    /// A3c: proven `Value16::int(i64)` — integer literal in the
    /// f64-exact range, or pure-integer arithmetic whose operands are
    /// both `Int`.  Used to select `IntAdd` / `IntSub` / `IntLt` / ...
    /// over `NumAdd` / `NumSub` / `NumLt` so the hot path stays in
    /// `i64` without a f64 widening round-trip.
    Int,
    #[allow(dead_code)]
    Bool,
    #[allow(dead_code)]
    Str,
    /// P2: proven array value — used to emit IndexArray / IndexAssignArray.
    #[allow(dead_code)]
    Array,
    Unknown,
}

/// A3c: does `t` refer to a numeric type (Int or Number)?  Used by the
/// arithmetic-emit path to decide whether operand widening is needed —
/// mixing `Int` and `Number` inserts an explicit `NumberFromInt` step.
pub(crate) fn is_numeric(t: ExprType) -> bool {
    matches!(t, ExprType::Int | ExprType::Number)
}

/// ISSUE-1: variant of `infer_type` that resolves identifiers through
/// a compile-time local-type table.  Enables `let i = 0; i = i + 1`
/// to emit `IntAdd` instead of generic `Add`.
pub(crate) fn infer_type_with_locals<F>(expr: &Expr, local_type: &F) -> ExprType
where
    F: Fn(&str) -> ExprType,
{
    match expr {
        Expr::Identifier(name, _) => local_type(name),

        Expr::Literal(Literal::Number(_, is_float), _) => {
            if *is_float {
                ExprType::Number
            } else {
                ExprType::Int
            }
        }
        Expr::Literal(Literal::Int(_), _) => ExprType::Int,
        Expr::Literal(Literal::BigInt(_), _) => ExprType::Int,
        Expr::Literal(Literal::Boolean(_), _) => ExprType::Bool,
        Expr::Literal(Literal::String(_), _) => ExprType::Str,
        Expr::Array { .. } => ExprType::Array,
        Expr::Literal(Literal::Null, _) => ExprType::Unknown,

        Expr::Binary {
            op, left, right, ..
        } => {
            match op {
                // Pure arithmetic on two numeric operands stays numeric.
                // `Add` is included only when BOTH sides are proven numeric —
                // otherwise it's the string-concat path.
                //
                // A3c: Int × Int → Int (fast path), mixed → Number.
                BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Add => {
                    let l = infer_type_with_locals(left, local_type);
                    let r = infer_type_with_locals(right, local_type);
                    // String concat: if either side is Str, result is Str
                    if l == ExprType::Str || r == ExprType::Str {
                        return ExprType::Str;
                    }
                    match (l, r) {
                        (ExprType::Int, ExprType::Int) => ExprType::Int,
                        (a, b) if is_numeric(a) && is_numeric(b) => ExprType::Number,
                        _ => ExprType::Unknown,
                    }
                }
                // IntMod returns Int for int%int; NumMod returns Number otherwise
                BinaryOp::Mod => {
                    let l = infer_type_with_locals(left, local_type);
                    let r = infer_type_with_locals(right, local_type);
                    if l == ExprType::Int && r == ExprType::Int {
                        ExprType::Int
                    } else if is_numeric(l) && is_numeric(r) {
                        ExprType::Number
                    } else {
                        ExprType::Unknown
                    }
                }
                // Integer division: int/int → int (truncation toward zero).
                // Matches C/Java/Rust/Go semantics. Float if either side is float.
                BinaryOp::Div => {
                    let l = infer_type_with_locals(left, local_type);
                    let r = infer_type_with_locals(right, local_type);
                    if l == ExprType::Int && r == ExprType::Int {
                        ExprType::Int
                    } else if is_numeric(l) && is_numeric(r) {
                        ExprType::Number
                    } else {
                        ExprType::Unknown
                    }
                }
                // Comparisons always yield a boolean (never a number).
                BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge
                | BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::And
                | BinaryOp::Or
                | BinaryOp::InstanceOf => ExprType::Bool,
                BinaryOp::NullCoalesce => ExprType::Unknown,
            }
        }

        Expr::Unary {
            op, expr: inner, ..
        } => match op {
            hudhudscript_ast::UnaryOp::Not => ExprType::Bool,
            hudhudscript_ast::UnaryOp::Neg | hudhudscript_ast::UnaryOp::Plus => {
                match infer_type_with_locals(inner, local_type) {
                    ExprType::Int => ExprType::Int,
                    ExprType::Number => ExprType::Number,
                    _ => ExprType::Unknown,
                }
            }
            hudhudscript_ast::UnaryOp::PostIncrement
            | hudhudscript_ast::UnaryOp::PostDecrement => {
                // i++/i-- returns the same type as the variable
                infer_type_with_locals(inner, local_type)
            }
        },

        // arr.length / str.length always returns Int
        Expr::Member { property, .. } | Expr::OptionalMember { property, .. } => {
            match property.as_str() {
                "length" | "size" => ExprType::Int,
                _ => ExprType::Unknown,
            }
        }

        // Everything else — calls, indexing, template strings,
        // object/array literals, arrow fns, await, new, this, spread,
        // yield, spawn — may be any type at runtime.  Be tutucu.
        _ => ExprType::Unknown,
    }
}

/// A3c: distinguishes `Int` (integer literal in f64-exact range, pure
/// integer arithmetic) from `Number` (fractional or mixed).  Int×Int
/// arithmetic stays `Int`; Int×Number or Number×Number becomes `Number`.

/// A3b: emit the optimal load instruction for a numeric literal.
///
/// Integer-valued finite `f64` values that fit in `i64` are stored in the
/// integer pool and loaded via `LoadIntConst`, producing `Value16::int(i64)`
/// on the stack.  Everything else (fractional, NaN, infinities, out-of-range
/// magnitudes) falls through to the existing packed-f64 pool via
/// `LoadNumConst`.
///
/// Kural 7c: exactly one emission path — no runtime fallback, no dual
/// instruction stream.  The runtime's `pop_number` helper widens `Int` →
/// `f64` transparently for all existing `NumAdd` / `NumSub` / ... arms,
/// so emitting `LoadIntConst` is semantically identical to
/// `LoadNumConst` for today's arithmetic.
#[inline]
pub(crate) fn emit_numeric_literal(target: &mut impl CompileTarget, n: f64) {
    let temp = crate::compiler::regalloc::temp_reg();
    // N4: genişletilmiş Int aralığı — i64::MAX'a kadar (BigInt overflow temeli)
    if n.is_finite()
        && n.fract() == 0.0
        && n >= (i64::MIN as f64)
        && n <= (i64::MAX as f64)
    {
        let i = n as i64;
        let idx = target.ct_emit_int_const(i);
        target.ct_emit(Instruction::LoadIntConst { dst: temp, const_idx: idx as u16 });
        target.emit_move(255, temp );
    } else {
        let idx = target.ct_emit_num_const(n);
        target.ct_emit(Instruction::LoadNumConst { dst: temp, const_idx: idx as u16 });
        target.emit_move(255, temp );
    }
}

/// Shared expression compilation — the single source of truth for
/// `compile_expr` logic used by both `Compiler` and `FunctionCompiler`.

pub mod compile_complex;
pub mod compile_complex_extra;
pub mod compile_reg_binary;
pub mod compile_reg;
