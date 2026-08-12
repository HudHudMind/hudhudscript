//! Tests extracted from hudhudscript-sandbox/src/namespace.rs

use hudhudscript_sandbox::namespace::{IsolationLevel, NamespaceBuilder, NamespaceConfig};

#[test]
fn test_none_config() {
    let cfg = NamespaceConfig::none();
    assert!(!cfg.mount_ns);
    assert!(!cfg.pid_ns);
    assert!(!cfg.net_ns);
    assert!(!cfg.user_ns);
    assert!(!cfg.ipc_ns);
    assert_eq!(cfg.isolation_level(), IsolationLevel::None);
    assert!(cfg.enabled_namespaces().is_empty());
}

#[test]
fn test_partial_config() {
    let cfg = NamespaceConfig::partial();
    assert!(cfg.mount_ns);
    assert!(cfg.pid_ns);
    assert!(!cfg.net_ns);
    assert!(!cfg.user_ns);
    assert!(!cfg.ipc_ns);
    assert_eq!(cfg.isolation_level(), IsolationLevel::Partial);
    assert_eq!(cfg.enabled_namespaces(), vec!["mount", "pid"]);
}

#[test]
fn test_full_config() {
    let cfg = NamespaceConfig::full();
    assert!(cfg.mount_ns);
    assert!(cfg.pid_ns);
    assert!(cfg.net_ns);
    assert!(cfg.user_ns);
    assert!(cfg.ipc_ns);
    assert_eq!(cfg.isolation_level(), IsolationLevel::Full);
    assert_eq!(
        cfg.enabled_namespaces(),
        vec!["mount", "pid", "net", "user", "ipc"]
    );
}

#[test]
fn test_from_level() {
    assert_eq!(
        NamespaceConfig::from_level(IsolationLevel::Full),
        NamespaceConfig::full()
    );
    assert_eq!(
        NamespaceConfig::from_level(IsolationLevel::None),
        NamespaceConfig::none()
    );
}

#[test]
fn test_builder_empty() {
    let cfg = NamespaceBuilder::new().build();
    assert_eq!(cfg, NamespaceConfig::none());
}

#[test]
fn test_builder_partial() {
    let cfg = NamespaceBuilder::new().mount_ns().pid_ns().build();
    assert_eq!(cfg, NamespaceConfig::partial());
}

#[test]
fn test_builder_full() {
    let cfg = NamespaceBuilder::new()
        .mount_ns()
        .pid_ns()
        .net_ns()
        .user_ns()
        .ipc_ns()
        .build();
    assert_eq!(cfg, NamespaceConfig::full());
}

#[test]
fn test_builder_from_level() {
    let cfg = NamespaceBuilder::from_level(IsolationLevel::Partial).build();
    assert_eq!(cfg, NamespaceConfig::partial());
}

#[test]
fn test_builder_custom() {
    let cfg = NamespaceBuilder::new().mount_ns().net_ns().build();
    assert!(cfg.mount_ns);
    assert!(!cfg.pid_ns);
    assert!(cfg.net_ns);
    assert!(!cfg.user_ns);
    assert!(!cfg.ipc_ns);
    // Custom combo — classified as Partial
    assert_eq!(cfg.isolation_level(), IsolationLevel::Partial);
}

#[test]
#[ignore = "process-global privileged syscall; unsafe in parallel test. Run: --ignored --test-threads=1"]
fn test_apply_succeeds() {
    let cfg = NamespaceConfig::full();
    assert!(cfg.apply().is_ok());
}

#[test]
fn test_default_is_none() {
    let cfg = NamespaceConfig::default();
    assert_eq!(cfg, NamespaceConfig::none());
}
