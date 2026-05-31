//! Tests extracted from hudhudscript-sandbox/src/config.rs

use hudhudscript_sandbox::{
    FileSystemConfig, NetworkConfig, ProcessConfig, ResourceConfig, SandboxConfig,
};

#[test]
fn test_restrictive_config() {
    let config = SandboxConfig::default_restrictive();
    assert!(config.filesystem.deny_all.contains(&"/etc".to_string()));
    assert!(config.network.deny_hosts.contains(&"localhost".to_string()));
    assert!(config.process.deny_commands.contains(&"rm".to_string()));
    assert_eq!(config.resources.max_memory_bytes, 100 * 1024 * 1024);
}

#[test]
fn test_restrictive_filesystem_config() {
    let fs = FileSystemConfig::default_restrictive();
    assert!(fs.allow_read.contains(&"/tmp".to_string()));
    assert!(fs.allow_write.contains(&"/tmp".to_string()));
    assert!(fs.deny_all.contains(&"/proc".to_string()));
    assert!(fs.deny_all.contains(&"/root".to_string()));
    assert!(fs.deny_all.contains(&"/boot".to_string()));
}

#[test]
fn test_permissive_filesystem_config() {
    let fs = FileSystemConfig::default_permissive();
    assert!(fs.allow_read.contains(&"/*".to_string()));
    assert!(fs.deny_all.contains(&"/etc/shadow".to_string()));
}

#[test]
fn test_restrictive_network_config() {
    let net = NetworkConfig::default_restrictive();
    assert!(net.deny_hosts.contains(&"0.0.0.0".to_string()));
    assert!(net.deny_hosts.contains(&"192.168.*".to_string()));
    assert!(net.deny_hosts.contains(&"10.*".to_string()));
    assert!(net.allow_ports.contains(&80));
    assert!(net.allow_ports.contains(&443));
}

#[test]
fn test_permissive_network_config() {
    let net = NetworkConfig::default_permissive();
    assert!(net.allow_ports.is_empty());
    assert!(net.deny_hosts.is_empty());
}

#[test]
fn test_restrictive_process_config() {
    let proc = ProcessConfig::default_restrictive();
    assert!(proc.deny_commands.contains(&"mkfs".to_string()));
    assert!(proc.deny_commands.contains(&"shutdown".to_string()));
    assert!(proc.deny_commands.contains(&"reboot".to_string()));
    assert_eq!(proc.max_processes, 10);
}

#[test]
fn test_permissive_process_config() {
    let proc = ProcessConfig::default_permissive();
    assert!(proc.allow_commands.contains(&"*".to_string()));
    assert!(proc.deny_commands.contains(&"rm".to_string()));
}

#[test]
fn test_resource_config_restrictive() {
    let res = ResourceConfig::default_restrictive();
    assert_eq!(res.max_memory_bytes, 100 * 1024 * 1024);
    assert_eq!(res.max_cpu_percent, 80.0);
    assert_eq!(res.max_execution_time_ms, 30_000);
}

#[test]
fn test_resource_config_permissive() {
    let res = ResourceConfig::default_permissive();
    assert_eq!(res.max_memory_bytes, 1024 * 1024 * 1024);
    assert_eq!(res.max_cpu_percent, 100.0);
    assert_eq!(res.max_execution_time_ms, 300_000);
}

#[test]
fn test_permissive_config() {
    let config = SandboxConfig::default_permissive();
    assert!(config.filesystem.allow_read.contains(&"/*".to_string()));
    assert!(config.network.allow_domains.contains(&"*".to_string()));
    assert_eq!(config.process.max_processes, 100);
}
