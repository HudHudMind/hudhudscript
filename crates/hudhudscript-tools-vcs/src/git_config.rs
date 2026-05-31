//! Git configuration reader/writer (Issue #621)
//!
//! Wraps `git config` to get/set configuration values.

use crate::git::GitError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git configuration accessor scoped to a repository (or global).
pub struct GitConfig {
    /// Repository root; `None` means use `--global`.
    repo_path: Option<PathBuf>,
}

impl GitConfig {
    /// Create a config accessor for a specific repository.
    pub fn for_repo(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: Some(repo_path.into()),
        }
    }

    /// Create a config accessor for global (`~/.gitconfig`) settings.
    pub fn global() -> Self {
        Self { repo_path: None }
    }

    /// Return the repository path, if any.
    pub fn repo_path(&self) -> Option<&Path> {
        self.repo_path.as_deref()
    }

    // -- helpers ----------------------------------------------------------

    fn run_git(&self, args: &[&str]) -> Result<String, GitError> {
        let mut cmd = Command::new("git");
        if let Some(ref path) = self.repo_path {
            cmd.current_dir(path);
        }
        cmd.args(args);

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                GitError::GitNotFound
            } else {
                GitError::SpawnFailed(e.to_string())
            }
        })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(GitError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            })
        }
    }

    // -- public API -------------------------------------------------------

    /// Get a configuration value by key (e.g. `"user.name"`).
    ///
    /// Returns `Ok(None)` when the key is not set.
    pub fn get(&self, key: &str) -> Result<Option<String>, GitError> {
        match self.run_git(&["config", "--get", key]) {
            Ok(val) if val.is_empty() => Ok(None),
            Ok(val) => Ok(Some(val)),
            Err(GitError::CommandFailed { code: 1, .. }) => Ok(None), // key not found
            Err(e) => Err(e),
        }
    }

    /// Set a configuration value.
    pub fn set(&self, key: &str, value: &str) -> Result<(), GitError> {
        if self.repo_path.is_some() {
            self.run_git(&["config", key, value])?;
        } else {
            self.run_git(&["config", "--global", key, value])?;
        }
        Ok(())
    }

    /// Convenience: get `user.name`.
    pub fn user_name(&self) -> Result<Option<String>, GitError> {
        self.get("user.name")
    }

    /// Convenience: get `user.email`.
    pub fn user_email(&self) -> Result<Option<String>, GitError> {
        self.get("user.email")
    }
}
