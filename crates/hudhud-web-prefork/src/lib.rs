//! HudHud Web Prefork — Gunicorn-style master/worker process model.
//!
//! `Web.run({script, host, port, workers})`:
//! - If `HUDHUD_WEB_WORKER` env is NOT set → master: spawns N worker processes
//!   using `hudhud run <script>`, each with `HUDHUD_WEB_WORKER=<i>`.
//! - If set → worker: returns `{role:"worker", id, host, port}`.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;
use std::process::Command;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}
fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

/// `Web.run({script, host, port, workers})` → role info.
pub fn run(args: &[Value16]) -> HudHudResult<Value16> {
    let opts = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| type_error("object", "", "Web.run"))?;

    let script = opts
        .get("script")
        .and_then(|v| v.as_str())
        .unwrap_or("app.hud");
    let host = opts
        .get("host")
        .and_then(|v| v.as_str())
        .unwrap_or("127.0.0.1");
    let port = opts
        .get("port")
        .and_then(|v| v.as_number())
        .unwrap_or(8080.0) as u16;
    let workers = opts
        .get("workers")
        .and_then(|v| v.as_number())
        .unwrap_or(1.0) as usize;

    // ── Worker detection ───────────────────────────────────────────
    if let Ok(worker_id_str) = std::env::var("HUDHUD_WEB_WORKER") {
        let id: usize = worker_id_str.parse().unwrap_or(0);
        let mut result = hudhudscript_bytecode::ObjMap::default();
        result.insert("role".to_string(), Value16::string("worker"));
        result.insert("id".to_string(), Value16::number(id as f64));
        result.insert("host".to_string(), Value16::string(host.to_string()));
        result.insert("port".to_string(), Value16::number(port as f64));
        return Ok(Value16::object(result));
    }

    // ── Master: spawn N workers ────────────────────────────────────
    let hudhud_bin = opts
        .get("bin")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "hudhud".to_string())
        });

    let mut pids: Vec<f64> = Vec::new();
    for i in 0..workers {
        match Command::new(&hudhud_bin)
            .arg("run")
            .arg(script)
            .env("HUDHUD_WEB_WORKER", i.to_string())
            .spawn()
        {
            Ok(child) => {
                pids.push(child.id() as f64);
                // Detach the child so it outlives the master if master exits
            }
            Err(e) => {
                return Err(runtime_error(format!("Web.run: spawn worker {}: {}", i, e)));
            }
        }
    }

    eprintln!(
        "[hudhud-web] master (pid {}): {} workers on {}:{}",
        std::process::id(),
        workers,
        host,
        port
    );

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("role".to_string(), Value16::string("master"));
    result.insert("workers".to_string(), Value16::number(workers as f64));
    result.insert(
        "pids".to_string(),
        Value16::array(pids.into_iter().map(Value16::number).collect()),
    );
    Ok(Value16::object(result))
}
