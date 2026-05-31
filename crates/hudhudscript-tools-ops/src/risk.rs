//! Risk Assessment Engine (Issue #632)
//!
//! Categorises tool operations into risk levels and provides a rule-based
//! engine for evaluating the risk of a given tool invocation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

// ---------------------------------------------------------------------------
// Risk level
// ---------------------------------------------------------------------------

/// The risk level assigned to a tool operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No confirmation required — read-only or side-effect-free operations.
    Safe,
    /// The user is warned but execution proceeds unless denied.
    Warning,
    /// Requires explicit approval before execution.
    Dangerous,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Safe => write!(f, "safe"),
            RiskLevel::Warning => write!(f, "warning"),
            RiskLevel::Dangerous => write!(f, "dangerous"),
        }
    }
}

// ---------------------------------------------------------------------------
// Risk rule
// ---------------------------------------------------------------------------

/// A single rule that matches a tool name pattern and assigns a risk level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskRule {
    /// A tool name pattern — either an exact name or a prefix ending with `*`.
    pub pattern: String,
    /// The risk level to assign when this rule matches.
    pub level: RiskLevel,
    /// Human-readable description of why this rule exists.
    pub reason: String,
}

impl RiskRule {
    /// Create a new risk rule.
    pub fn new(pattern: impl Into<String>, level: RiskLevel, reason: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            level,
            reason: reason.into(),
        }
    }

    /// Returns `true` if this rule matches the given tool name.
    pub fn matches(&self, tool_name: &str) -> bool {
        if let Some(prefix) = self.pattern.strip_suffix('*') {
            tool_name.starts_with(prefix)
        } else {
            self.pattern == tool_name
        }
    }
}

// ---------------------------------------------------------------------------
// Risk assessment result
// ---------------------------------------------------------------------------

/// The result of assessing a tool invocation's risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// The tool that was assessed.
    pub tool_name: String,
    /// The determined risk level.
    pub level: RiskLevel,
    /// The rule that matched (if any).
    pub matched_rule: Option<RiskRule>,
    /// Whether the operation requires explicit user approval.
    pub requires_approval: bool,
    /// Human-readable summary.
    pub summary: String,
}

// ---------------------------------------------------------------------------
// Risk assessment engine
// ---------------------------------------------------------------------------

/// Rule-based engine that evaluates the risk level of tool operations.
///
/// Rules are evaluated in order; the **first matching rule wins**.
/// If no rule matches, the configured default level is used.
#[derive(Debug, Clone)]
pub struct RiskEngine {
    rules: Vec<RiskRule>,
    /// Per-tool overrides that take priority over pattern rules.
    overrides: HashMap<String, RiskLevel>,
    /// Default risk level when no rule matches.
    default_level: RiskLevel,
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            overrides: HashMap::new(),
            default_level: RiskLevel::Safe,
        }
    }
}

