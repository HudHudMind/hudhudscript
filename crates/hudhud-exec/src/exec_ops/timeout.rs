//! exec.timeout implementation.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;
use std::process::{Command, Stdio};

use super::utils::{apply_opts, parse_cmd, runtime_error};

pub fn exec_timeout(args: &[Value16]) -> HudHudResult<Value16> {
    let (program, cmd_args) = parse_cmd(args)?;
    let timeout_ms = match args.get(1) {
        Some(v) if v.as_number().is_some() => v.as_number().unwrap() as u64,
        Some(v) if v.as_object().is_some() => {
            let obj = v.as_object().unwrap();
            match obj.get("timeout").and_then(|v| v.as_number()) {
                Some(n) => n as u64,
                _ => 30000,
            }
        }
        _ => 30000,
    };

    let mut cmd = Command::new(&program);
    cmd.args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_opts(&mut cmd, args)?;

    let mut child = cmd
        .spawn()
        .map_err(|e| runtime_error(format!("exec.timeout error: program '{}' not found or failed to start: {}", program, e)))?;

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child
                    .stdout
                    .take()
                    .map(|s| {
                        use std::io::Read;
                        let mut buf = String::new();
                        std::io::BufReader::new(s).read_to_string(&mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();
                let stderr = child
                    .stderr
                    .take()
                    .map(|s| {
                        use std::io::Read;
                        let mut buf = String::new();
                        std::io::BufReader::new(s).read_to_string(&mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();
                let mut result = hudhudscript_bytecode::ObjMap::default();
                result.insert(
                    "code".to_string(),
                    Value16::number(status.code().unwrap_or(-1) as f64),
                );
                result.insert("stdout".to_string(), Value16::string(stdout));
                result.insert("stderr".to_string(), Value16::string(stderr));
                result.insert("success".to_string(), Value16::bool_(status.success()));
                result.insert("timed_out".to_string(), Value16::bool_(false));
                return Ok(Value16::object(result));
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    let mut result = hudhudscript_bytecode::ObjMap::default();
                    result.insert("code".to_string(), Value16::number(-1.0));
                    result.insert("stdout".to_string(), Value16::string(String::new()));
                    result.insert(
                        "stderr".to_string(),
                        Value16::string("Process timed out".to_string()),
                    );
                    result.insert("success".to_string(), Value16::bool_(false));
                    result.insert("timed_out".to_string(), Value16::bool_(true));
                    return Ok(Value16::object(result));
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                return Err(runtime_error(format!("exec.timeout wait error: {}", e)));
            }
        }
    }
}
