//! GPU operation identifiers and zero-cost dispatch.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, HudHudResult};

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    List,
    Usage,
    Memory,
    Driver,
    CudaAvailable,
    RocmAvailable,
    SetVisible,
    Processes,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "list" => Ok(Self::List),
            "usage" => Ok(Self::Usage),
            "memory" => Ok(Self::Memory),
            "driver" => Ok(Self::Driver),
            "cuda_available" => Ok(Self::CudaAvailable),
            "rocm_available" => Ok(Self::RocmAvailable),
            "set_visible" => Ok(Self::SetVisible),
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
        ScriptMethodId::List => super::query::gpu_list(args),
        ScriptMethodId::Usage => super::query::gpu_usage(args),
        ScriptMethodId::Memory => super::query::gpu_memory(args),
        ScriptMethodId::Driver => super::query::gpu_driver(args),
        ScriptMethodId::CudaAvailable => super::control::gpu_cuda_available(args),
        ScriptMethodId::RocmAvailable => super::control::gpu_rocm_available(args),
        ScriptMethodId::SetVisible => super::control::gpu_set_visible(args),
        ScriptMethodId::Processes => super::query::gpu_processes(args),
    }
}
