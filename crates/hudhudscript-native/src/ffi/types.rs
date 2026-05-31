//! FFI types — [`NativeFunction`] and [`NativeLibrary`] metadata.

use std::collections::HashMap;

use crate::types::{NativeType, NativeValue};

/// Metadata describing a single exported native function.
#[derive(Debug, Clone)]
pub struct NativeFunction {
    /// Symbol name in the shared library.
    pub name: String,
    /// Expected parameter types (positional).
    pub param_types: Vec<NativeType>,
    /// Return type.
    pub return_type: NativeType,
}

/// A loaded native (C/C++) shared library together with its registered function metadata.
pub struct NativeLibrary {
    /// The underlying dynamically loaded library handle.
    #[allow(dead_code)]
    pub(crate) lib: libloading::Library,
    /// Human-readable name for diagnostics.
    pub(crate) name: String,
    /// Registered function signatures, keyed by symbol name.
    pub(crate) functions: HashMap<String, NativeFunction>,
}

// libloading::Library is Send+Sync when the platform supports it.
// We only call into it behind &self so this is safe in practice.
unsafe impl Send for NativeLibrary {}
unsafe impl Sync for NativeLibrary {}

impl std::fmt::Debug for NativeLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeLibrary")
            .field("name", &self.name)
            .field("functions", &self.functions.keys().collect::<Vec<_>>())
            .finish()
    }
}
