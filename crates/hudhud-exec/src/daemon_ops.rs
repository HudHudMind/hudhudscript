//! Shared daemon/service builtin — used by both VM and interpreter.
//!
//! Provides: daemon.pid, daemon.start, daemon.stop, daemon.write_pid,
//!           daemon.remove_pid, daemon.is_running, daemon.signal

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

/// Execute a daemon method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        "pid" => Ok(Value16::number(std::process::id() as f64)),
        "start" => {
            let pid = std::process::id();
            if let Some(opts) = args.first().and_then(|v| v.as_object()) {
                let pid_file = opts
                    .get("pidFile")
                    .or_else(|| opts.get("pid_file"))
                    .and_then(|v| v.as_str());
                if let Some(pf) = pid_file {
                    std::fs::write(pf, pid.to_string()).map_err(|e| {
                        runtime_error(format!("daemon.start: cannot write PID file: {}", e))
                    })?;
                }
            }
            let mut result = HashMap::new();
            result.insert("pid".to_string(), Value16::number(pid as f64));
            result.insert("started".to_string(), Value16::bool_(true));
            Ok(Value16::object(result))
        }
        "stop" => {
            if let Some(opts) = args.first().and_then(|v| v.as_object()) {
                let pid_file = opts
                    .get("pidFile")
                    .or_else(|| opts.get("pid_file"))
                    .and_then(|v| v.as_str());
                if let Some(pf) = pid_file {
                    let _ = std::fs::remove_file(pf);
                }
            }
            Ok(Value16::null())
        }
        "write_pid" => {
            let path = match args.first().and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => {
                    return Err(runtime_error(
                        "daemon.write_pid: expected string path".to_string(),
                    ))
                }
            };
            std::fs::write(&path, std::process::id().to_string())
                .map_err(|e| runtime_error(format!("daemon.write_pid error: {}", e)))?;
            Ok(Value16::null())
        }
        "remove_pid" => {
            if let Some(path) = args.first().and_then(|v| v.as_str()) {
                let _ = std::fs::remove_file(path);
            }
            Ok(Value16::null())
        }
        "is_running" => {
            let pid = match args.first().and_then(|v| v.as_number()) {
                Some(n) => n as i32,
                None => {
                    return Err(runtime_error(
                        "daemon.is_running: expected number".to_string(),
                    ))
                }
            };
            #[cfg(unix)]
            {
                let result = unsafe { libc::kill(pid, 0) };
                Ok(Value16::bool_(result == 0))
            }
            #[cfg(not(unix))]
            {
                let output = std::process::Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                    .output();
                match output {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        Ok(Value16::bool_(
                            stdout.contains(&pid.to_string()) && !stdout.contains("No tasks"),
                        ))
                    }
                    Err(_) => Ok(Value16::bool_(false)),
                }
            }
        }
        "signal" => {
            let pid = match args.first().and_then(|v| v.as_number()) {
                Some(n) => n as i32,
                None => {
                    return Err(runtime_error(
                        "daemon.signal: expected pid number".to_string(),
                    ))
                }
            };
            #[cfg(unix)]
            {
                let sig = match args.get(1) {
                    Some(v) if v.as_number().is_some() => v.as_number().unwrap() as i32,
                    Some(v) if v.as_str().is_some() => {
                        match v.as_str().unwrap().to_uppercase().as_str() {
                            "SIGTERM" | "TERM" => libc::SIGTERM,
                            "SIGHUP" | "HUP" => libc::SIGHUP,
                            "SIGINT" | "INT" => libc::SIGINT,
                            "SIGKILL" | "KILL" => libc::SIGKILL,
                            "SIGUSR1" | "USR1" => libc::SIGUSR1,
                            "SIGUSR2" | "USR2" => libc::SIGUSR2,
                            "SIGSTOP" | "STOP" => libc::SIGSTOP,
                            "SIGCONT" | "CONT" => libc::SIGCONT,
                            other => {
                                return Err(runtime_error(format!(
                                    "daemon.signal: unknown signal '{}'",
                                    other
                                )))
                            }
                        }
                    }
                    _ => libc::SIGTERM,
                };
                let result = unsafe { libc::kill(pid, sig) };
                Ok(Value16::bool_(result == 0))
            }
            #[cfg(not(unix))]
            {
                let force = match args.get(1) {
                    Some(v) if v.as_number().is_some() => v.as_number().unwrap() as i32 == 9,
                    Some(v) if v.as_str().is_some() => {
                        matches!(
                            v.as_str().unwrap().to_uppercase().as_str(),
                            "SIGKILL" | "KILL"
                        )
                    }
                    _ => false,
                };
                let mut cmd = std::process::Command::new("taskkill");
                if force {
                    cmd.arg("/F");
                }
                cmd.args(["/PID", &pid.to_string()]);
                let output = cmd.output();
                match output {
                    Ok(o) => Ok(Value16::bool_(o.status.success())),
                    Err(_) => Ok(Value16::bool_(false)),
                }
            }
        }
        _ => Err(runtime_error(format!("Unknown daemon method: {}", method))),
    }
}
