//! Runtime host-access policy for the VM.
//!
//! This module defines `HostAccessPolicy`, the in-VM representation of the
//! user-facing `[host_access]` config. It provides helper predicates that
//! each builtin dispatch arm calls before performing a host-side operation.
//!
//! Design goals:
//! - Keep TOML/config types out of the VM crate (they live in CLI).
//! - Default policy is **permissive** so existing CLI/scripts keep working
//!   when no `[host_access]` section is configured.
//! - Every denial returns a structured runtime error, never a panic or silent
//!   null.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use std::collections::HashSet;

/// Coarse access decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDecision {
    Allow,
    Deny,
}

impl Default for AccessDecision {
    fn default() -> Self {
        AccessDecision::Deny
    }
}

impl AccessDecision {
    pub fn is_allowed(self) -> bool {
        matches!(self, AccessDecision::Allow)
    }
}

/// Module-specific access decisions.
#[derive(Debug, Clone, Default)]
pub struct ModulePolicy {
    pub default: AccessDecision,
    pub http: Option<AccessDecision>,
    pub tcp: Option<AccessDecision>,
    pub udp: Option<AccessDecision>,
    pub unix: Option<AccessDecision>,
    pub fs: Option<AccessDecision>,
    pub process: Option<AccessDecision>,
    pub database: Option<AccessDecision>,
    pub dbus: Option<AccessDecision>,
    pub tts: Option<AccessDecision>,
}

impl ModulePolicy {
    fn decision_for(&self, name: &str) -> AccessDecision {
        let explicit = match name {
            "http" => self.http,
            "tcp" => self.tcp,
            "udp" => self.udp,
            "unix" => self.unix,
            "fs" => self.fs,
            "process" | "exec" => self.process,
            "database" => self.database,
            "dbus" => self.dbus,
            "tts" => self.tts,
            _ => None,
        };
        explicit.unwrap_or(self.default)
    }
}

/// Environment variable access rules.
#[derive(Debug, Clone, Default)]
pub struct EnvPolicy {
    pub default: AccessDecision,
    pub allow: HashSet<String>,
    pub deny: HashSet<String>,
}

impl EnvPolicy {
    fn read_allowed(&self, key: &str) -> bool {
        match self.default {
            AccessDecision::Allow => !self.deny.contains(key),
            AccessDecision::Deny => self.allow.contains(key),
        }
    }

    fn write_allowed(&self, key: &str) -> bool {
        // By default env writes are never permitted from scripts unless
        // explicitly allowed. This prevents accidental pollution of the
        // parent process environment.
        self.allow.contains(key)
    }

    fn all_allowed(&self) -> bool {
        self.default.is_allowed() && self.deny.is_empty()
    }

    fn all_unfiltered_allowed(&self) -> bool {
        // Reading the full environment without filtering is a higher risk
        // than reading a single key; require explicit allow-all policy.
        self.default.is_allowed() && self.deny.is_empty() && self.allow.is_empty()
    }
}

/// Child process / command execution rules.
#[derive(Debug, Clone, Default)]
pub struct ExecPolicy {
    pub default: AccessDecision,
    pub allow: HashSet<String>,
    pub deny: HashSet<String>,
    pub max_processes: usize,
}

impl ExecPolicy {
    fn method_allowed(&self, method: &str) -> bool {
        match self.default {
            AccessDecision::Allow => !self.deny.contains(method),
            AccessDecision::Deny => self.allow.contains(method),
        }
    }

    fn command_allowed(&self, command: &str) -> bool {
        // command may be a path; normalize to basename.
        let base = std::path::Path::new(command)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(command);
        match self.default {
            AccessDecision::Allow => !self.deny.contains(base),
            AccessDecision::Deny => self.allow.contains(base),
        }
    }
}

/// Runtime host-access policy attached to a VM instance.
#[derive(Debug, Clone, Default)]
pub struct HostAccessPolicy {
    pub default: AccessDecision,
    pub env: EnvPolicy,
    pub exec: ExecPolicy,
    pub modules: ModulePolicy,
}

