// Extracted from hudhudscript-validation inline #[cfg(test)] blocks
use hudhudscript_validation::{Schema, SchemaType, ValidationError, ValidationRule, Validator};
use serde_json::json;
use std::collections::HashMap;

// === from lib.rs ===

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

// === from error.rs ===

#[test]
fn test_error_display() {
    let err = ValidationError::TypeMismatch {
        expected: "number".to_string(),
        found: "string".to_string(),
    };
    assert!(err.to_string().contains("Type mismatch"));

    let err = ValidationError::RequiredFieldMissing {
        field: "name".to_string(),
    };
    assert!(err.to_string().contains("Required field missing"));
}

#[test]
fn test_error_display_out_of_range() {
    let err = ValidationError::OutOfRange {
        value: "15".to_string(),
        min: "0".to_string(),
        max: "10".to_string(),
    };
    assert_eq!(err.to_string(), "Value out of range: 15 not in [0, 10]");
}

#[test]
fn test_error_display_pattern_mismatch() {
    let err = ValidationError::PatternMismatch {
        pattern: "^[a-z]+$".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "Pattern mismatch: value does not match pattern ^[a-z]+$"
    );
}

#[test]
fn test_error_display_invalid_length() {
    let err = ValidationError::InvalidLength {
        expected: ">= 5".to_string(),
        found: 3,
    };
    assert_eq!(err.to_string(), "Invalid length: expected >= 5, found 3");
}

#[test]
fn test_error_display_invalid_format() {
    let err = ValidationError::InvalidFormat {
        message: "bad regex".to_string(),
    };
    assert_eq!(err.to_string(), "Invalid format: bad regex");
}

#[test]
fn test_error_display_custom() {
    let err = ValidationError::Custom {
        message: "custom error".to_string(),
    };
    assert_eq!(err.to_string(), "Validation failed: custom error");
}

#[test]
fn test_error_equality() {
    let err1 = ValidationError::TypeMismatch {
        expected: "string".to_string(),
        found: "number".to_string(),
    };
    let err2 = ValidationError::TypeMismatch {
        expected: "string".to_string(),
        found: "number".to_string(),
    };
    assert_eq!(err1, err2);
}

#[test]
fn test_error_clone() {
    let err = ValidationError::Custom {
        message: "cloned".to_string(),
    };
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

// === from sanitizer.rs ===

#[test]
fn test_sanitize_string() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();

    assert_eq!(sanitizer.sanitize_string("  hello  "), "hello");
    assert_eq!(sanitizer.sanitize_string("hello\x00world"), "helloworld");
}

#[test]
fn test_sanitize_html() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();

    assert_eq!(
        sanitizer.sanitize_html("<script>alert('xss')</script>"),
        "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;"
    );
}

#[test]
fn test_sanitize_sql() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();

    assert_eq!(
        sanitizer.sanitize_sql("'; DROP TABLE users; --"),
        "''; DROP TABLE users; --"
    );
}

#[test]
fn test_sanitize_json() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();

    let input = json!({
        "name": "  Alice  ",
        "bio": "<script>alert('xss')</script>"
    });

    let output = sanitizer.sanitize_json(&input);
    assert_eq!(output["name"], "Alice");
}

#[test]
fn test_normalize_whitespace() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();

    assert_eq!(
        sanitizer.normalize_whitespace("hello   world  \n  test"),
        "hello world test"
    );
}

#[test]
fn test_truncate() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();

    assert_eq!(sanitizer.truncate("hello world", 5), "hello");
    assert_eq!(sanitizer.truncate("hi", 10), "hi");
}

#[test]
fn test_sanitizer_default() {
    use hudhudscript_validation::Sanitizer;
    let s = Sanitizer::default();
    assert_eq!(s.sanitize_string("  test  "), "test");
}

#[test]
fn test_sanitize_string_preserves_newlines_tabs() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    let result = sanitizer.sanitize_string("line1\nline2\tindented");
    assert_eq!(result, "line1\nline2\tindented");
}

#[test]
fn test_sanitize_string_removes_control_chars() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    let input = "hello\x01\x02world\x03";
    assert_eq!(sanitizer.sanitize_string(input), "helloworld");
}

