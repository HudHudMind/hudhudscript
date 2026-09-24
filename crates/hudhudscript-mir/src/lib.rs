//! Hudhud MIR — backend-independent, SSA-like middle representation
//! (JIT_AOT_ARCHITECTURE.md §5).
//!
//! AŞAMA-0 skeleton scope (honest limitations, widened in later steps):
//! - linear SSA within blocks; no block parameters (phi) yet,
//! - verifier checks structure, types and def-before-use in block order
//!   (full dominance/phi verification arrives with the P3 gate),
//! - lowering covers the typed literal/local/binary/call-print subset.
//!
//! Semantics owner is the VM; MIR is verified against it via differential
//! tests — MIR is never a second source of truth.

pub mod builder;
pub mod lower;
pub mod lower_typed;
pub mod param_infer;
pub mod mir;
pub mod print;
pub mod specialize;
pub mod types;
pub mod verify;

pub use builder::MirFunctionBuilder;
pub use lower::{lower_function, LowerError};
pub use lower_typed::{lower_function_typed, lower_function_typed_in_module, lower_module_typed};
pub use mir::*;
pub use print::render_function;
pub use specialize::specialize_module;
pub use verify::{verify_function, VerifyError};

#[cfg(test)]
mod builder_tests;
#[cfg(test)]
mod mir_tests;
#[cfg(test)]
mod verify_tests;
