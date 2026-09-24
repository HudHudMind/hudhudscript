//! Hudhud target system — architecture ≠ OS ≠ ABI (JIT_AOT_ARCHITECTURE
//! §8). Everything addresses targets by full triple; device names never
//! enter the compiler.

pub mod arch;
pub mod spec;

pub use arch::{Abi, Architecture, Endianness, OperatingSystem};
pub use spec::{parse_triple, CpuSpec, TargetSpec};