impl RiskEngine {
    /// Create a new engine with no rules and `Safe` as the default.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an engine pre-loaded with common risk rules for typical tool operations.
    pub fn with_defaults() -> Self {
        let mut engine = Self::new();

        // Dangerous operations
        engine.add_rule(RiskRule::new(
            "delete_*",
            RiskLevel::Dangerous,
            "Deletion operations are irreversible",
        ));
        engine.add_rule(RiskRule::new(
            "drop_*",
            RiskLevel::Dangerous,
            "Drop operations are irreversible",
        ));
        engine.add_rule(RiskRule::new(
            "exec_*",
            RiskLevel::Dangerous,
            "Arbitrary command execution",
        ));
        engine.add_rule(RiskRule::new(
            "shell_*",
            RiskLevel::Dangerous,
            "Shell command execution",
        ));
        engine.add_rule(RiskRule::new(
            "rm_*",
            RiskLevel::Dangerous,
            "File removal is irreversible",
        ));
        engine.add_rule(RiskRule::new(
            "format_disk",
            RiskLevel::Dangerous,
            "Disk formatting is irreversible",
        ));

        // Warning operations
        engine.add_rule(RiskRule::new(
            "write_*",
            RiskLevel::Warning,
            "Write operations modify state",
        ));
        engine.add_rule(RiskRule::new(
            "update_*",
            RiskLevel::Warning,
            "Update operations modify existing data",
        ));
        engine.add_rule(RiskRule::new(
            "create_*",
            RiskLevel::Warning,
            "Create operations add new resources",
        ));
        engine.add_rule(RiskRule::new(
            "send_*",
            RiskLevel::Warning,
            "Send operations transmit data externally",
        ));
        engine.add_rule(RiskRule::new(
            "publish_*",
            RiskLevel::Warning,
            "Publish makes data publicly accessible",
        ));

        // Safe operations
        engine.add_rule(RiskRule::new(
            "read_*",
            RiskLevel::Safe,
            "Read-only operation",
        ));
        engine.add_rule(RiskRule::new(
            "get_*",
            RiskLevel::Safe,
            "Read-only retrieval",
        ));
        engine.add_rule(RiskRule::new(
            "list_*",
            RiskLevel::Safe,
            "Read-only listing",
        ));
        engine.add_rule(RiskRule::new(
            "search_*",
            RiskLevel::Safe,
            "Read-only search",
        ));

        engine
    }

    /// Add a rule to the engine. Rules are evaluated in insertion order.
    pub fn add_rule(&mut self, rule: RiskRule) {
        self.rules.push(rule);
    }

    /// Set a per-tool override that takes priority over pattern rules.
    pub fn set_override(&mut self, tool_name: impl Into<String>, level: RiskLevel) {
        self.overrides.insert(tool_name.into(), level);
    }

    /// Remove a per-tool override.
    pub fn remove_override(&mut self, tool_name: &str) -> Option<RiskLevel> {
        self.overrides.remove(tool_name)
    }

    /// Set the default risk level for tools that match no rule.
    pub fn set_default_level(&mut self, level: RiskLevel) {
        self.default_level = level;
    }

    /// Assess the risk level of a tool invocation.
    pub fn assess(&self, tool_name: &str) -> RiskAssessment {
        // Check per-tool overrides first
        if let Some(&level) = self.overrides.get(tool_name) {
            debug!(
                tool = tool_name,
                level = %level,
                "Risk assessed via per-tool override"
            );
            return RiskAssessment {
                tool_name: tool_name.to_string(),
                level,
                matched_rule: None,
                requires_approval: level >= RiskLevel::Dangerous,
                summary: format!("Tool '{}' has a per-tool override: {}", tool_name, level),
            };
        }

        // Evaluate rules in order — first match wins
        for rule in &self.rules {
            if rule.matches(tool_name) {
                debug!(
                    tool = tool_name,
                    level = %rule.level,
                    pattern = rule.pattern.as_str(),
                    "Risk assessed via rule match"
                );
                return RiskAssessment {
                    tool_name: tool_name.to_string(),
                    level: rule.level,
                    matched_rule: Some(rule.clone()),
                    requires_approval: rule.level >= RiskLevel::Dangerous,
                    summary: format!(
                        "Tool '{}' matched rule '{}': {} ({})",
                        tool_name, rule.pattern, rule.level, rule.reason
                    ),
                };
            }
        }

        // No match — use default
        debug!(
            tool = tool_name,
            level = %self.default_level,
            "Risk assessed as default (no rule matched)"
        );
        RiskAssessment {
            tool_name: tool_name.to_string(),
            level: self.default_level,
            matched_rule: None,
            requires_approval: self.default_level >= RiskLevel::Dangerous,
            summary: format!(
                "Tool '{}' has no matching rule, using default: {}",
                tool_name, self.default_level
            ),
        }
    }

    /// Return the current rules (read-only).
    pub fn rules(&self) -> &[RiskRule] {
        &self.rules
    }
}
