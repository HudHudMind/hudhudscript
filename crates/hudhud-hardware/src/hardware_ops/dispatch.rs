//! Dispatch enum and zero-cost dispatch function for hardware operations.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

use super::audio_display_ops::{hw_audio_devices, hw_display_info};
use super::cpu_ops::hw_cpu_info;
use super::device_ops::{hw_network_adapters, hw_usb_devices};
use super::gpu_ops::hw_gpu_info;
use super::memory_ops::hw_memory_info;
use super::storage_ops::hw_disk_info;

pub(super) fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    CpuInfo,
    MemoryInfo,
    GpuInfo,
    DiskInfo,
    NetworkAdapters,
    UsbDevices,
    AudioDevices,
    DisplayInfo,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cpu_info" => Ok(Self::CpuInfo),
            "memory_info" => Ok(Self::MemoryInfo),
            "gpu_info" => Ok(Self::GpuInfo),
            "disk_info" => Ok(Self::DiskInfo),
            "network_adapters" => Ok(Self::NetworkAdapters),
            "usb_devices" => Ok(Self::UsbDevices),
            "audio_devices" => Ok(Self::AudioDevices),
            "display_info" => Ok(Self::DisplayInfo),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::CpuInfo => hw_cpu_info(args),
        ScriptMethodId::MemoryInfo => hw_memory_info(args),
        ScriptMethodId::GpuInfo => hw_gpu_info(args),
        ScriptMethodId::DiskInfo => hw_disk_info(args),
        ScriptMethodId::NetworkAdapters => hw_network_adapters(args),
        ScriptMethodId::UsbDevices => hw_usb_devices(args),
        ScriptMethodId::AudioDevices => hw_audio_devices(args),
        ScriptMethodId::DisplayInfo => hw_display_info(args),
    }
}
