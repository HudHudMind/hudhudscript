//! Shared desktop notification / systemd journal builtin.
//!
//! Single source of truth for the VM and interpreter runtimes (Kural 7).
//! Wraps `notify-send` for desktop notifications and `logger` for systemd
//! journal.

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

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Send,
    SendWithIcon,
    SendUrgent,
    Journal,
    JournalStructured,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "send" => Ok(Self::Send),
            "send_with_icon" => Ok(Self::SendWithIcon),
            "send_urgent" => Ok(Self::SendUrgent),
            "journal" => Ok(Self::Journal),
            "journal_structured" => Ok(Self::JournalStructured),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Send => notify_send(args),
        ScriptMethodId::SendWithIcon => notify_send_with_icon(args),
        ScriptMethodId::SendUrgent => notify_send_urgent(args),
        ScriptMethodId::Journal => notify_journal(args),
        ScriptMethodId::JournalStructured => notify_journal_structured(args),
    }
}

/// Main entry point (kept for backward compat).

pub fn notify_send(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() < 2 {
        return Err(runtime_error(
            "notify.send() requires 2 arguments: title, body",
        ));
    }
    let title = require_str(&args[0], "notify.send title")?.to_string();
    let body = require_str(&args[1], "notify.send body")?.to_string();
    run_notify_send(&[&title, &body])
}

pub fn notify_send_with_icon(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() < 3 {
        return Err(runtime_error(
            "notify.send_with_icon() requires 3 arguments: title, body, icon",
        ));
    }
    let title = require_str(&args[0], "notify.send_with_icon title")?.to_string();
    let body = require_str(&args[1], "notify.send_with_icon body")?.to_string();
    let icon = require_str(&args[2], "notify.send_with_icon icon")?.to_string();
    run_notify_send(&["-i", &icon, &title, &body])
}

pub fn notify_send_urgent(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() < 2 {
        return Err(runtime_error(
            "notify.send_urgent() requires 2 arguments: title, body",
        ));
    }
    let title = require_str(&args[0], "notify.send_urgent title")?.to_string();
    let body = require_str(&args[1], "notify.send_urgent body")?.to_string();
    run_notify_send(&["-u", "critical", &title, &body])
}

pub fn notify_journal(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() < 2 {
        return Err(runtime_error(
            "notify.journal() requires 2 arguments: level, message",
        ));
    }
    let level = require_str(&args[0], "notify.journal level")?.to_string();
    let message = require_str(&args[1], "notify.journal message")?.to_string();
    let priority = level_to_priority(&level);
    run_logger(&["-p", &priority, &message])
}

pub fn notify_journal_structured(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() < 2 {
        return Err(runtime_error(
            "notify.journal_structured() requires 2 arguments: message, fields_object",
        ));
    }
    let message = require_str(&args[0], "notify.journal_structured message")?.to_string();
    let fields = args[1].as_object().ok_or_else(|| {
        type_error(
            "object",
            args[1].type_name_str(),
            "notify.journal_structured fields",
        )
    })?;

    let priority = fields
        .get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("info")
        .to_string();

    let tag = fields
        .get("tag")
        .or_else(|| fields.get("unit"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let prio = level_to_priority(&priority);
    let mut cmd_args: Vec<String> = vec!["-p".to_string(), prio];

    if let Some(t) = &tag {
        cmd_args.push("-t".to_string());
        cmd_args.push(t.clone());
    }

    let mut full_message = message;
    for (k, v) in fields.iter() {
        if k == "priority" || k == "tag" || k == "unit" {
            continue;
        }
        let val_str = if let Some(s) = v.as_str() {
            s.to_string()
        } else {
            v.display_string()
        };
        full_message.push_str(&format!(" {}={}", k, val_str));
    }

    cmd_args.push(full_message);
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_logger(&str_args)
}

// ── helpers ────────────────────────────────────────────────────────────────

fn level_to_priority(level: &str) -> String {
    match level.to_lowercase().as_str() {
        "emerg" | "emergency" => "emerg",
        "alert" => "alert",
        "crit" | "critical" => "crit",
        "err" | "error" => "err",
        "warn" | "warning" => "warning",
        "notice" => "notice",
        "info" => "info",
        "debug" => "debug",
        other => other,
    }
    .to_string()
}

fn run_notify_send(cmd_args: &[&str]) -> HudHudResult<Value16> {
    run_command("notify-send", cmd_args)
}

fn run_logger(cmd_args: &[&str]) -> HudHudResult<Value16> {
    run_command("logger", cmd_args)
}

fn run_command(program: &str, cmd_args: &[&str]) -> HudHudResult<Value16> {
    let result = std::process::Command::new(program).args(cmd_args).output();

    let mut obj: HashMap<String, Value16> = HashMap::new();
    match result {
        Ok(output) => {
            let success = output.status.success();
            obj.insert("ok".to_string(), Value16::bool_(success));
            obj.insert(
                "code".to_string(),
                Value16::number(output.status.code().unwrap_or(-1) as f64),
            );
            if !success {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                obj.insert("error".to_string(), Value16::string(stderr));
            }
        }
        Err(e) => {
            obj.insert("ok".to_string(), Value16::bool_(false));
            obj.insert("code".to_string(), Value16::number(-1.0));
            obj.insert(
                "error".to_string(),
                Value16::string(format!("Failed to execute {}: {}", program, e)),
            );
        }
    }
    Ok(Value16::object(obj))
}

fn require_str<'a>(val: &'a Value16, ctx: &str) -> HudHudResult<&'a str> {
    val.as_str()
        .ok_or_else(|| type_error("string", val.type_name_str(), ctx))
}
