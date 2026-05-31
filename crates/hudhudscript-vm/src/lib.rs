//! HudHudScript Virtual Machine — bytecode execution engine.
//!
//! This crate is intentionally a thin wrapper around `vm.rs`. It exists as a
//! standalone crate (rather than being absorbed into another) because:
//!
//! 1. **Dependency boundary**: the VM has heavy deps (parking_lot,
//!    chrono, regex, ...) that should not leak into crates that only need bytecode
//!    types (e.g. compilers, formatters). Keeping the VM separate prevents
//!    transitive bloat.
//!
//! 2. **Build parallelism**: cargo can build the VM in parallel with consumers
//!    of `hudhudscript-bytecode` because they don't share the VM's dep tree.
//!
//! 3. **Optional execution**: tooling (linters, formatters, LSP) can depend on
//!    `hudhudscript-bytecode` for AST/IR types without pulling in the runtime.
//!
//! Issue #823 (v0.4.47.9): documented the rationale for keeping this thin wrapper.

pub mod stdlib;
pub mod vm;
pub mod wasm_compat;

pub use stdlib::register_vm_stdlib_modules;
pub use vm::*;
