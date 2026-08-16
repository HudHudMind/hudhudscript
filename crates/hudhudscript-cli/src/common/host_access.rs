//! Host access policy configuration for `[host_access]` in `hudhud.toml`.
//!
//! This module defines the user-facing TOML schema that controls access to
//! host resources from HudHudScript code: environment variables, child
//! process execution, and built-in modules that touch the network, filesystem,
//! or system interfaces.
//!
//! The policy is intentionally coarse-grained at the top level. Each subtable
//! (`env`, `exec`, `modules`) can independently decide whether to allow or deny
//! access and can provide allow/deny lists for fine-grained exceptions.
//!
//! # TOML example
//!
//! ```toml
//! [host_access]
//! default = "deny"
//!
//! [host_access.env]
//! default = "deny"
//! allow = ["DEEPSEEK_API_KEY", "OPENAI_API_KEY"]
//! deny = ["PATH", "HOME"]
//!
//! [host_access.exec]
//! default = "deny"
//! allow = ["python", "node"]
//! deny = ["rm", "dd"]
//!
//! [host_access.modules]
//! default = "deny"
//! http = "allow"
//! tcp = "deny"
//! udp = "deny"
//! fs = "allow"
//! ```

use serde::{Deserialize, Serialize};

/// Coarse-grained access decision used across all host_access subtables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessDecision {
    /// Access is permitted unless explicitly denied.
    Allow,
    /// Access is denied unless explicitly allowed.
    Deny,
}

impl Default for AccessDecision {
    fn default() -> Self {
        AccessDecision::Deny
    }
}

impl AccessDecision {
    /// Return true if the decision is `Allow`.
    pub fn is_allowed(&self) -> bool {
        matches!(self, AccessDecision::Allow)
    }

    /// Return true if the decision is `Deny`.
    pub fn is_denied(&self) -> bool {
        matches!(self, AccessDecision::Deny)
    }
}

/// Per-module access configuration.
///
/// Each module (`http`, `tcp`, `udp`, `unix`, `fs`, `process`, `dbus`, `tts`)
/// can be set to `allow` or `deny` explicitly. When a field is `None`, the
/// top-level `HostAccessConfig::default_decision` applies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostModuleAccessConfig {
    /// Default for this subtable when not overridden per module.
    #[serde(default)]
    pub default: Option<AccessDecision>,
    /// HTTP / HTTPS client module (`http::get`, `http::post`, ...).
    #[serde(default)]
    pub http: Option<AccessDecision>,
    /// TCP client/server module.
    #[serde(default)]
    pub tcp: Option<AccessDecision>,
    /// UDP socket module.
    #[serde(default)]
    pub udp: Option<AccessDecision>,
    /// Unix domain socket module.
    #[serde(default)]
    pub unix: Option<AccessDecision>,
    /// Filesystem module (read/write/list).
    #[serde(default)]
    pub fs: Option<AccessDecision>,
    /// Child process / command execution (`exec.run`, ...).
    #[serde(default)]
    pub process: Option<AccessDecision>,
    /// D-Bus system/session bus access.
    #[serde(default)]
    pub dbus: Option<AccessDecision>,
    /// Text-to-speech subsystem.
    #[serde(default)]
    pub tts: Option<AccessDecision>,
}

/// Environment variable access configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostEnvAccessConfig {
    /// Coarse default for env access.
    #[serde(default)]
    pub default: AccessDecision,
    /// Env var names explicitly allowed. On `deny` default this acts as a
    /// whitelist; on `allow` default this is redundant but harmless.
    #[serde(default)]
    pub allow: Vec<String>,
    /// Env var names explicitly denied. On `allow` default this acts as a
    /// blacklist; on `deny` default this is redundant but harmless.
    #[serde(default)]
    pub deny: Vec<String>,
}

/// Child process execution configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostExecAccessConfig {
    /// Coarse default for command execution.
    #[serde(default)]
    pub default: AccessDecision,
    /// Command basenames explicitly allowed.
    #[serde(default)]
    pub allow: Vec<String>,
    /// Command basenames explicitly denied.
    #[serde(default)]
    pub deny: Vec<String>,
    /// Maximum number of concurrently spawned child processes.
    #[serde(default)]
    pub max_processes: usize,
}

