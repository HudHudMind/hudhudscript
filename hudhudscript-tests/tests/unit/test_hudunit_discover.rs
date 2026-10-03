//! Tests for hudhudscript-hudunit discover — `@group` annotation parsing,
//! path-derived group chains, extension filtering.

use std::path::Path;

use hudhudscript_hudunit::config::HudunitConfig;
use hudhudscript_hudunit::discover::{collect_group_annotations, derive_group_path};

// ── annotations ───────────────────────────────────────────────────

#[test]
fn group_annotations_attach_to_following_line() {
    let src = "// @group hizli\n// @group math\nfn test_a() {}\n\nfn test_b() {}\n";
    let groups = collect_group_annotations(src);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].0, 3);
    assert_eq!(groups[0].1, vec!["hizli".to_string(), "math".to_string()]);
}

#[test]
fn blank_line_resets_pending_groups() {
    let src = "// @group x\n\nfn test_a() {}\n";
    assert!(collect_group_annotations(src).is_empty());
}

#[test]
fn slash_comment_groups_supported() {
    let src = "// @group slow\nfn test_a() {}\n";
    let groups = collect_group_annotations(src);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].1, vec!["slow".to_string()]);
}

// ── group paths ───────────────────────────────────────────────────

#[test]
fn derive_group_path_nested() {
    let path = Path::new("/tmp/tests/math/test_a.hud");
    let chain = derive_group_path(path, Some(Path::new("/tmp/tests")));
    assert_eq!(chain, vec!["math".to_string()]);
}

#[test]
fn derive_group_path_root_file() {
    let path = Path::new("/tmp/tests/test_a.hud");
    let chain = derive_group_path(path, Some(Path::new("/tmp/tests")));
    assert!(chain.is_empty());
}

// ── extension filter ──────────────────────────────────────────────

#[test]
fn extension_filter() {
    let cfg = HudunitConfig::default();
    assert!(cfg.is_test_file(Path::new("a.hhs")));
    assert!(cfg.is_test_file(Path::new("a.hud")));
    assert!(cfg.is_test_file(Path::new("a.hudhud")));
    assert!(!cfg.is_test_file(Path::new("a.rs")));
}
