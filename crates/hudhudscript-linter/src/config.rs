//! Linter configuration — rule enable/disable and severity overrides.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use hudhudscript_errors::Severity;

/// Per-rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Whether this rule is enabled.
    pub enabled: bool,
    /// Override severity (uses default when `None`).
    pub severity: Option<Severity>,
}

impl RuleConfig {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            severity: None,
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            severity: None,
        }
    }
}

/// Top-level linter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintConfig {
    /// Per-rule overrides keyed by rule code.
    pub rules: HashMap<String, RuleConfig>,
    /// Maximum nesting depth before `deep-nesting` fires. Default: 4.
    pub max_nesting_depth: usize,
}

impl Default for LintConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();
        // `no-print` is disabled by default — opt-in via .hudlint config.
        rules.insert("no-print".to_string(), RuleConfig::disabled());
        Self {
            rules,
            max_nesting_depth: 4,
        }
    }
}

impl LintConfig {
    /// Check whether a rule is enabled (defaults to `true` if not overridden).
    pub fn is_enabled(&self, code: &str) -> bool {
        self.rules.get(code).is_none_or(|r| r.enabled)
    }

    /// Resolve the severity for a rule, falling back to the provided default.
    pub fn severity(&self, code: &str, default: Severity) -> Severity {
        self.rules
            .get(code)
            .and_then(|r| r.severity)
            .unwrap_or(default)
    }

    /// Builder helper: disable a rule.
    pub fn disable(mut self, code: &str) -> Self {
        self.rules.insert(code.to_string(), RuleConfig::disabled());
        self
    }

    /// Builder helper: enable a rule with an optional severity override.
    pub fn enable(mut self, code: &str, severity: Option<Severity>) -> Self {
        self.rules.insert(
            code.to_string(),
            RuleConfig {
                enabled: true,
                severity,
            },
        );
        self
    }
}
