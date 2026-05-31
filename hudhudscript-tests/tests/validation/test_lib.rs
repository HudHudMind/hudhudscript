//! Public API tests for hudhudscript-validation
//! Covers: validator.rs (Validator, register_validator, validate),
//!         schema.rs (Schema, SchemaType, ValidationRule),
//!         error.rs (ValidationError).

use hudhudscript_validation::{Schema, SchemaType, ValidationError, ValidationRule, Validator};
use serde_json::json;
use std::collections::HashMap;

// ── Validator construction ───────────────────────────────────────────

#[test]
fn validator_new_works() {
    let v = Validator::new();
    let schema = Schema::string();
    assert!(v.validate(&json!("hello"), &schema).is_ok());
}

#[test]
fn validator_default_is_same_as_new() {
    let v = Validator::default();
    let schema = Schema::string();
    assert!(v.validate(&json!("hi"), &schema).is_ok());
}

// ── String validation ───────────────────────────────────────────────

#[test]
fn validate_string_accepts_valid_string() {
    let v = Validator::new();
    assert!(v.validate(&json!("hello"), &Schema::string()).is_ok());
}

#[test]
fn validate_string_rejects_number() {
    let v = Validator::new();
    let err = v.validate(&json!(123), &Schema::string()).unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, ref found }
        if expected == "string" && found == "number")
    );
}

#[test]
fn validate_string_rejects_boolean() {
    let v = Validator::new();
    let err = v.validate(&json!(true), &Schema::string()).unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, .. } if expected == "string")
    );
}

#[test]
fn validate_string_rejects_null() {
    let v = Validator::new();
    let err = v.validate(&json!(null), &Schema::string()).unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, .. } if expected == "string")
    );
}

#[test]
fn validate_string_rejects_array() {
    let v = Validator::new();
    let err = v.validate(&json!(["a"]), &Schema::string()).unwrap_err();
    assert!(matches!(err, ValidationError::TypeMismatch { ref found, .. } if found == "array"));
}

#[test]
fn validate_string_rejects_object() {
    let v = Validator::new();
    let err = v.validate(&json!({"k": 1}), &Schema::string()).unwrap_err();
    assert!(matches!(err, ValidationError::TypeMismatch { ref found, .. } if found == "object"));
}

// ── String min_length ────────────────────────────────────────────────

#[test]
fn validate_string_min_length_accepts_exact_min() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: Some(3),
            max_length: None,
            pattern: None,
        },
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!("abc"), &schema).is_ok());
}

#[test]
fn validate_string_min_length_accepts_longer() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: Some(3),
            max_length: None,
            pattern: None,
        },
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!("abcdef"), &schema).is_ok());
}

#[test]
fn validate_string_min_length_rejects_shorter() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: Some(3),
            max_length: None,
            pattern: None,
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!("ab"), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::InvalidLength { ref expected, found }
        if expected == ">= 3" && found == 2)
    );
}

// ── String max_length ────────────────────────────────────────────────

#[test]
fn validate_string_max_length_accepts_exact_max() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: Some(5),
            pattern: None,
        },
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!("hello"), &schema).is_ok());
}

#[test]
fn validate_string_max_length_rejects_longer() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: Some(5),
            pattern: None,
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!("toolong"), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::InvalidLength { ref expected, found }
        if expected == "<= 5" && found == 7)
    );
}

// ── String pattern ───────────────────────────────────────────────────

#[test]
fn validate_string_pattern_accepts_matching() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: Some("^[a-z]+$".into()),
        },
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!("hello"), &schema).is_ok());
}

#[test]
fn validate_string_pattern_rejects_non_matching() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: Some("^[a-z]+$".into()),
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!("Hello123"), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::PatternMismatch { ref pattern } if pattern == "^[a-z]+$")
    );
}

#[test]
fn validate_string_invalid_regex_gives_invalid_format_error() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: Some("[invalid".into()),
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!("test"), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::InvalidFormat { ref message } if message.contains("Invalid regex"))
    );
}

#[test]
fn validate_string_email_pattern() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: Some(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".into()),
        },
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!("user@example.com"), &schema).is_ok());
    assert!(v.validate(&json!("not-an-email"), &schema).is_err());
}

