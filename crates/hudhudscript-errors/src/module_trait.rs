//! Unified module resolution trait — Issue #921.
//!
//! Both Interpreter and VM should use this trait for resolving module imports.
//! The actual loading mechanism differs (source vs bytecode) but path resolution
//! and module content abstraction are shared.

/// The content of a resolved module.
pub enum ModuleContent {
    /// Source code to be parsed (.hud, .hudhud files).
    ///
    /// `file` is the resolved module file path. The loader uses its parent
    /// directory as the base for the module's OWN `use` imports (BULGU 4:
    /// a module's nested imports must resolve relative to the module file,
    /// not relative to the importing script's root).
    Source {
        content: String,
        file: Option<std::path::PathBuf>,
    },
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
