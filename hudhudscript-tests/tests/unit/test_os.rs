//! Tests for hudhud-os — public API (non-destructive).
use hudhud_os::os_ops;
use hudhudscript_bytecode::Value16;

#[test]
fn test_os_name_returns_string() {
    let r = os_ops::name(&[]);
    assert!(r.is_ok());
    assert!(r.unwrap().as_string().is_some());
}

#[test]
fn test_os_arch_returns_string() {
    let r = os_ops::arch(&[]);
    assert!(r.is_ok());
}

#[test]
fn test_os_version_returns_string() {
    let r = os_ops::version(&[]);
    assert!(r.is_ok());
}

#[test]
fn test_os_hostname_returns_string() {
    let r = os_ops::hostname(&[]);
    assert!(r.is_ok());
}

#[test]
fn test_os_username_returns_string() {
    let r = os_ops::username(&[]);
    assert!(r.is_ok());
}

#[test]
fn test_os_homedir_returns_string() {
    let r = os_ops::homedir(&[]);
    assert!(r.is_ok());
    assert!(r.unwrap().as_string().is_some());
}

#[test]
fn test_os_tmpdir_returns_string() {
    let r = os_ops::tmpdir(&[]);
    assert!(r.is_ok());
}

#[test]
fn test_os_cpus_returns_number() {
    let r = os_ops::cpus(&[]);
    assert!(r.is_ok());
    let n = r.unwrap().as_number().unwrap();
    assert!(n >= 1.0, "cpus should be >= 1, got {n}");
}
