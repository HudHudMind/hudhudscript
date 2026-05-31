//! Input sanitizer

use serde_json::Value;

/// Sanitizer for input sanitization
pub struct Sanitizer;

impl Sanitizer {
    /// Create a new sanitizer
    pub fn new() -> Self {
        Self
    }

    /// Sanitize a string value
    pub fn sanitize_string(&self, value: &str) -> String {
        // Remove control characters
        let cleaned: String = value
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect();

        // Trim whitespace
        cleaned.trim().to_string()
    }

    /// Sanitize HTML (basic XSS prevention)
    pub fn sanitize_html(&self, value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2F;")
    }

    /// Sanitize SQL (basic SQL injection prevention)
    pub fn sanitize_sql(&self, value: &str) -> String {
        value
            .replace('\'', "''")
            .replace('\\', "\\\\")
            .replace('\0', "")
    }

    /// Sanitize JSON value recursively
    pub fn sanitize_json(&self, value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(self.sanitize_string(s)),
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.sanitize_json(v)).collect()),
            Value::Object(obj) => Value::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), self.sanitize_json(v)))
                    .collect(),
            ),
            _ => value.clone(),
        }
    }

    /// Remove null bytes from string
    pub fn remove_null_bytes(&self, value: &str) -> String {
        value.replace('\0', "")
    }

    /// Normalize whitespace
    pub fn normalize_whitespace(&self, value: &str) -> String {
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Truncate string to maximum length
    pub fn truncate(&self, value: &str, max_length: usize) -> String {
        if value.len() <= max_length {
            value.to_string()
        } else {
            value.chars().take(max_length).collect()
        }
    }
}

impl Default for Sanitizer {
    fn default() -> Self {
        Self::new()
    }
}