// ── Number validation ────────────────────────────────────────────────

#[test]
fn validate_number_accepts_float() {
    let v = Validator::new();
    assert!(v.validate(&json!(42.5), &Schema::number()).is_ok());
}

#[test]
fn validate_number_accepts_integer_json() {
    let v = Validator::new();
    assert!(v.validate(&json!(10), &Schema::number()).is_ok());
}

#[test]
fn validate_number_rejects_string() {
    let v = Validator::new();
    let err = v
        .validate(&json!("not a number"), &Schema::number())
        .unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, .. } if expected == "number")
    );
}

#[test]
fn validate_number_min_rejects_below_min() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Number {
            min: Some(0.0),
            max: None,
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!(-1.0), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::OutOfRange { ref min, ref max, .. }
        if min == "0" && max.contains('\u{221e}'))
    );
}

#[test]
fn validate_number_max_rejects_above_max() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Number {
            min: None,
            max: Some(100.0),
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!(101.0), &schema).unwrap_err();
    assert!(matches!(err, ValidationError::OutOfRange { ref min, .. } if min.contains('\u{221e}')));
}

#[test]
fn validate_number_accepts_boundary_values() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Number {
            min: Some(0.0),
            max: Some(100.0),
        },
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!(0.0), &schema).is_ok());
    assert!(v.validate(&json!(100.0), &schema).is_ok());
    assert!(v.validate(&json!(50.5), &schema).is_ok());
}

// ── Integer validation ───────────────────────────────────────────────

#[test]
fn validate_integer_accepts_valid_int() {
    let v = Validator::new();
    assert!(v.validate(&json!(42), &Schema::integer()).is_ok());
}

#[test]
fn validate_integer_rejects_string() {
    let v = Validator::new();
    let err = v
        .validate(&json!("not an int"), &Schema::integer())
        .unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, ref found }
        if expected == "integer" && found == "string")
    );
}

#[test]
fn validate_integer_range_rejects_below_min() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Integer {
            min: Some(1),
            max: Some(10),
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!(0), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::OutOfRange { ref value, ref min, ref max }
        if value == "0" && min == "1" && max == "10")
    );
}

#[test]
fn validate_integer_range_rejects_above_max() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Integer {
            min: Some(1),
            max: Some(10),
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!(11), &schema).unwrap_err();
    assert!(matches!(err, ValidationError::OutOfRange { ref value, .. } if value == "11"));
}

#[test]
fn validate_integer_range_accepts_boundary_values() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Integer {
            min: Some(1),
            max: Some(10),
        },
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!(1), &schema).is_ok());
    assert!(v.validate(&json!(10), &schema).is_ok());
    assert!(v.validate(&json!(5), &schema).is_ok());
}

#[test]
fn validate_integer_no_range_accepts_any_integer() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Integer {
            min: None,
            max: None,
        },
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!(-999999), &schema).is_ok());
    assert!(v.validate(&json!(999999), &schema).is_ok());
}

// ── Boolean validation ───────────────────────────────────────────────

#[test]
fn validate_boolean_accepts_true_and_false() {
    let v = Validator::new();
    let schema = Schema::boolean();
    assert!(v.validate(&json!(true), &schema).is_ok());
    assert!(v.validate(&json!(false), &schema).is_ok());
}

#[test]
fn validate_boolean_rejects_string_true() {
    let v = Validator::new();
    let err = v.validate(&json!("true"), &Schema::boolean()).unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, .. } if expected == "boolean")
    );
}

#[test]
fn validate_boolean_rejects_integer_one() {
    let v = Validator::new();
    let err = v.validate(&json!(1), &Schema::boolean()).unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, .. } if expected == "boolean")
    );
}

// ── Array validation ─────────────────────────────────────────────────

#[test]
fn validate_array_accepts_homogeneous_array() {
    let v = Validator::new();
    let schema = Schema::array(SchemaType::Number {
        min: None,
        max: None,
    });
    assert!(v.validate(&json!([1, 2, 3]), &schema).is_ok());
}

