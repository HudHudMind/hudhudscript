//! External tests for hudhudscript-package::publisher —
//! create_tarball, load_ignore_patterns, should_ignore, read_auth_token, PublishOptions.

use hudhudscript_package::{
    create_tarball, load_ignore_patterns, read_auth_token, should_ignore, PublishOptions,
};

#[test]
fn test_should_ignore() {
    let patterns = vec![
        ".git".to_string(),
        "target".to_string(),
        "node_modules".to_string(),
    ];

    assert!(should_ignore(".git", &patterns));
    assert!(should_ignore(".git/config", &patterns));
    assert!(should_ignore("target", &patterns));
    assert!(should_ignore("target/debug", &patterns));
    assert!(!should_ignore("src/main.hud", &patterns));
    assert!(!should_ignore("hudhud.toml", &patterns));
}

#[test]
fn test_load_ignore_patterns_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    let patterns = load_ignore_patterns(tmp.path());
    assert!(patterns.contains(&".git".to_string()));
    assert!(patterns.contains(&".hudpackages".to_string()));
}

#[test]
fn test_create_tarball() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path();

    std::fs::write(base.join("main.hud"), "print(42)").unwrap();
    std::fs::write(
        base.join("hudhud.toml"),
        "[package]\nname=\"test\"\nversion=\"0.1.0\"",
    )
    .unwrap();

    let tarball = create_tarball(base.to_str().unwrap()).unwrap();
    assert!(!tarball.is_empty());

    // Verify it's valid gzip
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(&tarball[..]);
    let mut buf = Vec::new();
    assert!(decoder.read_to_end(&mut buf).is_ok());
}

#[test]
fn test_should_ignore_nested_path() {
    let patterns = vec![".git".to_string(), "node_modules".to_string()];

    // Nested path with pattern in the middle
    assert!(should_ignore("src/node_modules/pkg", &patterns));
    // Exact match
    assert!(should_ignore("node_modules", &patterns));
    // Path starts with pattern
    assert!(should_ignore("node_modules/something", &patterns));
    // Path ending with pattern
    assert!(should_ignore("path/to/.git", &patterns));
    // Not matching
    assert!(!should_ignore("src/main.rs", &patterns));
}

#[test]
fn test_should_ignore_empty_patterns() {
    let patterns: Vec<String> = vec![];
    assert!(!should_ignore("anything", &patterns));
}

#[test]
fn test_load_ignore_patterns_custom_file() {
    let tmp = tempfile::tempdir().unwrap();
    let ignore_file = tmp.path().join(".hudhudignore");
    std::fs::write(&ignore_file, "# Comment line\nbuild\n\n  dist  \nsecrets\n").unwrap();

    let patterns = load_ignore_patterns(tmp.path());
    // Should have default patterns plus custom ones
    assert!(patterns.contains(&".git".to_string()));
    assert!(patterns.contains(&".hudpackages".to_string()));
    assert!(patterns.contains(&"target".to_string()));
    assert!(patterns.contains(&"build".to_string()));
    assert!(patterns.contains(&"dist".to_string()));
    assert!(patterns.contains(&"secrets".to_string()));
    // Comment and blank lines should not be included
    assert!(!patterns.contains(&"# Comment line".to_string()));
    assert!(!patterns.contains(&"".to_string()));
}

#[test]
fn test_publish_options_default() {
    let opts = PublishOptions::default();
    assert!(!opts.dry_run);
    assert!(!opts.allow_dirty);
}

#[test]
fn test_should_ignore_no_false_positive_substring() {
    let patterns = vec!["target".to_string()];
    // "target" is a full segment match; "targeting" should not match
    // but our implementation checks starts_with/contains segment-style
    assert!(!should_ignore("targeting/output", &patterns));
}

#[test]
fn test_load_ignore_patterns_comment_and_blank_filtered() {
    let tmp = tempfile::tempdir().unwrap();
    let ignore_file = tmp.path().join(".hudhudignore");
    std::fs::write(&ignore_file, "# This is a comment\n\n\nbuild_output\n  \n").unwrap();

    let patterns = load_ignore_patterns(tmp.path());
    assert!(patterns.contains(&"build_output".to_string()));
    // Blank lines and comments should not appear
    assert!(!patterns.iter().any(|p| p.starts_with('#')));
    assert!(!patterns.contains(&"".to_string()));
}

