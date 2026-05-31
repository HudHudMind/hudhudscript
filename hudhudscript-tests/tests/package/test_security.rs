//! External tests for hudhudscript-package::security —
//! SecurityChecker, Advisory, calculate_checksum.

use hudhudscript_package::security::calculate_checksum;
use hudhudscript_package::{Advisory, DependencySpec, SecurityChecker};
use std::collections::HashMap;

#[test]
fn test_calculate_checksum() {
    let data = b"hello world";
    let checksum = calculate_checksum(data);
    assert_eq!(
        checksum,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn test_calculate_checksum_empty() {
    let checksum = calculate_checksum(b"");
    assert_eq!(
        checksum,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_load_advisories_missing_file() {
    let advisories = SecurityChecker::load_advisories_from_file("/nonexistent/path");
    assert!(advisories.is_empty());
}

#[tokio::test]
async fn test_audit_with_local_advisory() {
    let advisories = vec![Advisory {
        id: "HUDHUD-2025-001".to_string(),
        package: "bad-pkg".to_string(),
        affected_versions: "<1.0.0".to_string(),
        title: "Remote code execution".to_string(),
        severity: "critical".to_string(),
        url: None,
    }];

    let checker = SecurityChecker {
        registry: None,
        advisories,
    };

    let mut deps = HashMap::new();
    deps.insert(
        "bad-pkg".to_string(),
        DependencySpec::Simple("0.9.0".to_string()),
    );

    let warnings = checker.audit_dependencies(&deps).await.unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("Remote code execution"));
}

#[tokio::test]
async fn test_audit_no_match() {
    let advisories = vec![Advisory {
        id: "HUDHUD-2025-001".to_string(),
        package: "bad-pkg".to_string(),
        affected_versions: "<1.0.0".to_string(),
        title: "Remote code execution".to_string(),
        severity: "critical".to_string(),
        url: None,
    }];

    let checker = SecurityChecker {
        registry: None,
        advisories,
    };

    let mut deps = HashMap::new();
    deps.insert(
        "bad-pkg".to_string(),
        DependencySpec::Simple("2.0.0".to_string()),
    );

    let warnings = checker.audit_dependencies(&deps).await.unwrap();
    assert!(warnings.is_empty());
}

#[tokio::test]
async fn test_verify_checksum_no_registry() {
    let checker = SecurityChecker::new();
    let result = checker.verify_checksum(b"data", "pkg", "1.0.0").await;
    assert!(result.is_ok());
}

#[test]
fn test_security_checker_default() {
    let checker = SecurityChecker::default();
    // default delegates to new(), which has no registry and empty advisories
    assert!(checker.registry.is_none());
    assert_eq!(checker.advisories.len(), 0);
}

#[test]
fn test_security_checker_with_registry() {
    let registry = hudhudscript_package::RegistryClient::new("https://example.com").unwrap();
    let advisories = vec![Advisory {
        id: "ADV-001".to_string(),
        package: "foo".to_string(),
        affected_versions: "<2.0.0".to_string(),
        title: "Test advisory".to_string(),
        severity: "high".to_string(),
        url: Some("https://example.com/advisory".to_string()),
    }];
    let checker = SecurityChecker::with_registry(registry, advisories.clone());
    assert!(checker.registry.is_some());
    assert_eq!(checker.advisories.len(), 1);
    assert_eq!(checker.advisories[0].id, "ADV-001");
}

#[tokio::test]
async fn test_audit_with_advisory_url_field() {
    let advisories = vec![Advisory {
        id: "ADV-002".to_string(),
        package: "vuln-pkg".to_string(),
        affected_versions: ">=0.1.0, <1.0.0".to_string(),
        title: "Data leak".to_string(),
        severity: "high".to_string(),
        url: Some("https://advisory.example.com/002".to_string()),
    }];

    let checker = SecurityChecker {
        registry: None,
        advisories,
    };

    let mut deps = HashMap::new();
    deps.insert(
        "vuln-pkg".to_string(),
        DependencySpec::Simple("0.5.0".to_string()),
    );

    let warnings = checker.audit_dependencies(&deps).await.unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("Data leak"));
    assert!(warnings[0].contains("https://advisory.example.com/002"));
}

#[tokio::test]
async fn test_audit_different_package_not_matched() {
    let advisories = vec![Advisory {
        id: "ADV-003".to_string(),
        package: "other-pkg".to_string(),
        affected_versions: "<1.0.0".to_string(),
        title: "Bug".to_string(),
        severity: "low".to_string(),
        url: None,
    }];

    let checker = SecurityChecker {
        registry: None,
        advisories,
    };

    let mut deps = HashMap::new();
    deps.insert(
        "my-pkg".to_string(),
        DependencySpec::Simple("0.5.0".to_string()),
    );

    let warnings = checker.audit_dependencies(&deps).await.unwrap();
    assert_eq!(warnings.len(), 0);
}

#[tokio::test]
async fn test_audit_non_semver_version_skipped() {
    let advisories = vec![Advisory {
        id: "ADV-004".to_string(),
        package: "pkg".to_string(),
        affected_versions: "<1.0.0".to_string(),
        title: "Issue".to_string(),
        severity: "medium".to_string(),
        url: None,
    }];

    let checker = SecurityChecker {
        registry: None,
        advisories,
    };

    let mut deps = HashMap::new();
    // Non-semver version string won't parse, so advisory check is skipped
    deps.insert(
        "pkg".to_string(),
        DependencySpec::Simple("^0.5".to_string()),
    );

    let warnings = checker.audit_dependencies(&deps).await.unwrap();
    // ^0.5 doesn't parse as a semver::Version, so no match
    assert_eq!(warnings.len(), 0);
}

#[test]
fn test_calculate_checksum_deterministic() {
    let data = b"deterministic check";
    let c1 = calculate_checksum(data);
    let c2 = calculate_checksum(data);
    assert_eq!(c1, c2);
    assert_eq!(c1.len(), 64); // SHA-256 hex is 64 chars
}

#[test]
fn test_load_advisories_invalid_json() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "not valid json").unwrap();
    let advisories = SecurityChecker::load_advisories_from_file(tmp.path().to_str().unwrap());
    assert_eq!(advisories.len(), 0);
}

