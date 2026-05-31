//! NativeLibrary construction, registration, and the public [`call`] API.

use std::path::Path;

use crate::error::{NativeError, Result};
use crate::types::{NativeType, NativeValue};

use super::{NativeFunction, NativeLibrary};

impl NativeLibrary {
    /// Load a shared library from `path`.
    ///
    /// No functions are registered yet — use [`register_function`](Self::register_function)
    /// to declare callable symbols.
    pub fn load(path: &Path) -> Result<Self> {
        let lib =
            unsafe { libloading::Library::new(path) }.map_err(|e| NativeError::LibraryLoad {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_owned();

        Ok(Self {
            lib,
            name,
            functions: std::collections::HashMap::new(),
        })
    }

    /// Register a function signature so that [`call`](Self::call) can invoke it.
    pub fn register_function(&mut self, func: NativeFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    /// The library's human-readable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Iterate over registered function metadata.
    pub fn functions(&self) -> impl Iterator<Item = &NativeFunction> {
        self.functions.values()
    }

    /// Check whether a symbol with the given name exists in the library,
    /// regardless of whether it has been registered.
    pub fn has_symbol(&self, name: &str) -> bool {
        unsafe { self.lib.get::<*const ()>(name.as_bytes()).is_ok() }
    }

    /// Call a registered native function by name.
    ///
    /// The function must have been previously registered with [`register_function`](Self::register_function).
    /// Argument count is validated against the registered signature.
    pub fn call(&self, name: &str, args: &[NativeValue]) -> Result<NativeValue> {
        let func_meta = self
            .functions
            .get(name)
            .ok_or_else(|| NativeError::FunctionNotFound {
                function: name.to_owned(),
                library: self.name.clone(),
            })?;

        if args.len() != func_meta.param_types.len() {
            return Err(NativeError::ArgumentCount {
                function: name.to_owned(),
                expected: func_meta.param_types.len(),
                got: args.len(),
            });
        }

        match func_meta.return_type {
            NativeType::Void => self.call_void(name, args),
            NativeType::Int32 => self.call_returning_i32(name, args),
            NativeType::Int64 => self.call_returning_i64(name, args),
            NativeType::Float64 => self.call_returning_f64(name, args),
            NativeType::Bool => self.call_returning_bool(name, args),
            NativeType::String => self.call_returning_string(name, args),
            NativeType::Pointer | NativeType::Array => self.call_returning_pointer(name, args),
        }
    }
}
