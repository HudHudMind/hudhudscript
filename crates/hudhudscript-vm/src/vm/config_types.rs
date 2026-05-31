/// Output locale for formatting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputLocale {
    Default,
    Arabic,
}

/// Sandbox configuration for file and network access control
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub allowed_paths: Vec<String>,
    pub allowed_hosts: Vec<String>,
    pub allow_file_read: bool,
    pub allow_file_write: bool,
    pub allow_network: bool,
}