/// Top-level `[host_access]` policy.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostAccessConfig {
    /// Coarse default for all host access decisions when a more specific
    /// subtable or module does not override it.
    #[serde(default)]
    pub default: AccessDecision,
    /// Environment variable access rules.
    #[serde(default)]
    pub env: HostEnvAccessConfig,
    /// Child process execution rules.
    #[serde(default)]
    pub exec: HostExecAccessConfig,
    /// Built-in module access rules.
    #[serde(default)]
    pub modules: HostModuleAccessConfig,
}

impl HostAccessConfig {
    /// Create a restrictive (secure-by-default) configuration.
    pub fn restrictive() -> Self {
        Self {
            default: AccessDecision::Deny,
            env: HostEnvAccessConfig {
                default: AccessDecision::Deny,
                allow: Vec::new(),
                deny: Vec::new(),
            },
            exec: HostExecAccessConfig {
                default: AccessDecision::Deny,
                allow: Vec::new(),
                deny: Vec::new(),
                max_processes: 0,
            },
            modules: HostModuleAccessConfig {
                default: Some(AccessDecision::Deny),
                http: None,
                tcp: None,
                udp: None,
                unix: None,
                fs: None,
                process: None,
                dbus: None,
                tts: None,
            },
        }
    }

    /// Create a permissive (development) configuration.
    pub fn permissive() -> Self {
        Self {
            default: AccessDecision::Allow,
            env: HostEnvAccessConfig {
                default: AccessDecision::Allow,
                allow: Vec::new(),
                deny: Vec::new(),
            },
            exec: HostExecAccessConfig {
                default: AccessDecision::Allow,
                allow: Vec::new(),
                deny: Vec::new(),
                max_processes: 100,
            },
            modules: HostModuleAccessConfig {
                default: Some(AccessDecision::Allow),
                http: None,
                tcp: None,
                udp: None,
                unix: None,
                fs: None,
                process: None,
                dbus: None,
                tts: None,
            },
        }
    }

    /// Merge another `HostAccessConfig` over `self`.
    ///
    /// Rules:
    /// - `default` is a scalar: overlay wins.
    /// - `env` and `exec` subtables merge field-by-field:
    ///   - `default` scalar: overlay wins.
    ///   - `allow` and `deny` lists: overlay wins if non-empty, otherwise base keeps.
    ///   - `max_processes`: overlay wins if non-zero, otherwise base keeps.
    /// - `modules` subtable merges per-module:
    ///   - `default` scalar: overlay wins if present.
    ///   - each module decision: overlay wins if present.
    pub fn merge(mut self, overlay: Option<&HostAccessConfig>) -> Self {
        let Some(overlay) = overlay else { return self };

        // Top-level default is scalar: overlay wins.
        self.default = overlay.default;

        // Env subtable merge.
        self.env.default = overlay.env.default;
        // `allow` is a whitelist: last explicit list wins.
        if !overlay.env.allow.is_empty() {
            self.env.allow = overlay.env.allow.clone();
        }
        // `deny` is a blacklist: union of base and overlay.
        for d in &overlay.env.deny {
            if !self.env.deny.contains(d) {
                self.env.deny.push(d.clone());
            }
        }

        // Exec subtable merge.
        self.exec.default = overlay.exec.default;
        // `allow` is a whitelist: last explicit list wins.
        if !overlay.exec.allow.is_empty() {
            self.exec.allow = overlay.exec.allow.clone();
        }
        // `deny` is a blacklist: union of base and overlay.
        for d in &overlay.exec.deny {
            if !self.exec.deny.contains(d) {
                self.exec.deny.push(d.clone());
            }
        }
        if overlay.exec.max_processes != 0 {
            self.exec.max_processes = overlay.exec.max_processes;
        }

        // Modules subtable merge.
        if let Some(d) = overlay.modules.default {
            self.modules.default = Some(d);
        }
        macro_rules! merge_module {
            ($field:ident) => {
                if overlay.modules.$field.is_some() {
                    self.modules.$field = overlay.modules.$field;
                }
            };
        }
        merge_module!(http);
        merge_module!(tcp);
        merge_module!(udp);
        merge_module!(unix);
        merge_module!(fs);
        merge_module!(process);
        merge_module!(dbus);
        merge_module!(tts);

        self
    }

