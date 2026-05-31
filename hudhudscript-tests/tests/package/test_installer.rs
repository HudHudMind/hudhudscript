//! External tests for hudhudscript-package::installer —
//! InstallOptions, Installer, Installer::install_to_local, resolve_import, init_project.

use hudhudscript_package::{
    InstallOptions, Installer, PackageCache, ResolvedDependency, SecurityChecker, PACKAGES_DIR,
};

#[test]
fn test_install_options_default() {
    let opts = InstallOptions::default();
    assert!(!opts.dev);
    assert!(!opts.optional);
    assert!(!opts.force);
}

#[test]
fn test_install_to_local_creates_directories() {
    let tmp = tempfile::tempdir().unwrap();
    let deps = vec![
        ResolvedDependency {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            resolved_path: None,
        },
        ResolvedDependency {
            name: "pkg-b".to_string(),
            version: "2.0.0".to_string(),
            dependencies: vec![],
            resolved_path: None,
        },
    ];

    Installer::install_to_local(tmp.path(), &deps).unwrap();

    let a_lib = tmp.path().join(PACKAGES_DIR).join("pkg-a").join("lib");
    let b_lib = tmp.path().join(PACKAGES_DIR).join("pkg-b").join("lib");
    assert!(a_lib.is_dir());
    assert!(b_lib.is_dir());
}

#[test]
fn test_install_to_local_empty_deps() {
    let tmp = tempfile::tempdir().unwrap();
    let deps: Vec<ResolvedDependency> = vec![];
    Installer::install_to_local(tmp.path(), &deps).unwrap();
    // .hudpackages directory should still be created
    assert!(tmp.path().join(PACKAGES_DIR).is_dir());
}

#[test]
fn test_resolve_import_existing() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_dir = tmp.path().join(PACKAGES_DIR).join("my-lib").join("lib");
    std::fs::create_dir_all(&lib_dir).unwrap();

    let result = Installer::resolve_import(tmp.path(), "my-lib");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), lib_dir);
}

#[test]
fn test_resolve_import_not_installed() {
    let tmp = tempfile::tempdir().unwrap();
    let result = Installer::resolve_import(tmp.path(), "nonexistent-pkg");
    assert!(result.is_none());
}

#[test]
fn test_resolve_import_directory_but_no_lib() {
    let tmp = tempfile::tempdir().unwrap();
    // Create the package dir but not the lib/ subdirectory
    let pkg_dir = tmp.path().join(PACKAGES_DIR).join("partial-pkg");
    std::fs::create_dir_all(&pkg_dir).unwrap();

    let result = Installer::resolve_import(tmp.path(), "partial-pkg");
    assert!(result.is_none());
}

#[test]
fn test_install_to_local_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    let deps = vec![ResolvedDependency {
        name: "pkg".to_string(),
        version: "1.0.0".to_string(),
        dependencies: vec![],
        resolved_path: None,
    }];

    // Install twice - should not error
    Installer::install_to_local(tmp.path(), &deps).unwrap();
    Installer::install_to_local(tmp.path(), &deps).unwrap();

    let lib = tmp.path().join(PACKAGES_DIR).join("pkg").join("lib");
    assert!(lib.is_dir());
}

#[test]
fn test_install_options_custom() {
    let opts = InstallOptions {
        dev: true,
        optional: true,
        force: true,
    };
    assert!(opts.dev);
    assert!(opts.optional);
    assert!(opts.force);
}

#[test]
fn test_install_options_debug() {
    let opts = InstallOptions::default();
    let debug = format!("{:?}", opts);
    assert!(debug.contains("dev"));
    assert!(debug.contains("optional"));
    assert!(debug.contains("force"));
}

#[tokio::test]
async fn test_init_project_application() {
    let tmp = tempfile::tempdir().unwrap();
    let project_path = tmp.path().join("my-app");

    let cache_dir = tmp.path().join("cache");
    let cache = PackageCache::new(&cache_dir).unwrap();
    let security = SecurityChecker::new();
    let installer = Installer::new(cache, security);

    installer
        .init_project("my-app", project_path.to_str().unwrap(), false)
        .await
        .unwrap();

    // Verify files were created
    assert!(project_path.join("hudhud.toml").exists());
    assert!(project_path.join("main.hud").exists());
    assert!(project_path.join("agent.hud").exists());
    assert!(project_path.join("README.md").exists());
    assert!(project_path.join("tests").is_dir());

    // Verify config content
    let config_content = std::fs::read_to_string(project_path.join("hudhud.toml")).unwrap();
    assert!(config_content.contains("name = \"my-app\""));
    assert!(config_content.contains("type = \"application\""));
}

#[tokio::test]
async fn test_init_project_library() {
    let tmp = tempfile::tempdir().unwrap();
    let project_path = tmp.path().join("my-lib");

    let cache_dir = tmp.path().join("cache");
    let cache = PackageCache::new(&cache_dir).unwrap();
    let security = SecurityChecker::new();
    let installer = Installer::new(cache, security);

    installer
        .init_project("my-lib", project_path.to_str().unwrap(), true)
        .await
        .unwrap();

    // Verify library layout
    assert!(project_path.join("hudhud.toml").exists());
    assert!(project_path.join("lib").is_dir());
    assert!(project_path.join("lib/utils.hud").exists());
    assert!(project_path.join("tests").is_dir());
    assert!(project_path.join("README.md").exists());
    // Application files should NOT exist
    assert!(!project_path.join("main.hud").exists());
    assert!(!project_path.join("agent.hud").exists());

    // Verify config content
    let config_content = std::fs::read_to_string(project_path.join("hudhud.toml")).unwrap();
    assert!(config_content.contains("name = \"my-lib\""));
    assert!(config_content.contains("type = \"library\""));
}

#[test]
fn test_resolve_import_multiple_packages() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_a = tmp.path().join(PACKAGES_DIR).join("pkg-a").join("lib");
    let lib_b = tmp.path().join(PACKAGES_DIR).join("pkg-b").join("lib");
    std::fs::create_dir_all(&lib_a).unwrap();
    std::fs::create_dir_all(&lib_b).unwrap();

    assert!(Installer::resolve_import(tmp.path(), "pkg-a").is_some());
    assert!(Installer::resolve_import(tmp.path(), "pkg-b").is_some());
    assert!(Installer::resolve_import(tmp.path(), "pkg-c").is_none());
}
