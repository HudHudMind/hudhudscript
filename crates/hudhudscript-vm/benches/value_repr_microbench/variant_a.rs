//! Variant A — Baseline (re-export from production crate; read-only).

use hudhudscript_bytecode::Value;

pub fn a_make_string(s: &str) -> Value {
    Value::String(s.to_string())
}

pub fn a_make_array(n: usize) -> Value {
    Value::Array((0..n).map(|i| Value::Number(i as f64)).collect())
}

pub fn a_make_number(x: f64) -> Value {
    Value::Number(x)
}

pub fn a_dispatch(v: &Value) -> u64 {
    match v {
        Value::Number(_) => 1,
        Value::String(_) => 2,
        Value::Array(_) => 3,
        _ => 0,
    }
}
