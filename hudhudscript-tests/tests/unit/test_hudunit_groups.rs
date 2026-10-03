//! Tests for hudhudscript-hudunit groups — path-derived groups, annotation
//! merging, include/exclude selection semantics.

use std::path::PathBuf;

use hudhudscript_hudunit::discover::{DiscoveredTest, TestFile};
use hudhudscript_hudunit::groups::{groups_of, selected};

// ── helpers ───────────────────────────────────────────────────────

fn file(chain: &[&str], stem: &str) -> TestFile {
    TestFile {
        path: PathBuf::from(format!("/t/{}/{}.hud", chain.join("/"), stem)),
        group_path: chain.iter().map(|s| s.to_string()).collect(),
        tests: vec![],
        has_setup: false,
        has_teardown: false,
    }
}

fn test(annotations: &[&str]) -> DiscoveredTest {
    DiscoveredTest {
        name: "test_x".into(),
        skip: false,
        annotations: annotations.iter().map(|s| s.to_string()).collect(),
        line: 1,
    }
}

// ── groups_of ─────────────────────────────────────────────────────

#[test]
fn root_file_groups_fall_back_to_stem() {
    let f = file(&[], "test_math");
    let g = groups_of(&f, &test(&[]));
    assert_eq!(g, vec!["test_math".to_string()]);
}

#[test]
fn annotations_merge_with_path_groups() {
    let f = file(&["math"], "test_arithmetic");
    let g = groups_of(&f, &test(&["hizli"]));
    assert_eq!(g, vec!["math".to_string(), "hizli".to_string()]);
}

// ── selection ─────────────────────────────────────────────────────

#[test]
fn include_selects_matching_group() {
    let f = file(&["math"], "t");
    assert!(selected(&f, &test(&[]), &["math".into()], &[]));
    assert!(!selected(&f, &test(&[]), &["string".into()], &[]));
}

#[test]
fn nested_chain_prefix_matches() {
    let f = file(&["math", "arithmetic"], "t");
    assert!(selected(&f, &test(&[]), &["math".into()], &[]));
}

#[test]
fn exclude_overrides_include() {
    let f = file(&["math"], "t");
    assert!(!selected(
        &f,
        &test(&["yavas"]),
        &["math".into()],
        &["yavas".into()]
    ));
}

#[test]
fn empty_include_selects_all() {
    let f = file(&["string"], "t");
    assert!(selected(&f, &test(&[]), &[], &[]));
}
