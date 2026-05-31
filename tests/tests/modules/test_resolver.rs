//! Tests extracted from hudhudscript-modules/src/resolver.rs

use hudhudscript_modules::resolver::{ModuleResolver, ResolverError};
use std::path::PathBuf;

#[test]
fn test_resolver_relative() {
    let resolver = ModuleResolver::new(PathBuf::from("/base"));
    // This will fail in test but shows the logic
    let _ = resolver.resolve("./module");
}

#[test]
fn test_resolver_relative_existing_file() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("mymod.hudhud"), "").unwrap();
    let resolver = ModuleResolver::new(tmp.path().to_path_buf());

    let result = resolver.resolve("./mymod");
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.ends_with("mymod.hudhud"));
}

#[test]
fn test_resolver_relative_with_extension() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("mymod.hudhud"), "").unwrap();
    let resolver = ModuleResolver::new(tmp.path().to_path_buf());

    let result = resolver.resolve("./mymod.hudhud");
    assert!(result.is_ok());
}

#[test]
fn test_resolver_parent_relative() {
    let tmp = tempfile::tempdir().unwrap();
    let sub = tmp.path().join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(tmp.path().join("root.hudhud"), "").unwrap();

    let resolver = ModuleResolver::new(sub.clone());
    let result = resolver.resolve("../root");
    assert!(result.is_ok());
}

#[test]
fn test_resolver_search_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let search = tmp.path().join("search");
    std::fs::create_dir_all(&search).unwrap();
    std::fs::write(search.join("found.hudhud"), "").unwrap();

    let mut resolver = ModuleResolver::new(tmp.path().to_path_buf());
    resolver.add_search_path(search);

    let result = resolver.resolve("found");
    assert!(result.is_ok());
}

#[test]
fn test_resolver_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let resolver = ModuleResolver::new(tmp.path().to_path_buf());
    let result = resolver.resolve("nonexistent");
    assert!(result.is_err());
    match result {
        Err(ResolverError::NotFound(msg)) => {
            assert!(msg.contains("nonexistent"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_resolver_relative_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let resolver = ModuleResolver::new(tmp.path().to_path_buf());
    let result = resolver.resolve("./missing");
    assert!(result.is_err());
}

#[test]
fn test_resolver_multiple_search_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let sp1 = tmp.path().join("sp1");
    let sp2 = tmp.path().join("sp2");
    std::fs::create_dir_all(&sp1).unwrap();
    std::fs::create_dir_all(&sp2).unwrap();
    std::fs::write(sp2.join("insp2.hudhud"), "").unwrap();

    let mut resolver = ModuleResolver::new(tmp.path().to_path_buf());
    resolver.add_search_path(sp1);
    resolver.add_search_path(sp2);

    let result = resolver.resolve("insp2");
    assert!(result.is_ok());
}

#[test]
fn test_resolver_error_display() {
    let err = ResolverError::InvalidPath("bad/path".to_string());
    assert!(err.to_string().contains("Invalid module path: bad/path"));

    let err = ResolverError::NotFound("missing.hudhud".to_string());
    assert!(err.to_string().contains("Module not found: missing.hudhud"));
}
