//! Rule types (Rule, Condition, Action)

use serde::{Deserialize, Serialize};

use super::RuleId;

/// Conditional rule with constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub priority: u32,
}

impl Rule {
    /// Create a new rule with the specified priority
    pub fn new(
        id: String,
        name: String,
        conditions: Vec<Condition>,
        actions: Vec<Action>,
        priority: u32,
    ) -> Self {
        Self {
            id,
            name,
            conditions,
            actions,
            priority,
        }
    }

    /// Get the current priority of the rule
    pub fn get_priority(&self) -> u32 {
        self.priority
    }

    /// Set a new priority for the rule
    pub fn set_priority(&mut self, new_priority: u32) {
        self.priority = new_priority;
    }

    /// Adjust the priority by a given delta, clamped to bounds
    pub fn adjust_priority(&mut self, delta: i32) {
        let new_priority = (self.priority as i64 + delta as i64).clamp(0, u32::MAX as i64) as u32;
        self.priority = new_priority;
    }

    /// Validate the priority is within acceptable range
    pub fn validate_priority(&self) -> bool {
        true
    }
}

/// Condition for rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Condition {
    Equals {
        field: String,
        value: serde_json::Value,
    },
    NotEquals {
        field: String,
        value: serde_json::Value,
    },
    GreaterThan {
        field: String,
        value: f64,
    },
    LessThan {
        field: String,
        value: f64,
    },
    Between {
        field: String,
        min: f64,
        max: f64,
    },
    In {
        field: String,
        values: Vec<serde_json::Value>,
    },
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
}

/// Action to execute when rule matches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    Allow,
    Deny,
    Require { permission: String },
    Execute { task: String },
    Notify { message: String },
}
