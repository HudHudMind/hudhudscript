//! Shared argument validators for builtin functions and operations (Value16)
//!
//! Provides reusable helpers that check argument count, extract typed values,
//! and produce consistent `RuntimeError` variants.

use hudhudscript_bytecode::shared_value::SharedResult;
use hudhudscript_bytecode::Value16;

/// Verify that `args` has exactly `expected` elements.
pub fn check_arg_count(args: &[Value16], expected: usize, method: &str) -> SharedResult<()> {
    if args.len() != expected {
        return Err(hudhudscript_bytecode::shared_value::call_error(
            format!(
                "{} expects {} argument(s), got {}",
                method,
                expected,
                args.len()
            ),
            method.to_string(),
        ));
    }
    Ok(())
}

/// Extract a `String` from `args[idx]`, returning a `TypeError` on mismatch
/// and a `CallError` when the index is out of bounds.
pub fn require_string(args: &[Value16], idx: usize, method: &str) -> SharedResult<String> {
    match args.get(idx) {
        Some(v) => match v.as_str() {
            Some(s) => Ok(s.to_string()),
            None => Err(hudhudscript_bytecode::shared_value::type_error_ctx(
                "string".to_string(),
                v.type_name_str().to_string(),
                method.to_string(),
            )),
        },
        None => Err(hudhudscript_bytecode::shared_value::call_error(
            format!("{} requires argument at position {}", method, idx),
            method.to_string(),
        )),
    }
}

/// Extract an `f64` from `args[idx]`.
pub fn require_number(args: &[Value16], idx: usize, method: &str) -> SharedResult<f64> {
    match args.get(idx) {
        Some(v) => match v.as_number() {
            Some(n) => Ok(n),
            None => Err(hudhudscript_bytecode::shared_value::type_error_ctx(
                "number".to_string(),
                v.type_name_str().to_string(),
                method.to_string(),
            )),
        },
        None => Err(hudhudscript_bytecode::shared_value::call_error(
            format!("{} requires argument at position {}", method, idx),
            method.to_string(),
        )),
    }
}

/// Extract a `Vec<Value16>` from `args[idx]` (cloned).
pub fn require_array(args: &[Value16], idx: usize, method: &str) -> SharedResult<Vec<Value16>> {
    match args.get(idx) {
        Some(v) => match v.as_array() {
            Some(a) => Ok(a.clone()),
            None => Err(hudhudscript_bytecode::shared_value::type_error_ctx(
                "array".to_string(),
                v.type_name_str().to_string(),
                method.to_string(),
            )),
        },
        None => Err(hudhudscript_bytecode::shared_value::call_error(
            format!("{} requires argument at position {}", method, idx),
            method.to_string(),
        )),
    }
}
