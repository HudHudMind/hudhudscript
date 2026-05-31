//! Validator implementation

use crate::error::{ValidationError, ValidationResult};
use crate::schema::{Schema, SchemaType};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// Validator for input validation
pub struct Validator {
    /// Custom validators
    #[allow(clippy::type_complexity)]
    custom_validators: HashMap<String, Box<dyn Fn(&Value) -> ValidationResult<()>>>,
}

impl Validator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            custom_validators: HashMap::new(),
        }
    }

    /// Register a custom validator
    pub fn register_validator<F>(&mut self, name: String, validator: F)
    where
        F: Fn(&Value) -> ValidationResult<()> + 'static,
    {
        self.custom_validators.insert(name, Box::new(validator));
    }

    /// Validate a value against a schema
    pub fn validate(&self, value: &Value, schema: &Schema) -> ValidationResult<()> {
        self.validate_type(value, &schema.schema_type)?;

        // Apply custom validation rules
        if let Some(rules) = &schema.rules {
            for rule in rules {
                if let Some(validator_name) = &rule.validator {
                    if let Some(validator) = self.custom_validators.get(validator_name) {
                        validator(value)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate value type
    fn validate_type(&self, value: &Value, schema_type: &SchemaType) -> ValidationResult<()> {
        match schema_type {
            SchemaType::String {
                min_length,
                max_length,
                pattern,
            } => self.validate_string(value, *min_length, *max_length, pattern.as_deref()),
            SchemaType::Number { min, max } => self.validate_number(value, *min, *max),
            SchemaType::Integer { min, max } => self.validate_integer(value, *min, *max),
            SchemaType::Boolean => self.validate_boolean(value),
            SchemaType::Array {
                items,
                min_items,
                max_items,
            } => self.validate_array(value, items, *min_items, *max_items),
            SchemaType::Object {
                properties,
                required,
            } => self.validate_object(value, properties, required.as_ref()),
            SchemaType::Null => self.validate_null(value),
            SchemaType::Any => Ok(()),
        }
    }

    /// Validate string
    fn validate_string(
        &self,
        value: &Value,
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<&str>,
    ) -> ValidationResult<()> {
        let s = value
            .as_str()
            .ok_or_else(|| ValidationError::TypeMismatch {
                expected: "string".to_string(),
                found: Self::type_name(value),
            })?;

        // Check length
        if let Some(min) = min_length {
            if s.len() < min {
                return Err(ValidationError::InvalidLength {
                    expected: format!(">= {}", min),
                    found: s.len(),
                });
            }
        }

        if let Some(max) = max_length {
            if s.len() > max {
                return Err(ValidationError::InvalidLength {
                    expected: format!("<= {}", max),
                    found: s.len(),
                });
            }
        }

        // Check pattern
        if let Some(pattern_str) = pattern {
            let regex = Regex::new(pattern_str).map_err(|_| ValidationError::InvalidFormat {
                message: format!("Invalid regex pattern: {}", pattern_str),
            })?;

            if !regex.is_match(s) {
                return Err(ValidationError::PatternMismatch {
                    pattern: pattern_str.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate number
    fn validate_number(
        &self,
        value: &Value,
        min: Option<f64>,
        max: Option<f64>,
    ) -> ValidationResult<()> {
        let n = value
            .as_f64()
            .ok_or_else(|| ValidationError::TypeMismatch {
                expected: "number".to_string(),
                found: Self::type_name(value),
            })?;

        if let Some(min_val) = min {
            if n < min_val {
                return Err(ValidationError::OutOfRange {
                    value: n.to_string(),
                    min: min_val.to_string(),
                    max: max
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "∞".to_string()),
                });
            }
        }

        if let Some(max_val) = max {
            if n > max_val {
                return Err(ValidationError::OutOfRange {
                    value: n.to_string(),
                    min: min
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "-∞".to_string()),
                    max: max_val.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate integer
    fn validate_integer(
        &self,
        value: &Value,
        min: Option<i64>,
        max: Option<i64>,
    ) -> ValidationResult<()> {
        let n = value
            .as_i64()
            .ok_or_else(|| ValidationError::TypeMismatch {
                expected: "integer".to_string(),
                found: Self::type_name(value),
            })?;

        if let Some(min_val) = min {
            if n < min_val {
                return Err(ValidationError::OutOfRange {
                    value: n.to_string(),
                    min: min_val.to_string(),
                    max: max
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "∞".to_string()),
                });
            }
        }

        if let Some(max_val) = max {
            if n > max_val {
                return Err(ValidationError::OutOfRange {
                    value: n.to_string(),
                    min: min
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "-∞".to_string()),
                    max: max_val.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate boolean
    fn validate_boolean(&self, value: &Value) -> ValidationResult<()> {
        if !value.is_boolean() {
            return Err(ValidationError::TypeMismatch {
                expected: "boolean".to_string(),
                found: Self::type_name(value),
            });
        }
        Ok(())
    }

    /// Validate array
    fn validate_array(
        &self,
        value: &Value,
        items: &SchemaType,
        min_items: Option<usize>,
        max_items: Option<usize>,
    ) -> ValidationResult<()> {
        let arr = value
            .as_array()
            .ok_or_else(|| ValidationError::TypeMismatch {
                expected: "array".to_string(),
                found: Self::type_name(value),
            })?;

        // Check length
        if let Some(min) = min_items {
            if arr.len() < min {
                return Err(ValidationError::InvalidLength {
                    expected: format!(">= {}", min),
                    found: arr.len(),
                });
            }
        }

        if let Some(max) = max_items {
            if arr.len() > max {
                return Err(ValidationError::InvalidLength {
                    expected: format!("<= {}", max),
                    found: arr.len(),
                });
            }
        }

        // Validate each item
        for item in arr {
            self.validate_type(item, items)?;
        }

        Ok(())
    }

    /// Validate object
    fn validate_object(
        &self,
        value: &Value,
        properties: &HashMap<String, Schema>,
        required: Option<&Vec<String>>,
    ) -> ValidationResult<()> {
        let obj = value
            .as_object()
            .ok_or_else(|| ValidationError::TypeMismatch {
                expected: "object".to_string(),
                found: Self::type_name(value),
            })?;

        // Check required fields
        if let Some(required_fields) = required {
            for field in required_fields {
                if !obj.contains_key(field) {
                    return Err(ValidationError::RequiredFieldMissing {
                        field: field.clone(),
                    });
                }
            }
        }

        // Validate each property
        for (key, schema) in properties {
            if let Some(prop_value) = obj.get(key) {
                self.validate(prop_value, schema)?;
            }
        }

        Ok(())
    }

    /// Validate null
    fn validate_null(&self, value: &Value) -> ValidationResult<()> {
        if !value.is_null() {
            return Err(ValidationError::TypeMismatch {
                expected: "null".to_string(),
                found: Self::type_name(value),
            });
        }
        Ok(())
    }

    /// Get type name of a value
    pub fn type_name(value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Object(_) => "object".to_string(),
        }
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
