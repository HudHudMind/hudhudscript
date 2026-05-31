//! Core EvaluationContext — construction and basic field access.

use serde_json::Value;
use std::collections::HashMap;

/// Evaluation context for rule conditions
///
/// Provides a type-safe wrapper around a HashMap with helper functions
/// for dynamic context construction, type validation, and field access.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationContext {
    pub(crate) fields: HashMap<String, Value>,
}

impl EvaluationContext {
    /// Create a new empty evaluation context
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Create an evaluation context from a HashMap
    pub fn from_map(fields: HashMap<String, Value>) -> Self {
        Self { fields }
    }

    /// Insert a field into the context
    pub fn insert(&mut self, key: String, value: Value) {
        self.fields.insert(key, value);
    }

    /// Get a field value from the context
    ///
    /// Returns None if the field doesn't exist, logging a warning.
    pub fn get(&self, key: &str) -> Option<&Value> {
        let value = self.fields.get(key);
        if value.is_none() {
            log::warn!("Field '{}' not found in evaluation context", key);
        }
        value
    }

    /// Check if a field exists in the context
    pub fn contains_key(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// Get the number of fields in the context
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if the context is empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Validate that a field exists and has the expected type
    pub fn validate_field_type(&self, key: &str, expected_type: &str) -> Result<(), String> {
        match self.get(key) {
            Some(value) => {
                let actual_type = match value {
                    Value::String(_) => "string",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "boolean",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                    Value::Null => "null",
                };

                if actual_type == expected_type {
                    Ok(())
                } else {
                    Err(format!(
                        "Field '{}' has type '{}' but expected '{}'",
                        key, actual_type, expected_type
                    ))
                }
            }
            None => Err(format!("Field '{}' not found in context", key)),
        }
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}
