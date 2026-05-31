//! ScriptMethodId enum and zero-cost dispatch for security operations.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, HudHudResult};

use super::audit_ops::{sec_check_permissions, sec_check_ssl, sec_failed_logins};
use super::helpers::runtime_error;
use super::scan_ops::{sec_open_ports, sec_suid_files, sec_world_writable};

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