#[test]
fn validate_array_rejects_mixed_types() {
    let v = Validator::new();
    let schema = Schema::array(SchemaType::Number {
        min: None,
        max: None,
    });
    assert!(v.validate(&json!([1, "two", 3]), &schema).is_err());
}

#[test]
fn validate_array_rejects_non_array() {
    let v = Validator::new();
    let schema = Schema::array(SchemaType::Number {
        min: None,
        max: None,
    });
    let err = v.validate(&json!("not array"), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, ref found }
        if expected == "array" && found == "string")
    );
}

#[test]
fn validate_array_min_items_rejects_too_few() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Array {
            items: Box::new(SchemaType::Number {
                min: None,
                max: None,
            }),
            min_items: Some(2),
            max_items: None,
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!([1]), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::InvalidLength { ref expected, found }
        if expected == ">= 2" && found == 1)
    );
}

#[test]
fn validate_array_max_items_rejects_too_many() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Array {
            items: Box::new(SchemaType::Number {
                min: None,
                max: None,
            }),
            min_items: None,
            max_items: Some(2),
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!([1, 2, 3]), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::InvalidLength { ref expected, found }
        if expected == "<= 2" && found == 3)
    );
}

#[test]
fn validate_array_accepts_empty_with_no_min() {
    let v = Validator::new();
    let schema = Schema::array(SchemaType::Number {
        min: None,
        max: None,
    });
    assert!(v.validate(&json!([]), &schema).is_ok());
}

#[test]
fn validate_array_of_strings() {
    let v = Validator::new();
    let schema = Schema::array(SchemaType::String {
        min_length: None,
        max_length: None,
        pattern: None,
    });
    assert!(v.validate(&json!(["a", "b", "c"]), &schema).is_ok());
    assert!(v.validate(&json!(["a", 1]), &schema).is_err());
}

// ── Object validation ────────────────────────────────────────────────

#[test]
fn validate_object_accepts_valid_object() {
    let v = Validator::new();
    let mut props = HashMap::new();
    props.insert("name".into(), Schema::string());
    props.insert("age".into(), Schema::integer());
    let schema = Schema::object(props);
    assert!(v
        .validate(&json!({"name": "Alice", "age": 30}), &schema)
        .is_ok());
}

#[test]
fn validate_object_rejects_wrong_property_type() {
    let v = Validator::new();
    let mut props = HashMap::new();
    props.insert("name".into(), Schema::string());
    props.insert("age".into(), Schema::integer());
    let schema = Schema::object(props);
    assert!(v
        .validate(&json!({"name": "Bob", "age": "thirty"}), &schema)
        .is_err());
}

#[test]
fn validate_object_rejects_non_object() {
    let v = Validator::new();
    let schema = Schema::object(HashMap::new());
    let err = v.validate(&json!("not object"), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, ref found }
        if expected == "object" && found == "string")
    );
}

#[test]
fn validate_object_required_field_missing() {
    let v = Validator::new();
    let mut props = HashMap::new();
    props.insert("name".into(), Schema::string());
    let schema = Schema {
        schema_type: SchemaType::Object {
            properties: props,
            required: Some(vec!["name".into()]),
        },
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!({}), &schema).unwrap_err();
    assert!(matches!(err, ValidationError::RequiredFieldMissing { ref field } if field == "name"));
}

#[test]
fn validate_object_required_field_present_passes() {
    let v = Validator::new();
    let mut props = HashMap::new();
    props.insert("name".into(), Schema::string());
    let schema = Schema {
        schema_type: SchemaType::Object {
            properties: props,
            required: Some(vec!["name".into()]),
        },
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!({"name": "Alice"}), &schema).is_ok());
}

#[test]
fn validate_object_extra_fields_allowed() {
    let v = Validator::new();
    let mut props = HashMap::new();
    props.insert("name".into(), Schema::string());
    let schema = Schema::object(props);
    // Extra fields not in schema properties are ignored
    assert!(v
        .validate(&json!({"name": "Alice", "extra": 42}), &schema)
        .is_ok());
}

// ── Null validation ──────────────────────────────────────────────────

