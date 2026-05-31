use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

/// Zero-cost builtin function pointer (real code, no wrapper).
pub type BuiltinFn = Box<dyn Fn(&[Value16]) -> HudHudResult<Value16> + Send + Sync>;

/// Legacy module-wide handler — dispatches every method on the module
/// via a single closure.  Used only while migrating modules from
/// stringly-typed `handle_method` to per-method `BuiltinFn`s.
pub type ModuleMethodHandler =
    Box<dyn Fn(&str, Vec<Value16>) -> CompileResult<Value16> + Send + Sync>;

/// Registry of builtin methods indexed by (module, method) string pair.
///
/// Supports both per-method registration (`register_method`) and legacy
/// per-module registration (`register_module`) during the transition.
pub struct ModuleRegistry {
    methods: rustc_hash::FxHashMap<String, rustc_hash::FxHashMap<String, BuiltinFn>>,
    modules: rustc_hash::FxHashMap<String, ModuleMethodHandler>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            methods: rustc_hash::FxHashMap::default(),
            modules: rustc_hash::FxHashMap::default(),
        }
    }

    /// Register a single builtin method under `module_name::method_name`.
    pub fn register_method(&mut self, module_name: &str, method_name: &str, handler: BuiltinFn) {
        self.methods
            .entry(module_name.to_string())
            .or_default()
            .insert(method_name.to_string(), handler);
    }

    /// Register a legacy module-wide handler.
    pub fn register_module(&mut self, module_name: &str, handler: ModuleMethodHandler) {
        self.modules.insert(module_name.to_string(), handler);
    }

    /// Look up and call a registered builtin.
    ///
    /// 1. Try per-method registry first (zero-cost fn pointer).
    /// 2. Fall back to legacy module-wide handler.
    pub fn call(
        &self,
        module_name: &str,
        method: &str,
        args: Vec<Value16>,
    ) -> Option<CompileResult<Value16>> {
        if let Some(methods) = self.methods.get(module_name) {
            if let Some(f) = methods.get(method) {
                return Some(f(&args).map_err(|e| {
                    hudhudscript_bytecode::error::compile_codes::runtime_error(e.to_string())
                }));
            }
        }
        self.modules.get(module_name).map(|h| h(method, args))
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
