//! Shared UFW firewall wrapper — single source of truth (Kural 7).
//!
//! Provides: status, rules, allow, deny, delete_rule, enable, disable, reset.

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
use std::process::Command;

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Status,
    Rules,
    Allow,
    Deny,
    DeleteRule,
    Enable,
    Disable,
    Reset,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "status" => Ok(Self::Status),
            "rules" => Ok(Self::Rules),
            "allow" => Ok(Self::Allow),
            "deny" => Ok(Self::Deny),
            "delete_rule" => Ok(Self::DeleteRule),
            "enable" => Ok(Self::Enable),
            "disable" => Ok(Self::Disable),
            "reset" => Ok(Self::Reset),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Status => fw_status(args),
        ScriptMethodId::Rules => fw_rules(args),
        ScriptMethodId::Allow => fw_allow(args),
        ScriptMethodId::Deny => fw_deny(args),
        ScriptMethodId::DeleteRule => fw_delete_rule(args),
        ScriptMethodId::Enable => fw_enable(args),
        ScriptMethodId::Disable => fw_disable(args),
        ScriptMethodId::Reset => fw_reset(args),
    }
}

/// Main entry point (kept for backward compat).

/// Guard + constructor for every `sudo` invocation in this module.
///
/// Privilege escalation is opt-in and off by default (see
/// `hudhudscript_bytecode::privileged_ops`): `dispatch` takes no policy context,
/// and unit tests call it directly, so without this guard `cargo test` really ran
/// `sudo` against the developer's machine.
#[inline]
fn sudo_cmd(op: &str) -> HudHudResult<Command> {
    hudhudscript_bytecode::privileged_ops::ensure_privileged_ops_allowed(op)?;
    Ok(Command::new("sudo"))
}

