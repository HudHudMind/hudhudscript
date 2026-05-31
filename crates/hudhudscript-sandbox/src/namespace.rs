//! Linux namespace isolation (Issue #603)
//!
//! Provides configuration types for Linux namespace-based isolation (mount,
//! PID, network, user, IPC). Actual `clone` / `unshare` calls are gated
//! behind `#[cfg(target_os = "linux")]` so the module compiles on all
//! platforms.

use crate::Result;

// ---------------------------------------------------------------------------
// IsolationLevel
// ---------------------------------------------------------------------------

/// Pre-defined isolation levels combining sets of namespaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsolationLevel {
    /// No namespace isolation at all.
    None,
    /// Partial isolation: mount and PID namespaces only.
    Partial,
    /// Full isolation: mount, PID, network, user, and IPC namespaces.
    Full,
}

// ---------------------------------------------------------------------------
// NamespaceConfig
// ---------------------------------------------------------------------------

/// Describes which Linux namespaces should be unshared for the sandbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceConfig {
    /// Isolate the mount namespace.
    pub mount_ns: bool,
    /// Isolate the PID namespace.
    pub pid_ns: bool,
    /// Isolate the network namespace.
    pub net_ns: bool,
    /// Isolate the user namespace.
    pub user_ns: bool,
    /// Isolate the IPC namespace.
    pub ipc_ns: bool,
}

impl NamespaceConfig {
    /// All namespaces disabled.
    pub fn none() -> Self {
        Self {
            mount_ns: false,
            pid_ns: false,
            net_ns: false,
            user_ns: false,
            ipc_ns: false,
        }
    }

    /// Partial isolation (mount + PID).
    pub fn partial() -> Self {
        Self {
            mount_ns: true,
            pid_ns: true,
            net_ns: false,
            user_ns: false,
            ipc_ns: false,
        }
    }

    /// Full isolation (all namespaces).
    pub fn full() -> Self {
        Self {
            mount_ns: true,
            pid_ns: true,
            net_ns: true,
            user_ns: true,
            ipc_ns: true,
        }
    }

    /// Create a config from a preset isolation level.
    pub fn from_level(level: IsolationLevel) -> Self {
        match level {
            IsolationLevel::None => Self::none(),
            IsolationLevel::Partial => Self::partial(),
            IsolationLevel::Full => Self::full(),
        }
    }

    /// Return the matching `IsolationLevel` for this config, if it matches a
    /// preset exactly.
    pub fn isolation_level(&self) -> IsolationLevel {
        if *self == Self::full() {
            IsolationLevel::Full
        } else if *self == Self::partial() {
            IsolationLevel::Partial
        } else if *self == Self::none() {
            IsolationLevel::None
        } else {
            // Custom combination — treat as Partial for safety classification.
            IsolationLevel::Partial
        }
    }

    /// Return the set of namespace flags as a human-readable list.
    pub fn enabled_namespaces(&self) -> Vec<&'static str> {
        let mut ns = Vec::new();
        if self.mount_ns {
            ns.push("mount");
        }
        if self.pid_ns {
            ns.push("pid");
        }
        if self.net_ns {
            ns.push("net");
        }
        if self.user_ns {
            ns.push("user");
        }
        if self.ipc_ns {
            ns.push("ipc");
        }
        ns
    }

    /// Apply the namespace configuration by calling `unshare(2)` on Linux.
    ///
    /// On non-Linux platforms this is a no-op.
    /// Note: requires `CAP_SYS_ADMIN` or user namespace support.
    pub fn apply(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::io;

            let mut flags: libc::c_int = 0;

            // User namespace must be first (grants capabilities for other namespaces)
            if self.user_ns {
                flags |= libc::CLONE_NEWUSER;
            }
            if self.mount_ns {
                flags |= libc::CLONE_NEWNS;
            }
            if self.pid_ns {
                flags |= libc::CLONE_NEWPID;
            }
            if self.net_ns {
                flags |= libc::CLONE_NEWNET;
            }
            if self.ipc_ns {
                flags |= libc::CLONE_NEWIPC;
            }

            if flags != 0 {
                let ret = unsafe { libc::unshare(flags) };
                if ret != 0 {
                    let err = io::Error::last_os_error();
                    return Err(crate::SandboxError::SystemCallFailed(format!(
                        "unshare({:#x}) failed: {} — may require CAP_SYS_ADMIN or unprivileged user namespaces",
                        flags,
                        err
                    )));
                }
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(crate::SandboxError::SystemCallFailed(
                "Linux namespaces not supported on this platform".to_string(),
            ))
        }
    }
}

impl Default for NamespaceConfig {
    fn default() -> Self {
        Self::none()
    }
}

// ---------------------------------------------------------------------------
// NamespaceBuilder
// ---------------------------------------------------------------------------

/// Fluent builder for `NamespaceConfig`.
#[derive(Debug, Clone, Default)]
pub struct NamespaceBuilder {
    config: NamespaceConfig,
}

impl NamespaceBuilder {
    /// Start with no namespaces enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start from a preset isolation level.
    pub fn from_level(level: IsolationLevel) -> Self {
        Self {
            config: NamespaceConfig::from_level(level),
        }
    }

    /// Enable mount namespace isolation.
    pub fn mount_ns(mut self) -> Self {
        self.config.mount_ns = true;
        self
    }

    /// Enable PID namespace isolation.
    pub fn pid_ns(mut self) -> Self {
        self.config.pid_ns = true;
        self
    }

    /// Enable network namespace isolation.
    pub fn net_ns(mut self) -> Self {
        self.config.net_ns = true;
        self
    }

    /// Enable user namespace isolation.
    pub fn user_ns(mut self) -> Self {
        self.config.user_ns = true;
        self
    }

    /// Enable IPC namespace isolation.
    pub fn ipc_ns(mut self) -> Self {
        self.config.ipc_ns = true;
        self
    }

    /// Consume the builder and return the finalised configuration.
    pub fn build(self) -> NamespaceConfig {
        self.config
    }
}
