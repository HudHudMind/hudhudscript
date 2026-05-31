//! Schema definition for validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SchemaType {
    /// String type
    String {
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
    },

    /// Number type
    Number {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },

    /// Integer type
    Integer {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
    },

    /// Boolean type
    Boolean,

    /// Array type
    Array {
        items: Box<SchemaType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_items: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_items: Option<usize>,
    },

    /// Object type
    Object {
        properties: HashMap<String, Schema>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<Vec<String>>,
    },

    /// Null type
    Null,

    /// Any type (no validation)
    Any,
}

/// Validation rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// Custom validation function name
    pub validator: Option<String>,
}

/// Schema definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    /// Schema type
    #[serde(flatten)]
    pub schema_type: SchemaType,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Custom validation rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<ValidationRule>>,
}

impl Schema {
    /// Create a string schema
    pub fn string() -> Self {
        Self {
            schema_type: SchemaType::String {
                min_length: None,
                max_length: None,
                pattern: None,
            },
            description: None,
            default: None,
            rules: None,
        }
    }

    /// Create a number schema
    pub fn number() -> Self {
        Self {
            schema_type: SchemaType::Number {
                min: None,
                max: None,
            },
            description: None,
            default: None,
            rules: None,
        }
    }

    /// Create an integer schema
    pub fn integer() -> Self {
        Self {
            schema_type: SchemaType::Integer {
                min: None,
                max: None,
            },
            description: None,
            default: None,
            rules: None,
        }
    }

    /// Create a boolean schema
    pub fn boolean() -> Self {
        Self {
            schema_type: SchemaType::Boolean,
            description: None,
            default: None,
            rules: None,
        }
    }

    /// Create an array schema
    pub fn array(items: SchemaType) -> Self {
        Self {
            schema_type: SchemaType::Array {
                items: Box::new(items),
                min_items: None,
                max_items: None,
            },
            description: None,
            default: None,
            rules: None,
        }
    }

    /// Create an object schema
    pub fn object(properties: HashMap<String, Schema>) -> Self {
        Self {
            schema_type: SchemaType::Object {
                properties,
                required: None,
            },
            description: None,
            default: None,
            rules: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set default value
    pub fn with_default(mut self, default: serde_json::Value) -> Self {
        self.default = Some(default);
        self
    }
}
