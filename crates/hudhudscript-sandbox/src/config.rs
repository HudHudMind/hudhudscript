//! Sandbox configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub filesystem: FileSystemConfig,
    pub network: NetworkConfig,
    pub process: ProcessConfig,
    pub resources: ResourceConfig,
}

impl SandboxConfig {
    /// Create a restrictive (secure) configuration
    pub fn default_restrictive() -> Self {
        Self {
            filesystem: FileSystemConfig::default_restrictive(),
            network: NetworkConfig::default_restrictive(),
            process: ProcessConfig::default_restrictive(),
            resources: ResourceConfig::default_restrictive(),
        }
    }

    /// Create a permissive (development) configuration
    pub fn default_permissive() -> Self {
        Self {
            filesystem: FileSystemConfig::default_permissive(),
            network: NetworkConfig::default_permissive(),
            process: ProcessConfig::default_permissive(),
            resources: ResourceConfig::default_permissive(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemConfig {
    pub allow_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub deny_all: Vec<String>,
}

impl FileSystemConfig {
    pub fn default_restrictive() -> Self {
        Self {
            allow_read: vec!["/tmp".to_string(), "/data/public".to_string()],
            allow_write: vec!["/tmp".to_string()],
            deny_all: vec![
                "/etc".to_string(),
                "/sys".to_string(),
                "/proc".to_string(),
                "/root".to_string(),
                "/boot".to_string(),
            ],
        }
    }

    pub fn default_permissive() -> Self {
        Self {
            allow_read: vec!["/*".to_string()],
            allow_write: vec!["/tmp".to_string(), "/data".to_string()],
            deny_all: vec!["/etc/shadow".to_string(), "/etc/passwd".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub allow_domains: Vec<String>,
    pub allow_ports: Vec<u16>,
    pub deny_hosts: Vec<String>,
}

impl NetworkConfig {
    pub fn default_restrictive() -> Self {
        Self {
            allow_domains: vec!["api.example.com".to_string(), "*.trusted.com".to_string()],
            allow_ports: vec![80, 443],
            deny_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "::1".to_string(),
                "0.0.0.0".to_string(),
                "192.168.*".to_string(),
                "10.*".to_string(),
            ],
        }
    }

    pub fn default_permissive() -> Self {
        Self {
            allow_domains: vec!["*".to_string()],
            allow_ports: vec![], // Empty means all ports
            deny_hosts: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub allow_commands: Vec<String>,
    pub deny_commands: Vec<String>,
    pub max_processes: usize,
}

impl ProcessConfig {
    pub fn default_restrictive() -> Self {
        Self {
            allow_commands: vec!["python".to_string(), "node".to_string()],
            deny_commands: vec![
                "rm".to_string(),
                "dd".to_string(),
                "mkfs".to_string(),
                "fdisk".to_string(),
                "shutdown".to_string(),
                "reboot".to_string(),
            ],
            max_processes: 10,
        }
    }

    pub fn default_permissive() -> Self {
        Self {
            allow_commands: vec!["*".to_string()],
            deny_commands: vec!["rm".to_string(), "dd".to_string()],
            max_processes: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub max_memory_bytes: usize,
    pub max_cpu_percent: f64,
    pub max_execution_time_ms: u64,
}

impl ResourceConfig {
    pub fn default_restrictive() -> Self {
        Self {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_percent: 80.0,
            max_execution_time_ms: 30_000, // 30 seconds
        }
    }

    pub fn default_permissive() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 1024, // 1 GB
            max_cpu_percent: 100.0,
            max_execution_time_ms: 300_000, // 5 minutes
        }
    }
}
