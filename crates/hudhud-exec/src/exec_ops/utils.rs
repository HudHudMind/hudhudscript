//! Shared helpers for process execution builtins.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode};
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

/// Env var names that must never be passed to child processes.
pub const BLACKLISTED_ENV_VARS: &[&str] = &[
    "LD_PRELOAD",
    "DYLD_INSERT_LIBRARIES",
    "LD_LIBRARY_PATH",
    "DYLD_LIBRARY_PATH",
];

/// Parse a shell command string respecting quotes.
pub fn split_shell_args(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = cmd.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '\\' if in_double_quote => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            c if c.is_whitespace() && !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            c => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

pub fn parse_cmd(args: &[Value16]) -> Result<(String, Vec<String>), Error> {
    let first = args
        .first()
        .ok_or_else(|| runtime_error("exec: command argument required".to_string()))?;

    if let Some(cmd) = first.as_str() {
        let parts = split_shell_args(cmd);
        if parts.is_empty() {
            return Err(runtime_error("exec: empty command".to_string()));
        }
        return Ok((parts[0].clone(), parts[1..].to_vec()));
    }

    if let Some(arr) = first.as_array() {
        if arr.is_empty() {
            return Err(runtime_error("exec: empty command array".to_string()));
        }
        let program = arr[0]
            .as_str()
            .ok_or_else(|| runtime_error("exec: program must be a string".to_string()))?
            .to_string();
        let cmd_args: Result<Vec<String>, _> = arr[1..]
            .iter()
            .map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| runtime_error("exec: args must be strings".to_string()))
            })
            .collect();
        return Ok((program, cmd_args?));
    }

    Err(runtime_error("exec: command argument required".to_string()))
}

pub fn apply_opts(cmd: &mut Command, args: &[Value16]) -> Result<(), Error> {
    if let Some(opts) = args.get(1).and_then(|v| v.as_object()) {
        if let Some(cwd) = opts.get("cwd").and_then(|v| v.as_str()) {
            cmd.current_dir(cwd);
        }
        if let Some(env_map) = opts.get("env").and_then(|v| v.as_object()) {
            for (k, v) in env_map {
                if BLACKLISTED_ENV_VARS
                    .iter()
                    .any(|b| b.eq_ignore_ascii_case(k))
                {
                    return Err(runtime_error(format!(
                        "exec: setting env var '{}' is blocked for security",
                        k
                    )));
                }
                if let Some(val) = v.as_str() {
                    cmd.env(k, val);
                }
            }
        }
    }
    Ok(())
}
