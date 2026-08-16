//! Variant A — Baseline (re-export from production crate; read-only).

use hudhudscript_bytecode::Value16;

pub fn a_make_string(s: &str) -> Value16 {
    Value16::string(s.to_string())
}

pub fn a_make_array(n: usize) -> Value16 {
    Value16::array((0..n).map(|i| Value16::number(i as f64)).collect())
}

pub fn a_make_number(x: f64) -> Value16 {
    Value16::number(x)
}

pub fn a_dispatch(v: &Value16) -> u64 {
    if v.as_number().is_some() {
        1
    } else if v.as_string().is_some() {
        2
    } else if v.as_array().is_some() {
        3
    } else {
        0
    }
}
