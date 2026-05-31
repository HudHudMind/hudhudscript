//! Evaluation context for rule conditions

use serde_json::Value;
use std::collections::HashMap;

/// Evaluation context for rule conditions
///
/// Provides a type-safe wrapper around a HashMap for field values.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationContext {
    fields: HashMap<String, Value>,
}

impl EvaluationContext {
    /// Create a new empty evaluation context
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Insert a field into the context
    pub fn insert(&mut self, key: String, value: Value) {
        self.fields.insert(key, value);
    }

    /// Get a field value from the context
    pub fn get(&self, key: &str) -> Option<&Value> {
        let value = self.fields.get(key);
        if value.is_none() {
            log::warn!("Field '{}' not found in evaluation context", key);
        }
        value
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}
