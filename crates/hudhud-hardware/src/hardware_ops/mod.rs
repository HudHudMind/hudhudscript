//! Shared hardware detection builtins — reads /proc, /sys, lspci, lsblk, lsusb, aplay, xrandr.
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).
//! All lookups are Linux-specific; non-Linux returns empty / "unknown".

mod audio_display_ops;
mod cpu_ops;
mod device_ops;
mod dispatch;
mod gpu_ops;
mod memory_ops;
mod storage_ops;

pub use audio_display_ops::{hw_audio_devices, hw_display_info};
pub use cpu_ops::hw_cpu_info;
pub use device_ops::{hw_network_adapters, hw_usb_devices};
pub use dispatch::{dispatch, ScriptMethodId};
pub use gpu_ops::hw_gpu_info;
pub use memory_ops::hw_memory_info;
pub use storage_ops::hw_disk_info;