#[test]
fn test_remove_null_bytes() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    assert_eq!(sanitizer.remove_null_bytes("a\0b\0c"), "abc");
    assert_eq!(sanitizer.remove_null_bytes("clean"), "clean");
}

#[test]
fn test_sanitize_html_ampersand() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    assert_eq!(sanitizer.sanitize_html("a & b"), "a &amp; b");
}

#[test]
fn test_sanitize_html_quotes() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    assert_eq!(
        sanitizer.sanitize_html("attr=\"val\""),
        "attr=&quot;val&quot;"
    );
}

#[test]
fn test_sanitize_sql_backslash() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    assert_eq!(sanitizer.sanitize_sql("a\\b"), "a\\\\b");
}

#[test]
fn test_sanitize_sql_null_bytes() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    assert_eq!(sanitizer.sanitize_sql("before\0after"), "beforeafter");
}

#[test]
fn test_sanitize_json_array() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    let input = json!(["  hello  ", "  world  "]);
    let output = sanitizer.sanitize_json(&input);
    assert_eq!(output[0], "hello");
    assert_eq!(output[1], "world");
}

#[test]
fn test_sanitize_json_nested_object() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    let input = json!({
        "outer": {
            "inner": "  nested  "
        }
    });
    let output = sanitizer.sanitize_json(&input);
    assert_eq!(output["outer"]["inner"], "nested");
}

#[test]
fn test_sanitize_json_preserves_non_string() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    let input = json!({
        "num": 42,
        "bool": true,
        "null_val": null
    });
    let output = sanitizer.sanitize_json(&input);
    assert_eq!(output["num"], 42);
    assert_eq!(output["bool"], true);
    assert!(output["null_val"].is_null());
}

#[test]
fn test_truncate_exact_length() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    assert_eq!(sanitizer.truncate("hello", 5), "hello");
}

#[test]
fn test_truncate_empty_string() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    assert_eq!(sanitizer.truncate("", 10), "");
}

#[test]
fn test_normalize_whitespace_empty() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    assert_eq!(sanitizer.normalize_whitespace(""), "");
}

#[test]
fn test_normalize_whitespace_tabs_and_newlines() {
    use hudhudscript_validation::Sanitizer;
    let sanitizer = Sanitizer::new();
    assert_eq!(sanitizer.normalize_whitespace("a\t\tb\n\nc"), "a b c");
}

// === from schema.rs ===

#[test]
fn test_schema_creation() {
    let schema = Schema::string();
    assert!(matches!(schema.schema_type, SchemaType::String { .. }));

    let schema = Schema::number();
    assert!(matches!(schema.schema_type, SchemaType::Number { .. }));

    let schema = Schema::boolean();
    assert!(matches!(schema.schema_type, SchemaType::Boolean));
}

#[test]
fn test_schema_with_description() {
    let schema = Schema::string().with_description("A test string");
    assert_eq!(schema.description, Some("A test string".to_string()));
}

#[test]
fn test_schema_serialization() {
    let schema = Schema::string();
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("string"));
}

#[test]
fn test_schema_integer() {
    let schema = Schema::integer();
    match &schema.schema_type {
        SchemaType::Integer { min, max } => {
            assert_eq!(*min, None);
            assert_eq!(*max, None);
        }
        _ => panic!("Expected Integer type"),
    }
}

#[test]
fn test_schema_array() {
    let schema = Schema::array(SchemaType::String {
        min_length: None,
        max_length: None,
        pattern: None,
    });
    match &schema.schema_type {
        SchemaType::Array {
            items,
            min_items,
            max_items,
        } => {
            assert!(matches!(items.as_ref(), SchemaType::String { .. }));
            assert_eq!(*min_items, None);
            assert_eq!(*max_items, None);
        }
        _ => panic!("Expected Array type"),
    }
}

#[test]
fn test_schema_object() {
    let mut props = HashMap::new();
    props.insert("key".to_string(), Schema::string());
    let schema = Schema::object(props);
    match &schema.schema_type {
        SchemaType::Object {
            properties,
            required,
        } => {
            assert_eq!(properties.len(), 1);
            assert!(properties.contains_key("key"));
            assert_eq!(*required, None);
        }
        _ => panic!("Expected Object type"),
    }
}

