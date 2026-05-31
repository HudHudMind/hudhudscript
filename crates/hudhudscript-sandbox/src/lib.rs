//! Sandbox module for HudHudScript
//!
//! Provides security features including:
//! - File system restrictions
//! - Network restrictions
//! - Process restrictions
//! - Resource limits
//! - Seccomp BPF filters (Issue #603)
//! - Landlock LSM integration (Issue #603)
//! - Linux namespace isolation (Issue #603)
//! - Linux capability management (Issue #603)

pub mod capability;
pub mod config;
pub mod filesystem;
pub mod landlock;
pub mod namespace;
pub mod network;
pub mod process;
pub mod resources;
pub mod seccomp;
pub mod tool_dispatch;
pub mod vfs; // Issue #123 – Virtual Filesystem Workspace Isolation

pub use capability::{effective_capabilities, Capability, CapabilitySet};
pub use config::{FileSystemConfig, NetworkConfig, ProcessConfig, ResourceConfig, SandboxConfig};
pub use filesystem::FileSystemSandbox;
pub use landlock::LandlockRuleset;
pub use namespace::{IsolationLevel, NamespaceBuilder, NamespaceConfig};
pub use network::NetworkSandbox;
pub use process::ProcessSandbox;
pub use resources::{BandwidthLimit, IoLimit, ResourceLimits};
pub use seccomp::{SeccompFilter, SeccompPolicy, SeccompProfile};
pub use tool_dispatch::{DispatchResult, ToolDispatcher, ToolHandler, ToolPermissions};
pub use vfs::WorkspaceVfs;

#[derive(Debug)]
pub enum SandboxError {
    FileSystemDenied(String),
    NetworkDenied(String),
    ProcessDenied(String),
    ResourceLimitExceeded(String),
    InvalidConfig(String),
    SystemCallFailed(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            SandboxError::FileSystemDenied(s) => write!(f, "File system access denied: {}", s),
            SandboxError::NetworkDenied(s) => write!(f, "Network access denied: {}", s),
            SandboxError::ProcessDenied(s) => write!(f, "Process execution denied: {}", s),
            SandboxError::ResourceLimitExceeded(s) => write!(f, "Resource limit exceeded: {}", s),
            SandboxError::InvalidConfig(s) => write!(f, "Invalid configuration: {}", s),
            SandboxError::SystemCallFailed(s) => write!(f, "System call failed: {}", s),
        }
    }
}

impl std::error::Error for SandboxError {}

pub type Result<T> = std::result::Result<T, SandboxError>;

impl SandboxError {
    /// Construct a structured filesystem denied error.
    pub fn fs_denied(path: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::FileSystemDenied(format!("{} on path '{}'", operation.into(), path.into()))
    }

    /// Construct a structured network denied error.
    pub fn net_denied(host: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::NetworkDenied(format!("{} to host '{}'", operation.into(), host.into()))
    }

    /// Construct a structured process denied error.
    pub fn proc_denied(command: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ProcessDenied(format!("command '{}': {}", command.into(), reason.into()))
    }

    /// Construct a structured resource limit error.
    pub fn resource_limit(resource: impl Into<String>, limit: impl Into<String>) -> Self {
        Self::ResourceLimitExceeded(format!("{}: {}", resource.into(), limit.into()))
    }

    /// Construct a structured config error.
    pub fn invalid_config(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidConfig(format!("{}: {}", field.into(), reason.into()))
    }

    /// Construct a structured syscall failure error.
    pub fn syscall_failed(syscall: impl Into<String>, errno: impl Into<String>) -> Self {
        Self::SystemCallFailed(format!("{}: {}", syscall.into(), errno.into()))
    }

    /// Stable error code for documentation lookup. (v0.4.47.9 — Issue #847)
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::FileSystemDenied(_) => "E_SANDBOX_FS",
            Self::NetworkDenied(_) => "E_SANDBOX_NET",
            Self::ProcessDenied(_) => "E_SANDBOX_PROC",
            Self::ResourceLimitExceeded(_) => "E_SANDBOX_RESOURCE",
            Self::InvalidConfig(_) => "E_SANDBOX_CONFIG",
            Self::SystemCallFailed(_) => "E_SANDBOX_SYSCALL",
        }
    }
}

/// Main sandbox coordinator
/// Sandbox config field is kept for future use
pub struct Sandbox {
    _config: SandboxConfig,
    filesystem: FileSystemSandbox,
    network: NetworkSandbox,
    process: ProcessSandbox,
    resources: ResourceLimits,
}

impl Sandbox {
    /// Create a new sandbox with the given configuration
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            filesystem: FileSystemSandbox::new(config.filesystem.clone()),
            network: NetworkSandbox::new(config.network.clone()),
            process: ProcessSandbox::new(config.process.clone()),
            resources: ResourceLimits::new(config.resources.clone()),
            _config: config,
        }
    }

    /// Create a sandbox with default (restrictive) configuration
    pub fn default_restrictive() -> Self {
        Self::new(SandboxConfig::default_restrictive())
    }

    /// Create a sandbox with permissive configuration (for development)
    pub fn default_permissive() -> Self {
        Self::new(SandboxConfig::default_permissive())
    }

    /// Check if file system access is allowed
    pub fn check_file_access(&self, path: &str, write: bool) -> Result<()> {
        self.filesystem.check_access(path, write)
    }

    /// Check if network access is allowed
    pub fn check_network_access(&self, host: &str, port: u16) -> Result<()> {
        self.network.check_access(host, port)
    }

    /// Check if process execution is allowed
    pub fn check_process_execution(&self, command: &str) -> Result<()> {
        self.process.check_execution(command)
    }

    /// Check if resource usage is within limits
    pub fn check_resource_usage(&self, memory_bytes: usize, cpu_percent: f64) -> Result<()> {
        self.resources.check_usage(memory_bytes, cpu_percent)
    }

    /// Get current resource usage
    pub fn get_resource_usage(&self) -> ResourceUsage {
        self.resources.get_usage()
    }
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_used: usize,
    pub memory_limit: usize,
    pub cpu_percent: f64,
    pub cpu_limit: f64,
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl SandboxError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            SandboxError::FileSystemDenied(..) => {
                hudhudscript_errors::ErrorCode::SandboxFileSystemDenied
            }
            SandboxError::InvalidConfig(..) => hudhudscript_errors::ErrorCode::SandboxInvalidConfig,
            SandboxError::NetworkDenied(..) => hudhudscript_errors::ErrorCode::SandboxNetworkDenied,
            SandboxError::ProcessDenied(..) => hudhudscript_errors::ErrorCode::SandboxProcessDenied,
            SandboxError::ResourceLimitExceeded(..) => {
                hudhudscript_errors::ErrorCode::SandboxResourceLimitExceeded
            }
            SandboxError::SystemCallFailed(..) => {
                hudhudscript_errors::ErrorCode::SandboxSystemCallFailed
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

impl From<SandboxError> for hudhudscript_errors::Error {
    fn from(e: SandboxError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
