//! VM host-access policy tests: permissive/restrictive defaults, env
//! allow/deny lists, command basename checks and per-module overrides.

use hudhudscript_vm::vm::host_access::{AccessDecision, HostAccessPolicy};

#[test]
fn permissive_keeps_database_opt_in() {
    let p = HostAccessPolicy::permissive();
    assert!(p.ensure_module_allowed("http").is_ok());
    assert!(p.ensure_env_read("ANY").is_ok());
    assert!(p.ensure_exec_method("run").is_ok());
    assert!(p.ensure_command_allowed("/bin/ls").is_ok());
    assert!(p.ensure_module_allowed("database").is_err());
}

#[test]
fn restrictive_denies_by_default() {
    let p = HostAccessPolicy::restrictive();
    assert!(p.ensure_module_allowed("http").is_err());
    assert!(p.ensure_env_read("ANY").is_err());
    assert!(p.ensure_exec_method("run").is_err());
    assert!(p.ensure_command_allowed("/bin/ls").is_err());
}

#[test]
fn env_whitelist_on_deny_default() {
    let mut p = HostAccessPolicy::restrictive();
    p.env.allow.insert("ALLOWED".to_string());
    assert!(p.ensure_env_read("ALLOWED").is_ok());
    assert!(p.ensure_env_read("OTHER").is_err());
}

#[test]
fn env_blacklist_on_allow_default() {
    let mut p = HostAccessPolicy::permissive();
    p.env.deny.insert("SECRET".to_string());
    assert!(p.ensure_env_read("OTHER").is_ok());
    assert!(p.ensure_env_read("SECRET").is_err());
}

#[test]
fn command_basename_check() {
    let mut p = HostAccessPolicy::restrictive();
    p.exec.allow.insert("python".to_string());
    assert!(p.ensure_command_allowed("/usr/bin/python").is_ok());
    assert!(p.ensure_command_allowed("python").is_ok());
    assert!(p.ensure_command_allowed("/bin/bash").is_err());
}

#[test]
fn module_per_module_override() {
    let mut p = HostAccessPolicy::restrictive();
    p.modules.http = Some(AccessDecision::Allow);
    assert!(p.ensure_module_allowed("http").is_ok());
    assert!(p.ensure_module_allowed("tcp").is_err());
}
