//! Tool Schema Definitions

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Tool schema with JSON Schema validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: Option<String>,

    /// Input schema (JSON Schema)
    pub input_schema: JsonSchema,

    /// Server that provides this tool
    pub server: String,
}

/// Tool metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: Option<String>,

    /// Server name
    pub server: String,

    /// When was this tool discovered
    pub discovered_at: std::time::SystemTime,

    /// Last used timestamp
    pub last_used: Option<std::time::SystemTime>,

    /// Usage count
    pub usage_count: u64,

    /// Tags for categorization
    pub tags: Vec<String>,
}

impl ToolMetadata {
    /// Create new tool metadata
    pub fn new(name: String, server: String, description: Option<String>) -> Self {
        Self {
            name,
            description,
            server,
            discovered_at: std::time::SystemTime::now(),
            last_used: None,
            usage_count: 0,
            tags: Vec::new(),
        }
    }

    /// Record tool usage
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
        self.last_used = Some(std::time::SystemTime::now());
    }

    /// Add tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

/// JSON Schema representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    #[serde(rename = "type")]
    pub schema_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, JsonSchemaProperty>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// JSON Schema property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaProperty {
    #[serde(rename = "type")]
    pub property_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
}

impl JsonSchema {
    /// Validate value against schema
    pub fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        match self.schema_type.as_str() {
            "object" => self.validate_object(value),
            "array" => self.validate_array(value),
            "string" => self.validate_string(value),
            "number" => self.validate_number(value),
            "integer" => self.validate_integer(value),
            "boolean" => self.validate_boolean(value),
            "null" => self.validate_null(value),
            _ => Err(ValidationError::UnknownType(self.schema_type.clone())),
        }
    }

    fn validate_object(&self, value: &Value) -> Result<(), ValidationError> {
        let obj = value
            .as_object()
            .ok_or_else(|| ValidationError::TypeMismatch {
                expected: "object".to_string(),
                found: value_type_name(value),
            })?;

        // Check required fields
        if let Some(required) = &self.required {
            for field in required {
                if !obj.contains_key(field) {
                    return Err(ValidationError::MissingRequired(field.clone()));
                }
            }
        }

        // Validate properties
        if let Some(properties) = &self.properties {
            for (key, prop_value) in obj {
                if let Some(prop_schema) = properties.get(key) {
                    // Validate property type
                    validate_property_type(prop_value, &prop_schema.property_type)?;
                }
            }
        }

        Ok(())
    }

    fn validate_array(&self, value: &Value) -> Result<(), ValidationError> {
        let arr = value
            .as_array()
            .ok_or_else(|| ValidationError::TypeMismatch {
                expected: "array".to_string(),
                found: value_type_name(value),
            })?;

        // Validate items if schema provided
        if let Some(items_schema) = &self.items {
            for item in arr {
                items_schema.validate(item)?;
            }
        }

        Ok(())
    }

    fn validate_string(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_string() {
            return Err(ValidationError::TypeMismatch {
                expected: "string".to_string(),
                found: value_type_name(value),
            });
        }
        Ok(())
    }

    fn validate_number(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_number() {
            return Err(ValidationError::TypeMismatch {
                expected: "number".to_string(),
                found: value_type_name(value),
            });
        }
        Ok(())
    }

    fn validate_integer(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_i64() && !value.is_u64() {
            return Err(ValidationError::TypeMismatch {
                expected: "integer".to_string(),
                found: value_type_name(value),
            });
        }
        Ok(())
    }

    fn validate_boolean(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_boolean() {
            return Err(ValidationError::TypeMismatch {
                expected: "boolean".to_string(),
                found: value_type_name(value),
            });
        }
        Ok(())
    }

    fn validate_null(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_null() {
            return Err(ValidationError::TypeMismatch {
                expected: "null".to_string(),
                found: value_type_name(value),
            });
        }
        Ok(())
    }
}

/// Validation error
#[derive(Debug, Clone)]
pub enum ValidationError {
    TypeMismatch { expected: String, found: String },
    MissingRequired(String),
    UnknownType(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ValidationError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            ValidationError::MissingRequired(name) => {
                write!(f, "Missing required field: {}", name)
            }
            ValidationError::UnknownType(t) => write!(f, "Unknown schema type: {}", t),
        }
    }
}

impl std::error::Error for ValidationError {}

pub fn value_type_name(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

pub fn validate_property_type(value: &Value, expected_type: &str) -> Result<(), ValidationError> {
    let matches = match expected_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => true, // Unknown types pass
    };

    if !matches {
        return Err(ValidationError::TypeMismatch {
            expected: expected_type.to_string(),
            found: value_type_name(value),
        });
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl ValidationError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ValidationError::MissingRequired(..) => {
                hudhudscript_errors::ErrorCode::ValidationMissingRequired
            }
            ValidationError::TypeMismatch { .. } => {
                hudhudscript_errors::ErrorCode::ValidationTypeMismatch
            }
            ValidationError::UnknownType(..) => {
                hudhudscript_errors::ErrorCode::ValidationUnknownType
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<ValidationError> for hudhudscript_errors::Error {
    fn from(e: ValidationError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
