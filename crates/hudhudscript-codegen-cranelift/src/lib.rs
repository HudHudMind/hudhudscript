//! Cranelift backend — primary JIT engine (JIT_AOT_ARCHITECTURE §12).
//!
//! First-light scope (REAL, nothing silent):
//! - lowering: single-block functions of ConstInt terminated by Return
//!   (the `five() -> i64 { 5 }` class). Parameters, arithmetic and every
//!   other MIR construct are REJECTED with a clear BackendError naming
//!   the offending instruction — the accepted set widens one lane at a
//!   time, each widening parity-tested against the VM.
//! - codegen: host ISA via cranelift-native, executable memory via
//!   cranelift-jit (W^X handled by the JITModule).
//! - AOT object emission is NOT implemented: `emit_object` refuses with
//!   UNSUPPORTED and capabilities().aot == false.

mod aot;
mod backend;
mod translate;

pub use aot::{compile_to_object, entry_symbol, entry_symbols};

pub use backend::CraneliftBackend;