#[test]
fn validate_null_accepts_json_null() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Null,
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!(null), &schema).is_ok());
}

#[test]
fn validate_null_rejects_string() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Null,
        description: None,
        default: None,
        rules: None,
    };
    let err = v.validate(&json!("not null"), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::TypeMismatch { ref expected, .. } if expected == "null")
    );
}

#[test]
fn validate_null_rejects_zero() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Null,
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!(0), &schema).is_err());
}

// ── Any validation ───────────────────────────────────────────────────

#[test]
fn validate_any_accepts_all_types() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Any,
        description: None,
        default: None,
        rules: None,
    };
    assert!(v.validate(&json!(null), &schema).is_ok());
    assert!(v.validate(&json!("anything"), &schema).is_ok());
    assert!(v.validate(&json!(42), &schema).is_ok());
    assert!(v.validate(&json!([1, 2, 3]), &schema).is_ok());
    assert!(v.validate(&json!({"key": "val"}), &schema).is_ok());
    assert!(v.validate(&json!(true), &schema).is_ok());
}

// ── Custom validator ─────────────────────────────────────────────────

#[test]
fn custom_validator_invoked_and_passes() {
    let mut v = Validator::new();
    v.register_validator("even_check".into(), |val| {
        if let Some(n) = val.as_i64() {
            if n % 2 == 0 {
                return Ok(());
            }
        }
        Err(ValidationError::Custom {
            message: "Value must be even".into(),
        })
    });

    let schema = Schema {
        schema_type: SchemaType::Integer {
            min: None,
            max: None,
        },
        description: None,
        default: None,
        rules: Some(vec![ValidationRule {
            name: "even".into(),
            description: None,
            validator: Some("even_check".into()),
        }]),
    };
    assert!(v.validate(&json!(4), &schema).is_ok());
}

#[test]
fn custom_validator_invoked_and_fails() {
    let mut v = Validator::new();
    v.register_validator("even_check".into(), |val| {
        if let Some(n) = val.as_i64() {
            if n % 2 == 0 {
                return Ok(());
            }
        }
        Err(ValidationError::Custom {
            message: "Value must be even".into(),
        })
    });

    let schema = Schema {
        schema_type: SchemaType::Integer {
            min: None,
            max: None,
        },
        description: None,
        default: None,
        rules: Some(vec![ValidationRule {
            name: "even".into(),
            description: None,
            validator: Some("even_check".into()),
        }]),
    };
    let err = v.validate(&json!(3), &schema).unwrap_err();
    assert!(
        matches!(err, ValidationError::Custom { ref message } if message == "Value must be even")
    );
}

#[test]
fn unregistered_custom_validator_is_silently_skipped() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: None,
        },
        description: None,
        default: None,
        rules: Some(vec![ValidationRule {
            name: "unregistered".into(),
            description: None,
            validator: Some("does_not_exist".into()),
        }]),
    };
    // Should pass since unregistered validator is skipped
    assert!(v.validate(&json!("hello"), &schema).is_ok());
}

#[test]
fn rule_without_validator_field_is_skipped() {
    let v = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: None,
        },
        description: None,
        default: None,
        rules: Some(vec![ValidationRule {
            name: "info-only".into(),
            description: Some("Just metadata".into()),
            validator: None,
        }]),
    };
    assert!(v.validate(&json!("hello"), &schema).is_ok());
}

// ── ValidationError display ──────────────────────────────────────────

#[test]
fn validation_error_type_mismatch_display() {
    let e = ValidationError::TypeMismatch {
        expected: "number".into(),
        found: "string".into(),
    };
    assert!(e.to_string().contains("Type mismatch"));
    assert!(e.to_string().contains("number"));
    assert!(e.to_string().contains("string"));
}

#[test]
fn validation_error_out_of_range_display() {
    let e = ValidationError::OutOfRange {
        value: "15".into(),
        min: "0".into(),
        max: "10".into(),
    };
    assert_eq!(e.to_string(), "Value out of range: 15 not in [0, 10]");
}

