//! exec.stream and exec.spawn implementations.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;
use std::io::BufRead;
use std::process::{Command, Stdio};

use super::utils::{apply_opts, parse_cmd, runtime_error};

pub fn exec_stream(args: &[Value16]) -> HudHudResult<Value16> {
    let (program, cmd_args) = parse_cmd(args)?;
    let mut cmd = Command::new(&program);
    cmd.args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_opts(&mut cmd, args)?;

    let mut child = cmd
        .spawn()
        .map_err(|e| runtime_error(format!("exec.stream error: program '{}' not found or failed to start: {}", program, e)))?;

    let mut lines = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let mut entry = hudhudscript_bytecode::ObjMap::default();
            entry.insert("stream".to_string(), Value16::string("stdout".to_string()));
            entry.insert("line".to_string(), Value16::string(line));
            lines.push(Value16::object(entry));
        }
    }
    if let Some(stderr) = child.stderr.take() {
        let reader = std::io::BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let mut entry = hudhudscript_bytecode::ObjMap::default();
            entry.insert("stream".to_string(), Value16::string("stderr".to_string()));
            entry.insert("line".to_string(), Value16::string(line));
            lines.push(Value16::object(entry));
        }
    }

    let status = child
        .wait()
        .map_err(|e| runtime_error(format!("exec.stream wait error: {}", e)))?;

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("lines".to_string(), Value16::array(lines));
    result.insert(
        "code".to_string(),
        Value16::number(status.code().unwrap_or(-1) as f64),
    );
    result.insert("success".to_string(), Value16::bool_(status.success()));
    Ok(Value16::object(result))
}

pub fn exec_spawn(args: &[Value16]) -> HudHudResult<Value16> {
    let (program, cmd_args) = parse_cmd(args)?;
    let mut cmd = Command::new(&program);
    cmd.args(&cmd_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    apply_opts(&mut cmd, args)?;

    let child = cmd
        .spawn()
        .map_err(|e| runtime_error(format!("exec.spawn error: program '{}' not found or failed to start: {}", program, e)))?;

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("pid".to_string(), Value16::number(child.id() as f64));
    Ok(Value16::object(result))
}
