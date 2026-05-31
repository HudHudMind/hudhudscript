//! Shared helpers for archive builtins.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
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

pub fn shell_pipe(
    program: &str,
    args: &[&str],
    stdin_bytes: &[u8],
    context: &str,
) -> HudHudResult<Vec<u8>> {
    use std::io::Write;

    let mut child = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| runtime_error(format!("{}: {}", context, e)))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(stdin_bytes)
            .map_err(|e| runtime_error(format!("{} write error: {}", context, e)))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| runtime_error(format!("{}: {}", context, e)))?;

    if !output.status.success() {
        return Err(runtime_error(format!(
            "{} failed: {}",
            context,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(output.stdout)
}

pub fn run_command(mut cmd: Command, context: &str) -> HudHudResult<Vec<u8>> {
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("{}: {}", context, e)))?;

    if !output.status.success() {
        return Err(runtime_error(format!(
            "{} failed: {}",
            context,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(output.stdout)
}

pub fn require_str<'a>(args: &'a [Value16], idx: usize, method: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            method, idx
        ))),
    }
}

pub fn require_string_array(
    args: &[Value16],
    idx: usize,
    method: &str,
) -> HudHudResult<Vec<String>> {
    let val = args
        .get(idx)
        .ok_or_else(|| type_error("array of strings", "missing argument", method))?;
    let arr = val
        .as_array()
        .ok_or_else(|| type_error("array", val.type_name_str(), method))?;
    let mut result = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let s = v.as_str().ok_or_else(|| {
            type_error("string", v.type_name_str(), &format!("{}[{}]", method, i))
        })?;
        result.push(s.to_string());
    }
    Ok(result)
}
