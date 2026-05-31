//! External tests for hudhudscript-package::resolver —
//! parse_version_req, select_best_version, topological_sort, ResolvedDependency, DependencyResolver.

use hudhudscript_package::{
    parse_version_req, select_best_version, topological_sort, DependencyResolver, PackageError,
    RegistryClient, ResolvedDependency,
};
use semver::{Version, VersionReq};
use std::collections::HashMap;

#[test]
fn test_parse_version_req_caret() {
    let req = parse_version_req("^1.0").unwrap();
    assert!(req.matches(&Version::parse("1.2.3").unwrap()));
    assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
}

#[test]
fn test_parse_version_req_tilde() {
    let req = parse_version_req("~1.2").unwrap();
    assert!(req.matches(&Version::parse("1.2.5").unwrap()));
    assert!(!req.matches(&Version::parse("1.3.0").unwrap()));
}

#[test]
fn test_parse_version_req_latest() {
    let req = parse_version_req("latest").unwrap();
    assert!(req.matches(&Version::parse("99.99.99").unwrap()));
}

#[test]
fn test_select_best_version() {
    let versions = vec![
        "1.0.0".to_string(),
        "1.1.0".to_string(),
        "1.2.0".to_string(),
        "2.0.0".to_string(),
    ];
    let req = VersionReq::parse("^1.0").unwrap();
    assert_eq!(
        select_best_version(&versions, &req),
        Some("1.2.0".to_string())
    );
}

#[test]
fn test_topological_sort_simple() {
    let mut graph = HashMap::new();
    graph.insert(
        "a".to_string(),
        ResolvedDependency {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["b".to_string()],
            resolved_path: None,
        },
    );
    graph.insert(
        "b".to_string(),
        ResolvedDependency {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            resolved_path: None,
        },
    );

    let sorted = topological_sort(&graph).unwrap();
    let pos_a = sorted.iter().position(|d| d.name == "a").unwrap();
    let pos_b = sorted.iter().position(|d| d.name == "b").unwrap();
    assert!(pos_b < pos_a);
}

#[test]
fn test_parse_version_req_star() {
    let req = parse_version_req("*").unwrap();
    assert!(req.matches(&Version::parse("0.0.1").unwrap()));
    assert!(req.matches(&Version::parse("99.99.99").unwrap()));
}

#[test]
fn test_parse_version_req_latest_whitespace() {
    let req = parse_version_req("  latest  ").unwrap();
    assert!(req.matches(&Version::parse("1.0.0").unwrap()));
}

#[test]
fn test_parse_version_req_invalid() {
    let result = parse_version_req("not-a-version");
    assert!(result.is_err());
    match result {
        Err(PackageError::InvalidVersion(msg)) => {
            assert!(msg.contains("not-a-version"));
        }
        _ => panic!("Expected InvalidVersion error"),
    }
}

#[test]
fn test_parse_version_req_range() {
    let req = parse_version_req(">=1.0.0, <2.0.0").unwrap();
    assert!(req.matches(&Version::parse("1.5.0").unwrap()));
    assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
    assert!(!req.matches(&Version::parse("0.9.0").unwrap()));
}

#[test]
fn test_select_best_version_no_match() {
    let versions = vec!["1.0.0".to_string(), "1.1.0".to_string()];
    let req = VersionReq::parse("^3.0").unwrap();
    assert_eq!(select_best_version(&versions, &req), None);
}

#[test]
fn test_select_best_version_single_match() {
    let versions = vec![
        "1.0.0".to_string(),
        "2.0.0".to_string(),
        "3.0.0".to_string(),
    ];
    let req = VersionReq::parse("~2.0").unwrap();
    assert_eq!(
        select_best_version(&versions, &req),
        Some("2.0.0".to_string())
    );
}

#[test]
fn test_select_best_version_invalid_versions_ignored() {
    let versions = vec![
        "1.0.0".to_string(),
        "not-semver".to_string(),
        "1.1.0".to_string(),
    ];
    let req = VersionReq::parse("^1.0").unwrap();
    assert_eq!(
        select_best_version(&versions, &req),
        Some("1.1.0".to_string())
    );
}