pub fn fw_status(_args: &[Value16]) -> HudHudResult<Value16> {
    let output = sudo_cmd("firewall")?
        .args(["ufw", "status", "verbose"])
        .output()
        .map_err(|e| runtime_error(format!("firewall.status: {e}")))?;

    if !output.status.success() {
        return Err(runtime_error(format!(
            "firewall.status: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut active = false;
    let mut default_incoming = String::new();
    let mut default_outgoing = String::new();
    let mut rules_count: usize = 0;
    let mut in_rules = false;

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Status:") {
            active = line.contains("active");
        } else if line.starts_with("Default:") {
            let parts = line.strip_prefix("Default:").unwrap_or("").trim();
            for segment in parts.split(',') {
                let segment = segment.trim();
                if segment.contains("(incoming)") {
                    default_incoming = segment.replace("(incoming)", "").trim().to_string();
                } else if segment.contains("(outgoing)") {
                    default_outgoing = segment.replace("(outgoing)", "").trim().to_string();
                }
            }
        } else if line.starts_with("--") {
            in_rules = true;
        } else if in_rules && !line.is_empty() {
            rules_count += 1;
        }
    }

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("active".to_string(), Value16::bool_(active));
    obj.insert(
        "rules_count".to_string(),
        Value16::number(rules_count as f64),
    );
    obj.insert(
        "default_incoming".to_string(),
        Value16::string(default_incoming),
    );
    obj.insert(
        "default_outgoing".to_string(),
        Value16::string(default_outgoing),
    );
    Ok(Value16::object(obj))
}

pub fn fw_rules(_args: &[Value16]) -> HudHudResult<Value16> {
    let output = sudo_cmd("firewall")?
        .args(["ufw", "status", "numbered"])
        .output()
        .map_err(|e| runtime_error(format!("firewall.rules: {e}")))?;

    if !output.status.success() {
        return Err(runtime_error(format!(
            "firewall.rules: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut rules: Vec<Value16> = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with('[') {
            continue;
        }
        let close_bracket = match line.find(']') {
            Some(idx) => idx,
            None => continue,
        };
        let num_str = line[1..close_bracket].trim();
        let number: f64 = match num_str.parse() {
            Ok(n) => n,
            Err(_) => continue,
        };

        let rest = line[close_bracket + 1..].trim();
        let tokens: Vec<&str> = rest.split_whitespace().collect();

        let (port, protocol) = if let Some(target) = tokens.first() {
            let target = target.trim_end_matches("(v6)").trim();
            if let Some((p, proto)) = target.split_once('/') {
                (p.to_string(), proto.to_string())
            } else {
                (target.to_string(), String::new())
            }
        } else {
            (String::new(), String::new())
        };

        let action = tokens.get(1).unwrap_or(&"").to_string();
        let direction = tokens.get(2).unwrap_or(&"").to_string();

        let from = if direction == "IN" {
            tokens.get(3).unwrap_or(&"Anywhere").to_string()
        } else {
            String::from("Anywhere")
        };
        let to = if direction == "OUT" {
            tokens.get(3).unwrap_or(&"Anywhere").to_string()
        } else {
            String::from("Anywhere")
        };

        let mut rule = hudhudscript_bytecode::ObjMap::default();
        rule.insert("number".to_string(), Value16::number(number));
        rule.insert("action".to_string(), Value16::string(action));
        rule.insert("direction".to_string(), Value16::string(direction));
        rule.insert("from".to_string(), Value16::string(from));
        rule.insert("to".to_string(), Value16::string(to));
        rule.insert("port".to_string(), Value16::string(port));
        rule.insert("protocol".to_string(), Value16::string(protocol));
        rules.push(Value16::object(rule));
    }
    Ok(Value16::array(rules))
}

pub fn fw_allow(args: &[Value16]) -> HudHudResult<Value16> {
    let spec = port_spec(args, "firewall.allow")?;
    run_cmd_result(
        sudo_cmd("firewall")?.args(["ufw", "allow", &spec]),
        "firewall.allow",
    )
}

pub fn fw_deny(args: &[Value16]) -> HudHudResult<Value16> {
    let spec = port_spec(args, "firewall.deny")?;
    run_cmd_result(
        sudo_cmd("firewall")?.args(["ufw", "deny", &spec]),
        "firewall.deny",
    )
}

pub fn fw_delete_rule(args: &[Value16]) -> HudHudResult<Value16> {
    let number = require_number(args, 0, "firewall.delete_rule")? as u64;
    run_cmd_result(
        sudo_cmd("firewall")?.args(["ufw", "--force", "delete", &number.to_string()]),
        "firewall.delete_rule",
    )
}

pub fn fw_enable(_args: &[Value16]) -> HudHudResult<Value16> {
    run_cmd_result(
        sudo_cmd("firewall")?.args(["ufw", "--force", "enable"]),
        "firewall.enable",
    )
}

pub fn fw_disable(_args: &[Value16]) -> HudHudResult<Value16> {
    run_cmd_result(
        sudo_cmd("firewall")?.args(["ufw", "disable"]),
        "firewall.disable",
    )
}

pub fn fw_reset(_args: &[Value16]) -> HudHudResult<Value16> {
    run_cmd_result(
        sudo_cmd("firewall")?.args(["ufw", "reset", "--force"]),
        "firewall.reset",
    )
}

// ── helpers ────────────────────────────────────────────────────────────────

fn port_spec(args: &[Value16], op: &str) -> HudHudResult<String> {
    let port = require_string(args, 0, op)?;
    let protocol = args
        .get(1)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    match protocol {
        Some(proto) => Ok(format!("{}/{}", port, proto)),
        None => Ok(port),
    }
}

fn require_string(args: &[Value16], idx: usize, op: &str) -> HudHudResult<String> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| type_error("string", v.type_name_str(), op)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            op, idx
        ))),
    }
}

fn require_number(args: &[Value16], idx: usize, op: &str) -> HudHudResult<f64> {
    match args.get(idx) {
        Some(v) => v
            .as_number()
            .ok_or_else(|| type_error("number", v.type_name_str(), op)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            op, idx
        ))),
    }
}

fn run_cmd_result(cmd: &mut Command, op: &str) -> HudHudResult<Value16> {
    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("{}: {}", op, e)))?;
    let ok = output.status.success();
    let msg = if ok {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    };
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("ok".to_string(), Value16::bool_(ok));
    obj.insert("message".to_string(), Value16::string(msg));
    Ok(Value16::object(obj))
}
