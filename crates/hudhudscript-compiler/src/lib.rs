//! HudHudScript Bytecode Compiler
//!
//! Compiles HudHudScript AST to bytecode for faster execution

pub mod bundle;
pub mod bytecode;
pub mod compiler;
pub mod error;
pub mod optimizer;
pub mod serialize;
// VM has been moved to hudhudscript-vm crate
mod vm;

pub use bundle::{create_bundle, load_bundle, BundleManifest};
pub use bytecode::{Bytecode, Instruction, Value16};
pub use compiler::Compiler;
pub use error::{compile_codes, CompileError, CompileResult, SourcePosition};
pub use optimizer::{optimize, OptimizationLevel};
pub use serialize::{
    deserialize_bytecode, load_hudc, save_hudc, serialize_bytecode, HudcHeader, MAGIC_BYTES,
};
// Re-export VM and OutputLocale so external code and tests can use
// `hudhudscript_compiler::VM` without depending on hudhudscript-vm directly.
pub use hudhudscript_vm::OutputLocale;
pub use hudhudscript_vm::VM;
pub use vm::RuntimeVM;
