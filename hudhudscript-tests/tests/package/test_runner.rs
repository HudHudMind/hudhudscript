//! External tests for hudhudscript-package::runner —
//! find_entry_point, is_application_package, get_exported_modules, collect_modules, PackageRunner.

use hudhudscript_package::{
    collect_modules, find_entry_point, get_exported_modules, is_application_package, HudhudConfig,
    PackageRunner, PackageType,
};

#[test]
fn test_find_entry_point_main_hud() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("main.hud"), "print(1)").unwrap();
    let entry = find_entry_point(dir.path());
    assert!(entry.is_some());
    assert!(entry.unwrap().ends_with("main.hud"));
}

#[test]
fn test_find_entry_point_missing() {
    let dir = tempfile::tempdir().unwrap();
    assert!(find_entry_point(dir.path()).is_none());
}

#[test]
fn test_is_application_package() {
    let mut config = HudhudConfig::default();
    assert!(is_application_package(&config));
    config.package.package_type = PackageType::Library;
    assert!(!is_application_package(&config));
}

#[test]
fn test_get_exported_modules() {
    let dir = tempfile::tempdir().unwrap();
    let lib = dir.path().join("lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(lib.join("utils.hud"), "").unwrap();
    std::fs::write(lib.join("helpers.hud"), "").unwrap();
    // Root-level module (not main)
    std::fs::write(dir.path().join("extra.hud"), "").unwrap();
    // main.hud should be excluded
    std::fs::write(dir.path().join("main.hud"), "").unwrap();

    let modules = get_exported_modules(dir.path());
    assert!(modules.contains(&"utils".to_string()));
    assert!(modules.contains(&"helpers".to_string()));
    assert!(modules.contains(&"extra".to_string()));
    assert!(!modules.contains(&"main".to_string()));
}

#[test]
fn test_find_entry_point_src_main_hud() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("main.hud"), "print(1)").unwrap();
    let entry = find_entry_point(dir.path());
    assert!(entry.is_some());
    let path = entry.unwrap();
    assert!(path.ends_with("src/main.hud"));
}

#[test]
fn test_find_entry_point_manifest_entry_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        r#"
[package]
name = "test"
version = "0.1.0"
entry = "app.hud"
"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("app.hud"), "print(42)").unwrap();
    // Also have main.hud to verify entry field takes precedence
    std::fs::write(dir.path().join("main.hud"), "print(1)").unwrap();
    let entry = find_entry_point(dir.path());
    assert!(entry.is_some());
    assert!(entry.unwrap().ends_with("app.hud"));
}

#[test]
fn test_find_entry_point_manifest_entry_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        r#"
[package]
name = "test"
version = "0.1.0"
entry = "nonexistent.hud"
"#,
    )
    .unwrap();
    // Falls through to conventional locations which don't exist either
    let entry = find_entry_point(dir.path());
    assert!(entry.is_none());
}

#[test]
fn test_package_runner_no_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let runner = PackageRunner::new(dir.path()).unwrap();
    // Should use default config
    assert_eq!(runner.config.package.name, "my-project");
}

#[test]
fn test_package_runner_with_manifest() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        r#"
[package]
name = "my-app"
version = "2.0.0"
type = "application"
entry = "main.hud"

[ai-providers.openai]
model = "gpt-4"

[mcp-servers]
github = "^1.0"
postgres = "^2.0"
"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("main.hud"), "print(1)").unwrap();

    let runner = PackageRunner::new(dir.path()).unwrap();
    assert_eq!(runner.config.package.name, "my-app");
    assert_eq!(runner.config.package.version, "2.0.0");

    let entry = runner.entry_point().unwrap();
    assert!(entry.ends_with("main.hud"));

    let providers = runner.ai_providers();
    assert_eq!(providers.len(), 1);
    assert!(providers.contains(&"openai".to_string()));

    let servers = runner.mcp_servers();
    assert_eq!(servers.len(), 2);
}

