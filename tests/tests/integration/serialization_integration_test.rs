//! Integration tests for cache serialization with metadata

use chrono::Utc;
use hudhudscript_cache::{deserialize_definitions, serialize_definitions, CommandCache};
use hudhudscript_governance::{Action, Constitution, EnforcementLevel, Law, Rule};
use std::collections::HashMap;

#[test]
fn test_full_cache_serialization_workflow() {
    // Create a cache with various governance structures
    let mut cache = CommandCache::new();

    // Add constitution
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Data Governance Constitution".to_string(),
        description: Some("Governs data processing and privacy".to_string()),
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();

    // Add law
    let law = Law {
        id: "cons1.law1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "Data Privacy Law".to_string(),
        description: "All personal data must be encrypted".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };
    cache.store_law(law).unwrap();

    // Add rule
    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Data Size Limit".to_string(),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    };
    cache.store_rule(rule).unwrap();

    // Serialize with metadata
    let json = serialize_definitions(&cache).unwrap();

    // Verify JSON contains metadata
    assert!(json.contains("\"version\""));
    assert!(json.contains("\"timestamp\""));
    assert!(json.contains("\"item_count\":3"));

    // Verify JSON is compact (no pretty printing)
    assert!(!json.contains("\n"));
    assert!(!json.contains("  "));

    // Deserialize and validate
    let restored = deserialize_definitions(&json).unwrap();

    // Verify all items are restored
    assert_eq!(restored.constitutions.len(), 1);
    assert_eq!(restored.laws.len(), 1);
    assert_eq!(restored.rules.len(), 1);

    // Verify content integrity
    let restored_const = restored.constitutions.get("cons.1").unwrap();
    assert_eq!(restored_const.name, "Data Governance Constitution");
    assert_eq!(
        restored_const.description.as_ref().unwrap(),
        "Governs data processing and privacy"
    );

    let restored_law = restored.laws.get("cons1.law1").unwrap();
    assert_eq!(restored_law.name, "Data Privacy Law");
    assert_eq!(restored_law.enforcement_level, EnforcementLevel::Mandatory);

    let restored_rule = restored.rules.get("rule.1").unwrap();
    assert_eq!(restored_rule.name, "Data Size Limit");
    assert_eq!(restored_rule.priority, 10);
}

#[test]
fn test_multilingual_cache_serialization() {
    let mut cache = CommandCache::new();

    // Add constitutions with multilingual names
    let constitutions = vec![
        ("cons.1", "English Constitution"),
        ("cons.2", "Türkçe Anayasa"),
        ("cons.3", "日本国憲法"),
        ("cons.4", "الدستور العربي"),
    ];

    for (id, name) in constitutions {
        let constitution = Constitution {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            laws: HashMap::new(),
            created_at: Utc::now(),
            version: 1,
        };
        cache.store_constitution(constitution).unwrap();
    }

    // Serialize and deserialize
    let json = serialize_definitions(&cache).unwrap();
    let restored = deserialize_definitions(&json).unwrap();

    // Verify all multilingual names are preserved
    assert_eq!(
        restored.constitutions.get("cons.1").unwrap().name,
        "English Constitution"
    );
    assert_eq!(
        restored.constitutions.get("cons.2").unwrap().name,
        "Türkçe Anayasa"
    );
    assert_eq!(
        restored.constitutions.get("cons.3").unwrap().name,
        "日本国憲法"
    );
    assert_eq!(
        restored.constitutions.get("cons.4").unwrap().name,
        "الدستور العربي"
    );
}

#[test]
fn test_large_cache_serialization() {
    let mut cache = CommandCache::new();

    // Add many items to test performance and correctness
    for i in 1..=50 {
        let constitution = Constitution {
            id: format!("cons.{}", i),
            name: format!("Constitution {}", i),
            description: Some(format!("Description for constitution {}", i)),
            laws: HashMap::new(),
            created_at: Utc::now(),
            version: 1,
        };
        cache.store_constitution(constitution).unwrap();

        let law = Law {
            id: format!("cons{}.law1", i),
            constitution_id: format!("cons.{}", i),
            name: format!("Law {} of Constitution {}", 1, i),
            description: format!("Description for law 1 of constitution {}", i),
            enforcement_level: EnforcementLevel::Mandatory,
            conditions: vec![],
        };
        cache.store_law(law).unwrap();

        let rule = Rule {
            id: format!("rule.{}", i),
            name: format!("Rule {}", i),
            conditions: vec![],
            actions: vec![Action::Allow],
            priority: i as u32,
        };
        cache.store_rule(rule).unwrap();
    }

    // Serialize
    let json = serialize_definitions(&cache).unwrap();

    // Verify metadata
    assert!(json.contains("\"item_count\":150")); // 50 * 3 = 150 items

    // Deserialize
    let restored = deserialize_definitions(&json).unwrap();

    // Verify all items are present
    assert_eq!(restored.constitutions.len(), 50);
    assert_eq!(restored.laws.len(), 50);
    assert_eq!(restored.rules.len(), 50);

    // Spot check a few items
    assert!(restored.constitutions.contains_key("cons.1"));
    assert!(restored.constitutions.contains_key("cons.25"));
    assert!(restored.constitutions.contains_key("cons.50"));
}

#[test]
fn test_empty_cache_serialization() {
    let cache = CommandCache::new();

    // Serialize empty cache
    let json = serialize_definitions(&cache).unwrap();

    // Verify metadata for empty cache
    assert!(json.contains("\"item_count\":0"));

    // Deserialize
    let restored = deserialize_definitions(&json).unwrap();

    // Verify empty cache is restored correctly
    assert_eq!(restored.constitutions.len(), 0);
    assert_eq!(restored.laws.len(), 0);
    assert_eq!(restored.rules.len(), 0);
}

#[test]
fn test_serialization_preserves_enforcement_levels() {
    let mut cache = CommandCache::new();

    // Add laws with different enforcement levels
    let enforcement_levels = vec![
        ("cons1.law1", EnforcementLevel::Mandatory),
        ("cons1.law2", EnforcementLevel::Advisory),
        ("cons1.law3", EnforcementLevel::Optional),
    ];

    for (id, level) in enforcement_levels {
        let law = Law {
            id: id.to_string(),
            constitution_id: "cons.1".to_string(),
            name: format!("Law {}", id),
            description: format!("Description for {}", id),
            enforcement_level: level,
            conditions: vec![],
        };
        cache.store_law(law).unwrap();
    }

    // Serialize and deserialize
    let json = serialize_definitions(&cache).unwrap();
    let restored = deserialize_definitions(&json).unwrap();

    // Verify enforcement levels are preserved
    assert_eq!(
        restored.laws.get("cons1.law1").unwrap().enforcement_level,
        EnforcementLevel::Mandatory
    );
    assert_eq!(
        restored.laws.get("cons1.law2").unwrap().enforcement_level,
        EnforcementLevel::Advisory
    );
    assert_eq!(
        restored.laws.get("cons1.law3").unwrap().enforcement_level,
        EnforcementLevel::Optional
    );
}
