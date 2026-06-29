//! Tests extracted from hudhudscript-sandbox/src/landlock.rs

use hudhudscript_sandbox::landlock::LandlockRuleset;
use std::path::PathBuf;

#[test]
fn test_empty_ruleset() {
    let rs = LandlockRuleset::new();
    assert!(rs.read_paths().is_empty());
    assert!(rs.write_paths().is_empty());
    assert!(rs.exec_paths().is_empty());
}

#[test]
fn test_add_paths() {
    let mut rs = LandlockRuleset::new();
    rs.add_read_path("/data");
    rs.add_write_path("/tmp");
    rs.add_exec_path("/usr/bin");

    assert_eq!(rs.read_paths(), &[PathBuf::from("/data")]);
    assert_eq!(rs.write_paths(), &[PathBuf::from("/tmp")]);
    assert_eq!(rs.exec_paths(), &[PathBuf::from("/usr/bin")]);
}

#[test]
fn test_check_read_allowed() {
    let mut rs = LandlockRuleset::new();
    rs.add_read_path("/data");

    assert!(rs.check_read("/data/file.txt"));
    assert!(rs.check_read("/data/sub/dir/file"));
    assert!(!rs.check_read("/etc/passwd"));
}

#[test]
fn test_check_write_allowed() {
    let mut rs = LandlockRuleset::new();
    rs.add_write_path("/tmp");

    assert!(rs.check_write("/tmp/output.log"));
    assert!(!rs.check_write("/data/output.log"));
}

#[test]
fn test_check_exec_allowed() {
    let mut rs = LandlockRuleset::new();
    rs.add_exec_path("/usr/bin");

    assert!(rs.check_exec("/usr/bin/python"));
    assert!(!rs.check_exec("/home/user/evil"));
}

#[test]
    #[ignore = "process-global privileged syscall; unsafe in parallel test. Run: --ignored --test-threads=1"]
fn test_apply_succeeds() {
    let rs = LandlockRuleset::new();
    assert!(rs.apply().is_ok());
}

#[test]
fn test_is_supported() {
    // On CI / non-Linux this will be false — that is correct.
    let _ = LandlockRuleset::is_supported();
}

#[test]
fn test_read_only_preset() {
    let rs = LandlockRuleset::read_only(&["/data", "/config"]);
    assert!(rs.check_read("/data/file"));
    assert!(rs.check_read("/config/app.toml"));
    assert!(!rs.check_write("/data/file"));
}

#[test]
fn test_workspace_preset() {
    let rs = LandlockRuleset::workspace("/workspace");
    assert!(rs.check_read("/workspace/src/main.rs"));
    assert!(rs.check_write("/workspace/target/output"));
    assert!(rs.check_read("/usr/lib/libc.so"));
    assert!(rs.check_exec("/usr/bin/python"));
    assert!(!rs.check_write("/etc/passwd"));
}
