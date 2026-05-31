// VM has been moved to hudhudscript-vm crate.
// Re-export for backward compatibility so `crate::vm::VM` and
// `crate::vm::OutputLocale` continue to resolve in tests and other code.
// Re-exports removed — no longer used from this path.

// --- Runtime trait implementation for VM via newtype (Issue #912 Phase 2) ---
//
// The VM crate cannot depend on hudhudscript-compiler (circular dependency),
// and the orphan rule prevents implementing a foreign trait for a foreign type.
// We use a newtype wrapper that delegates to the inner VM and additionally
// provides the compile step needed by `execute_source`.

use std::ops::{Deref, DerefMut};

/// Newtype wrapper around [`hudhudscript_vm::VM`] that implements
/// [`hudhudscript_errors::Runtime`].
///
/// Use this when you need a `Runtime`-compatible VM. It derefs to the
/// inner `VM` so all existing VM methods remain accessible.
pub struct RuntimeVM(pub hudhudscript_vm::VM);

impl RuntimeVM {
    pub fn new() -> Self {
        Self(hudhudscript_vm::VM::new())
    }
}

impl Default for RuntimeVM {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for RuntimeVM {
    type Target = hudhudscript_vm::VM;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RuntimeVM {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl hudhudscript_errors::Runtime for RuntimeVM {
    type Value = hudhudscript_bytecode::Value16;

    fn execute_source(&mut self, source: &str) -> Result<(), hudhudscript_errors::Error> {
        let ast = hudhudscript_parser::parse(source)?;
        let mut compiler = crate::Compiler::new();
        let bytecode = compiler.compile(&ast)?;
        self.0.execute(&bytecode)?;
        Ok(())
    }

    fn get_variable(&self, name: &str) -> Option<Self::Value> {
        self.0.get_variable(name).cloned()
    }
}
