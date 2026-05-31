//! Shared security-scanning builtins — suid_files, check_ssl,
//! world_writable, open_ports, failed_logins, check_permissions.
//!
//! Single source of truth for the VM and interpreter runtimes (Kural 7).
//! Wraps system tools: find, openssl, ss/netstat, journalctl, stat.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

pub fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use super::*;
use std::collections::HashMap;
use std::process::Command;

pub fn require_str<'a>(args: &'a [Value16], idx: usize, method: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(type_error("string", "missing", method)),
    }
}