    /// Convert the user-facing TOML config into the VM runtime policy.
    ///
    /// The VM policy is intentionally separate from the TOML type so the VM
    /// crate does not depend on serde/TOML details.
    pub fn to_policy(&self) -> hudhudscript_vm::HostAccessPolicy {
        use hudhudscript_vm::host_access::{EnvPolicy, ExecPolicy, ModulePolicy};
        use hudhudscript_vm::{HostAccessDecision as PDecision, HostAccessPolicy};

        fn convert(d: AccessDecision) -> PDecision {
            match d {
                AccessDecision::Allow => PDecision::Allow,
                AccessDecision::Deny => PDecision::Deny,
            }
        }

        let mut policy = HostAccessPolicy {
            default: convert(self.default),
            env: EnvPolicy {
                default: convert(self.env.default),
                allow: self.env.allow.iter().cloned().collect(),
                deny: self.env.deny.iter().cloned().collect(),
            },
            exec: ExecPolicy {
                default: convert(self.exec.default),
                allow: self.exec.allow.iter().cloned().collect(),
                deny: self.exec.deny.iter().cloned().collect(),
                max_processes: self.exec.max_processes,
            },
            modules: ModulePolicy {
                default: self
                    .modules
                    .default
                    .map(convert)
                    .unwrap_or_else(|| convert(self.default)),
                http: self.modules.http.map(convert),
                tcp: self.modules.tcp.map(convert),
                udp: self.modules.udp.map(convert),
                unix: self.modules.unix.map(convert),
                fs: self.modules.fs.map(convert),
                process: self.modules.process.map(convert),
                dbus: self.modules.dbus.map(convert),
                tts: self.modules.tts.map(convert),
            },
        };

        // When the top-level default is deny and no module default is set,
        // inherit deny into modules so the policy is consistent.
        if policy.modules.default == PDecision::Allow
            && self.default == AccessDecision::Deny
            && self.modules.default.is_none()
        {
            policy.modules.default = PDecision::Deny;
        }

        policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_deny() {
        let cfg = HostAccessConfig::default();
        assert_eq!(cfg.default, AccessDecision::Deny);
        assert_eq!(cfg.env.default, AccessDecision::Deny);
        assert_eq!(cfg.exec.default, AccessDecision::Deny);
        assert_eq!(cfg.modules.default, None);
    }

    #[test]
    fn restrictive_is_deny() {
        let cfg = HostAccessConfig::restrictive();
        assert_eq!(cfg.default, AccessDecision::Deny);
        assert_eq!(cfg.exec.max_processes, 0);
        assert_eq!(cfg.modules.default, Some(AccessDecision::Deny));
    }

    #[test]
    fn permissive_is_allow() {
        let cfg = HostAccessConfig::permissive();
        assert_eq!(cfg.default, AccessDecision::Allow);
        assert_eq!(cfg.exec.max_processes, 100);
        assert_eq!(cfg.modules.default, Some(AccessDecision::Allow));
    }

    #[test]
    fn toml_round_trip() {
        let toml = r#"
default = "deny"

[env]
default = "allow"
allow = ["DEEPSEEK_API_KEY"]
deny = ["HOME"]

[exec]
default = "deny"
allow = ["python", "node"]
deny = ["rm"]
max_processes = 10

[modules]
default = "deny"
http = "allow"
process = "deny"
"#;
        let cfg: HostAccessConfig = toml::from_str(toml).expect("valid TOML");
        assert_eq!(cfg.default, AccessDecision::Deny);
        assert_eq!(cfg.env.default, AccessDecision::Allow);
        assert!(cfg.env.allow.contains(&"DEEPSEEK_API_KEY".to_string()));
        assert!(cfg.env.deny.contains(&"HOME".to_string()));
        assert_eq!(cfg.exec.default, AccessDecision::Deny);
        assert!(cfg.exec.allow.contains(&"python".to_string()));
        assert_eq!(cfg.exec.max_processes, 10);
        assert_eq!(cfg.modules.default, Some(AccessDecision::Deny));
        assert_eq!(cfg.modules.http, Some(AccessDecision::Allow));
        assert_eq!(cfg.modules.process, Some(AccessDecision::Deny));
    }

    #[test]
    fn invalid_decision_is_parse_error() {
        let toml = r#"default = "maybe""#;
        let result: Result<HostAccessConfig, _> = toml::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn merge_scalar_default_overlay_wins() {
        let base = HostAccessConfig::restrictive();
        let mut overlay = HostAccessConfig::default();
        overlay.default = AccessDecision::Allow;
        let merged = base.merge(Some(&overlay));
        assert_eq!(merged.default, AccessDecision::Allow);
        // subtables unchanged because overlay left them at default
        assert_eq!(merged.env.default, AccessDecision::Deny);
    }

    #[test]
    fn merge_env_lists_allow_replace_deny_union() {
        let mut base = HostAccessConfig::default();
        base.env.allow = vec!["BASE_ALLOW".to_string()];
        base.env.deny = vec!["BASE_DENY".to_string()];

        let mut overlay = HostAccessConfig::default();
        overlay.env.allow = vec!["OVERLAY_ALLOW".to_string()];
        overlay.env.deny = vec!["OVERLAY_DENY".to_string()];

        let merged = base.merge(Some(&overlay));
        // allow whitelist: overlay replaces
        assert_eq!(merged.env.allow, vec!["OVERLAY_ALLOW".to_string()]);
        // deny blacklist: union of base and overlay
        assert!(merged.env.deny.contains(&"BASE_DENY".to_string()));
        assert!(merged.env.deny.contains(&"OVERLAY_DENY".to_string()));
    }

    #[test]
    fn merge_exec_max_processes_overlay_wins_when_non_zero() {
        let mut base = HostAccessConfig::default();
        base.exec.max_processes = 5;
        let mut overlay = HostAccessConfig::default();
        overlay.exec.max_processes = 10;
        let merged = base.merge(Some(&overlay));
        assert_eq!(merged.exec.max_processes, 10);
    }

    #[test]
    fn merge_modules_per_module_overlay_wins() {
        let mut base = HostAccessConfig::default();
        base.modules.default = Some(AccessDecision::Deny);
        base.modules.http = Some(AccessDecision::Deny);
        base.modules.tcp = Some(AccessDecision::Allow);

        let mut overlay = HostAccessConfig::default();
        overlay.modules.http = Some(AccessDecision::Allow);
        overlay.modules.process = Some(AccessDecision::Deny);

        let merged = base.merge(Some(&overlay));
        // default scalar unchanged because overlay left it at None
        assert_eq!(merged.modules.default, Some(AccessDecision::Deny));
        assert_eq!(merged.modules.http, Some(AccessDecision::Allow));
        assert_eq!(merged.modules.tcp, Some(AccessDecision::Allow));
        assert_eq!(merged.modules.process, Some(AccessDecision::Deny));
    }

    #[test]
    fn merge_none_returns_base() {
        let base = HostAccessConfig::restrictive();
        let merged = base.merge(None);
        assert_eq!(merged.default, AccessDecision::Deny);
    }

    #[test]
    fn to_policy_converts_decisions_and_lists() {
        let mut cfg = HostAccessConfig::default();
        cfg.default = AccessDecision::Allow;
        cfg.env.default = AccessDecision::Deny;
        cfg.env.allow = vec!["KEY".to_string()];
        cfg.exec.default = AccessDecision::Allow;
        cfg.exec.deny = vec!["rm".to_string()];
        cfg.modules.http = Some(AccessDecision::Deny);

        let policy = cfg.to_policy();
        assert!(policy.default.is_allowed());
        assert!(policy.ensure_env_read("KEY").is_ok());
        assert!(policy.ensure_env_read("OTHER").is_err());
        assert!(policy.ensure_exec_method("run").is_ok());
        assert!(policy.ensure_command_allowed("rm").is_err());
        assert!(policy.ensure_module_allowed("http").is_err());
    }

    #[test]
    fn to_policy_default_deny_inherits_to_modules() {
        let cfg = HostAccessConfig::default(); // default Deny, modules.default None
        let policy = cfg.to_policy();
        assert!(policy.ensure_module_allowed("http").is_err());
    }
}
