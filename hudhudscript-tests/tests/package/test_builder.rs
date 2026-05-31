//! External tests for hudhudscript-package::builder — PackageBuilder, BuildResult.

use hudhudscript_package::PackageBuilder;

#[test]
fn test_builder_no_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let result = PackageBuilder::new(dir.path());
    assert!(result.is_err());
}

#[test]
fn test_builder_builds_archive() {
    let dir = tempfile::tempdir().unwrap();

    // Write a minimal manifest
    std::fs::write(
        dir.path().join("hudhud.toml"),
        r#"
[package]
name = "test-pkg"
version = "0.1.0"
"#,
    )
    .unwrap();

    // Write a source file
    std::fs::write(dir.path().join("main.hud"), "print(1)").unwrap();

    let builder = PackageBuilder::new(dir.path()).unwrap();
    let result = builder.build().unwrap();

    assert_eq!(result.source_file_count, 1);
    assert!(result.archive_path.exists());
    assert!(result.archive_path.to_string_lossy().ends_with(".hudpkg"));
}

#[test]
fn test_builder_skips_hidden_and_target_dirs() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    // Create hidden dir, target dir, and node_modules — all should be skipped
    let hidden = dir.path().join(".hidden");
    std::fs::create_dir_all(&hidden).unwrap();
    std::fs::write(hidden.join("secret.hud"), "secret()").unwrap();

    let target = dir.path().join("target");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("build.hud"), "build()").unwrap();

    let nm = dir.path().join("node_modules");
    std::fs::create_dir_all(&nm).unwrap();
    std::fs::write(nm.join("dep.hud"), "dep()").unwrap();

    // Only the main source should be included
    std::fs::write(dir.path().join("main.hud"), "print(1)").unwrap();

    let builder = PackageBuilder::new(dir.path()).unwrap();
    let result = builder.build().unwrap();
    assert_eq!(result.source_file_count, 1);
}

#[test]
fn test_builder_empty_source_warning() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    // Create an empty .hud file
    std::fs::write(dir.path().join("empty.hud"), "   ").unwrap();

    let builder = PackageBuilder::new(dir.path()).unwrap();
    // Should succeed, empty files generate a warning but not an error
    let result = builder.build().unwrap();
    assert_eq!(result.source_file_count, 1);
}

#[test]
fn test_builder_nested_sources() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        "[package]\nname = \"nested\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let sub = dir.path().join("src");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("a.hud"), "a()").unwrap();
    std::fs::write(sub.join("b.hud"), "b()").unwrap();
    std::fs::write(dir.path().join("main.hud"), "main()").unwrap();

    let builder = PackageBuilder::new(dir.path()).unwrap();
    let result = builder.build().unwrap();
    assert_eq!(result.source_file_count, 3);
}

#[test]
fn test_build_result_total_size() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        "[package]\nname = \"size-test\"\nversion = \"0.1.0\"",
    )
    .unwrap();
    std::fs::write(dir.path().join("main.hud"), "print(42)").unwrap();

    let builder = PackageBuilder::new(dir.path()).unwrap();
    let result = builder.build().unwrap();
    assert!(result.total_size > 0);
}

#[test]
fn test_builder_archive_name_includes_name_and_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        "[package]\nname = \"my-pkg\"\nversion = \"3.2.1\"",
    )
    .unwrap();
    std::fs::write(dir.path().join("main.hud"), "print(1)").unwrap();

    let builder = PackageBuilder::new(dir.path()).unwrap();
    let result = builder.build().unwrap();
    let archive_name = result
        .archive_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert_eq!(archive_name, "my-pkg-3.2.1.hudpkg");
}

#[test]
fn test_builder_no_hud_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        "[package]\nname = \"empty\"\nversion = \"0.1.0\"",
    )
    .unwrap();
    // No .hud files at all

    let builder = PackageBuilder::new(dir.path()).unwrap();
    let result = builder.build().unwrap();
    assert_eq!(result.source_file_count, 0);
    assert!(result.archive_path.exists());
}

#[test]
fn test_builder_with_dependencies_message() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        r#"[package]
name = "with-deps"
version = "0.1.0"

[dependencies]
some-lib = "^1.0"
"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("main.hud"), "import some_lib").unwrap();

    let builder = PackageBuilder::new(dir.path()).unwrap();
    let result = builder.build().unwrap();
    assert_eq!(result.source_file_count, 1);
}

#[test]
fn test_builder_deeply_nested_sources() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        "[package]\nname = \"deep\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let deep = dir.path().join("src").join("lib").join("utils");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("helper.hud"), "fn help() {}").unwrap();
    std::fs::write(dir.path().join("main.hud"), "print(1)").unwrap();

    let builder = PackageBuilder::new(dir.path()).unwrap();
    let result = builder.build().unwrap();
    assert_eq!(result.source_file_count, 2);
}

#[test]
fn test_build_result_debug() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("hudhud.toml"),
        "[package]\nname = \"debug-test\"\nversion = \"0.1.0\"",
    )
    .unwrap();
    std::fs::write(dir.path().join("main.hud"), "print(1)").unwrap();

    let builder = PackageBuilder::new(dir.path()).unwrap();
    let result = builder.build().unwrap();
    let debug = format!("{:?}", result);
    assert!(debug.contains("archive_path"));
    assert!(debug.contains("source_file_count"));
    assert!(debug.contains("total_size"));
}
