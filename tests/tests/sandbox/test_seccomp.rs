//! Tests extracted from hudhudscript-sandbox/src/seccomp.rs

use hudhudscript_sandbox::seccomp::{SeccompFilter, SeccompPolicy, SeccompProfile};

#[test]
fn test_filter_default_deny() {
    let filter = SeccompFilter::new();
    assert_eq!(filter.get_default_policy(), SeccompPolicy::Deny);
    assert_eq!(filter.effective_policy(999), SeccompPolicy::Deny);
}

#[test]
fn test_filter_allow_syscall() {
    let mut filter = SeccompFilter::new();
    filter.allow_syscall(0); // read
    assert_eq!(filter.effective_policy(0), SeccompPolicy::Allow);
    assert_eq!(filter.effective_policy(1), SeccompPolicy::Deny);
}

#[test]
fn test_filter_deny_syscall() {
    let mut filter = SeccompFilter::new();
    filter.default_policy(SeccompPolicy::Allow);
    filter.deny_syscall(165); // mount
    assert_eq!(filter.effective_policy(165), SeccompPolicy::Deny);
    assert_eq!(filter.effective_policy(0), SeccompPolicy::Allow);
}

#[test]
fn test_filter_log_syscall() {
    let mut filter = SeccompFilter::new();
    filter.log_syscall(42);
    assert_eq!(filter.effective_policy(42), SeccompPolicy::Log);
}

#[test]
fn test_last_rule_wins() {
    let mut filter = SeccompFilter::new();
    filter.allow_syscall(0);
    filter.deny_syscall(0);
    assert_eq!(filter.effective_policy(0), SeccompPolicy::Deny);
}

#[test]
fn test_apply_succeeds() {
    let filter = SeccompFilter::new();
    assert!(filter.apply().is_ok());
}

#[test]
fn test_minimal_profile() {
    let profile = SeccompProfile::minimal();
    assert_eq!(profile.name(), "minimal");
    assert_eq!(profile.filter().get_default_policy(), SeccompPolicy::Deny);
    // read (0) should be allowed
    assert_eq!(profile.filter().effective_policy(0), SeccompPolicy::Allow);
    // mount (165) should fall through to deny
    assert_eq!(profile.filter().effective_policy(165), SeccompPolicy::Deny);
}

#[test]
fn test_standard_profile() {
    let profile = SeccompProfile::standard();
    assert_eq!(profile.name(), "standard");
    assert_eq!(profile.filter().get_default_policy(), SeccompPolicy::Allow);
    // mount (165) explicitly denied
    assert_eq!(profile.filter().effective_policy(165), SeccompPolicy::Deny);
    // read (0) allowed by default
    assert_eq!(profile.filter().effective_policy(0), SeccompPolicy::Allow);
}

#[test]
fn test_permissive_profile() {
    let profile = SeccompProfile::permissive();
    assert_eq!(profile.name(), "permissive");
    // reboot (169) denied even in permissive
    assert_eq!(profile.filter().effective_policy(169), SeccompPolicy::Deny);
    // everything else allowed
    assert_eq!(profile.filter().effective_policy(0), SeccompPolicy::Allow);
}

#[test]
fn test_profile_apply() {
    let profile = SeccompProfile::minimal();
    assert!(profile.apply().is_ok());
}

#[test]
fn test_allowed_denied_sets() {
    let profile = SeccompProfile::minimal();
    let allowed = profile.allowed_syscalls();
    let denied = profile.denied_syscalls();
    assert!(allowed.contains(&0)); // read
    assert!(!denied.contains(&0));
}

#[test]
fn test_filter_default_impl() {
    let filter = SeccompFilter::default();
    assert_eq!(filter.get_default_policy(), SeccompPolicy::Deny);
    assert!(filter.rules().is_empty());
}

#[test]
fn test_profile_filter_mut() {
    let mut profile = SeccompProfile::minimal();
    profile.filter_mut().deny_syscall(999);
    assert_eq!(profile.filter().effective_policy(999), SeccompPolicy::Deny);
}

#[test]
fn test_standard_denied_syscalls() {
    let profile = SeccompProfile::standard();
    let denied = profile.denied_syscalls();
    assert!(denied.contains(&165)); // mount
    assert!(denied.contains(&169)); // reboot
    assert!(denied.contains(&246)); // kexec_load
}

#[test]
fn test_permissive_denied_syscalls() {
    let profile = SeccompProfile::permissive();
    let denied = profile.denied_syscalls();
    assert!(denied.contains(&169)); // reboot
    assert!(denied.contains(&246)); // kexec_load
    assert_eq!(denied.len(), 2);
}

#[test]
fn test_filter_rules_count() {
    let mut filter = SeccompFilter::new();
    filter.allow_syscall(0);
    filter.deny_syscall(1);
    filter.log_syscall(2);
    assert_eq!(filter.rules().len(), 3);
}

#[test]
fn test_custom_profile() {
    let mut filter = SeccompFilter::new();
    filter.default_policy(SeccompPolicy::Allow);
    filter.deny_syscall(100);
    let profile = SeccompProfile::custom("my_profile", filter);
    assert_eq!(profile.name(), "my_profile");
    assert_eq!(profile.filter().effective_policy(100), SeccompPolicy::Deny);
}
