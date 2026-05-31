//! Tests for hudhudscript-tools-schema: schema validation logic

use hudhudscript_tools_schema::{
    validate_property_type, value_type_name, JsonSchema, JsonSchemaProperty, ValidationError,
};
use serde_json::json;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helper: build an object schema with given properties + required list
// ---------------------------------------------------------------------------
fn object_schema(props: Vec<(&str, &str)>, required: Vec<&str>) -> JsonSchema {
    let mut properties = HashMap::new();
    for (name, ty) in props {
        properties.insert(
            name.to_string(),
            JsonSchemaProperty {
                property_type: ty.to_string(),
                description: None,
                default: None,
                enum_values: None,
            },
        );
    }
    JsonSchema {
        schema_type: "object".to_string(),
        properties: Some(properties),
        required: if required.is_empty() {
            None
        } else {
            Some(required.into_iter().map(String::from).collect())
        },
        items: None,
        description: None,
    }
}

// ===== 1. Primitive type validation (string, number, integer, boolean, null) =====
#[test]
fn validate_primitive_types_happy_path() {
    let string_schema = JsonSchema {
        schema_type: "string".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(string_schema.validate(&json!("hello")).is_ok());

    let number_schema = JsonSchema {
        schema_type: "number".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(number_schema.validate(&json!(3.14)).is_ok());
    // integers also pass as numbers
    assert!(number_schema.validate(&json!(42)).is_ok());

    let integer_schema = JsonSchema {
        schema_type: "integer".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(integer_schema.validate(&json!(42)).is_ok());

    let bool_schema = JsonSchema {
        schema_type: "boolean".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(bool_schema.validate(&json!(true)).is_ok());

    let null_schema = JsonSchema {
        schema_type: "null".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(null_schema.validate(&json!(null)).is_ok());
}

// ===== 2. Primitive type mismatch errors =====
#[test]
fn validate_primitive_type_mismatch_returns_error() {
    let string_schema = JsonSchema {
        schema_type: "string".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    let err = string_schema.validate(&json!(123)).unwrap_err();
    match err {
        ValidationError::TypeMismatch { expected, found } => {
            assert_eq!(expected, "string");
            assert_eq!(found, "number");
        }
        other => panic!("Expected TypeMismatch, got: {:?}", other),
    }
}

// ===== 3. Object validation — required fields missing =====
#[test]
fn validate_object_missing_required_field() {
    let schema = object_schema(
        vec![("name", "string"), ("age", "integer")],
        vec!["name", "age"],
    );
    // Missing "age"
    let value = json!({"name": "Alice"});
    let err = schema.validate(&value).unwrap_err();
    match err {
        ValidationError::MissingRequired(field) => {
            assert_eq!(field, "age");
        }
        other => panic!("Expected MissingRequired, got: {:?}", other),
    }
}

// ===== 4. Object validation — property type mismatch =====
#[test]
fn validate_object_property_type_mismatch() {
    let schema = object_schema(vec![("count", "integer")], vec![]);
    // "count" is a string instead of integer
    let value = json!({"count": "not-a-number"});
    let err = schema.validate(&value).unwrap_err();
    match err {
        ValidationError::TypeMismatch { expected, found } => {
            assert_eq!(expected, "integer");
            assert_eq!(found, "string");
        }
        other => panic!("Expected TypeMismatch, got: {:?}", other),
    }
}

// ===== 5. Object validation — valid object passes =====
#[test]
fn validate_object_with_all_required_fields_passes() {
    let schema = object_schema(
        vec![("name", "string"), ("active", "boolean")],
        vec!["name"],
    );
    let value = json!({"name": "Bob", "active": true});
    assert!(schema.validate(&value).is_ok());
}

// ===== 6. Array validation with item schema =====
#[test]
fn validate_array_with_item_schema() {
    let items = JsonSchema {
        schema_type: "string".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    let schema = JsonSchema {
        schema_type: "array".to_string(),
        properties: None,
        required: None,
        items: Some(Box::new(items)),
        description: None,
    };

    // Valid
    assert!(schema.validate(&json!(["a", "b", "c"])).is_ok());

    // Invalid item
    let err = schema.validate(&json!(["a", 42])).unwrap_err();
    match err {
        ValidationError::TypeMismatch { expected, found } => {
            assert_eq!(expected, "string");
            assert_eq!(found, "number");
        }
        other => panic!("Expected TypeMismatch, got: {:?}", other),
    }
}

// ===== 7. Unknown schema type produces UnknownType error =====
#[test]
fn validate_unknown_schema_type() {
    let schema = JsonSchema {
        schema_type: "unicorn".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    let err = schema.validate(&json!("anything")).unwrap_err();
    match err {
        ValidationError::UnknownType(ty) => assert_eq!(ty, "unicorn"),
        other => panic!("Expected UnknownType, got: {:?}", other),
    }
}

// ===== 8. value_type_name returns correct names for all JSON types =====
#[test]
fn value_type_name_covers_all_json_types() {
    assert_eq!(value_type_name(&json!(null)), "null");
    assert_eq!(value_type_name(&json!(true)), "boolean");
    assert_eq!(value_type_name(&json!(1)), "number");
    assert_eq!(value_type_name(&json!("x")), "string");
    assert_eq!(value_type_name(&json!([])), "array");
    assert_eq!(value_type_name(&json!({})), "object");
}

// ===== 9. validate_property_type — unknown types pass through =====
#[test]
fn validate_property_type_unknown_type_passes() {
    // The function explicitly allows unknown types to pass
    assert!(validate_property_type(&json!("anything"), "custom_type").is_ok());
    assert!(validate_property_type(&json!(42), "fancy").is_ok());
}

// ===== 10. Non-object value against object schema =====
#[test]
fn validate_non_object_against_object_schema_fails() {
    let schema = object_schema(vec![], vec![]);
    let err = schema.validate(&json!("not an object")).unwrap_err();
    match err {
        ValidationError::TypeMismatch { expected, found } => {
            assert_eq!(expected, "object");
            assert_eq!(found, "string");
        }
        other => panic!("Expected TypeMismatch, got: {:?}", other),
    }
}
