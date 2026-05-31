//! Shared security-scanning builtins — suid_files, check_ssl,
//! world_writable, open_ports, failed_logins, check_permissions.
//!
//! Single source of truth for the VM and interpreter runtimes (Kural 7).
//! Wraps system tools: find, openssl, ss/netstat, journalctl, stat.

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

/// Main entry point used by the VM's module dispatcher.
/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    SuidFiles,
    CheckSsl,
    WorldWritable,
    OpenPorts,
    FailedLogins,
    CheckPermissions,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "suid_files" => Ok(Self::SuidFiles),
            "check_ssl" => Ok(Self::CheckSsl),
            "world_writable" => Ok(Self::WorldWritable),
            "open_ports" => Ok(Self::OpenPorts),
            "failed_logins" => Ok(Self::FailedLogins),
            "check_permissions" => Ok(Self::CheckPermissions),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::SuidFiles => sec_suid_files(args),
        ScriptMethodId::CheckSsl => sec_check_ssl(args),
        ScriptMethodId::WorldWritable => sec_world_writable(args),
        ScriptMethodId::OpenPorts => sec_open_ports(args),
        ScriptMethodId::FailedLogins => sec_failed_logins(args),
        ScriptMethodId::CheckPermissions => sec_check_permissions(args),
    }
}

mod check;
/// Main entry point (kept for backward compat).
mod scan;
mod util;

pub use check::*;
pub use scan::*;
pub use util::*;
