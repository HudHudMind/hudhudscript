//! Arithmetic Operations Module (Value16)
//!
//! This module contains all arithmetic and comparison operations for Value16.
//! Single source of truth for the VM (Kural 7).

use hudhudscript_bytecode::shared_value::{
    num_div, num_mod, shared_add, shared_compare, SharedError, SharedResult,
};
use hudhudscript_bytecode::Value16;

/// Build a `TypeError` from static string slices — avoids heap allocation for
/// the `expected` and `operation` fields, which are almost always literals.
#[inline]
fn type_error_static(
    expected: &'static str,
    found: String,
    operation: &'static str,
) -> SharedError {
    hudhudscript_bytecode::shared_value::type_error_ctx(
        expected.to_string(),
        found,
        operation.to_string(),
    )
}

/// Evaluate addition (handles both numbers and string concatenation).
///
/// Delegates to shared `shared_add` (Kural 7 — single source of truth).
#[inline]
pub fn eval_add(left: &Value16, right: &Value16) -> SharedResult<Value16> {
    shared_add(left, right)
}

/// Evaluate arithmetic operation.
#[inline]
pub fn eval_arithmetic<F>(
    left: &Value16,
    right: &Value16,
    op: F,
    op_name: &'static str,
) -> SharedResult<Value16>
where
    F: Fn(f64, f64) -> f64,
{
    match (left.as_number(), right.as_number()) {
        (Some(a), Some(b)) => Ok(Value16::number(op(a, b))),
        _ => Err(type_error_static(
            "number",
            format!("{} and {}", left.type_name_str(), right.type_name_str()),
            op_name,
        )),
    }
}

/// Evaluate division.
#[inline]
pub fn eval_div(left: &Value16, right: &Value16) -> SharedResult<Value16> {
    match (left.as_number(), right.as_number()) {
        (Some(a), Some(b)) => {
            if b == 0.0 {
                Err(hudhudscript_bytecode::shared_value::division_by_zero())
            } else {
                Ok(Value16::number(num_div(a, b)))
            }
        }
        _ => Err(type_error_static(
            "number",
            format!("{} and {}", left.type_name_str(), right.type_name_str()),
            "division",
        )),
    }
}

/// Evaluate modulo.
#[inline]
pub fn eval_mod(left: &Value16, right: &Value16) -> SharedResult<Value16> {
    match (left.as_number(), right.as_number()) {
        (Some(a), Some(b)) => {
            if b == 0.0 {
                Err(hudhudscript_bytecode::shared_value::division_by_zero())
            } else {
                Ok(Value16::number(num_mod(a, b)))
            }
        }
        _ => Err(type_error_static(
            "number",
            format!("{} and {}", left.type_name_str(), right.type_name_str()),
            "modulo",
        )),
    }
}

/// Evaluate comparison.
///
/// Delegates to shared `shared_compare` (Kural 7) so VM has identical
/// mixed-type → false semantics.
#[inline]
pub fn eval_comparison(
    left: &Value16,
    right: &Value16,
    op: fn(f64, f64) -> bool,
) -> SharedResult<Value16> {
    Ok(shared_compare(left, right, op))
}
