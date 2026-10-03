//! Tests for hudhud-fs patch.apply — SEARCH/REPLACE diff applier (moved from src/patch.rs).

use std::fs;

use hudhud_fs::patch::patch_apply;

fn setup(path: &str, content: &str) {
    fs::write(path, content).unwrap();
}

fn cleanup(path: &str) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(format!("{}.hudhud_patch_tmp", path));
}

#[test]
fn exact_match_single_replacement() {
    let p = "/tmp/test_patch_exact.txt";
    setup(p, "hello world\nfoo bar\n");
    let r = patch_apply(p, "hello world", "HELLO WORLD").unwrap();
    assert!(r.modified);
    assert_eq!(r.replacements, 1);
    assert_eq!(fs::read_to_string(p).unwrap(), "HELLO WORLD\nfoo bar\n");
    cleanup(p);
}

#[test]
fn no_match_returns_error() {
    let p = "/tmp/test_patch_nomatch.txt";
    setup(p, "hello world\n");
    let r = patch_apply(p, "nonexistent", "x");
    assert!(r.is_err());
    cleanup(p);
}

#[test]
fn multi_match_returns_error() {
    let p = "/tmp/test_patch_multi.txt";
    setup(p, "foo\nfoo\n");
    let r = patch_apply(p, "foo", "bar");
    assert!(r.is_err());
    assert!(r.unwrap_err().contains("matches 2 times"));
    cleanup(p);
}

#[test]
fn no_change_when_search_equals_replace() {
    let p = "/tmp/test_patch_nochange.txt";
    setup(p, "hello world\n");
    let r = patch_apply(p, "hello world", "hello world").unwrap();
    assert!(!r.modified);
    assert_eq!(r.replacements, 1);
    cleanup(p);
}

#[test]
fn file_not_found() {
    let r = patch_apply("/tmp/no_such_file_xyz.txt", "a", "b");
    assert!(r.is_err());
}
