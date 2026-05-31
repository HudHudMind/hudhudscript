//! Shared docker CLI wrapper — docker.ps/images/run/stop/rm/logs/exec/build.
//!
//! Single source of truth for the VM and interpreter runtimes (Kural 7).
//! All functions shell out to the `docker` CLI.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use std::collections::HashMap;
use std::process::{Command, Stdio};

/// Main entry point used by the VM's module dispatcher.
/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Ps,
    Images,
    Run,
    Stop,
    Rm,
    Logs,
    Exec,
    Build,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ps" => Ok(Self::Ps),
            "images" => Ok(Self::Images),
            "run" => Ok(Self::Run),
            "stop" => Ok(Self::Stop),
            "rm" => Ok(Self::Rm),
            "logs" => Ok(Self::Logs),
            "exec" => Ok(Self::Exec),
            "build" => Ok(Self::Build),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Ps => docker_ps(args),
        ScriptMethodId::Images => docker_images(args),
        ScriptMethodId::Run => docker_run(args),
        ScriptMethodId::Stop => docker_stop(args),
        ScriptMethodId::Rm => docker_rm(args),
        ScriptMethodId::Logs => docker_logs(args),
        ScriptMethodId::Exec => docker_exec(args),
        ScriptMethodId::Build => docker_build(args),
    }
}

/// Main entry point (kept for backward compat).

pub fn docker_ps(_args: &[Value16]) -> HudHudResult<Value16> {
    let output = run_docker(&["ps", "--format", "{{json .}}"])?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(runtime_error(format!(
            "docker.ps failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers: Vec<Value16> = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
            let mut entry = HashMap::new();
            entry.insert(
                "id".to_string(),
                Value16::string(str_field(&parsed, "ID").to_string()),
            );
            entry.insert(
                "name".to_string(),
                Value16::string(str_field(&parsed, "Names").to_string()),
            );
            entry.insert(
                "image".to_string(),
                Value16::string(str_field(&parsed, "Image").to_string()),
            );
            entry.insert(
                "status".to_string(),
                Value16::string(str_field(&parsed, "Status").to_string()),
            );
            entry.insert(
                "ports".to_string(),
                Value16::string(str_field(&parsed, "Ports").to_string()),
            );
            containers.push(Value16::object(entry));
        }
    }
    Ok(Value16::array(containers))
}

pub fn docker_images(_args: &[Value16]) -> HudHudResult<Value16> {
    let output = run_docker(&["images", "--format", "{{json .}}"])?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(runtime_error(format!(
            "docker.images failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut images: Vec<Value16> = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
            let mut entry = HashMap::new();
            entry.insert(
                "id".to_string(),
                Value16::string(str_field(&parsed, "ID").to_string()),
            );
            entry.insert(
                "repository".to_string(),
                Value16::string(str_field(&parsed, "Repository").to_string()),
            );
            entry.insert(
                "tag".to_string(),
                Value16::string(str_field(&parsed, "Tag").to_string()),
            );
            entry.insert(
                "size".to_string(),
                Value16::string(str_field(&parsed, "Size").to_string()),
            );
            images.push(Value16::object(entry));
        }
    }
    Ok(Value16::array(images))
}

pub fn docker_run(args: &[Value16]) -> HudHudResult<Value16> {
    let image = require_str(args, 0, "docker.run")?.to_string();

    let mut cmd_args: Vec<String> = vec!["run".to_string()];

    if let Some(opts_val) = args.get(1) {
        if let Some(opts) = opts_val.as_object() {
            if let Some(name) = opts.get("name").and_then(|v| v.as_str()) {
                cmd_args.push("--name".to_string());
                cmd_args.push(name.to_string());
            }
            match opts.get("detach").and_then(|v| v.as_bool()) {
                Some(true) => cmd_args.push("--detach".to_string()),
                None => cmd_args.push("--detach".to_string()),
                Some(false) => {}
            }
            if let Some(ports) = opts.get("ports") {
                if let Some(s) = ports.as_str() {
                    cmd_args.push("-p".to_string());
                    cmd_args.push(s.to_string());
                } else if let Some(arr) = ports.as_array() {
                    for p in arr {
                        if let Some(s) = p.as_str() {
                            cmd_args.push("-p".to_string());
                            cmd_args.push(s.to_string());
                        }
                    }
                }
            }
            if let Some(volumes) = opts.get("volumes") {
                if let Some(s) = volumes.as_str() {
                    cmd_args.push("-v".to_string());
                    cmd_args.push(s.to_string());
                } else if let Some(arr) = volumes.as_array() {
                    for v in arr {
                        if let Some(s) = v.as_str() {
                            cmd_args.push("-v".to_string());
                            cmd_args.push(s.to_string());
                        }
                    }
                }
            }
            if let Some(env_val) = opts.get("env") {
                if let Some(map) = env_val.as_object() {
                    for (k, v) in map {
                        if let Some(val) = v.as_str() {
                            cmd_args.push("-e".to_string());
                            cmd_args.push(format!("{}={}", k, val));
                        }
                    }
                } else if let Some(arr) = env_val.as_array() {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            cmd_args.push("-e".to_string());
                            cmd_args.push(s.to_string());
                        }
                    }
                }
            }
        }
    } else {
        cmd_args.push("--detach".to_string());
    }

    cmd_args.push(image);

    let arg_refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    let output = run_docker(&arg_refs)?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let mut result = HashMap::new();
    result.insert("container_id".to_string(), Value16::string(stdout));
    result.insert(
        "status".to_string(),
        Value16::string(if output.status.success() {
            "created".to_string()
        } else {
            format!("error: {}", stderr)
        }),
    );
    Ok(Value16::object(result))
}

