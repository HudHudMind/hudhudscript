//! Public API tests for hudhudscript-sandbox

use hudhudscript_sandbox::{
    capability::{effective_capabilities, Capability, CapabilitySet},
    resources::{BandwidthLimit, IoLimit, ResourceLimits},
    vfs::WorkspaceVfs,
    FileSystemConfig, NetworkConfig, ProcessConfig, ResourceConfig, Sandbox, SandboxConfig,
    SandboxError,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

// ── Sandbox ─────────────────────────────────────────────────────────

#[test]
fn test_restrictive_sandbox_file_deny() {
    let s = Sandbox::default_restrictive();
    assert!(s.check_file_access("/etc/passwd", false).is_err());
    assert!(s.check_file_access("/sys/kernel", false).is_err());
}
#[test]
fn test_restrictive_sandbox_network_deny() {
    let s = Sandbox::default_restrictive();
    assert!(s.check_network_access("localhost", 8080).is_err());
    assert!(s.check_network_access("127.0.0.1", 3000).is_err());
    assert!(s.check_network_access("::1", 3000).is_err());
}
#[test]
fn test_restrictive_sandbox_process_deny() {
    let s = Sandbox::default_restrictive();
    assert!(s.check_process_execution("rm").is_err());
    assert!(s.check_process_execution("dd").is_err());
}
#[test]
fn test_restrictive_sandbox_write_deny() {
    assert!(Sandbox::default_restrictive()
        .check_file_access("/sys/test", true)
        .is_err());
}
#[test]
fn test_permissive_sandbox_file() {
    assert!(Sandbox::default_permissive()
        .check_file_access("/tmp/test.txt", true)
        .is_ok());
}
#[test]
fn test_permissive_sandbox_network() {
    assert!(Sandbox::default_permissive()
        .check_network_access("api.example.com", 443)
        .is_ok());
}
#[test]
fn test_permissive_sandbox_process() {
    assert!(Sandbox::default_permissive()
        .check_process_execution("python")
        .is_ok());
}
#[test]
fn test_sandbox_resource_usage() {
    let u = Sandbox::default_permissive().get_resource_usage();
    assert_eq!(u.memory_used, 0);
    assert_eq!(u.cpu_percent, 0.0);
}
#[test]
fn test_sandbox_resource_within() {
    assert!(Sandbox::default_permissive()
        .check_resource_usage(1000, 10.0)
        .is_ok());
}
#[test]
fn test_sandbox_memory_exceeded() {
    assert!(Sandbox::default_restrictive()
        .check_resource_usage(200 * 1024 * 1024, 10.0)
        .is_err());
}
#[test]
fn test_sandbox_cpu_exceeded() {
    assert!(Sandbox::default_restrictive()
        .check_resource_usage(1000, 95.0)
        .is_err());
}

// ── SandboxError ────────────────────────────────────────────────────

#[test]
fn test_error_fs() {
    assert!(format!("{}", SandboxError::FileSystemDenied("t".into()))
        .contains("File system access denied: t"));
}
#[test]
fn test_error_net() {
    assert!(
        format!("{}", SandboxError::NetworkDenied("t".into())).contains("Network access denied: t")
    );
}
#[test]
fn test_error_proc() {
    assert!(format!("{}", SandboxError::ProcessDenied("t".into()))
        .contains("Process execution denied: t"));
}
#[test]
fn test_error_resource() {
    assert!(
        format!("{}", SandboxError::ResourceLimitExceeded("t".into()))
            .contains("Resource limit exceeded: t")
    );
}
#[test]
fn test_error_config() {
    assert!(
        format!("{}", SandboxError::InvalidConfig("t".into())).contains("Invalid configuration: t")
    );
}
#[test]
fn test_error_is_std() {
    let e: Box<dyn std::error::Error> = Box::new(SandboxError::FileSystemDenied("t".into()));
    assert!(e.to_string().contains("File system"));
}

// ── SandboxConfig ───────────────────────────────────────────────────

#[test]
fn test_restrictive_config() {
    let c = SandboxConfig::default_restrictive();
    assert!(c.filesystem.deny_all.contains(&"/etc".into()));
    assert!(c.network.deny_hosts.contains(&"localhost".into()));
}
#[test]
fn test_permissive_config() {
    let c = SandboxConfig::default_permissive();
    assert!(c.filesystem.allow_read.contains(&"/*".into()));
    assert!(c.network.allow_domains.contains(&"*".into()));
}
#[test]
fn test_restrictive_fs_config() {
    let f = FileSystemConfig::default_restrictive();
    assert!(f.allow_read.contains(&"/tmp".into()));
    assert!(f.deny_all.contains(&"/proc".into()));
}
#[test]
fn test_permissive_fs_config() {
    let f = FileSystemConfig::default_permissive();
    assert!(f.deny_all.contains(&"/etc/shadow".into()));
}
#[test]
fn test_restrictive_net_config() {
    let n = NetworkConfig::default_restrictive();
    assert!(n.deny_hosts.contains(&"0.0.0.0".into()));
    assert!(n.allow_ports.contains(&443));
}
#[test]
fn test_permissive_net_config() {
    let n = NetworkConfig::default_permissive();
    assert!(n.deny_hosts.is_empty());
}
#[test]
fn test_restrictive_proc_config() {
    let p = ProcessConfig::default_restrictive();
    assert!(p.deny_commands.contains(&"shutdown".into()));
    assert_eq!(p.max_processes, 10);
}
#[test]
fn test_permissive_proc_config() {
    assert!(ProcessConfig::default_permissive()
        .allow_commands
        .contains(&"*".into()));
}
#[test]
fn test_resource_config_restrictive() {
    let r = ResourceConfig::default_restrictive();
    assert_eq!(r.max_memory_bytes, 100 * 1024 * 1024);
    assert_eq!(r.max_cpu_percent, 80.0);
}
#[test]
fn test_resource_config_permissive() {
    let r = ResourceConfig::default_permissive();
    assert_eq!(r.max_memory_bytes, 1024 * 1024 * 1024);
}

// ── Capability ──────────────────────────────────────────────────────

#[test]
fn test_capability_set_empty() {
    let s = CapabilitySet::new();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
}
#[test]
fn test_capability_add_remove() {
    let mut s = CapabilitySet::new();
    s.add(Capability::NetAdmin);
    assert!(s.contains(Capability::NetAdmin));
    s.remove(Capability::NetAdmin);
    assert!(s.is_empty());
}
#[test]
fn test_capability_full() {
    let s = CapabilitySet::full();
    assert_eq!(s.len(), Capability::all().len());
    assert!(s.contains(Capability::SysAdmin));
}
#[test]
fn test_capability_drop_all() {
    let mut s = CapabilitySet::full();
    s.drop_all();
    assert!(s.is_empty());
}
#[test]
fn test_capability_retain_only() {
    let mut s = CapabilitySet::full();
    let k: HashSet<_> = [Capability::Chown, Capability::NetBindService]
        .into_iter()
        .collect();
    s.retain_only(&k);
    assert_eq!(s.len(), 2);
}
#[test]
fn test_capability_names() {
    let mut s = CapabilitySet::new();
    s.add(Capability::NetAdmin);
    assert!(s.names().contains(&"CAP_NET_ADMIN"));
}
#[test]
fn test_capability_name() {
    assert_eq!(Capability::SysAdmin.name(), "CAP_SYS_ADMIN");
}
#[test]
fn test_capability_all() {
    assert!(Capability::all().len() >= 20);
}
#[test]
fn test_effective_caps() {
    // v0.4.47.9: effective_capabilities returns Result; Linux ok, others err.
    #[cfg(target_os = "linux")]
    assert!(effective_capabilities().is_ok());
    #[cfg(not(target_os = "linux"))]
    assert!(effective_capabilities().is_err());
}
#[test]
    #[ignore = "process-global privileged syscall; unsafe in parallel test. Run: --ignored --test-threads=1"]
fn test_capability_apply() {
    assert!(CapabilitySet::new().apply().is_ok());
}
#[test]
fn test_capability_sorted() {
    let mut s = CapabilitySet::new();
    s.add(Capability::SysAdmin);
    s.add(Capability::Chown);
    let names: Vec<_> = s.to_sorted_vec().iter().map(|c| c.name()).collect();
    assert_eq!(names, vec!["CAP_CHOWN", "CAP_SYS_ADMIN"]);
}

// ── ResourceLimits / BandwidthLimit / IoLimit ───────────────────────

#[test]
fn test_bandwidth_unlimited() {
    let b = BandwidthLimit::unlimited();
    assert!(b.is_unlimited());
    assert!(b.check(1_000_000, Duration::from_secs(1)));
}
#[test]
fn test_bandwidth_within() {
    assert!(BandwidthLimit::new(1_000_000).check(500_000, Duration::from_secs(1)));
}
#[test]
fn test_bandwidth_exceeded() {
    assert!(!BandwidthLimit::new(1_000_000).check(2_000_000, Duration::from_secs(1)));
}
#[test]
fn test_bandwidth_zero_elapsed() {
    let b = BandwidthLimit::new(1_000_000);
    assert!(b.check(0, Duration::ZERO));
    assert!(!b.check(100, Duration::ZERO));
}
#[test]
fn test_io_limit_unlimited() {
    assert!(IoLimit::unlimited().is_unlimited());
}
#[test]
fn test_io_limit_new() {
    let io = IoLimit::new(10_000, 5_000);
    assert!(!io.is_unlimited());
    assert_eq!(io.read_bps, 10_000);
}
#[test]
fn test_resource_limits_disk_within() {
    let mut l = ResourceLimits::new(ResourceConfig::default_restrictive());
    l.set_max_disk_bytes(1_000_000);
    assert!(l.check_disk_usage(500_000).is_ok());
}
#[test]
fn test_resource_limits_disk_exceeded() {
    let mut l = ResourceLimits::new(ResourceConfig::default_restrictive());
    l.set_max_disk_bytes(1_000_000);
    assert!(l.check_disk_usage(2_000_000).is_err());
}
#[test]
fn test_resource_limits_disk_unlimited() {
    let l = ResourceLimits::new(ResourceConfig::default_restrictive());
    assert_eq!(l.max_disk_bytes(), 0);
    assert!(l.check_disk_usage(999_999).is_ok());
}
#[test]
fn test_resource_limits_bandwidth_ok() {
    let mut l = ResourceLimits::new(ResourceConfig::default_restrictive());
    l.set_bandwidth_limit(BandwidthLimit::new(1_000_000));
    assert!(l.check_bandwidth(500_000, Duration::from_secs(1)).is_ok());
}
#[test]
fn test_resource_limits_bandwidth_exceeded() {
    let mut l = ResourceLimits::new(ResourceConfig::default_restrictive());
    l.set_bandwidth_limit(BandwidthLimit::new(1_000_000));
    assert!(l
        .check_bandwidth(5_000_000, Duration::from_secs(1))
        .is_err());
}
#[test]
fn test_resource_limits_io_set_get() {
    let mut l = ResourceLimits::new(ResourceConfig::default_restrictive());
    l.set_io_limit(IoLimit::new(500, 250));
    assert_eq!(l.io_limit().read_bps, 500);
}
#[test]
fn test_resource_limits_reset_timer() {
    let mut l = ResourceLimits::new(ResourceConfig::default_restrictive());
    l.reset_timer();
    assert!(l.check_usage(0, 0.0).is_ok());
}

// ── WorkspaceVfs ────────────────────────────────────────────────────

#[test]
fn test_vfs_resolve_simple() {
    let v = WorkspaceVfs::new("/workspace").unwrap();
    assert_eq!(
        v.resolve("data/file.txt").unwrap(),
        PathBuf::from("/workspace/data/file.txt")
    );
}
#[test]
fn test_vfs_resolve_absolute_virtual() {
    assert_eq!(
        WorkspaceVfs::new("/workspace")
            .unwrap()
            .resolve("/data/file.txt")
            .unwrap(),
        PathBuf::from("/workspace/data/file.txt")
    );
}
#[test]
fn test_vfs_escape_rejected() {
    assert!(WorkspaceVfs::new("/workspace")
        .unwrap()
        .resolve("../../../etc/passwd")
        .is_err());
}
#[test]
fn test_vfs_complex_traversal_rejected() {
    assert!(WorkspaceVfs::new("/workspace")
        .unwrap()
        .resolve("data/../../etc/shadow")
        .is_err());
}
#[test]
fn test_vfs_dot_normalised() {
    assert_eq!(
        WorkspaceVfs::new("/workspace")
            .unwrap()
            .resolve("data/./subdir/../file.txt")
            .unwrap(),
        PathBuf::from("/workspace/data/file.txt")
    );
}
#[test]
fn test_vfs_check_read() {
    assert!(WorkspaceVfs::new("/workspace")
        .unwrap()
        .check_read("data/file.txt")
        .is_ok());
}
#[test]
fn test_vfs_check_read_escape() {
    assert!(WorkspaceVfs::new("/workspace")
        .unwrap()
        .check_read("../../etc/passwd")
        .is_err());
}
#[test]
fn test_vfs_check_write() {
    assert!(WorkspaceVfs::new("/workspace")
        .unwrap()
        .check_write("output/r.json")
        .is_ok());
}
#[test]
fn test_vfs_read_only_rejects_write() {
    assert!(WorkspaceVfs::read_only("/workspace")
        .unwrap()
        .check_write("output/r.json")
        .is_err());
}
#[test]
fn test_vfs_read_only_allows_read() {
    assert!(WorkspaceVfs::read_only("/workspace")
        .unwrap()
        .check_read("data/file.txt")
        .is_ok());
}
#[test]
fn test_vfs_relative_root_rejected() {
    assert!(WorkspaceVfs::new("relative/path").is_err());
}
#[test]
fn test_vfs_to_virtual() {
    assert_eq!(
        WorkspaceVfs::new("/workspace")
            .unwrap()
            .to_virtual("/workspace/data/file.txt"),
        Some(PathBuf::from("data/file.txt"))
    );
}
#[test]
fn test_vfs_to_virtual_outside() {
    assert!(WorkspaceVfs::new("/workspace")
        .unwrap()
        .to_virtual("/etc/passwd")
        .is_none());
}
#[test]
fn test_vfs_path_components() {
    assert_eq!(
        WorkspaceVfs::new("/workspace")
            .unwrap()
            .path_components("a/b/c.txt")
            .unwrap(),
        vec!["a", "b", "c.txt"]
    );
}
#[test]
fn test_vfs_is_read_only() {
    assert!(WorkspaceVfs::read_only("/workspace")
        .unwrap()
        .is_read_only());
    assert!(!WorkspaceVfs::new("/workspace").unwrap().is_read_only());
}
#[test]
fn test_vfs_workspace_root() {
    assert_eq!(
        WorkspaceVfs::new("/workspace").unwrap().workspace_root(),
        Path::new("/workspace")
    );
}

// ── Additional tests from lib.rs inline block ────────────────────────

#[test]
fn test_restrictive_sandbox() {
    let sandbox = hudhudscript_sandbox::Sandbox::default_restrictive();

    // Should deny access to system directories
    assert!(sandbox.check_file_access("/etc/passwd", false).is_err());
    assert!(sandbox.check_file_access("/sys/kernel", false).is_err());

    // Should deny localhost access (including IPv6 loopback)
    assert!(sandbox.check_network_access("localhost", 8080).is_err());
    assert!(sandbox.check_network_access("127.0.0.1", 3000).is_err());
    assert!(sandbox.check_network_access("::1", 3000).is_err());

    // Should deny dangerous commands
    assert!(sandbox.check_process_execution("rm").is_err());
    assert!(sandbox.check_process_execution("dd").is_err());
}

#[test]
fn test_sandbox_error_display() {
    let e = hudhudscript_sandbox::SandboxError::FileSystemDenied("test".to_string());
    assert!(format!("{}", e).contains("File system access denied: test"));

    let e = hudhudscript_sandbox::SandboxError::NetworkDenied("test".to_string());
    assert!(format!("{}", e).contains("Network access denied: test"));

    let e = hudhudscript_sandbox::SandboxError::ProcessDenied("test".to_string());
    assert!(format!("{}", e).contains("Process execution denied: test"));

    let e = hudhudscript_sandbox::SandboxError::ResourceLimitExceeded("test".to_string());
    assert!(format!("{}", e).contains("Resource limit exceeded: test"));

    let e = hudhudscript_sandbox::SandboxError::InvalidConfig("test".to_string());
    assert!(format!("{}", e).contains("Invalid configuration: test"));
}

#[test]
fn test_sandbox_resource_check_within_limits() {
    let sandbox = hudhudscript_sandbox::Sandbox::default_permissive();
    assert!(sandbox.check_resource_usage(1000, 10.0).is_ok());
}

#[test]
fn test_sandbox_resource_check_memory_exceeded() {
    let sandbox = hudhudscript_sandbox::Sandbox::default_restrictive();
    let result = sandbox.check_resource_usage(200 * 1024 * 1024, 10.0);
    assert!(result.is_err());
}

#[test]
fn test_restrictive_sandbox_write_denied() {
    let sandbox = hudhudscript_sandbox::Sandbox::default_restrictive();
    // Writing to /sys should be denied (in deny_all)
    assert!(sandbox.check_file_access("/sys/test", true).is_err());
}

#[test]
fn test_permissive_sandbox() {
    let sandbox = hudhudscript_sandbox::Sandbox::default_permissive();

    // Should allow most access in permissive mode
    assert!(sandbox.check_file_access("/tmp/test.txt", true).is_ok());
    assert!(sandbox.check_network_access("api.example.com", 443).is_ok());
    assert!(sandbox.check_process_execution("python").is_ok());
}

#[test]
fn test_sandbox_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(
        hudhudscript_sandbox::SandboxError::FileSystemDenied("test".to_string()),
    );
    assert!(err.to_string().contains("File system access denied"));
}

#[test]
fn test_resource_usage_debug() {
    let usage = hudhudscript_sandbox::ResourceUsage {
        memory_used: 1024,
        memory_limit: 2048,
        cpu_percent: 50.0,
        cpu_limit: 100.0,
    };
    let debug_str = format!("{:?}", usage);
    assert!(debug_str.contains("1024"));
    assert!(debug_str.contains("2048"));
}

#[test]
fn test_sandbox_resource_check_cpu_exceeded() {
    let sandbox = hudhudscript_sandbox::Sandbox::default_restrictive();
    let result = sandbox.check_resource_usage(1000, 95.0);
    assert!(result.is_err());
}