#[test]
fn test_select_best_version_empty_list() {
    let versions: Vec<String> = vec![];
    let req = VersionReq::parse("^1.0").unwrap();
    assert_eq!(select_best_version(&versions, &req), None);
}

#[test]
fn test_topological_sort_cycle_detection() {
    let mut graph = HashMap::new();
    graph.insert(
        "a".to_string(),
        ResolvedDependency {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["b".to_string()],
            resolved_path: None,
        },
    );
    graph.insert(
        "b".to_string(),
        ResolvedDependency {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["a".to_string()],
            resolved_path: None,
        },
    );

    let result = topological_sort(&graph);
    assert!(result.is_err());
    match result {
        Err(PackageError::DependencyResolution(msg)) => {
            assert!(msg.contains("Cycle"));
        }
        _ => panic!("Expected DependencyResolution error"),
    }
}

#[test]
fn test_topological_sort_no_deps() {
    let mut graph = HashMap::new();
    graph.insert(
        "x".to_string(),
        ResolvedDependency {
            name: "x".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            resolved_path: None,
        },
    );
    graph.insert(
        "y".to_string(),
        ResolvedDependency {
            name: "y".to_string(),
            version: "2.0.0".to_string(),
            dependencies: vec![],
            resolved_path: None,
        },
    );

    let sorted = topological_sort(&graph).unwrap();
    assert_eq!(sorted.len(), 2);
}

#[test]
fn test_topological_sort_chain() {
    let mut graph = HashMap::new();
    graph.insert(
        "a".to_string(),
        ResolvedDependency {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["b".to_string()],
            resolved_path: None,
        },
    );
    graph.insert(
        "b".to_string(),
        ResolvedDependency {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["c".to_string()],
            resolved_path: None,
        },
    );
    graph.insert(
        "c".to_string(),
        ResolvedDependency {
            name: "c".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            resolved_path: None,
        },
    );

    let sorted = topological_sort(&graph).unwrap();
    assert_eq!(sorted.len(), 3);
    let pos_a = sorted.iter().position(|d| d.name == "a").unwrap();
    let pos_b = sorted.iter().position(|d| d.name == "b").unwrap();
    let pos_c = sorted.iter().position(|d| d.name == "c").unwrap();
    assert!(pos_c < pos_b);
    assert!(pos_b < pos_a);
}

#[test]
fn test_topological_sort_external_deps_ignored() {
    // Dependencies referencing packages not in the graph are ignored
    let mut graph = HashMap::new();
    graph.insert(
        "a".to_string(),
        ResolvedDependency {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["external-not-in-graph".to_string()],
            resolved_path: None,
        },
    );

    let sorted = topological_sort(&graph).unwrap();
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0].name, "a");
}

#[test]
fn test_topological_sort_empty_graph() {
    let graph: HashMap<String, ResolvedDependency> = HashMap::new();
    let sorted = topological_sort(&graph).unwrap();
    assert_eq!(sorted.len(), 0);
}

#[test]
fn test_resolved_dependency_clone() {
    let dep = ResolvedDependency {
        name: "pkg".to_string(),
        version: "1.2.3".to_string(),
        dependencies: vec!["dep-a".to_string()],
        resolved_path: None,
    };
    let cloned = dep.clone();
    assert_eq!(cloned.name, "pkg");
    assert_eq!(cloned.version, "1.2.3");
    assert_eq!(cloned.dependencies, vec!["dep-a"]);
}

#[test]
fn test_parse_version_req_exact() {
    let req = parse_version_req("=1.2.3").unwrap();
    assert!(req.matches(&Version::parse("1.2.3").unwrap()));
    assert!(!req.matches(&Version::parse("1.2.4").unwrap()));
}