#[test]
fn test_schema_with_default() {
    let schema = Schema::string().with_default(serde_json::json!("default_val"));
    assert_eq!(schema.default, Some(serde_json::json!("default_val")));
}

#[test]
fn test_schema_number() {
    let schema = Schema::number();
    match &schema.schema_type {
        SchemaType::Number { min, max } => {
            assert_eq!(*min, None);
            assert_eq!(*max, None);
        }
        _ => panic!("Expected Number type"),
    }
    assert_eq!(schema.description, None);
    assert_eq!(schema.default, None);
    assert_eq!(schema.rules, None);
}

#[test]
fn test_schema_boolean() {
    let schema = Schema::boolean();
    assert!(matches!(schema.schema_type, SchemaType::Boolean));
}

#[test]
fn test_validation_rule_serde() {
    let rule = ValidationRule {
        name: "my_rule".to_string(),
        description: Some("A test rule".to_string()),
        validator: Some("custom_fn".to_string()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: ValidationRule = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "my_rule");
    assert_eq!(deserialized.description, Some("A test rule".to_string()));
    assert_eq!(deserialized.validator, Some("custom_fn".to_string()));
}

#[test]
fn test_schema_roundtrip_serde() {
    let schema = Schema::string()
        .with_description("A name field")
        .with_default(serde_json::json!("unnamed"));

    let json = serde_json::to_string(&schema).unwrap();
    let deserialized: Schema = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.description, Some("A name field".to_string()));
    assert_eq!(deserialized.default, Some(serde_json::json!("unnamed")));
    assert!(matches!(
        deserialized.schema_type,
        SchemaType::String { .. }
    ));
}

// === from validator.rs ===

#[test]
fn test_validate_string() {
    let validator = Validator::new();
    let schema = Schema::string();

    assert!(validator.validate(&json!("hello"), &schema).is_ok());
    assert!(validator.validate(&json!(123), &schema).is_err());
}

#[test]
fn test_validate_number() {
    let validator = Validator::new();
    let schema = Schema::number();

    assert!(validator.validate(&json!(42.5), &schema).is_ok());
    assert!(validator.validate(&json!("not a number"), &schema).is_err());
}

#[test]
fn test_validate_boolean() {
    let validator = Validator::new();
    let schema = Schema::boolean();

    assert!(validator.validate(&json!(true), &schema).is_ok());
    assert!(validator.validate(&json!(false), &schema).is_ok());
    assert!(validator.validate(&json!("true"), &schema).is_err());
}

#[test]
fn test_validate_array() {
    let validator = Validator::new();
    let schema = Schema::array(SchemaType::Number {
        min: None,
        max: None,
    });

    assert!(validator.validate(&json!([1, 2, 3]), &schema).is_ok());
    assert!(validator.validate(&json!([1, "two", 3]), &schema).is_err());
}

#[test]
fn test_validate_object() {
    let validator = Validator::new();
    let mut properties = HashMap::new();
    properties.insert("name".to_string(), Schema::string());
    properties.insert("age".to_string(), Schema::integer());

    let schema = Schema::object(properties);

    assert!(validator
        .validate(&json!({"name": "Alice", "age": 30}), &schema)
        .is_ok());
    assert!(validator
        .validate(&json!({"name": "Bob", "age": "thirty"}), &schema)
        .is_err());
}

#[test]
fn test_validate_null() {
    let validator = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Null,
        description: None,
        default: None,
        rules: None,
    };

    assert!(validator.validate(&json!(null), &schema).is_ok());
    assert!(validator.validate(&json!("not null"), &schema).is_err());
    assert!(validator.validate(&json!(0), &schema).is_err());
}

#[test]
fn test_validate_any() {
    let validator = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Any,
        description: None,
        default: None,
        rules: None,
    };

    assert!(validator.validate(&json!(null), &schema).is_ok());
    assert!(validator.validate(&json!("anything"), &schema).is_ok());
    assert!(validator.validate(&json!(42), &schema).is_ok());
    assert!(validator.validate(&json!([1, 2, 3]), &schema).is_ok());
}

