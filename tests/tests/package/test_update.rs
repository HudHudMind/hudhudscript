//! External tests for hudhudscript-package::update —
//! UpdateInfo, InstalledPackage, UpdateChecker, is_newer.

use chrono::Utc;
use hudhudscript_package::update::is_newer;
use hudhudscript_package::{InstalledPackage, RegistryClient, UpdateChecker, UpdateInfo};

#[test]
fn test_is_newer() {
    assert!(is_newer("1.0.0", "1.0.1"));
    assert!(is_newer("1.0.0", "1.1.0"));
    assert!(is_newer("1.0.0", "2.0.0"));
    assert!(!is_newer("1.0.0", "1.0.0"));
    assert!(!is_newer("2.0.0", "1.0.0"));
    assert!(!is_newer("invalid", "1.0.0"));
}

#[test]
fn test_update_info_major() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.2.3".to_string(),
        latest_version: "2.0.0".to_string(),
        changelog: None,
    };
    assert!(info.is_major_update());
    assert!(!info.is_minor_update());
    assert!(!info.is_patch_update());
}

#[test]
fn test_update_info_minor() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.2.3".to_string(),
        latest_version: "1.3.0".to_string(),
        changelog: None,
    };
    assert!(!info.is_major_update());
    assert!(info.is_minor_update());
    assert!(!info.is_patch_update());
}

#[test]
fn test_update_info_patch() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.2.3".to_string(),
        latest_version: "1.2.5".to_string(),
        changelog: None,
    };
    assert!(!info.is_major_update());
    assert!(!info.is_minor_update());
    assert!(info.is_patch_update());
}

#[test]
fn test_installed_package_serialization() {
    let pkg = InstalledPackage {
        name: "my-pkg".to_string(),
        version: "1.0.0".to_string(),
        install_date: Utc::now(),
    };
    let json = serde_json::to_string(&pkg).unwrap();
    let deserialized: InstalledPackage = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "my-pkg");
    assert_eq!(deserialized.version, "1.0.0");
}

#[test]
fn test_update_info_serialization() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: "2.0.0".to_string(),
        changelog: Some("Breaking changes".to_string()),
    };
    let json = serde_json::to_string(&info).unwrap();
    let deserialized: UpdateInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.package_name, "pkg");
    assert_eq!(deserialized.changelog, Some("Breaking changes".to_string()));
}

#[test]
fn test_update_checker_creation() {
    let registry = RegistryClient::new("https://registry.hudhudscript.org").unwrap();
    let checker = UpdateChecker::new(registry);
    let _ = checker;
}

#[test]
fn test_is_newer_invalid_latest() {
    assert!(!is_newer("1.0.0", "not-valid"));
}

#[test]
fn test_is_newer_both_invalid() {
    assert!(!is_newer("bad", "worse"));
}

#[test]
fn test_update_info_invalid_versions_major() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "not-semver".to_string(),
        latest_version: "2.0.0".to_string(),
        changelog: None,
    };
    assert!(!info.is_major_update());
    assert!(!info.is_minor_update());
    assert!(!info.is_patch_update());
}

#[test]
fn test_update_info_same_version() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: "1.0.0".to_string(),
        changelog: None,
    };
    assert!(!info.is_major_update());
    assert!(!info.is_minor_update());
    assert!(!info.is_patch_update());
}

#[test]
fn test_is_newer_equal() {
    assert!(!is_newer("1.2.3", "1.2.3"));
}

#[test]
fn test_is_newer_prerelease() {
    assert!(is_newer("1.0.0-alpha", "1.0.0"));
}

#[test]
fn test_update_info_no_update_older_latest() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "2.0.0".to_string(),
        latest_version: "1.0.0".to_string(),
        changelog: None,
    };
    assert!(!info.is_major_update());
    assert!(!info.is_minor_update());
    assert!(!info.is_patch_update());
}

#[test]
fn test_update_info_invalid_latest_version() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: "not-semver".to_string(),
        changelog: None,
    };
    assert!(!info.is_major_update());
    assert!(!info.is_minor_update());
    assert!(!info.is_patch_update());
}