#[test]
fn test_package_runner_entry_point_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let runner = PackageRunner::new(dir.path()).unwrap();
    let result = runner.entry_point();
    assert!(result.is_err());
}

#[test]
fn test_get_exported_modules_nested_lib() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("lib").join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("deep.hud"), "").unwrap();

    let modules = get_exported_modules(dir.path());
    assert!(modules.contains(&"sub/deep".to_string()));
}

#[test]
fn test_get_exported_modules_empty() {
    let dir = tempfile::tempdir().unwrap();
    let modules = get_exported_modules(dir.path());
    assert_eq!(modules.len(), 0);
}

#[test]
fn test_get_exported_modules_dedup_and_sort() {
    let dir = tempfile::tempdir().unwrap();
    // Create lib/ with file and root-level with same name (if it happened)
    let lib = dir.path().join("lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(lib.join("alpha.hud"), "").unwrap();
    std::fs::write(lib.join("beta.hud"), "").unwrap();
    std::fs::write(dir.path().join("gamma.hud"), "").unwrap();

    let modules = get_exported_modules(dir.path());
    // Verify sorted order
    let mut expected = modules.clone();
    expected.sort();
    assert_eq!(modules, expected);
}

#[test]
fn test_get_exported_modules_ignores_non_hud_files() {
    let dir = tempfile::tempdir().unwrap();
    let lib = dir.path().join("lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(lib.join("readme.md"), "# readme").unwrap();
    std::fs::write(lib.join("data.json"), "{}").unwrap();
    std::fs::write(lib.join("utils.hud"), "").unwrap();
    // Also a non-hud file at root
    std::fs::write(dir.path().join("config.toml"), "").unwrap();

    let modules = get_exported_modules(dir.path());
    assert_eq!(modules.len(), 1);
    assert!(modules.contains(&"utils".to_string()));
}

#[test]
fn test_collect_modules_unreadable_dir() {
    // When dir doesn't exist, collect_modules returns early without error
    let mut modules = Vec::new();
    let fake_dir = std::path::Path::new("/nonexistent/path/that/does/not/exist");
    collect_modules(fake_dir, fake_dir, &mut modules);
    assert!(modules.is_empty());
}

#[test]
fn test_find_entry_point_manifest_no_entry_field() {
    // Manifest exists but has no entry field, falls through to conventional locations
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        r#"
[package]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("main.hud"), "print(1)").unwrap();
    let entry = find_entry_point(dir.path());
    assert!(entry.is_some());
    assert!(entry.unwrap().ends_with("main.hud"));
}

#[test]
fn test_package_runner_entry_point_error_message() {
    let dir = tempfile::tempdir().unwrap();
    let runner = PackageRunner::new(dir.path()).unwrap();
    let err = runner.entry_point().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("No entry point found"));
    assert!(msg.contains("Application packages require"));
}

#[test]
fn test_package_runner_empty_ai_providers_and_mcp_servers() {
    let dir = tempfile::tempdir().unwrap();
    let runner = PackageRunner::new(dir.path()).unwrap();
    assert!(runner.ai_providers().is_empty());
    assert!(runner.mcp_servers().is_empty());
}

#[test]
fn test_get_exported_modules_deeply_nested() {
    let dir = tempfile::tempdir().unwrap();
    let deep = dir.path().join("lib").join("a").join("b").join("c");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("deep_module.hud"), "").unwrap();

    let modules = get_exported_modules(dir.path());
    assert!(modules.contains(&"a/b/c/deep_module".to_string()));
}

#[test]
fn test_is_application_package_default_is_true() {
    let config = HudhudConfig::default();
    assert!(is_application_package(&config));
}

#[test]
fn test_get_exported_modules_root_only_no_lib() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("utils.hud"), "").unwrap();
    std::fs::write(dir.path().join("helpers.hud"), "").unwrap();

    let modules = get_exported_modules(dir.path());
    assert_eq!(modules.len(), 2);
    assert!(modules.contains(&"helpers".to_string()));
    assert!(modules.contains(&"utils".to_string()));
}
