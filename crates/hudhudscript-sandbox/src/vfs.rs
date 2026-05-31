//! Virtual File System (VFS) Workspace Isolation (Issue #123)
//!
//! Provides a `WorkspaceVfs` that restricts all file-path resolution to a
//! configurable workspace root, preventing path-traversal and escape attacks.
//! All paths exposed to callers are *virtual* (relative to the workspace root).
//!
//! The VFS can optionally delegate real I/O operations through the existing
//! `FileSystemSandbox` for an additional layer of enforcement.

use std::path::{Component, Path, PathBuf};

use crate::{FileSystemConfig, FileSystemSandbox, Result, SandboxError};

// ---------------------------------------------------------------------------
// WorkspaceVfs
// ---------------------------------------------------------------------------

/// A virtual filesystem scoped to a single workspace directory.
///
/// # Path resolution
///
/// *Virtual paths* are relative to `workspace_root`.  The VFS resolves them to
/// absolute paths and verifies they are inside the workspace before any
/// operation is allowed.  Symbolic-link traversal is not followed.
#[derive(Debug)]
pub struct WorkspaceVfs {
    /// Absolute path to the workspace root.
    workspace_root: PathBuf,
    /// Underlying file-system sandbox (enforces allow/deny lists).
    fs_sandbox: FileSystemSandbox,
    /// Whether the VFS is read-only (all write operations are rejected).
    read_only: bool,
}

impl WorkspaceVfs {
    /// Create a new VFS rooted at `workspace_root`.
    ///
    /// The root must be an absolute path; relative roots are an error.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Result<Self> {
        let root = workspace_root.into();
        if !root.is_absolute() {
            return Err(SandboxError::InvalidConfig(format!(
                "WorkspaceVfs root must be absolute, got: {}",
                root.display()
            )));
        }

        let root_str = root.to_string_lossy().into_owned();
        let config = FileSystemConfig {
            allow_read: vec![format!("{}/", root_str), root_str.clone()],
            allow_write: vec![format!("{}/", root_str), root_str.clone()],
            deny_all: vec![],
        };

        Ok(Self {
            workspace_root: root,
            fs_sandbox: FileSystemSandbox::new(config),
            read_only: false,
        })
    }

    /// Create a read-only VFS.  All write operations will be rejected.
    pub fn read_only(workspace_root: impl Into<PathBuf>) -> Result<Self> {
        let mut vfs = Self::new(workspace_root)?;
        vfs.read_only = true;
        Ok(vfs)
    }

    /// Return the absolute workspace root path.
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Return `true` if this VFS is read-only.
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    // -----------------------------------------------------------------------
    // Core resolution
    // -----------------------------------------------------------------------

    /// Resolve a virtual (potentially relative) path to a canonical absolute
    /// path inside the workspace.
    ///
    /// Returns `Err` if the resolved path would escape the workspace root.
    pub fn resolve(&self, virtual_path: impl AsRef<Path>) -> Result<PathBuf> {
        let vp = virtual_path.as_ref();

        // Strip a leading `/` — callers may pass absolute-looking virtual paths.
        let relative = if vp.is_absolute() {
            vp.strip_prefix("/").unwrap_or(vp)
        } else {
            vp
        };

        // Build a lexically normalised absolute path without touching the FS.
        let joined = self.workspace_root.join(relative);
        let normalised = lexical_normalise(&joined);

        // Security check: the normalised path must be inside the workspace.
        if !normalised.starts_with(&self.workspace_root) {
            return Err(SandboxError::FileSystemDenied(format!(
                "Path escape detected: '{}' resolves outside workspace '{}'",
                vp.display(),
                self.workspace_root.display()
            )));
        }

        Ok(normalised)
    }

    // -----------------------------------------------------------------------
    // Permission checks
    // -----------------------------------------------------------------------

    /// Check whether reading `virtual_path` is permitted.
    pub fn check_read(&self, virtual_path: impl AsRef<Path>) -> Result<()> {
        let abs = self.resolve(virtual_path)?;
        self.fs_sandbox.check_access(&abs.to_string_lossy(), false)
    }

    /// Check whether writing to `virtual_path` is permitted.
    pub fn check_write(&self, virtual_path: impl AsRef<Path>) -> Result<()> {
        if self.read_only {
            return Err(SandboxError::FileSystemDenied(
                "VFS is read-only".to_string(),
            ));
        }
        let abs = self.resolve(virtual_path)?;
        self.fs_sandbox.check_access(&abs.to_string_lossy(), true)
    }

    // -----------------------------------------------------------------------
    // Path helpers
    // -----------------------------------------------------------------------

    /// Convert an absolute real path back to its virtual path (relative to root).
    ///
    /// Returns `None` if `real_path` is outside the workspace.
    pub fn to_virtual(&self, real_path: impl AsRef<Path>) -> Option<PathBuf> {
        let real = real_path.as_ref();
        real.strip_prefix(&self.workspace_root)
            .ok()
            .map(|p| p.to_path_buf())
    }

    /// List the virtual path components of `virtual_path`, verifying it is
    /// inside the workspace.
    pub fn path_components(&self, virtual_path: impl AsRef<Path>) -> Result<Vec<String>> {
        let abs = self.resolve(virtual_path)?;
        let rel = abs
            .strip_prefix(&self.workspace_root)
            .map_err(|_| SandboxError::FileSystemDenied("Path outside workspace".to_string()))?;

        Ok(rel
            .components()
            .filter_map(|c| match c {
                Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect())
    }
}

// ---------------------------------------------------------------------------
// Lexical path normalisation (no FS access)
// ---------------------------------------------------------------------------

/// Normalise a path lexically by resolving `.` and `..` components without
/// hitting the file system (and therefore without following symlinks).
fn lexical_normalise(path: &Path) -> PathBuf {
    let mut stack: Vec<std::ffi::OsString> = Vec::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Prefix(_) => {
                stack.clear();
                stack.push(component.as_os_str().to_owned());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if stack.len() > 1 {
                    stack.pop();
                }
            }
            Component::Normal(s) => {
                stack.push(s.to_owned());
            }
        }
    }
    stack.iter().collect()
}
