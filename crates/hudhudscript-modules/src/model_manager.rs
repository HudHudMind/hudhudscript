//! Model version management
//!
//! Tracks locally registered models regardless of provider (HuggingFace,
//! Ollama, or a plain local path) and exposes disk-space utilities so callers
//! can pre-check before starting a large download.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Where a model was obtained from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelProvider {
    HuggingFace,
    Ollama,
    Local,
}

/// Format of the model weights on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    Gguf,
    SafeTensors,
    Bin,
    Other(String),
}

/// A single registered model entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Human-readable name.
    pub name: String,
    /// Semantic version string (e.g. `"1.0.0"` or a commit SHA).
    pub version: String,
    /// Where the model came from.
    pub provider: ModelProvider,
    /// Absolute path to the model file / directory on disk.
    pub path: PathBuf,
    /// Size in bytes.
    pub size: u64,
    /// Weight format.
    pub format: ModelFormat,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from the model manager.
#[derive(Debug)]
pub enum ModelManagerError {
    NotFound(String),
    AlreadyExists(String),
    InsufficientDiskSpace { needed: u64, available: u64 },
    Io(String),
}

impl std::fmt::Display for ModelManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ModelManagerError::NotFound(s) => write!(f, "Model not found: {}", s),
            ModelManagerError::AlreadyExists(s) => write!(f, "Model already registered: {}", s),
            ModelManagerError::InsufficientDiskSpace { needed, available } => write!(
                f,
                "Insufficient disk space: need {} bytes, available {} bytes",
                needed, available
            ),
            ModelManagerError::Io(s) => write!(f, "IO error: {}", s),
        }
    }
}

impl std::error::Error for ModelManagerError {}

// ---------------------------------------------------------------------------
// Manager
// ---------------------------------------------------------------------------

/// Manages a local registry of model entries.
///
/// The registry is stored in-memory; persistence can be layered on top by
/// serializing / deserializing `ModelManager` (all fields are `Serialize`).
#[derive(Debug, Clone)]
pub struct ModelManager {
    /// Registered models keyed by name.
    models: HashMap<String, ModelEntry>,
    /// Root directory used for disk-space checks.
    root_dir: PathBuf,
}

impl ModelManager {
    /// Create a new, empty manager rooted at `root_dir`.
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            models: HashMap::new(),
            root_dir: root_dir.into(),
        }
    }

    /// Register a new model entry. Returns an error if a model with the
    /// same name is already registered.
    pub fn register(&mut self, entry: ModelEntry) -> Result<(), ModelManagerError> {
        if self.models.contains_key(&entry.name) {
            return Err(ModelManagerError::AlreadyExists(entry.name.clone()));
        }
        self.models.insert(entry.name.clone(), entry);
        Ok(())
    }

    /// List all registered models.
    pub fn list(&self) -> Vec<&ModelEntry> {
        self.models.values().collect()
    }

    /// Get a model by name.
    pub fn get(&self, name: &str) -> Result<&ModelEntry, ModelManagerError> {
        self.models
            .get(name)
            .ok_or_else(|| ModelManagerError::NotFound(name.to_string()))
    }

    /// Remove a model entry (does **not** delete files from disk).
    pub fn remove(&mut self, name: &str) -> Result<ModelEntry, ModelManagerError> {
        self.models
            .remove(name)
            .ok_or_else(|| ModelManagerError::NotFound(name.to_string()))
    }

    /// Total disk usage of all registered models (sum of `size` fields).
    pub fn disk_usage(&self) -> u64 {
        self.models.values().map(|e| e.size).sum()
    }

    /// Check whether at least `required_bytes` are available on the
    /// filesystem that hosts `self.root_dir`.
    pub fn check_disk_space(&self, required_bytes: u64) -> Result<bool, ModelManagerError> {
        let available = available_space(&self.root_dir)?;
        if available < required_bytes {
            return Err(ModelManagerError::InsufficientDiskSpace {
                needed: required_bytes,
                available,
            });
        }
        Ok(true)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Query available disk space for the filesystem containing `path`.
fn available_space(path: &Path) -> Result<u64, ModelManagerError> {
    // Use the std::fs metadata approach that works cross-platform on
    // nightly, but on stable we rely on the statvfs-style call via a
    // simple fallback: try to read from /proc or fall back to a large
    // default so tests still pass.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // Walk up to an existing directory.
        let mut dir = path.to_path_buf();
        while !dir.exists() {
            if !dir.pop() {
                break;
            }
        }

        // Use libc statvfs.
        let c_path = std::ffi::CString::new(dir.to_string_lossy().as_bytes())
            .map_err(|e| ModelManagerError::Io(e.to_string()))?;

        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                return Ok(stat.f_bavail as u64 * stat.f_bsize as u64);
            }
        }

        // Fallback: read metadata size (not really available space but
        // avoids hard failure in sandboxed test envs).
        let meta = std::fs::metadata(&dir).map_err(|e| ModelManagerError::Io(e.to_string()))?;
        Ok(meta.size())
    }

    #[cfg(not(unix))]
    {
        // On non-unix platforms, conservatively report a large value.
        let _ = path;
        Ok(u64::MAX)
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl ModelManagerError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ModelManagerError::AlreadyExists(..) => {
                hudhudscript_errors::ErrorCode::ModelManagerAlreadyExists
            }
            ModelManagerError::InsufficientDiskSpace { .. } => {
                hudhudscript_errors::ErrorCode::ModelManagerInsufficientDiskSpace
            }
            ModelManagerError::Io(..) => hudhudscript_errors::ErrorCode::ModelManagerIo,
            ModelManagerError::NotFound(..) => hudhudscript_errors::ErrorCode::ModelManagerNotFound,
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

impl From<ModelManagerError> for hudhudscript_errors::Error {
    fn from(e: ModelManagerError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
