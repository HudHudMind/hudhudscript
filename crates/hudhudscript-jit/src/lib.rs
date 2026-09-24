//! Hudhud JIT orchestration: source → parse → HIR → MIR → native.
//!
//! First-light scope: whole-module eager compilation with default i64
//! param types; scripts with a `main()` function run natively. Hotness
//! tiering and mixed VM/JIT execution arrive with later milestones.

#[cfg(any(feature = "gccjit", feature = "llvm"))]
mod extern_backend;

pub mod precheck;
pub mod runtime;

pub use runtime::{JitRunResult, JitRuntime};

#[cfg(test)]
mod tests;
