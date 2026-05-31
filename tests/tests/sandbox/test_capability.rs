//! Tests extracted from hudhudscript-sandbox/src/capability.rs

use hudhudscript_sandbox::capability::{effective_capabilities, Capability, CapabilitySet};
use std::collections::HashSet;

#[test]
fn test_empty_set() {
    let set = CapabilitySet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn test_add_remove() {
    let mut set = CapabilitySet::new();
    set.add(Capability::NetAdmin);
    assert!(set.contains(Capability::NetAdmin));
    assert_eq!(set.len(), 1);

    set.remove(Capability::NetAdmin);
    assert!(!set.contains(Capability::NetAdmin));
    assert!(set.is_empty());
}

#[test]
fn test_full_set() {
    let set = CapabilitySet::full();
    assert_eq!(set.len(), Capability::all().len());
    assert!(set.contains(Capability::SysAdmin));
    assert!(set.contains(Capability::NetRaw));
}

#[test]
fn test_drop_all() {
    let mut set = CapabilitySet::full();
    set.drop_all();
    assert!(set.is_empty());
}

#[test]
fn test_retain_only() {
    let mut set = CapabilitySet::full();
    let keep: HashSet<Capability> = [Capability::NetBindService, Capability::Chown]
        .iter()
        .copied()
        .collect();
    set.retain_only(&keep);
    assert_eq!(set.len(), 2);
    assert!(set.contains(Capability::NetBindService));
    assert!(set.contains(Capability::Chown));
    assert!(!set.contains(Capability::SysAdmin));
}

#[test]
fn test_names() {
    let mut set = CapabilitySet::new();
    set.add(Capability::NetAdmin);
    set.add(Capability::SysAdmin);
    let names = set.names();
    assert!(names.contains(&"CAP_NET_ADMIN"));
    assert!(names.contains(&"CAP_SYS_ADMIN"));
}

#[test]
fn test_capability_name() {
    assert_eq!(Capability::SysAdmin.name(), "CAP_SYS_ADMIN");
    assert_eq!(Capability::NetRaw.name(), "CAP_NET_RAW");
}

#[test]
fn test_all_capabilities() {
    let all = Capability::all();
    assert!(all.len() >= 20);
}

#[test]
fn test_effective_capabilities() {
    // v0.4.47.9: effective_capabilities now returns Result.
    // On Linux it should succeed; on other platforms it returns an error.
    let result = effective_capabilities();
    #[cfg(target_os = "linux")]
    {
        assert!(result.is_ok(), "Linux should return Ok: {:?}", result);
    }
    #[cfg(not(target_os = "linux"))]
    {
        assert!(result.is_err(), "Non-Linux should return Err");
    }
}

#[test]
fn test_apply_succeeds() {
    let set = CapabilitySet::new();
    assert!(set.apply().is_ok());
}

#[test]
fn test_to_sorted_vec_deterministic() {
    let mut set = CapabilitySet::new();
    set.add(Capability::SysAdmin);
    set.add(Capability::Chown);
    set.add(Capability::NetRaw);
    let sorted = set.to_sorted_vec();
    let names: Vec<&str> = sorted.iter().map(|c| c.name()).collect();
    // Alphabetical by CAP_ name
    assert_eq!(names, vec!["CAP_CHOWN", "CAP_NET_RAW", "CAP_SYS_ADMIN"]);
}
