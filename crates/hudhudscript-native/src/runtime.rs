//! Integration trait and default runtime for connecting native libraries
//! to the HudHudScript interpreter.
//!
//! The interpreter does NOT need to depend on this crate.  Instead, it can
//! accept any `dyn NativeCallable` and dispatch native calls through it.

use crate::error::NativeError;
use crate::ffi::NativeFunction;
use crate::loader::NativeLoader;
use crate::types::{NativeType, NativeValue};

/// Trait that the HudHudScript interpreter can use to invoke native C/C++ functions
/// without depending on this crate directly.
///
/// A blanket implementation is provided by [`NativeRuntime`].
pub trait NativeCallable {
    /// Call `library::function(args)` and return the result.
    fn call_native(
        &mut self,
        library: &str,
        function: &str,
        args: Vec<NativeValue>,
    ) -> Result<NativeValue, String>;

    /// Check whether a particular library/function pair is available.
    fn is_native_available(&self, library: &str, function: &str) -> bool;
}

/// Default implementation of [`NativeCallable`] backed by [`NativeLoader`].
pub struct NativeRuntime {
    loader: NativeLoader,
}

impl std::fmt::Debug for NativeRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeRuntime")
            .field("loader", &self.loader)
            .finish()
    }
}

impl Default for NativeRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRuntime {
    /// Create a runtime with default search paths.
    pub fn new() -> Self {
        Self {
            loader: NativeLoader::new(),
        }
    }

    /// Create a runtime backed by the provided loader.
    pub fn with_loader(loader: NativeLoader) -> Self {
        Self { loader }
    }

    /// Get a mutable reference to the underlying loader (e.g. to add search paths
    /// or register functions).
    pub fn loader_mut(&mut self) -> &mut NativeLoader {
        &mut self.loader
    }

    /// Get an immutable reference to the underlying loader.
    pub fn loader(&self) -> &NativeLoader {
        &self.loader
    }

    /// Convenience: register a function on an already-loaded library.
    pub fn register_function(
        &mut self,
        library: &str,
        name: &str,
        param_types: Vec<NativeType>,
        return_type: NativeType,
    ) -> Result<(), NativeError> {
        self.loader.register_function(
            library,
            NativeFunction {
                name: name.to_owned(),
                param_types,
                return_type,
            },
        )
    }
}

impl NativeCallable for NativeRuntime {
    fn call_native(
        &mut self,
        library: &str,
        function: &str,
        args: Vec<NativeValue>,
    ) -> Result<NativeValue, String> {
        // Ensure library is loaded.
        if !self.loader.is_loaded(library) {
            self.loader
                .load_library(library)
                .map_err(|e| e.to_string())?;
        }

        self.loader
            .call_function(library, function, &args)
            .map_err(|e| e.to_string())
    }

    fn is_native_available(&self, library: &str, function: &str) -> bool {
        if let Some(lib) = self.loader.get_library(library) {
            lib.has_symbol(function)
        } else {
            false
        }
    }
}
