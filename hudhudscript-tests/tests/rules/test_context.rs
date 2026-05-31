//! External tests for hudhudscript_rules::context

use hudhudscript_rules::context::EvaluationContext;
use serde_json::{json, Value};
use std::collections::HashMap;

#[test]
fn test_new_context() {
    let context = EvaluationContext::new();
    assert!(context.is_empty());
    assert_eq!(context.len(), 0);
}

#[test]
fn test_from_map() {
    let mut fields = HashMap::new();
    fields.insert("status".to_string(), json!("active"));
    fields.insert("count".to_string(), json!(42));

    let context = EvaluationContext::from_map(fields);
    assert_eq!(context.len(), 2);
    assert!(context.contains_key("status"));
    assert!(context.contains_key("count"));
}

#[test]
fn test_insert_and_get() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    assert_eq!(context.get("status"), Some(&json!("active")));
    assert_eq!(context.get("missing"), None);
}

#[test]
fn test_contains_key() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    assert!(context.contains_key("status"));
    assert!(!context.contains_key("missing"));
}

#[test]
fn test_get_string() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));
    context.insert("count".to_string(), json!(42));

    assert_eq!(context.get_string("status"), Some("active"));
    assert_eq!(context.get_string("count"), None);
    assert_eq!(context.get_string("missing"), None);
}

#[test]
fn test_get_number() {
    let mut context = EvaluationContext::new();
    context.insert("count".to_string(), json!(42));
    context.insert("price".to_string(), json!(19.99));
    context.insert("status".to_string(), json!("active"));

    assert_eq!(context.get_number("count"), Some(42.0));
    assert_eq!(context.get_number("price"), Some(19.99));
    assert_eq!(context.get_number("status"), None);
    assert_eq!(context.get_number("missing"), None);
}

#[test]
fn test_get_bool() {
    let mut context = EvaluationContext::new();
    context.insert("active".to_string(), json!(true));
    context.insert("disabled".to_string(), json!(false));
    context.insert("count".to_string(), json!(42));

    assert_eq!(context.get_bool("active"), Some(true));
    assert_eq!(context.get_bool("disabled"), Some(false));
    assert_eq!(context.get_bool("count"), None);
    assert_eq!(context.get_bool("missing"), None);
}

#[test]
fn test_get_array() {
    let mut context = EvaluationContext::new();
    context.insert("tags".to_string(), json!(["rust", "programming"]));
    context.insert("count".to_string(), json!(42));

    assert_eq!(context.get_array("tags").map(|a| a.len()), Some(2));
    assert_eq!(context.get_array("count"), None);
    assert_eq!(context.get_array("missing"), None);
}

#[test]
fn test_get_object() {
    let mut context = EvaluationContext::new();
    context.insert("user".to_string(), json!({"name": "Alice", "age": 30}));
    context.insert("count".to_string(), json!(42));

    assert!(context.get_object("user").is_some());
    assert_eq!(
        context.get_object("user").and_then(|o| o.get("name")),
        Some(&json!("Alice"))
    );
    assert_eq!(context.get_object("count"), None);
    assert_eq!(context.get_object("missing"), None);
}

#[test]
fn test_validate_field_type() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));
    context.insert("count".to_string(), json!(42));
    context.insert("active".to_string(), json!(true));
    context.insert("tags".to_string(), json!(["rust", "programming"]));
    context.insert("user".to_string(), json!({"name": "Alice"}));

    assert!(context.validate_field_type("status", "string").is_ok());
    assert!(context.validate_field_type("count", "number").is_ok());
    assert!(context.validate_field_type("active", "boolean").is_ok());
    assert!(context.validate_field_type("tags", "array").is_ok());
    assert!(context.validate_field_type("user", "object").is_ok());

    assert!(context.validate_field_type("status", "number").is_err());
    assert!(context.validate_field_type("count", "string").is_err());
    assert!(context.validate_field_type("missing", "string").is_err());
}

#[test]
fn test_keys_values_iter() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));
    context.insert("count".to_string(), json!(42));

    let keys: Vec<&String> = context.keys().collect();
    assert_eq!(keys.len(), 2);

    let values: Vec<&Value> = context.values().collect();
    assert_eq!(values.len(), 2);

    let mut count = 0;
    for (key, value) in context.iter() {
        assert!(key == "status" || key == "count");
        assert!(value == &json!("active") || value == &json!(42));
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn test_to_map_and_into_map() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    let map_ref = context.to_map();
    assert_eq!(map_ref.len(), 1);

    let map_owned = context.into_map();
    assert_eq!(map_owned.len(), 1);
}

#[test]
fn test_from_hashmap() {
    let mut fields = HashMap::new();
    fields.insert("status".to_string(), json!("active"));

    let context: EvaluationContext = fields.into();
    assert_eq!(context.len(), 1);
}

#[test]
fn test_into_hashmap() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    let map: HashMap<String, Value> = context.into();
    assert_eq!(map.len(), 1);
}

#[test]
fn test_default() {
    let context = EvaluationContext::default();
    assert!(context.is_empty());
}

#[test]
fn test_validate_field_type_null() {
    let mut context = EvaluationContext::new();
    context.insert("nothing".to_string(), Value::Null);

    assert!(context.validate_field_type("nothing", "null").is_ok());
    let err = context
        .validate_field_type("nothing", "string")
        .unwrap_err();
    assert!(err.contains("has type 'null' but expected 'string'"));
}

#[test]
fn test_all_supported_types() {
    let mut context = EvaluationContext::new();

    context.insert("name".to_string(), json!("Alice"));
    assert_eq!(context.get_string("name"), Some("Alice"));

    context.insert("age".to_string(), json!(30));
    assert_eq!(context.get_number("age"), Some(30.0));

    context.insert("price".to_string(), json!(19.99));
    assert_eq!(context.get_number("price"), Some(19.99));

    context.insert("active".to_string(), json!(true));
    assert_eq!(context.get_bool("active"), Some(true));

    context.insert("tags".to_string(), json!(["rust", "programming"]));
    assert_eq!(context.get_array("tags").map(|a| a.len()), Some(2));

    context.insert("user".to_string(), json!({"name": "Bob", "age": 25}));
    assert!(context.get_object("user").is_some());
}
