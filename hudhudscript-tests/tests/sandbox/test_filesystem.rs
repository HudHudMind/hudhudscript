//! Tests extracted from hudhudscript-sandbox/src/filesystem.rs

use hudhudscript_sandbox::{FileSystemConfig, FileSystemSandbox};

#[test]
fn test_read_access() {
    let config = FileSystemConfig {
        allow_read: vec!["/tmp".to_string(), "/data/public".to_string()],
        allow_write: vec!["/tmp".to_string()],
        deny_all: vec!["/etc".to_string()],
    };

    let sandbox = FileSystemSandbox::new(config);

    // Should allow read from allowed paths
    assert!(sandbox.check_access("/tmp/file.txt", false).is_ok());
    assert!(sandbox.check_access("/data/public/file.txt", false).is_ok());

    // Should deny read from denied paths
    assert!(sandbox.check_access("/etc/passwd", false).is_err());

    // Should deny read from non-allowed paths
    assert!(sandbox.check_access("/root/file.txt", false).is_err());
}

#[test]
fn test_write_access() {
    let config = FileSystemConfig {
        allow_read: vec!["/tmp".to_string(), "/data".to_string()],
        allow_write: vec!["/tmp".to_string()],
        deny_all: vec![],
    };

    let sandbox = FileSystemSandbox::new(config);

    // Should allow write to allowed paths
    assert!(sandbox.check_access("/tmp/file.txt", true).is_ok());

    // Should deny write to read-only paths
    assert!(sandbox.check_access("/data/file.txt", true).is_err());
}

#[test]
fn test_wildcard_patterns() {
    let config = FileSystemConfig {
        allow_read: vec!["/data/*".to_string()],
        allow_write: vec![],
        deny_all: vec![],
    };

    let sandbox = FileSystemSandbox::new(config);

    // Should match wildcard patterns
    assert!(sandbox.check_access("/data/public/file.txt", false).is_ok());
    assert!(sandbox
        .check_access("/data/private/file.txt", false)
        .is_ok());

    // Should not match outside wildcard
    assert!(sandbox.check_access("/other/file.txt", false).is_err());
}

#[test]
fn test_all_wildcard_allows_everything() {
    let config = FileSystemConfig {
        allow_read: vec!["/*".to_string()],
        allow_write: vec!["/*".to_string()],
        deny_all: vec![],
    };
    let sandbox = FileSystemSandbox::new(config);
    assert!(sandbox.check_access("/anything/at/all", false).is_ok());
    assert!(sandbox.check_access("/anything/at/all", true).is_ok());
}

#[test]
fn test_exact_path_match() {
    let config = FileSystemConfig {
        allow_read: vec!["/specific/file.txt".to_string()],
        allow_write: vec![],
        deny_all: vec![],
    };
    let sandbox = FileSystemSandbox::new(config);
    assert!(sandbox.check_access("/specific/file.txt", false).is_ok());
    assert!(sandbox.check_access("/specific/other.txt", false).is_err());
}

#[test]
fn test_deny_takes_precedence_over_allow() {
    let config = FileSystemConfig {
        allow_read: vec!["/tmp".to_string()],
        allow_write: vec![],
        deny_all: vec!["/tmp/secret".to_string()],
    };
    let sandbox = FileSystemSandbox::new(config);
    assert!(sandbox.check_access("/tmp/file.txt", false).is_ok());
    assert!(sandbox.check_access("/tmp/secret/data", false).is_err());
}

#[test]
fn test_path_component_boundary() {
    let config = FileSystemConfig {
        allow_read: vec!["/tmp".to_string()],
        allow_write: vec!["/tmp".to_string()],
        deny_all: vec!["/etc".to_string()],
    };

    let sandbox = FileSystemSandbox::new(config);

    // /tmp/foo should be allowed
    assert!(sandbox.check_access("/tmp/foo.txt", false).is_ok());
    // /tmpevil should NOT match /tmp (component boundary)
    assert!(sandbox.check_access("/tmpevil/foo.txt", false).is_err());
    // /etcetera should NOT match /etc deny rule (so it won't be denied by /etc)
    // but it's also not in the allow list, so it should be denied
    assert!(sandbox.check_access("/etcetera/file", false).is_err());
}