#[test]
fn validation_error_pattern_mismatch_display() {
    let e = ValidationError::PatternMismatch {
        pattern: "^[a-z]+$".into(),
    };
    assert!(e.to_string().contains("^[a-z]+$"));
}

#[test]
fn validation_error_required_field_missing_display() {
    let e = ValidationError::RequiredFieldMissing {
        field: "email".into(),
    };
    assert!(e.to_string().contains("email"));
    assert!(e.to_string().contains("Required field missing"));
}

#[test]
fn validation_error_invalid_length_display() {
    let e = ValidationError::InvalidLength {
        expected: ">= 5".into(),
        found: 3,
    };
    assert_eq!(e.to_string(), "Invalid length: expected >= 5, found 3");
}

#[test]
fn validation_error_invalid_format_display() {
    let e = ValidationError::InvalidFormat {
        message: "bad regex".into(),
    };
    assert_eq!(e.to_string(), "Invalid format: bad regex");
}

#[test]
fn validation_error_custom_display() {
    let e = ValidationError::Custom {
        message: "custom error".into(),
    };
    assert_eq!(e.to_string(), "Validation failed: custom error");
}

#[test]
fn validation_error_equality() {
    let e1 = ValidationError::TypeMismatch {
        expected: "string".into(),
        found: "number".into(),
    };
    let e2 = ValidationError::TypeMismatch {
        expected: "string".into(),
        found: "number".into(),
    };
    assert_eq!(e1, e2);
}

#[test]
fn validation_error_clone_equals_original() {
    let e = ValidationError::Custom {
        message: "cloned".into(),
    };
    assert_eq!(e.clone(), e);
}

// ── Schema construction helpers ──────────────────────────────────────

#[test]
fn schema_string_has_string_type() {
    let s = Schema::string();
    assert!(matches!(
        s.schema_type,
        SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: None
        }
    ));
    assert!(s.description.is_none());
    assert!(s.default.is_none());
    assert!(s.rules.is_none());
}

#[test]
fn schema_number_has_number_type() {
    let s = Schema::number();
    assert!(matches!(
        s.schema_type,
        SchemaType::Number {
            min: None,
            max: None
        }
    ));
}

#[test]
fn schema_integer_has_integer_type() {
    let s = Schema::integer();
    assert!(matches!(
        s.schema_type,
        SchemaType::Integer {
            min: None,
            max: None
        }
    ));
}

#[test]
fn schema_boolean_has_boolean_type() {
    let s = Schema::boolean();
    assert!(matches!(s.schema_type, SchemaType::Boolean));
}

#[test]
fn schema_array_wraps_items_type() {
    let s = Schema::array(SchemaType::Number {
        min: None,
        max: None,
    });
    assert!(matches!(s.schema_type, SchemaType::Array { .. }));
}

#[test]
fn schema_object_stores_properties() {
    let mut props = HashMap::new();
    props.insert("key".into(), Schema::string());
    let s = Schema::object(props);
    if let SchemaType::Object { ref properties, .. } = s.schema_type {
        assert!(properties.contains_key("key"));
    } else {
        panic!("Expected Object type");
    }
}

#[test]
fn schema_with_description_sets_field() {
    let s = Schema::string().with_description("A name field");
    assert_eq!(s.description, Some("A name field".into()));
}

#[test]
fn schema_with_default_sets_field() {
    let s = Schema::string().with_default(json!("default_val"));
    assert_eq!(s.default, Some(json!("default_val")));
}

#[test]
fn schema_serialization_roundtrip() {
    let s = Schema::string()
        .with_description("test")
        .with_default(json!("def"));
    let json_str = serde_json::to_string(&s).unwrap();
    let back: Schema = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.description, Some("test".into()));
    assert_eq!(back.default, Some(json!("def")));
}

#[test]
fn validation_rule_serde_roundtrip() {
    let rule = ValidationRule {
        name: "my_rule".into(),
        description: Some("A test rule".into()),
        validator: Some("custom_fn".into()),
    };
    let json_str = serde_json::to_string(&rule).unwrap();
    let back: ValidationRule = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.name, "my_rule");
    assert_eq!(back.validator.as_deref(), Some("custom_fn"));
}
