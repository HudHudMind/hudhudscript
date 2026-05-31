//! System metrics operation identifiers and zero-cost dispatch.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, HudHudResult};

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    CpuCount,
    CpuUsage,
    Memory,
    DiskUsage,
    LoadAverage,
    Uptime,
    Hostname,
    NetworkInterfaces,
    Processes,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cpu_count" => Ok(Self::CpuCount),
            "cpu_usage" => Ok(Self::CpuUsage),
            "memory" => Ok(Self::Memory),
            "disk_usage" => Ok(Self::DiskUsage),
            "load_average" => Ok(Self::LoadAverage),
            "uptime" => Ok(Self::Uptime),
            "hostname" => Ok(Self::Hostname),
            "network_interfaces" => Ok(Self::NetworkInterfaces),
            "processes" => Ok(Self::Processes),
            _ => Err(super::utils::runtime_error(format!(
                "Unknown method: {}",
                s
            ))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::CpuCount => super::cpu::sys_cpu_count(args),
        ScriptMethodId::CpuUsage => super::cpu::sys_cpu_usage(args),
        ScriptMethodId::Memory => super::memory::sys_memory(args),
        ScriptMethodId::DiskUsage => super::disk::sys_disk_usage(args),
        ScriptMethodId::LoadAverage => super::system::sys_load_average(args),
        ScriptMethodId::Uptime => super::system::sys_uptime(args),
        ScriptMethodId::Hostname => super::system::sys_hostname(args),
        ScriptMethodId::NetworkInterfaces => super::network::sys_network_interfaces(args),
        ScriptMethodId::Processes => super::process::sys_processes(args),
    }
}
