//! exec.kill implementation with platform-specific signalling.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::process::Command;

use super::utils::runtime_error;

pub fn exec_kill(args: &[Value16]) -> HudHudResult<Value16> {
    let pid = match args.first().and_then(|v| v.as_number()) {
        Some(n) => n as i32,
        None => return Err(runtime_error("exec.kill: expected pid number".to_string())),
    };

    #[cfg(unix)]
    {
        let sig = match args.get(1) {
            Some(v) if v.as_number().is_some() => v.as_number().unwrap() as i32,
            Some(v) if v.as_str().is_some() => match v.as_str().unwrap().to_uppercase().as_str() {
                "SIGTERM" | "TERM" => libc::SIGTERM,
                "SIGKILL" | "KILL" => libc::SIGKILL,
                "SIGINT" | "INT" => libc::SIGINT,
                "SIGHUP" | "HUP" => libc::SIGHUP,
                _ => libc::SIGTERM,
            },
            _ => libc::SIGTERM,
        };
        let result = unsafe { libc::kill(pid, sig) };
        Ok(Value16::bool_(result == 0))
    }

    #[cfg(not(unix))]
    {
        let force = match args.get(1) {
            Some(v) if v.as_str().is_some() => {
                matches!(
                    v.as_str().unwrap().to_uppercase().as_str(),
                    "SIGKILL" | "KILL"
                )
            }
            Some(v) if v.as_number().is_some() => v.as_number().unwrap() as i32 == 9,
            _ => false,
        };
        let mut cmd = Command::new("taskkill");
        if force {
            cmd.arg("/F");
        }
        cmd.args(["/PID", &pid.to_string()]);
        let output = cmd.output();
        Ok(Value16::bool_(
            output.map(|o| o.status.success()).unwrap_or(false),
        ))
    }
}
