//! Skill definition types for YAML event-action automation
//!
//! A Skill represents a named automation unit with triggers, conditions, and actions.
//! Skills are defined in YAML and loaded by the SkillEngine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A fully resolved skill definition ready for engine registration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    /// Unique skill name
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    /// Triggers that activate this skill
    pub triggers: Vec<SkillTrigger>,
    /// Conditions that must hold for actions to execute (raw string expressions)
    #[serde(default)]
    pub conditions: Vec<String>,
    /// Ordered list of actions to execute when triggered
    pub actions: Vec<SkillAction>,
}

/// A trigger entry inside a skill YAML definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillTrigger {
    /// Event-based trigger with optional glob/regex pattern
    Event {
        event: String,
        #[serde(default)]
        pattern: Option<String>,
    },
    /// Cron-schedule trigger
    Cron { cron: String },
    /// Manual / on-demand trigger
    Manual { manual: bool },
}

/// An action entry inside a skill YAML definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillAction {
    /// Tool / plugin name to invoke
    pub tool: String,
    /// Arguments passed to the tool (supports `{{var}}` template placeholders)
    #[serde(default)]
    pub args: HashMap<String, String>,
    /// Optional timeout in seconds
    #[serde(default)]
    pub timeout: Option<u64>,
}
