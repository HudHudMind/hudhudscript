//! Shared Env builtin — used by both VM and interpreter.
//!
//! Provides Env.get(), set(), remove(), has(), all(), all_unfiltered().

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvMethodId {
    Get,
    Set,
    Remove,
    Has,
    All,
    AllUnfiltered,
}

impl std::str::FromStr for EnvMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Self::Get),
            "set" => Ok(Self::Set),
            "remove" => Ok(Self::Remove),
            "has" => Ok(Self::Has),
            "all" => Ok(Self::All),
            "all_unfiltered" => Ok(Self::AllUnfiltered),
            _ => Err(runtime_error(format!("Unknown env method: {}", s))),
        }
    }
}

impl EnvMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Get => get(args),
            Self::Set => set(args),
            Self::Remove => remove(args),
            Self::Has => has(args),
            Self::All => all(args),
            Self::AllUnfiltered => all_unfiltered(args),
        }
    }
}

pub fn get(args: &[Value16]) -> HudHudResult<Value16> {
    let key = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Env.get: expected string key"))?
        .to_string();
    match std::env::var(&key) {
        Ok(val) => Ok(Value16::string(val)),
        Err(_) => match args.get(1) {
            Some(v) => Ok(v.clone()),
            None => Ok(Value16::null()),
        },
    }
}

pub fn set(args: &[Value16]) -> HudHudResult<Value16> {
    let key = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Env.set: expected string key"))?
        .to_string();
    let value = match args.get(1) {
        Some(v) => match v.as_str() {
            Some(s) => s.to_string(),
            None => v.display_string(),
        },
        None => String::new(),
    };
    std::env::set_var(&key, &value);
    Ok(Value16::null())
}

pub fn remove(args: &[Value16]) -> HudHudResult<Value16> {
    if let Some(key) = args.first().and_then(|v| v.as_str()) {
        std::env::remove_var(key);
    }
    Ok(Value16::null())
}

pub fn has(args: &[Value16]) -> HudHudResult<Value16> {
    let key = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Env.has: expected string key"))?
        .to_string();
    Ok(Value16::bool_(std::env::var(&key).is_ok()))
}

pub fn all(_args: &[Value16]) -> HudHudResult<Value16> {
    const SENSITIVE_PATTERNS: &[&str] = &[
        "SECRET",
        "KEY",
        "TOKEN",
        "PASSWORD",
        "CREDENTIAL",
        "PRIVATE",
    ];
    let mut obj = HashMap::new();
    for (key, val) in std::env::vars() {
        let upper = key.to_uppercase();
        if SENSITIVE_PATTERNS.iter().any(|p| upper.contains(p)) {
            continue;
        }
        obj.insert(key, Value16::string(val));
    }
    Ok(Value16::object(obj))
}

pub fn all_unfiltered(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut obj = HashMap::new();
    for (key, val) in std::env::vars() {
        obj.insert(key, Value16::string(val));
    }
    Ok(Value16::object(obj))
}
