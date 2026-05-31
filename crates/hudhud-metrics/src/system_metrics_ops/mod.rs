//! Shared system-metrics builtins — CPU, memory, disk, load, uptime,
//! hostname, network interfaces, processes (Kural 7).
//!
//! Single source of truth for the VM and interpreter runtimes.
//! Reads from `/proc` on Linux; sensible defaults / platform-specific fall
//! back elsewhere. Uses `libc` for `sysconf`, `statvfs`, `gethostname`.

pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod process;
pub mod system;
pub mod types;
pub mod utils;

pub use types::{dispatch, ScriptMethodId};
