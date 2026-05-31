//! Typed field accessors — string, number, boolean, array, object.

use serde_json::Value;

use super::EvaluationContext;

impl EvaluationContext {
    /// Get a string field value with type validation
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    /// Get a number field value with type validation
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_f64())
    }

    /// Get a boolean field value with type validation
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    /// Get an array field value with type validation
    pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        self.get(key).and_then(|v| v.as_array())
    }

    /// Get an object field value with type validation
    pub fn get_object(&self, key: &str) -> Option<&serde_json::Map<String, Value>> {
        self.get(key).and_then(|v| v.as_object())
    }
}