pub fn docker_stop(args: &[Value16]) -> HudHudResult<Value16> {
    let container = require_str(args, 0, "docker.stop")?.to_string();
    let output = run_docker(&["stop", &container])?;
    Ok(ok_message(
        output.status.success(),
        if output.status.success() {
            format!("Stopped {}", String::from_utf8_lossy(&output.stdout).trim())
        } else {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        },
    ))
}

pub fn docker_rm(args: &[Value16]) -> HudHudResult<Value16> {
    let container = require_str(args, 0, "docker.rm")?.to_string();
    let output = run_docker(&["rm", &container])?;
    Ok(ok_message(
        output.status.success(),
        if output.status.success() {
            format!("Removed {}", String::from_utf8_lossy(&output.stdout).trim())
        } else {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        },
    ))
}

pub fn docker_logs(args: &[Value16]) -> HudHudResult<Value16> {
    let container = require_str(args, 0, "docker.logs")?.to_string();
    let tail = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| format!("{}", n as u64))
        .unwrap_or_else(|| "100".to_string());

    let output = run_docker(&["logs", "--tail", &tail, &container])?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(runtime_error(format!(
            "docker.logs failed: {}",
            stderr.trim()
        )));
    }

    let mut combined = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
    if !stderr_str.is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(&stderr_str);
    }
    Ok(Value16::string(combined))
}

pub fn docker_exec(args: &[Value16]) -> HudHudResult<Value16> {
    let container = require_str(args, 0, "docker.exec")?.to_string();
    let command = require_str(args, 1, "docker.exec")?.to_string();

    let output = Command::new("docker")
        .args(["exec", &container, "sh", "-c", &command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| runtime_error(format!("docker.exec error: {}", e)))?;

    let mut result = HashMap::new();
    result.insert(
        "stdout".to_string(),
        Value16::string(String::from_utf8_lossy(&output.stdout).to_string()),
    );
    result.insert(
        "stderr".to_string(),
        Value16::string(String::from_utf8_lossy(&output.stderr).to_string()),
    );
    result.insert(
        "exit_code".to_string(),
        Value16::number(output.status.code().unwrap_or(-1) as f64),
    );
    Ok(Value16::object(result))
}

pub fn docker_build(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "docker.build")?.to_string();
    let tag = require_str(args, 1, "docker.build")?.to_string();

    let output = run_docker(&["build", "-t", &tag, &path])?;
    Ok(ok_message(
        output.status.success(),
        if output.status.success() {
            format!("Built image {}", tag)
        } else {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        },
    ))
}

// ── helpers ────────────────────────────────────────────────────────────────

fn run_docker(args: &[&str]) -> HudHudResult<std::process::Output> {
    Command::new("docker")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| runtime_error(format!("docker error: {}", e)))
}

fn require_str<'a>(args: &'a [Value16], idx: usize, method: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(runtime_error(format!(
            "{}: argument {} required",
            method, idx
        ))),
    }
}

fn str_field<'a>(v: &'a serde_json::Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

fn ok_message(ok: bool, msg: String) -> Value16 {
    let mut m = HashMap::new();
    m.insert("ok".to_string(), Value16::bool_(ok));
    m.insert("message".to_string(), Value16::string(msg));
    Value16::object(m)
}