#[test]
fn test_load_advisories_valid_json() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let json = serde_json::json!([
        {
            "id": "ADV-100",
            "package": "test",
            "affected_versions": "<1.0.0",
            "title": "Test",
            "severity": "low",
            "url": null
        }
    ]);
    std::fs::write(tmp.path(), json.to_string()).unwrap();
    let advisories = SecurityChecker::load_advisories_from_file(tmp.path().to_str().unwrap());
    assert_eq!(advisories.len(), 1);
    assert_eq!(advisories[0].id, "ADV-100");
}

#[tokio::test]
async fn test_audit_empty_deps() {
    let checker = SecurityChecker::new();
    let deps = HashMap::new();
    let warnings = checker.audit_dependencies(&deps).await.unwrap();
    assert_eq!(warnings.len(), 0);
}

#[tokio::test]
async fn test_audit_multiple_advisories_same_package() {
    let advisories = vec![
        Advisory {
            id: "ADV-A".to_string(),
            package: "pkg".to_string(),
            affected_versions: "<2.0.0".to_string(),
            title: "First issue".to_string(),
            severity: "high".to_string(),
            url: None,
        },
        Advisory {
            id: "ADV-B".to_string(),
            package: "pkg".to_string(),
            affected_versions: "<1.5.0".to_string(),
            title: "Second issue".to_string(),
            severity: "critical".to_string(),
            url: Some("https://example.com".to_string()),
        },
    ];

    let checker = SecurityChecker {
        registry: None,
        advisories,
    };

    let mut deps = HashMap::new();
    deps.insert(
        "pkg".to_string(),
        DependencySpec::Simple("1.0.0".to_string()),
    );

    let warnings = checker.audit_dependencies(&deps).await.unwrap();
    // Version 1.0.0 matches both advisories
    assert_eq!(warnings.len(), 2);
    assert!(warnings.iter().any(|w| w.contains("First issue")));
    assert!(warnings.iter().any(|w| w.contains("Second issue")));
    // Second advisory has a URL
    assert!(warnings.iter().any(|w| w.contains("https://example.com")));
}

#[tokio::test]
async fn test_audit_invalid_advisory_version_req() {
    let advisories = vec![Advisory {
        id: "ADV-BAD".to_string(),
        package: "pkg".to_string(),
        affected_versions: "not-a-valid-req".to_string(),
        title: "Bad req".to_string(),
        severity: "low".to_string(),
        url: None,
    }];

    let checker = SecurityChecker {
        registry: None,
        advisories,
    };

    let mut deps = HashMap::new();
    deps.insert(
        "pkg".to_string(),
        DependencySpec::Simple("1.0.0".to_string()),
    );

    // Invalid version req means the advisory won't match
    let warnings = checker.audit_dependencies(&deps).await.unwrap();
    assert_eq!(warnings.len(), 0);
}

#[tokio::test]
async fn test_audit_detailed_dependency_spec() {
    let advisories = vec![Advisory {
        id: "ADV-D".to_string(),
        package: "detail-pkg".to_string(),
        affected_versions: "<2.0.0".to_string(),
        title: "Detail vuln".to_string(),
        severity: "medium".to_string(),
        url: None,
    }];

    let checker = SecurityChecker {
        registry: None,
        advisories,
    };

    let mut deps = HashMap::new();
    deps.insert(
        "detail-pkg".to_string(),
        DependencySpec::Detailed {
            version: "1.5.0".to_string(),
            features: vec![],
            registry: None,
            git: None,
            branch: None,
            tag: None,
            path: None,
            optional: false,
        },
    );

    let warnings = checker.audit_dependencies(&deps).await.unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("Detail vuln"));
}

#[test]
fn test_calculate_checksum_large_data() {
    let data = vec![0xAB_u8; 10000];
    let checksum = calculate_checksum(&data);
    assert_eq!(checksum.len(), 64);
    // Deterministic
    assert_eq!(checksum, calculate_checksum(&data));
}

#[test]
fn test_load_advisories_valid_with_url_field() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let json = serde_json::json!([
        {
            "id": "ADV-URL",
            "package": "test",
            "affected_versions": ">=1.0.0, <2.0.0",
            "title": "URL advisory",
            "severity": "high",
            "url": "https://advisory.example.com/1"
        }
    ]);
    std::fs::write(tmp.path(), json.to_string()).unwrap();
    let advisories = SecurityChecker::load_advisories_from_file(tmp.path().to_str().unwrap());
    assert_eq!(advisories.len(), 1);
    assert_eq!(
        advisories[0].url,
        Some("https://advisory.example.com/1".to_string())
    );
}

#[tokio::test]
async fn test_verify_checksum_no_registry_any_data() {
    let checker = SecurityChecker::new();
    // Without a registry, any data passes
    let result = checker
        .verify_checksum(b"arbitrary data", "any-pkg", "99.0.0")
        .await;
    assert!(result.is_ok());
}
