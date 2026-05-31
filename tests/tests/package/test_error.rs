//! External tests for hudhudscript-package::error — PackageError display and conversions.

use hudhudscript_package::PackageError;

#[test]
fn test_error_display_package_not_found() {
    let err = PackageError::PackageNotFound("my-pkg".to_string());
    assert!(err.to_string().contains("Package not found: my-pkg"));
}

#[test]
fn test_error_display_version_not_found() {
    let err = PackageError::VersionNotFound("my-pkg@2.0.0".to_string());
    assert!(err.to_string().contains("Version not found: my-pkg@2.0.0"));
}

#[test]
fn test_error_display_checksum_mismatch() {
    let err = PackageError::ChecksumMismatch("my-pkg".to_string());
    assert!(err
        .to_string()
        .contains("Checksum mismatch for package: my-pkg"));
}

#[test]
fn test_error_display_dependency_resolution() {
    let err = PackageError::DependencyResolution("cycle detected".to_string());
    assert!(err
        .to_string()
        .contains("Dependency resolution failed: cycle detected"));
}

#[test]
fn test_error_display_invalid_package_name() {
    let err = PackageError::InvalidPackageName("bad name".to_string());
    assert!(err.to_string().contains("Invalid package name: bad name"));
}

#[test]
fn test_error_display_invalid_version() {
    let err = PackageError::InvalidVersion("not-semver".to_string());
    assert!(err.to_string().contains("Invalid version: not-semver"));
}

#[test]
fn test_error_display_security_vulnerability() {
    let err = PackageError::SecurityVulnerability("CVE-2025-001".to_string());
    assert!(err
        .to_string()
        .contains("Security vulnerability detected: CVE-2025-001"));
}

#[test]
fn test_error_display_build_failed() {
    let err = PackageError::BuildFailed("compilation error".to_string());
    assert!(err.to_string().contains("Build failed: compilation error"));
}

#[test]
fn test_error_display_entry_point_not_found() {
    let err = PackageError::EntryPointNotFound("main.hud".to_string());
    assert!(err.to_string().contains("Entry point not found: main.hud"));
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let pkg_err: PackageError = io_err.into();
    assert!(pkg_err.to_string().contains("IO error"));
}

#[test]
fn test_error_from_anyhow() {
    let anyhow_err = anyhow::anyhow!("something went wrong");
    let pkg_err: PackageError = anyhow_err.into();
    assert!(pkg_err.to_string().contains("something went wrong"));
}

#[test]
fn test_error_display_other() {
    let err = PackageError::Other(anyhow::anyhow!("custom error"));
    assert!(err.to_string().contains("custom error"));
}

#[test]
fn test_error_display_toml_serialize() {
    // Create a TOML serialization error by serializing an unsupported type
    use serde::ser::Error;
    let toml_err = toml::ser::Error::custom("bad value");
    let err = PackageError::TomlSerialize(toml_err);
    assert!(err.to_string().contains("TOML serialization error"));
}

#[test]
fn test_error_from_serde_json() {
    let json_err: serde_json::Error =
        serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let pkg_err: PackageError = json_err.into();
    assert!(pkg_err.to_string().contains("Serialization error"));
}

#[test]
fn test_error_debug_format() {
    let err = PackageError::PackageNotFound("test-pkg".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("PackageNotFound"));
    assert!(debug.contains("test-pkg"));
}