#[test]
fn test_parse_version_req_greater_than() {
    let req = parse_version_req(">1.0.0").unwrap();
    assert!(req.matches(&Version::parse("1.0.1").unwrap()));
    assert!(req.matches(&Version::parse("2.0.0").unwrap()));
    assert!(!req.matches(&Version::parse("1.0.0").unwrap()));
}

#[test]
fn test_select_best_version_prerelease() {
    let versions = vec![
        "1.0.0-alpha".to_string(),
        "1.0.0-beta".to_string(),
        "1.0.0".to_string(),
    ];
    let req = VersionReq::parse(">=1.0.0-alpha").unwrap();
    let best = select_best_version(&versions, &req);
    assert_eq!(best, Some("1.0.0".to_string()));
}

#[test]
fn test_select_best_version_many_matching() {
    let versions = vec![
        "1.0.0".to_string(),
        "1.0.1".to_string(),
        "1.0.2".to_string(),
        "1.1.0".to_string(),
        "1.2.0".to_string(),
        "1.9.9".to_string(),
    ];
    let req = VersionReq::parse("^1.0").unwrap();
    assert_eq!(
        select_best_version(&versions, &req),
        Some("1.9.9".to_string())
    );
}

#[test]
fn test_topological_sort_diamond() {
    // Diamond dependency: a -> b, a -> c, b -> d, c -> d
    let mut graph = HashMap::new();
    graph.insert(
        "a".to_string(),
        ResolvedDependency {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["b".to_string(), "c".to_string()],
            resolved_path: None,
        },
    );
    graph.insert(
        "b".to_string(),
        ResolvedDependency {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["d".to_string()],
            resolved_path: None,
        },
    );
    graph.insert(
        "c".to_string(),
        ResolvedDependency {
            name: "c".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["d".to_string()],
            resolved_path: None,
        },
    );
    graph.insert(
        "d".to_string(),
        ResolvedDependency {
            name: "d".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            resolved_path: None,
        },
    );

    let sorted = topological_sort(&graph).unwrap();
    assert_eq!(sorted.len(), 4);
    let pos_a = sorted.iter().position(|d| d.name == "a").unwrap();
    let pos_b = sorted.iter().position(|d| d.name == "b").unwrap();
    let pos_c = sorted.iter().position(|d| d.name == "c").unwrap();
    let pos_d = sorted.iter().position(|d| d.name == "d").unwrap();
    assert!(pos_d < pos_b);
    assert!(pos_d < pos_c);
    assert!(pos_b < pos_a);
    assert!(pos_c < pos_a);
}

#[test]
fn test_topological_sort_three_way_cycle() {
    let mut graph = HashMap::new();
    graph.insert(
        "a".to_string(),
        ResolvedDependency {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["b".to_string()],
            resolved_path: None,
        },
    );
    graph.insert(
        "b".to_string(),
        ResolvedDependency {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["c".to_string()],
            resolved_path: None,
        },
    );
    graph.insert(
        "c".to_string(),
        ResolvedDependency {
            name: "c".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["a".to_string()],
            resolved_path: None,
        },
    );

    let result = topological_sort(&graph);
    assert!(result.is_err());
}

#[test]
fn test_parse_version_req_whitespace_star() {
    let req = parse_version_req("  *  ").unwrap();
    assert!(req.matches(&Version::parse("1.0.0").unwrap()));
}

#[test]
fn test_select_best_version_all_invalid() {
    let versions = vec!["not-semver".to_string(), "also-not".to_string()];
    let req = VersionReq::parse("^1.0").unwrap();
    assert_eq!(select_best_version(&versions, &req), None);
}

#[test]
fn test_resolved_dependency_debug_format() {
    let dep = ResolvedDependency {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        dependencies: vec![],
        resolved_path: None,
    };
    let debug = format!("{:?}", dep);
    assert!(debug.contains("test"));
    assert!(debug.contains("0.1.0"));
}

#[test]
fn test_dependency_resolver_clone() {
    let registry = RegistryClient::new("https://example.com").unwrap();
    let resolver = DependencyResolver::new(registry);
    let cloned = resolver.clone();
    let _ = cloned;
}
