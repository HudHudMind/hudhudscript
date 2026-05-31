//! Module path resolver

use std::path::PathBuf;

/// Resolver error
#[derive(Debug)]
pub enum ResolverError {
    InvalidPath(String),
    NotFound(String),
}

impl std::fmt::Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ResolverError::InvalidPath(s) => write!(f, "Invalid module path: {}", s),
            ResolverError::NotFound(s) => write!(f, "Module not found: {}", s),
        }
    }
}

impl std::error::Error for ResolverError {}

/// Module resolver - resolves module paths
pub struct ModuleResolver {
    /// Base directory
    base_dir: PathBuf,

    /// Search paths
    search_paths: Vec<PathBuf>,
}

impl ModuleResolver {
    /// Create new resolver
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            search_paths: vec![],
        }
    }

    /// Add search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Resolve module path
    pub fn resolve(&self, path: &str) -> Result<PathBuf, ResolverError> {
        // Relative path
        if path.starts_with("./") || path.starts_with("../") {
            let resolved = self.base_dir.join(path);
            return self.check_path(resolved);
        }

        // Search in search paths
        for search_path in &self.search_paths {
            let resolved = search_path.join(path);
            if let Ok(path) = self.check_path(resolved) {
                return Ok(path);
            }
        }

        Err(ResolverError::NotFound(path.to_string()))
    }

    fn check_path(&self, mut path: PathBuf) -> Result<PathBuf, ResolverError> {
        // Add extension if missing
        if path.extension().is_none() {
            path.set_extension("hudhud");
        }

        if path.exists() {
            Ok(path)
        } else {
            Err(ResolverError::NotFound(path.display().to_string()))
        }
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl ResolverError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ResolverError::InvalidPath(..) => hudhudscript_errors::ErrorCode::ResolverInvalidPath,
            ResolverError::NotFound(..) => hudhudscript_errors::ErrorCode::ResolverNotFound,
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

impl From<ResolverError> for hudhudscript_errors::Error {
    fn from(e: ResolverError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
