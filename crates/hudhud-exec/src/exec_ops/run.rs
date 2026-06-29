//! exec.run, exec.output, exec.lines implementations.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;
use std::process::{Command, Stdio};

use super::utils::{apply_opts, parse_cmd, runtime_error};

pub fn exec_run(args: &[Value16]) -> HudHudResult<Value16> {
    let (program, cmd_args) = parse_cmd(args)?;
    let mut cmd = Command::new(&program);
    cmd.args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_opts(&mut cmd, args)?;

    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("exec.run error: {}", e)))?;

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert(
        "code".to_string(),
        Value16::number(output.status.code().unwrap_or(-1) as f64),
    );
    result.insert(
        "stdout".to_string(),
        Value16::string(String::from_utf8_lossy(&output.stdout).to_string()),
    );
    result.insert(
        "stderr".to_string(),
        Value16::string(String::from_utf8_lossy(&output.stderr).to_string()),
    );
    result.insert(
        "success".to_string(),
        Value16::bool_(output.status.success()),
    );
    Ok(Value16::object(result))
}

pub fn exec_output(args: &[Value16]) -> HudHudResult<Value16> {
    let (program, cmd_args) = parse_cmd(args)?;
    let mut cmd = Command::new(&program);
    cmd.args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_opts(&mut cmd, args)?;

    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("exec.output error: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(runtime_error(format!(
            "exec.output: command failed with code {}: {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }
    Ok(Value16::string(
        String::from_utf8_lossy(&output.stdout).to_string(),
    ))
}

pub fn exec_lines(args: &[Value16]) -> HudHudResult<Value16> {
    let (program, cmd_args) = parse_cmd(args)?;
    let mut cmd = Command::new(&program);
    cmd.args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_opts(&mut cmd, args)?;

    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("exec.lines error: {}", e)))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<Value16> = stdout
        .lines()
        .map(|l| Value16::string(l.to_string()))
        .collect();
    Ok(Value16::array(lines))
}