#[test]
fn test_installed_package_roundtrip() {
    let pkg = InstalledPackage {
        name: "test-pkg".to_string(),
        version: "2.3.4".to_string(),
        install_date: Utc::now(),
    };
    let json = serde_json::to_string(&pkg).unwrap();
    let deserialized: InstalledPackage = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "test-pkg");
    assert_eq!(deserialized.version, "2.3.4");
}

#[test]
fn test_update_info_changelog_none() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: "1.0.1".to_string(),
        changelog: None,
    };
    assert!(info.changelog.is_none());
    assert!(info.is_patch_update());
}

#[test]
fn test_update_info_debug_format() {
    let info = UpdateInfo {
        package_name: "test-pkg".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: "2.0.0".to_string(),
        changelog: Some("Breaking changes".to_string()),
    };
    let debug = format!("{:?}", info);
    assert!(debug.contains("test-pkg"));
    assert!(debug.contains("Breaking changes"));
}

#[test]
fn test_installed_package_debug_format() {
    let pkg = InstalledPackage {
        name: "debug-pkg".to_string(),
        version: "0.1.0".to_string(),
        install_date: Utc::now(),
    };
    let debug = format!("{:?}", pkg);
    assert!(debug.contains("debug-pkg"));
}

#[test]
fn test_is_newer_prerelease_comparison() {
    // Pre-release versions should sort correctly
    assert!(is_newer("1.0.0-alpha.1", "1.0.0-alpha.2"));
    assert!(is_newer("1.0.0-beta", "1.0.0"));
}

#[test]
fn test_update_info_major_version_jump() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.5.3".to_string(),
        latest_version: "5.0.0".to_string(),
        changelog: None,
    };
    assert!(info.is_major_update());
    assert!(!info.is_minor_update());
    assert!(!info.is_patch_update());
}

#[test]
fn test_update_info_minor_same_major_higher_minor() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "2.1.0".to_string(),
        latest_version: "2.5.0".to_string(),
        changelog: None,
    };
    assert!(!info.is_major_update());
    assert!(info.is_minor_update());
    assert!(!info.is_patch_update());
}

#[test]
fn test_update_checker_clone() {
    let registry = RegistryClient::new("https://registry.hudhudscript.org").unwrap();
    let checker = UpdateChecker::new(registry);
    let cloned = checker.clone();
    let _ = cloned;
}

#[test]
fn test_is_newer_patch_bump() {
    assert!(is_newer("1.0.0", "1.0.1"));
    assert!(!is_newer("1.0.1", "1.0.0"));
}

#[test]
fn test_is_newer_minor_bump() {
    assert!(is_newer("1.0.0", "1.1.0"));
    assert!(!is_newer("1.1.0", "1.0.0"));
}

#[test]
fn test_is_newer_major_bump() {
    assert!(is_newer("1.0.0", "2.0.0"));
    assert!(!is_newer("2.0.0", "1.0.0"));
}

#[test]
fn test_update_info_minor_with_patch_increase() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.2.3".to_string(),
        latest_version: "1.4.1".to_string(),
        changelog: None,
    };
    // minor is higher, so is_minor_update
    assert!(info.is_minor_update());
    assert!(!info.is_major_update());
    assert!(!info.is_patch_update());
}

#[test]
fn test_update_info_clone() {
    let info = UpdateInfo {
        package_name: "pkg".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: "2.0.0".to_string(),
        changelog: Some("Breaking change".to_string()),
    };
    let cloned = info.clone();
    assert_eq!(cloned.package_name, "pkg");
    assert_eq!(cloned.changelog, Some("Breaking change".to_string()));
}

#[test]
fn test_installed_package_clone() {
    let pkg = InstalledPackage {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        install_date: Utc::now(),
    };
    let cloned = pkg.clone();
    assert_eq!(cloned.name, "test");
    assert_eq!(cloned.version, "1.0.0");
}

#[test]
fn test_is_newer_with_build_metadata() {
    // Build metadata is ignored in semver comparison
    assert!(!is_newer("1.0.0+build1", "1.0.0+build2"));
}
