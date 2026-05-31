use chrono::Utc;
use hudhudscript_cache::{
    deserialize_definitions, serialize_definitions, CacheError, CacheSerializationFormat,
    CommandCache, SERIALIZATION_VERSION,
};
use hudhudscript_governance::{Action, Constitution, EnforcementLevel, Law, Rule};
use std::collections::HashMap;

#[test]
fn test_serialize_empty_cache() {
    let cache = CommandCache::new();
    let json = serialize_definitions(&cache).unwrap();

    assert!(json.starts_with('{'));
    assert!(json.ends_with('}'));
    assert!(json.contains("\"version\""));
    assert!(json.contains("\"timestamp\""));
    assert!(json.contains("\"item_count\""));
    assert!(json.contains("\"cache\""));
}

#[test]
fn test_serialize_cache_with_items() {
    let mut cache = CommandCache::new();

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test Constitution".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();

    let json = serialize_definitions(&cache).unwrap();

    assert!(json.contains("\"version\":1"));
    assert!(json.contains("\"item_count\":1"));
    assert!(json.contains("cons.1"));
}

#[test]
fn test_deserialize_valid_cache() {
    let mut cache = CommandCache::new();

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test Constitution".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();

    let json = serialize_definitions(&cache).unwrap();
    let restored = deserialize_definitions(&json).unwrap();

    assert_eq!(restored.constitutions.len(), 1);
    assert!(restored.constitutions.contains_key("cons.1"));
}

#[test]
fn test_round_trip_with_multiple_items() {
    let mut cache = CommandCache::new();

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test Constitution".to_string(),
        description: Some("A test constitution".to_string()),
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();

    let law = Law {
        id: "cons1.law1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "Test Law".to_string(),
        description: "A test law".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };
    cache.store_law(law).unwrap();

    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test Rule".to_string(),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    };
    cache.store_rule(rule).unwrap();

    let json = serialize_definitions(&cache).unwrap();
    let restored = deserialize_definitions(&json).unwrap();

    assert_eq!(restored.constitutions.len(), 1);
    assert_eq!(restored.laws.len(), 1);
    assert_eq!(restored.rules.len(), 1);

    assert_eq!(
        restored.constitutions.get("cons.1").unwrap().name,
        "Test Constitution"
    );
    assert_eq!(restored.laws.get("cons1.law1").unwrap().name, "Test Law");
    assert_eq!(restored.rules.get("rule.1").unwrap().name, "Test Rule");
}

#[test]
fn test_compact_json_format() {
    let mut cache = CommandCache::new();

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();

    let json = serialize_definitions(&cache).unwrap();

    assert!(!json.contains("  "));
    assert!(!json.contains("\n"));
    assert!(!json.contains("\r"));
}

#[test]
fn test_utf8_encoding() {
    let mut cache = CommandCache::new();

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Anayasa 憲法 دستور".to_string(),
        description: Some("Multi-language test: Türkçe 日本語 العربية".to_string()),
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();

    let json = serialize_definitions(&cache).unwrap();
    let restored = deserialize_definitions(&json).unwrap();

    let restored_const = restored.constitutions.get("cons.1").unwrap();
    assert_eq!(restored_const.name, "Anayasa 憲法 دستور");
    assert_eq!(
        restored_const.description.as_ref().unwrap(),
        "Multi-language test: Türkçe 日本語 العربية"
    );
}

#[test]
fn test_metadata_validation() {
    let cache = CommandCache::new();
    let format = CacheSerializationFormat::new(cache);
    assert!(format.validate().is_ok());
}

#[test]
fn test_item_count_validation() {
    let mut cache = CommandCache::new();

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();

    let mut format = CacheSerializationFormat::new(cache);
    format.item_count = 999;

    let result = format.validate();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CacheError::DeserializationError(_)
    ));
}

#[test]
fn test_version_compatibility() {
    let cache = CommandCache::new();
    let mut format = CacheSerializationFormat::new(cache);
    format.version = 999;

    let result = format.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    if let CacheError::DeserializationError(msg) = err {
        assert!(msg.contains("Unsupported serialization version"));
    } else {
        panic!("Expected DeserializationError");
    }
}

#[test]
fn test_invalid_json_deserialization() {
    let invalid_json = "{ invalid json }";
    let result = deserialize_definitions(invalid_json);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CacheError::DeserializationError(_)
    ));
}

#[test]
fn test_metadata_includes_timestamp() {
    let cache = CommandCache::new();
    let json = serialize_definitions(&cache).unwrap();

    let format: CacheSerializationFormat = serde_json::from_str(&json).unwrap();

    let now = Utc::now();
    let diff = now.signed_duration_since(format.timestamp);
    assert!(diff.num_seconds() < 60);
}

#[test]
fn test_serialization_format_version() {
    let cache = CommandCache::new();
    let json = serialize_definitions(&cache).unwrap();

    let format: CacheSerializationFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(format.version, SERIALIZATION_VERSION);
}
