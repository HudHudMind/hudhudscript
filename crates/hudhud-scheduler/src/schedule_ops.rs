//! Shared schedule/cron builtin — used by both VM and interpreter.
//!
//! Provides: schedule.cron, schedule.parse_cron

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

/// Execute a schedule method on the given arguments.
/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Cron,
    ParseCron,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cron" => Ok(Self::Cron),
            "parse_cron" => Ok(Self::ParseCron),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Cron => schedule_cron(args),
        ScriptMethodId::ParseCron => parse_cron(args),
    }
}

/// Main entry point (kept for backward compat).

fn schedule_cron(args: &[Value16]) -> HudHudResult<Value16> {
    let expr = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("schedule.cron: expected string"))?
        .to_string();

    let fields: Vec<&str> = expr.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(runtime_error(format!(
            "schedule.cron: expected 5-field cron expression, got {} fields",
            fields.len()
        )));
    }

    let mut result = HashMap::new();
    result.insert("expression".to_string(), Value16::string(expr));
    result.insert("type".to_string(), Value16::string("cron".to_string()));
    result.insert("active".to_string(), Value16::bool_(true));
    Ok(Value16::object(result))
}

fn parse_cron(args: &[Value16]) -> HudHudResult<Value16> {
    let expr = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("schedule.parse_cron: expected string"))?
        .to_string();

    let fields: Vec<&str> = expr.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(runtime_error(format!(
            "schedule.parse_cron: expected 5 fields, got {}",
            fields.len()
        )));
    }

    let mut result = HashMap::new();
    result.insert("minute".to_string(), Value16::string(fields[0].to_string()));
    result.insert("hour".to_string(), Value16::string(fields[1].to_string()));
    result.insert("dom".to_string(), Value16::string(fields[2].to_string()));
    result.insert("month".to_string(), Value16::string(fields[3].to_string()));
    result.insert("dow".to_string(), Value16::string(fields[4].to_string()));
    Ok(Value16::object(result))
}