impl HostAccessPolicy {
    /// Permissive policy: matches legacy CLI behaviour where no `[host_access]`
    /// section is configured.
    pub fn permissive() -> Self {
        Self {
            default: AccessDecision::Allow,
            env: EnvPolicy {
                default: AccessDecision::Allow,
                allow: HashSet::new(),
                deny: HashSet::new(),
            },
            exec: ExecPolicy {
                default: AccessDecision::Allow,
                allow: HashSet::new(),
                deny: HashSet::new(),
                max_processes: usize::MAX,
            },
            modules: ModulePolicy {
                default: AccessDecision::Allow,
                http: None,
                tcp: None,
                udp: None,
                unix: None,
                fs: None,
                process: None,
                // Durable database mutation is opt-in even for the legacy
                // permissive profile.
                database: Some(AccessDecision::Deny),
                dbus: None,
                tts: None,
            },
        }
    }

    /// Restrictive policy: deny everything by default.
    pub fn restrictive() -> Self {
        Self {
            default: AccessDecision::Deny,
            env: EnvPolicy {
                default: AccessDecision::Deny,
                allow: HashSet::new(),
                deny: HashSet::new(),
            },
            exec: ExecPolicy {
                default: AccessDecision::Deny,
                allow: HashSet::new(),
                deny: HashSet::new(),
                max_processes: 0,
            },
            modules: ModulePolicy {
                default: AccessDecision::Deny,
                http: None,
                tcp: None,
                udp: None,
                unix: None,
                fs: None,
                process: None,
                database: None,
                dbus: None,
                tts: None,
            },
        }
    }

    // ── Module helpers ───────────────────────────────────────────────

    pub fn ensure_module_allowed(&self, module: &str) -> SharedResult<()> {
        let decision = self.modules.decision_for(module);
        if decision.is_allowed() {
            Ok(())
        } else {
            Err(runtime_error(format!(
                "Host access denied: module '{}' is not allowed by host_access policy",
                module
            )))
        }
    }

    // ── Env helpers ────────────────────────────────────────────────

    pub fn ensure_env_read(&self, key: &str) -> SharedResult<()> {
        if self.env.read_allowed(key) {
            Ok(())
        } else {
            Err(runtime_error(format!(
                "Host access denied: env('{}') is not allowed by host_access policy",
                key
            )))
        }
    }

    pub fn ensure_env_write(&self, key: &str) -> SharedResult<()> {
        if self.env.write_allowed(key) {
            Ok(())
        } else {
            Err(runtime_error(format!(
                "Host access denied: Env.set('{}') is not allowed by host_access policy",
                key
            )))
        }
    }

    pub fn ensure_env_remove(&self, key: &str) -> SharedResult<()> {
        // remove is a write-like operation.
        self.ensure_env_write(key)
    }

    pub fn ensure_env_all(&self) -> SharedResult<()> {
        if self.env.all_allowed() {
            Ok(())
        } else {
            Err(runtime_error(
                "Host access denied: Env.all() is not allowed by host_access policy".to_string(),
            ))
        }
    }

    pub fn ensure_env_all_unfiltered(&self) -> SharedResult<()> {
        if self.env.all_unfiltered_allowed() {
            Ok(())
        } else {
            Err(runtime_error(
                "Host access denied: Env.all_unfiltered() is not allowed by host_access policy"
                    .to_string(),
            ))
        }
    }

    // ── Exec helpers ─────────────────────────────────────────────────

    pub fn ensure_exec_method(&self, method: &str) -> SharedResult<()> {
        if self.exec.method_allowed(method) {
            Ok(())
        } else {
            Err(runtime_error(format!(
                "Host access denied: exec.{}() is not allowed by host_access policy",
                method
            )))
        }
    }

    pub fn ensure_command_allowed(&self, command: &str) -> SharedResult<()> {
        if self.exec.command_allowed(command) {
            Ok(())
        } else {
            Err(runtime_error(format!(
                "Host access denied: command '{}' is not allowed by host_access policy",
                command
            )))
        }
    }

    pub fn ensure_process_spawn_allowed(&self) -> SharedResult<()> {
        // process module permission is checked separately via ensure_module_allowed.
        // max_processes is tracked by the VM runtime, not this policy object.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
