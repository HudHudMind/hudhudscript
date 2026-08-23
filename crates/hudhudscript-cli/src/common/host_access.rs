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
/// Each module (`http`, `tcp`, `udp`, `unix`, `fs`, `process`, `database`, `dbus`, `tts`)
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
    /// PostgreSQL, MySQL, and SQLite database access.
    #[serde(default)]
    pub database: Option<AccessDecision>,
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
                database: Some(AccessDecision::Deny),
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
                database: Some(AccessDecision::Deny),
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
        merge_module!(database);
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
                database: self.modules.database.map(convert),
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
#[path = "host_access_tests.rs"]
mod tests;