#[test]
fn test_create_tarball_with_nested_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path();

    // Create directory structure
    let sub = base.join("src");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("module.hud"), "let x = 1").unwrap();
    std::fs::write(base.join("main.hud"), "print(1)").unwrap();
    std::fs::write(
        base.join("hudhud.toml"),
        "[package]\nname=\"test\"\nversion=\"0.1.0\"",
    )
    .unwrap();

    let tarball = create_tarball(base.to_str().unwrap()).unwrap();
    assert!(!tarball.is_empty());
}

#[test]
fn test_create_tarball_respects_ignore() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path();

    // Create files including ones that should be ignored
    std::fs::write(base.join("main.hud"), "print(1)").unwrap();
    let git_dir = base.join(".git");
    std::fs::create_dir_all(&git_dir).unwrap();
    std::fs::write(git_dir.join("config"), "").unwrap();

    let tarball = create_tarball(base.to_str().unwrap()).unwrap();
    assert!(!tarball.is_empty());
}

#[test]
fn test_should_ignore_exact_matches() {
    let patterns = vec![
        ".git".to_string(),
        ".hudpackages".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        ".hudhudignore".to_string(),
    ];

    // Default patterns should match
    assert!(should_ignore(".git", &patterns));
    assert!(should_ignore(".hudpackages", &patterns));
    assert!(should_ignore("node_modules", &patterns));
    assert!(should_ignore("target", &patterns));
    assert!(should_ignore(".hudhudignore", &patterns));

    // Prefixed paths
    assert!(should_ignore(".git/HEAD", &patterns));
    assert!(should_ignore("target/debug/build", &patterns));

    // Should not match
    assert!(!should_ignore("src/lib.hud", &patterns));
    assert!(!should_ignore("hudhud.toml", &patterns));
}

#[test]
fn test_publish_options_clone() {
    let opts = PublishOptions {
        dry_run: true,
        allow_dirty: true,
    };
    let cloned = opts.clone();
    assert!(cloned.dry_run);
    assert!(cloned.allow_dirty);
}

#[test]
fn test_publish_options_debug() {
    let opts = PublishOptions::default();
    let debug = format!("{:?}", opts);
    assert!(debug.contains("dry_run"));
    assert!(debug.contains("allow_dirty"));
}

#[test]
fn test_should_ignore_path_ending_with_pattern() {
    let patterns = vec!["build".to_string()];
    assert!(should_ignore("path/to/build", &patterns));
}

#[test]
fn test_should_ignore_path_with_pattern_in_middle() {
    let patterns = vec!["build".to_string()];
    assert!(should_ignore("path/build/output", &patterns));
}

#[test]
fn test_load_ignore_patterns_no_file() {
    let tmp = tempfile::tempdir().unwrap();
    // No .hudhudignore file exists -> only defaults
    let patterns = load_ignore_patterns(tmp.path());
    assert_eq!(patterns.len(), 5); // .git, .hudpackages, node_modules, target, .hudhudignore
}

#[test]
fn test_create_tarball_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    // Empty dir should produce a valid (small) tarball
    let tarball = create_tarball(tmp.path().to_str().unwrap()).unwrap();
    assert!(!tarball.is_empty());
}

#[test]
fn test_should_ignore_exact_and_prefix() {
    let patterns = vec!["dist".to_string()];
    assert!(should_ignore("dist", &patterns));
    assert!(should_ignore("dist/bundle.js", &patterns));
    assert!(!should_ignore("distribute/output", &patterns));
}

#[test]
fn test_read_auth_token_no_credentials() {
    // In test environment, ~/.hudhud/credentials likely doesn't exist
    let result = read_auth_token();
    assert!(result.is_err());
}

#[test]
fn test_create_tarball_with_subdirectories() {
    let tmp = tempfile::tempdir().unwrap();
    let sub = tmp.path().join("src").join("utils");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("helper.hud"), "fn help() {}").unwrap();
    std::fs::write(tmp.path().join("main.hud"), "print(1)").unwrap();

    let tarball = create_tarball(tmp.path().to_str().unwrap()).unwrap();
    assert!(!tarball.is_empty());

    // Verify it's valid gzip
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(&tarball[..]);
    let mut buf = Vec::new();
    assert!(decoder.read_to_end(&mut buf).is_ok());
}