#[test]
fn test_validate_string_min_length() {
    let validator = Validator::new();
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

    assert!(validator.validate(&json!("abc"), &schema).is_ok());
    assert!(validator.validate(&json!("abcd"), &schema).is_ok());
    let err = validator.validate(&json!("ab"), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::InvalidLength { expected, found } => {
            assert_eq!(expected, ">= 3");
            assert_eq!(found, 2);
        }
        _ => panic!("Expected InvalidLength error"),
    }
}

#[test]
fn test_validate_string_max_length() {
    let validator = Validator::new();
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

    assert!(validator.validate(&json!("hello"), &schema).is_ok());
    let err = validator.validate(&json!("toolong"), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::InvalidLength { expected, found } => {
            assert_eq!(expected, "<= 5");
            assert_eq!(found, 7);
        }
        _ => panic!("Expected InvalidLength error"),
    }
}

#[test]
fn test_validate_string_pattern() {
    let validator = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: Some("^[a-z]+$".to_string()),
        },
        description: None,
        default: None,
        rules: None,
    };

    assert!(validator.validate(&json!("hello"), &schema).is_ok());
    let err = validator.validate(&json!("Hello123"), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::PatternMismatch { pattern } => {
            assert_eq!(pattern, "^[a-z]+$");
        }
        _ => panic!("Expected PatternMismatch error"),
    }
}

#[test]
fn test_validate_number_min() {
    let validator = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Number {
            min: Some(0.0),
            max: None,
        },
        description: None,
        default: None,
        rules: None,
    };

    assert!(validator.validate(&json!(0.0), &schema).is_ok());
    assert!(validator.validate(&json!(100.5), &schema).is_ok());
    let err = validator.validate(&json!(-1.0), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::OutOfRange { value, min, max } => {
            assert_eq!(value, "-1");
            assert_eq!(min, "0");
            assert_eq!(max, "\u{221e}"); // infinity symbol
        }
        _ => panic!("Expected OutOfRange error"),
    }
}

#[test]
fn test_validate_number_max() {
    let validator = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Number {
            min: None,
            max: Some(100.0),
        },
        description: None,
        default: None,
        rules: None,
    };

    assert!(validator.validate(&json!(100.0), &schema).is_ok());
    let err = validator.validate(&json!(101.0), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::OutOfRange { min, .. } => {
            assert_eq!(min, "-\u{221e}");
        }
        _ => panic!("Expected OutOfRange error"),
    }
}

#[test]
fn test_validate_integer_range() {
    let validator = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::Integer {
            min: Some(1),
            max: Some(10),
        },
        description: None,
        default: None,
        rules: None,
    };

    assert!(validator.validate(&json!(1), &schema).is_ok());
    assert!(validator.validate(&json!(10), &schema).is_ok());
    assert!(validator.validate(&json!(5), &schema).is_ok());

    let err_min = validator.validate(&json!(0), &schema);
    assert!(err_min.is_err());
    match err_min.unwrap_err() {
        ValidationError::OutOfRange { value, min, max } => {
            assert_eq!(value, "0");
            assert_eq!(min, "1");
            assert_eq!(max, "10");
        }
        _ => panic!("Expected OutOfRange"),
    }

    let err_max = validator.validate(&json!(11), &schema);
    assert!(err_max.is_err());
    match err_max.unwrap_err() {
        ValidationError::OutOfRange { value, min, max } => {
            assert_eq!(value, "11");
            assert_eq!(min, "1");
            assert_eq!(max, "10");
        }
        _ => panic!("Expected OutOfRange"),
    }
}

#[test]
fn test_validate_integer_type_mismatch() {
    let validator = Validator::new();
    let schema = Schema::integer();
    let err = validator.validate(&json!("not an int"), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::TypeMismatch { expected, found } => {
            assert_eq!(expected, "integer");
            assert_eq!(found, "string");
        }
        _ => panic!("Expected TypeMismatch"),
    }
}

#[test]
fn test_validate_array_min_items() {
    let validator = Validator::new();
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

    assert!(validator.validate(&json!([1, 2]), &schema).is_ok());
    let err = validator.validate(&json!([1]), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::InvalidLength { expected, found } => {
            assert_eq!(expected, ">= 2");
            assert_eq!(found, 1);
        }
        _ => panic!("Expected InvalidLength"),
    }
}

