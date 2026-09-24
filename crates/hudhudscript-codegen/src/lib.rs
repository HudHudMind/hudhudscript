//! Backend abstraction layer (JIT_AOT_ARCHITECTURE.md §7).
//!
//! This crate contains ONLY traits, capability structs and shared
//! codegen types. Backend implementations (cranelift/llvm/gccjit) live in
//! their own crates and depend on this one; nothing here may depend on a
//! concrete backend, and no upper layer may branch on backend names —
//! behavior is selected exclusively through `BackendCapabilities`.

pub mod backend;
pub mod capabilities;
pub mod jit;

pub use backend::{BackendError, CodegenContext, CompiledFunction, CompiledModule, NativeBackend, OptGoal, OptLevel};
pub use capabilities::BackendCapabilities;
pub use jit::{JitConfig, JitEngine, NativeAddress, NativeFunction};
