//! Unified module resolution trait — Issue #921.
//!
//! Both Interpreter and VM should use this trait for resolving module imports.
//! The actual loading mechanism differs (source vs bytecode) but path resolution
//! and module content abstraction are shared.

/// The content of a resolved module.
pub enum ModuleContent {
    /// Source code to be parsed (.hud, .hudhud files)
    Source(String),
    /// Pre-compiled bytecode (.hudb files)
    Bytecode(Vec<u8>),
    /// Native/built-in module (no file needed)
    Native { name: String },
}

/// Trait for resolving module imports across runtimes.
///
/// `Send + Sync` bound lets `VM` (which holds `Option<Box<dyn
/// ModuleResolver>>`) cross thread boundaries — required by the
/// `hudi debug` REPL which spawns the VM on a worker thread.
pub trait ModuleResolver: Send + Sync {
    /// Resolve a module path relative to the importing file.
    /// Returns the module content (source, bytecode, or native).
    fn resolve(&self, path: &str, from: Option<&str>) -> Result<ModuleContent, crate::Error>;

    /// Check if a module exists at the given path.
    fn exists(&self, path: &str) -> bool;

    /// List available native modules.
    fn native_modules(&self) -> Vec<String> {
        Vec::new()
    }
}
