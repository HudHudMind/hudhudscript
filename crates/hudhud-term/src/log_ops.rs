//! Shared structured logging builtin — used by both VM and interpreter.
//!
//! Provides log.debug(), info(), warn(), error(), level(), setLevel().

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
use std::sync::Mutex;

/// Global log level storage (shared across runtimes).
fn shared_log_level() -> &'static Mutex<String> {
    static LOG_LEVEL: std::sync::OnceLock<Mutex<String>> = std::sync::OnceLock::new();
    LOG_LEVEL.get_or_init(|| Mutex::new("info".to_string()))
}

/// Enum identifying each log operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogMethodId {
    Level,
    SetLevel,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::str::FromStr for LogMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "level" => Ok(Self::Level),
            "setLevel" => Ok(Self::SetLevel),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => Err(runtime_error(format!("Unknown log method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for log operations.
pub fn dispatch(method: LogMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        LogMethodId::Level => {
            let level = shared_log_level().lock().unwrap().clone();
            return Ok(Value16::string(level));
        }
        LogMethodId::SetLevel => {
            let level = args
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("info")
                .to_string();
            let valid = matches!(
                level.as_str(),
                "debug" | "info" | "warn" | "error" | "trace"
            );
            if !valid {
                return Err(runtime_error(format!(
                    "Invalid log level: '{}'. Valid: debug, info, warn, error, trace",
                    level
                )));
            }
            *shared_log_level().lock().unwrap() = level;
            return Ok(Value16::bool_(true));
        }
        LogMethodId::Debug | LogMethodId::Info | LogMethodId::Warn | LogMethodId::Error => {}
    }

    let message = match args.first() {
        Some(v) => match v.as_str() {
            Some(s) => s.to_string(),
            None => v.display_string(),
        },
        None => String::new(),
    };

    let structured = if let Some(obj) = args.get(1).and_then(|v| v.as_object()) {
        let pairs: Vec<String> = obj
            .iter()
            .filter(|(k, _)| k.as_str() != "__module")
            .map(|(k, v)| format!("{}={}", k, v.display_string()))
            .collect();
        if pairs.is_empty() {
            String::new()
        } else {
            format!(" {}", pairs.join(" "))
        }
    } else {
        String::new()
    };

    let (level_str, color_code) = match method {
        LogMethodId::Debug => ("DEBUG", "36"),
        LogMethodId::Info => ("INFO", "32"),
        LogMethodId::Warn => ("WARN", "33"),
        LogMethodId::Error => ("ERROR", "31"),
        _ => unreachable!(),
    };

    eprintln!(
        "\x1b[{}m{}\x1b[0m — {}{}",
        color_code, level_str, message, structured
    );

    let mut entry = HashMap::new();
    entry.insert(
        "level".to_string(),
        Value16::string(level_str.to_lowercase()),
    );
    entry.insert("message".to_string(), Value16::string(message));
    if let Some(data) = args.get(1) {
        entry.insert("data".to_string(), data.clone());
    }
    Ok(Value16::object(entry))
}
