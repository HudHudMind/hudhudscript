//! Module loader - loads modules from files

use crate::module::{Module, ModuleId};
use hudhudscript_ast::{Decl, Stmt};
use hudhudscript_errors::module_trait::{ModuleContent, ModuleResolver as ModuleResolverTrait};
use hudhudscript_parser::parse;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Module loader error
#[derive(Debug)]
pub enum ModuleLoaderError {
    ModuleNotFound(String),
    ReadError(String),
    ParseError(String),
    AlreadyLoaded(String),
}

impl std::fmt::Display for ModuleLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ModuleLoaderError::ModuleNotFound(s) => write!(f, "Module not found: {}", s),
            ModuleLoaderError::ReadError(s) => write!(f, "Failed to read module: {}", s),
            ModuleLoaderError::ParseError(s) => write!(f, "Failed to parse module: {}", s),
            ModuleLoaderError::AlreadyLoaded(s) => write!(f, "Module already loaded: {}", s),
        }
    }
}

impl std::error::Error for ModuleLoaderError {}

/// Module loader - loads and caches modules
pub struct ModuleLoader {
    /// Module cache (path -> module)
    cache: Arc<RwLock<HashMap<ModuleId, Module>>>,

    /// Base path for module resolution
    base_path: PathBuf,
}

impl ModuleLoader {
    /// Create new module loader
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            base_path,
        }
    }

    /// Load a module from file
    pub async fn load(&self, path: &str) -> Result<Module, ModuleLoaderError> {
        let resolved_path = self.resolve_path(path)?;

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(module) = cache.get(&resolved_path) {
                return Ok(module.clone());
            }
        }

        // Read file
        let source = fs::read_to_string(&resolved_path)
            .map_err(|e| ModuleLoaderError::ReadError(e.to_string()))?;

        // Parse
        let ast = parse(&source).map_err(|e| ModuleLoaderError::ParseError(format!("{:?}", e)))?;

        // Create module
        let mut module = Module::new(resolved_path.clone(), source, ast);

        // Extract imports and exports
        self.extract_imports_exports(&mut module);

        // Cache module
        self.cache
            .write()
            .await
            .insert(resolved_path, module.clone());

        Ok(module)
    }

    /// Get cached module
    pub async fn get_cached(&self, path: &str) -> Option<Module> {
        let resolved_path = self.resolve_path(path).ok()?;
        self.cache.read().await.get(&resolved_path).cloned()
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// Resolve module path
    pub fn resolve_path(&self, path: &str) -> Result<PathBuf, ModuleLoaderError> {
        let path = if path.starts_with("./") || path.starts_with("../") {
            // Relative path
            self.base_path.join(path)
        } else {
            // Absolute or package path
            PathBuf::from(path)
        };

        // Add .hudhud extension if missing
        let path = if path.extension().is_none() {
            path.with_extension("hudhud")
        } else {
            path
        };

        if !path.exists() {
            return Err(ModuleLoaderError::ModuleNotFound(
                path.display().to_string(),
            ));
        }

        Ok(path)
    }

    /// Extract imports and exports from AST
    fn extract_imports_exports(&self, module: &mut Module) {
        let mut imports = Vec::new();

        for stmt in &module.ast {
            if let Stmt::Decl(Decl::Import {
                module: import_module,
                alias,
                ..
            }) = stmt
            {
                let import_name = alias.clone().unwrap_or_else(|| import_module.clone());
                imports.push((import_module.clone(), vec![import_name]));
            }
        }

        for (module_path, names) in imports {
            module.add_import(module_path, names);
        }
    }
}

// ---------------------------------------------------------------------------
// ModuleResolver trait implementation (Issue #921 Phase 2)
// ---------------------------------------------------------------------------
impl ModuleResolverTrait for ModuleLoader {
    /// Resolve a module path and return its source content.
    /// Uses sync `std::fs::read_to_string` since the trait is sync.
    /// The `from` parameter adjusts the base path for relative imports.
    fn resolve(
        &self,
        path: &str,
        from: Option<&str>,
    ) -> Result<ModuleContent, hudhudscript_errors::Error> {
        // If `from` is provided, resolve relative to the importing file's directory
        let resolved = if let Some(from_path) = from {
            let from_dir = std::path::Path::new(from_path)
                .parent()
                .unwrap_or(self.base_path.as_path());
            if path.starts_with("./") || path.starts_with("../") {
                let mut p = from_dir.join(path);
                if p.extension().is_none() {
                    p.set_extension("hudhud");
                }
                p
            } else {
                // Absolute or package path — fall through to normal resolution
                self.resolve_path(path)?
            }
        } else {
            self.resolve_path(path)?
        };

        // Check for bytecode companion file (.hudb)
        let bytecode_path = resolved.with_extension("hudb");
        if bytecode_path.exists() {
            let bytes = fs::read(&bytecode_path).map_err(|e| {
                hudhudscript_errors::Error::new(
                    hudhudscript_errors::ErrorCode::ModuleLoaderReadError,
                    format!(
                        "Failed to read bytecode file {}: {}",
                        bytecode_path.display(),
                        e
                    ),
                )
            })?;
            return Ok(ModuleContent::Bytecode(bytes));
        }

        // Read source file
        let source = fs::read_to_string(&resolved).map_err(|e| {
            hudhudscript_errors::Error::new(
                hudhudscript_errors::ErrorCode::ModuleLoaderReadError,
                format!("Failed to read module {}: {}", resolved.display(), e),
            )
        })?;

        Ok(ModuleContent::Source(source))
    }

    /// Check if a module exists at the given path.
    fn exists(&self, path: &str) -> bool {
        self.resolve_path(path).is_ok()
    }

    /// The ModuleLoader is file-based; native modules are registered elsewhere.
    /// Returns an empty list (uses the trait default).
    fn native_modules(&self) -> Vec<String> {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl ModuleLoaderError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ModuleLoaderError::AlreadyLoaded(..) => {
                hudhudscript_errors::ErrorCode::ModuleLoaderAlreadyLoaded
            }
            ModuleLoaderError::ModuleNotFound(..) => {
                hudhudscript_errors::ErrorCode::ModuleLoaderModuleNotFound
            }
            ModuleLoaderError::ParseError(..) => {
                hudhudscript_errors::ErrorCode::ModuleLoaderParseError
            }
            ModuleLoaderError::ReadError(..) => {
                hudhudscript_errors::ErrorCode::ModuleLoaderReadError
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<ModuleLoaderError> for hudhudscript_errors::Error {
    fn from(e: ModuleLoaderError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
