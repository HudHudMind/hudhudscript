//! Shared Duration builtin — used by both VM and interpreter.
//!
//! Provides Duration.seconds(), minutes(), hours(), days(), millis().

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

/// Enum identifying each Duration operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationMethodId {
    Seconds,
    Minutes,
    Hours,
    Days,
    Millis,
}

impl std::str::FromStr for DurationMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seconds" => Ok(Self::Seconds),
            "minutes" => Ok(Self::Minutes),
            "hours" => Ok(Self::Hours),
            "days" => Ok(Self::Days),
            "millis" => Ok(Self::Millis),
            _ => Err(runtime_error(format!("Unknown Duration method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for Duration operations.
pub fn dispatch(method: DurationMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    let n = args.first().and_then(|v| v.as_number()).ok_or_else(|| {
        runtime_error(format!("Duration.{:?}: expected number", method).to_lowercase())
    })?;

    let (seconds, millis) = match method {
        DurationMethodId::Seconds => (n, n * 1000.0),
        DurationMethodId::Minutes => (n * 60.0, n * 60_000.0),
        DurationMethodId::Hours => (n * 3600.0, n * 3_600_000.0),
        DurationMethodId::Days => (n * 86400.0, n * 86_400_000.0),
        DurationMethodId::Millis => (n / 1000.0, n),
    };

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("seconds".to_string(), Value16::number(seconds));
    obj.insert("millis".to_string(), Value16::number(millis));
    Ok(Value16::object(obj))
}
