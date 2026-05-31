//! Tests extracted from hudhudscript-modules/src/loader.rs

use hudhudscript_modules::loader::{ModuleLoader, ModuleLoaderError};
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_loader_resolve_path() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    // Create test file
    let test_file = temp_dir.path().join("test.hudhud");
    fs::write(&test_file, "").unwrap();

    let resolved = loader.resolve_path("./test").unwrap();
    assert!(resolved.exists());
}

#[tokio::test]
async fn test_loader_cache() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    // Create test file - empty file should parse
    let test_file = temp_dir.path().join("test.hudhud");
    fs::write(&test_file, "").unwrap();

    // Load module
    let module1 = loader.load("./test").await.unwrap();

    // Load again (should come from cache)
    let module2 = loader.get_cached("./test").await.unwrap();

    assert_eq!(module1.id, module2.id);
}

#[tokio::test]
async fn test_loader_module_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let result = loader.load("./nonexistent").await;
    assert!(result.is_err());
    match result {
        Err(ModuleLoaderError::ModuleNotFound(msg)) => {
            assert!(msg.contains("nonexistent"));
        }
        _ => panic!("Expected ModuleNotFound error"),
    }
}

#[tokio::test]
async fn test_loader_get_cached_miss() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let result = loader.get_cached("./nonexistent").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_loader_clear_cache() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("mod.hudhud");
    fs::write(&test_file, "").unwrap();

    loader.load("./mod").await.unwrap();
    assert!(loader.get_cached("./mod").await.is_some());

    loader.clear_cache().await;
    assert!(loader.get_cached("./mod").await.is_none());
}

#[tokio::test]
async fn test_loader_with_extension() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("mymod.hudhud");
    fs::write(&test_file, "let x = 1").unwrap();

    let module = loader.load("./mymod.hudhud").await.unwrap();
    assert_eq!(module.source, "let x = 1");
}

#[test]
fn test_loader_resolve_path_absolute() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("abs.hudhud");
    fs::write(&test_file, "").unwrap();

    let result = loader.resolve_path(test_file.to_str().unwrap());
    assert!(result.is_ok());
}

#[test]
fn test_loader_error_display() {
    let err = ModuleLoaderError::ModuleNotFound("test.hudhud".to_string());
    assert!(err.to_string().contains("Module not found: test.hudhud"));

    let err = ModuleLoaderError::ReadError("permission denied".to_string());
    assert!(err
        .to_string()
        .contains("Failed to read module: permission denied"));

    let err = ModuleLoaderError::ParseError("syntax error".to_string());
    assert!(err
        .to_string()
        .contains("Failed to parse module: syntax error"));

    let err = ModuleLoaderError::AlreadyLoaded("mod.hudhud".to_string());
    assert!(err
        .to_string()
        .contains("Module already loaded: mod.hudhud"));
}

#[tokio::test]
async fn test_loader_load_returns_cached_on_second_call() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("cached.hudhud");
    fs::write(&test_file, "let a = 1").unwrap();

    let m1 = loader.load("./cached").await.unwrap();
    let m2 = loader.load("./cached").await.unwrap();
    assert_eq!(m1.id, m2.id);
    assert_eq!(m1.source, m2.source);
}

#[tokio::test]
async fn test_loader_load_simple_module() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("simple.hudhud");
    fs::write(&test_file, "let x = 1\nlet y = 2").unwrap();

    let module = loader.load("./simple").await.unwrap();
    assert_eq!(module.source, "let x = 1\nlet y = 2");
    assert!(!module.executed);
}

#[test]
fn test_loader_resolve_path_adds_extension() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    // Create file with .hudhud extension
    let test_file = temp_dir.path().join("noext.hudhud");
    fs::write(&test_file, "").unwrap();

    // Resolve without extension
    let resolved = loader.resolve_path("./noext").unwrap();
    assert!(resolved.to_string_lossy().ends_with(".hudhud"));
}

#[test]
fn test_loader_resolve_path_parent_relative() {
    let temp_dir = TempDir::new().unwrap();
    let sub = temp_dir.path().join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(temp_dir.path().join("root_mod.hudhud"), "").unwrap();

    let loader = ModuleLoader::new(sub);
    let resolved = loader.resolve_path("../root_mod");
    assert!(resolved.is_ok());
}

#[test]
fn test_loader_resolve_path_not_found_error_message() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let result = loader.resolve_path("./does_not_exist");
    assert!(result.is_err());
    match result {
        Err(ModuleLoaderError::ModuleNotFound(msg)) => {
            assert!(msg.contains("does_not_exist"));
        }
        _ => panic!("Expected ModuleNotFound"),
    }
}

#[tokio::test]
async fn test_loader_load_parse_error() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    // Write syntactically invalid content
    let test_file = temp_dir.path().join("bad.hudhud");
    fs::write(&test_file, "fn (( {{{").unwrap();

    let result = loader.load("./bad").await;
    assert!(result.is_err());
    match result {
        Err(ModuleLoaderError::ParseError(_)) => {} // expected
        other => panic!("Expected ParseError, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_loader_get_cached_after_load() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("check_cache.hudhud");
    fs::write(&test_file, "let y = 2").unwrap();

    // Before loading, cache is empty
    assert!(loader.get_cached("./check_cache").await.is_none());

    // Load
    loader.load("./check_cache").await.unwrap();

    // After loading, cache has it
    let cached = loader.get_cached("./check_cache").await;
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().source, "let y = 2");
}

#[tokio::test]
async fn test_loader_clear_cache_then_reload() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ModuleLoader::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("reload.hudhud");
    fs::write(&test_file, "let a = 1").unwrap();

    loader.load("./reload").await.unwrap();
    loader.clear_cache().await;

    // Modify file content
    fs::write(&test_file, "let a = 2").unwrap();

    // Reload should get new content
    let module = loader.load("./reload").await.unwrap();
    assert_eq!(module.source, "let a = 2");
}
