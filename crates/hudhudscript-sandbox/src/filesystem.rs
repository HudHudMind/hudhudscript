//! File system sandbox

use crate::{FileSystemConfig, Result, SandboxError};

#[derive(Debug)]
pub struct FileSystemSandbox {
    config: FileSystemConfig,
}

impl FileSystemSandbox {
    pub fn new(config: FileSystemConfig) -> Self {
        Self { config }
    }

    /// Check if file access is allowed
    pub fn check_access(&self, path: &str, write: bool) -> Result<()> {
        // First check deny list
        for denied in &self.config.deny_all {
            if self.path_matches(path, denied) {
                return Err(SandboxError::FileSystemDenied(format!(
                    "Access to {} is explicitly denied",
                    path
                )));
            }
        }

        // Check appropriate allow list
        let allow_list = if write {
            &self.config.allow_write
        } else {
            &self.config.allow_read
        };

        for allowed in allow_list {
            if self.path_matches(path, allowed) {
                return Ok(());
            }
        }

        Err(SandboxError::FileSystemDenied(format!(
            "Access to {} is not allowed",
            path
        )))
    }

    /// Check if a path matches a pattern using proper path component boundaries.
    /// e.g. pattern "/tmp" matches "/tmp/foo" but NOT "/tmpevil/foo".
    /// Resolves symlinks and canonicalizes the path before matching.
    fn path_matches(&self, path: &str, pattern: &str) -> bool {
        use std::path::Path;

        // First, try to canonicalize the path to resolve symlinks and `..` components.
        let resolved = std::fs::canonicalize(path)
            .ok()
            .map(|p| p.to_string_lossy().to_string());
        let path_to_check = resolved.as_deref().unwrap_or(path);

        if pattern == "/*" {
            return true;
        }

        // Strip trailing wildcard to get the directory prefix pattern
        let base_pattern = pattern.strip_suffix('*').unwrap_or(pattern);
        // Also strip a trailing slash from the base so "/tmp/" and "/tmp" are treated the same
        let base_pattern = base_pattern.strip_suffix('/').unwrap_or(base_pattern);

        let pattern_path = Path::new(base_pattern);
        let check_path = Path::new(path_to_check);

        // Use Path::starts_with which compares full path components.
        // "/tmpevil".starts_with("/tmp") is false because "tmpevil" != "tmp".
        check_path.starts_with(pattern_path)
    }
}
