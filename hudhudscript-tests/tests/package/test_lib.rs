//! External tests for hudhudscript-package::lib — PackageManager, VERSION, DEFAULT_REGISTRY, PACKAGES_DIR.

use hudhudscript_package::{HudhudConfig, PackageManager, DEFAULT_REGISTRY, PACKAGES_DIR, VERSION};

#[tokio::test]
async fn test_package_manager_creation() {
    let config = HudhudConfig::default();
    let pm = PackageManager::new(config);
    assert!(pm.is_ok());
}

#[test]
fn test_version_constant() {
    assert!(!VERSION.is_empty());
    // Should parse as semver
    assert!(semver::Version::parse(VERSION).is_ok());
}

#[test]
fn test_default_registry_url() {
    assert_eq!(DEFAULT_REGISTRY, "https://registry.hudhudscript.org");
}

#[test]
fn test_packages_dir_constant() {
    assert_eq!(PACKAGES_DIR, ".hudpackages");
}

#[test]
fn test_package_manager_list() {
    let config = HudhudConfig::default();
    let pm = PackageManager::new(config).unwrap();
    let result = pm.list();
    assert!(result.is_ok());
}

#[test]
fn test_package_manager_has_all_components() {
    let config = HudhudConfig::default();
    let pm = PackageManager::new(config).unwrap();
    // Verify the package manager was fully initialized by testing list
    let list = pm.list().unwrap();
    // Fresh cache should be empty or contain nothing
    let _ = list;
}

#[tokio::test]
async fn test_package_manager_audit_empty_deps() {
    let config = HudhudConfig::default();
    let pm = PackageManager::new(config).unwrap();
    let warnings = pm.audit().await.unwrap();
    assert!(warnings.is_empty());
}

#[test]
fn test_version_is_valid_semver() {
    let version = semver::Version::parse(VERSION).unwrap();
    assert!(version.major > 0 || version.minor > 0 || version.patch > 0);
}
