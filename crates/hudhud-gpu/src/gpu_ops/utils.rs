//! Shared helpers for GPU detection.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode};
use std::collections::HashMap;
use std::process::Command;

pub fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

pub fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

pub fn which_exists(program: &str) -> bool {
    Command::new("which")
        .arg(program)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn make_obj(pairs: Vec<(&str, Value16)>) -> Value16 {
    let mut m = hudhudscript_bytecode::ObjMap::default();
    for (k, v) in pairs {
        m.insert(k.to_string(), v);
    }
    Value16::object(m)
}

pub fn extract_number(line: &str) -> Option<f64> {
    line.split_whitespace()
        .filter_map(|w| w.trim_end_matches('%').parse::<f64>().ok())
        .next()
}