#[test]
fn test_validate_array_max_items() {
    let validator = Validator::new();
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

    assert!(validator.validate(&json!([1, 2]), &schema).is_ok());
    let err = validator.validate(&json!([1, 2, 3]), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::InvalidLength { expected, found } => {
            assert_eq!(expected, "<= 2");
            assert_eq!(found, 3);
        }
        _ => panic!("Expected InvalidLength"),
    }
}

#[test]
fn test_validate_array_type_mismatch() {
    let validator = Validator::new();
    let schema = Schema::array(SchemaType::Number {
        min: None,
        max: None,
    });
    let err = validator.validate(&json!("not array"), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::TypeMismatch { expected, found } => {
            assert_eq!(expected, "array");
            assert_eq!(found, "string");
        }
        _ => panic!("Expected TypeMismatch"),
    }
}

#[test]
fn test_validate_object_required_field() {
    let validator = Validator::new();
    let mut properties = HashMap::new();
    properties.insert("name".to_string(), Schema::string());

    let schema = Schema {
        schema_type: SchemaType::Object {
            properties,
            required: Some(vec!["name".to_string()]),
        },
        description: None,
        default: None,
        rules: None,
    };

    assert!(validator
        .validate(&json!({"name": "Alice"}), &schema)
        .is_ok());
    let err = validator.validate(&json!({}), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::RequiredFieldMissing { field } => {
            assert_eq!(field, "name");
        }
        _ => panic!("Expected RequiredFieldMissing"),
    }
}

#[test]
fn test_validate_object_type_mismatch() {
    let validator = Validator::new();
    let schema = Schema::object(HashMap::new());
    let err = validator.validate(&json!("not object"), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::TypeMismatch { expected, found } => {
            assert_eq!(expected, "object");
            assert_eq!(found, "string");
        }
        _ => panic!("Expected TypeMismatch"),
    }
}

#[test]
fn test_custom_validator() {
    let mut validator = Validator::new();
    validator.register_validator("even_check".to_string(), |val| {
        if let Some(n) = val.as_i64() {
            if n % 2 == 0 {
                return Ok(());
            }
        }
        Err(ValidationError::Custom {
            message: "Value must be even".to_string(),
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
            name: "even".to_string(),
            description: None,
            validator: Some("even_check".to_string()),
        }]),
    };

    assert!(validator.validate(&json!(4), &schema).is_ok());
    let err = validator.validate(&json!(3), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::Custom { message } => {
            assert_eq!(message, "Value must be even");
        }
        _ => panic!("Expected Custom error"),
    }
}

#[test]
fn test_custom_validator_not_registered() {
    let validator = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: None,
        },
        description: None,
        default: None,
        rules: Some(vec![ValidationRule {
            name: "unregistered".to_string(),
            description: None,
            validator: Some("does_not_exist".to_string()),
        }]),
    };

    // Should pass since the validator is not registered, it's simply skipped
    assert!(validator.validate(&json!("hello"), &schema).is_ok());
}

#[test]
fn test_validator_default() {
    let v = Validator::default();
    let schema = Schema::string();
    assert!(v.validate(&json!("test"), &schema).is_ok());
}

#[test]
fn test_type_name_all_variants() {
    assert_eq!(Validator::type_name(&json!(null)), "null");
    assert_eq!(Validator::type_name(&json!(true)), "boolean");
    assert_eq!(Validator::type_name(&json!(42)), "number");
    assert_eq!(Validator::type_name(&json!("hi")), "string");
    assert_eq!(Validator::type_name(&json!([1])), "array");
    assert_eq!(Validator::type_name(&json!({"a": 1})), "object");
}

#[test]
fn test_validate_string_invalid_regex_pattern() {
    let validator = Validator::new();
    let schema = Schema {
        schema_type: SchemaType::String {
            min_length: None,
            max_length: None,
            pattern: Some("[invalid".to_string()),
        },
        description: None,
        default: None,
        rules: None,
    };

    let err = validator.validate(&json!("test"), &schema);
    assert!(err.is_err());
    match err.unwrap_err() {
        ValidationError::InvalidFormat { message } => {
            assert!(message.contains("Invalid regex pattern"));
        }
        _ => panic!("Expected InvalidFormat error"),
    }
}
