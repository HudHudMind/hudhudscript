//! Landlock LSM integration (Issue #603)
//!
//! Provides a safe abstraction over the Linux Landlock security module for
//! restricting filesystem access at the kernel level. Actual landlock syscalls
//! are gated behind `#[cfg(target_os = "linux")]` so the module compiles and
//! tests pass on all platforms.

use crate::Result;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// LandlockRuleset
// ---------------------------------------------------------------------------

/// A set of Landlock filesystem access rules.
///
/// Paths can be added for read, write, or execute access.  When [`apply`] is
/// called the ruleset is submitted to the kernel and enforced for the current
/// thread (and any future children).
#[derive(Debug, Clone, Default)]
pub struct LandlockRuleset {
    /// Paths allowed for read access.
    allowed_read_paths: Vec<PathBuf>,
    /// Paths allowed for write access.
    allowed_write_paths: Vec<PathBuf>,
    /// Paths allowed for execute access.
    allowed_exec_paths: Vec<PathBuf>,
}

impl LandlockRuleset {
    /// Create an empty ruleset (nothing is allowed).
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow read access under `path` (and its children).
    pub fn add_read_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.allowed_read_paths.push(path.into());
        self
    }

    /// Allow write access under `path` (and its children).
    pub fn add_write_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.allowed_write_paths.push(path.into());
        self
    }

    /// Allow execute access under `path` (and its children).
    pub fn add_exec_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.allowed_exec_paths.push(path.into());
        self
    }

    /// Return the read-allowed paths.
    pub fn read_paths(&self) -> &[PathBuf] {
        &self.allowed_read_paths
    }

    /// Return the write-allowed paths.
    pub fn write_paths(&self) -> &[PathBuf] {
        &self.allowed_write_paths
    }

    /// Return the execute-allowed paths.
    pub fn exec_paths(&self) -> &[PathBuf] {
        &self.allowed_exec_paths
    }

    /// Check whether `path` would be permitted for reading under this ruleset.
    pub fn check_read(&self, path: impl AsRef<Path>) -> bool {
        let p = path.as_ref();
        self.allowed_read_paths
            .iter()
            .any(|allowed| p.starts_with(allowed))
    }

    /// Check whether `path` would be permitted for writing under this ruleset.
    pub fn check_write(&self, path: impl AsRef<Path>) -> bool {
        let p = path.as_ref();
        self.allowed_write_paths
            .iter()
            .any(|allowed| p.starts_with(allowed))
    }

    /// Check whether `path` would be permitted for execution under this ruleset.
    pub fn check_exec(&self, path: impl AsRef<Path>) -> bool {
        let p = path.as_ref();
        self.allowed_exec_paths
            .iter()
            .any(|allowed| p.starts_with(allowed))
    }

    /// Install the ruleset into the current thread.
    ///
    /// On Linux with Landlock support: creates a ruleset, adds path rules,
    /// and calls `landlock_restrict_self`. On non-Linux or unsupported
    /// kernels returns an error.
    pub fn apply(&self) -> Result<()> {
        if !Self::is_supported() {
            return Err(crate::SandboxError::SystemCallFailed(
                "Landlock not supported on this kernel".to_string(),
            ));
        }

        #[cfg(target_os = "linux")]
        {
            use std::io;
            use std::os::unix::io::AsRawFd;

            // Landlock ABI v1 constants
            #[allow(dead_code)]
            const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1 << 0;
            const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
            const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 5;
            const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
            const LANDLOCK_RULE_PATH_BENEATH: u32 = 1;

            // All FS access flags for the ruleset
            let handled_access = LANDLOCK_ACCESS_FS_READ_FILE
                | LANDLOCK_ACCESS_FS_WRITE_FILE
                | LANDLOCK_ACCESS_FS_EXECUTE;

            #[repr(C)]
            struct LandlockRulesetAttr {
                handled_access_fs: u64,
            }

            #[repr(C)]
            struct LandlockPathBeneathAttr {
                allowed_access: u64,
                parent_fd: i32,
            }

            let attr = LandlockRulesetAttr {
                handled_access_fs: handled_access,
            };

            // SYS_landlock_create_ruleset = 444 on x86_64
            let ruleset_fd = unsafe {
                libc::syscall(
                    444, // SYS_landlock_create_ruleset
                    &attr as *const _ as *const libc::c_void,
                    std::mem::size_of::<LandlockRulesetAttr>(),
                    0u32,
                )
            };
            if ruleset_fd < 0 {
                return Err(crate::SandboxError::SystemCallFailed(format!(
                    "landlock_create_ruleset failed: {}",
                    io::Error::last_os_error()
                )));
            }

            // Helper to add a path rule
            let add_path_rule = |path: &std::path::Path, access: u64| -> Result<()> {
                if let Ok(file) = std::fs::File::open(path) {
                    let rule = LandlockPathBeneathAttr {
                        allowed_access: access,
                        parent_fd: file.as_raw_fd(),
                    };
                    // SYS_landlock_add_rule = 445
                    let ret = unsafe {
                        libc::syscall(
                            445,
                            ruleset_fd,
                            LANDLOCK_RULE_PATH_BENEATH,
                            &rule as *const _ as *const libc::c_void,
                            0u32,
                        )
                    };
                    if ret < 0 {
                        return Err(crate::SandboxError::SystemCallFailed(format!(
                            "landlock_add_rule failed for {:?}: {}",
                            path,
                            io::Error::last_os_error()
                        )));
                    }
                }
                Ok(())
            };

            for p in &self.allowed_read_paths {
                add_path_rule(p, LANDLOCK_ACCESS_FS_READ_FILE)?;
            }
            for p in &self.allowed_write_paths {
                add_path_rule(
                    p,
                    LANDLOCK_ACCESS_FS_WRITE_FILE | LANDLOCK_ACCESS_FS_READ_FILE,
                )?;
            }
            for p in &self.allowed_exec_paths {
                add_path_rule(p, LANDLOCK_ACCESS_FS_EXECUTE | LANDLOCK_ACCESS_FS_READ_FILE)?;
            }

            // SYS_landlock_restrict_self = 446
            let ret = unsafe { libc::syscall(446, ruleset_fd, 0u32) };
            unsafe { libc::close(ruleset_fd as i32) };
            if ret < 0 {
                return Err(crate::SandboxError::SystemCallFailed(format!(
                    "landlock_restrict_self failed: {}",
                    io::Error::last_os_error()
                )));
            }
        }

        Ok(())
    }

    /// Check whether the running kernel supports Landlock.
    ///
    /// On Linux: probes `landlock_create_ruleset(NULL, 0, VERSION)`.
    /// On non-Linux: always returns false.
    pub fn is_supported() -> bool {
        #[cfg(target_os = "linux")]
        {
            // Probe: SYS_landlock_create_ruleset with LANDLOCK_CREATE_RULESET_VERSION flag
            let ret = unsafe { libc::syscall(444, std::ptr::null::<u8>(), 0usize, 1u32) };
            ret >= 0 // returns ABI version on success
        }

        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Create a minimal ruleset suitable for a read-only sandbox.
    pub fn read_only(paths: &[impl AsRef<Path>]) -> Self {
        let mut rs = Self::new();
        for p in paths {
            rs.add_read_path(p.as_ref());
        }
        rs
    }

    /// Create a ruleset for a workspace: read+write inside `root`, read-only
    /// for shared libraries.
    pub fn workspace(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let mut rs = Self::new();
        rs.add_read_path(&root);
        rs.add_write_path(&root);
        rs.add_read_path("/usr/lib");
        rs.add_read_path("/lib");
        rs.add_exec_path("/usr/bin");
        rs.add_exec_path("/bin");
        rs
    }
}
