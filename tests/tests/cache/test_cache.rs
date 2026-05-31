use chrono::Utc;
use hudhudscript_cache::*;
use hudhudscript_governance::{Action, Condition, Constitution, EnforcementLevel, Law, Rule};
use std::collections::HashMap;

fn make_rule(id: &str) -> Rule {
    Rule {
        id: id.to_string(),
        name: format!("Rule {}", id),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    }
}

#[test]
fn test_cache_creation() {
    let cache = CommandCache::new();
    assert_eq!(cache.constitutions.len(), 0);
    assert_eq!(cache.laws.len(), 0);
    assert_eq!(cache.rules.len(), 0);
}

#[test]
fn test_store_and_resolve_constitution() {
    let mut cache = CommandCache::new();
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test Constitution".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let id = cache.store_constitution(constitution.clone()).unwrap();
    assert_eq!(id, "cons.1");

    let resolved = cache.resolve_constitution(&id);
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name, "Test Constitution");
}

#[test]
fn test_store_duplicate_constitution_fails() {
    let mut cache = CommandCache::new();
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    cache.store_constitution(constitution.clone()).unwrap();
    let result = cache.store_constitution(constitution);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CacheError::IdCollision(_)));
}

#[test]
fn test_store_and_resolve_law() {
    let mut cache = CommandCache::new();
    let law = Law {
        id: "cons1.law1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "Test Law".to_string(),
        description: "A test law".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };

    let id = cache.store_law(law.clone()).unwrap();
    assert_eq!(id, "cons1.law1");

    let resolved = cache.resolve_law(&id);
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name, "Test Law");
}

#[test]
fn test_store_and_resolve_rule() {
    let mut cache = CommandCache::new();
    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test Rule".to_string(),
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: serde_json::json!("active"),
        }],
        actions: vec![Action::Allow],
        priority: 10,
    };

    let id = cache.store_rule(rule.clone()).unwrap();
    assert_eq!(id, "rule.1");

    let resolved = cache.resolve_rule(&id);
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name, "Test Rule");
}

#[test]
fn test_resolve_nonexistent_returns_none() {
    let mut cache = CommandCache::new();
    assert!(cache
        .resolve_constitution(&"cons.999".to_string())
        .is_none());
    assert!(cache.resolve_law(&"cons1.law999".to_string()).is_none());
    assert!(cache.resolve_rule(&"rule.999".to_string()).is_none());
}

#[test]
fn test_cache_serialization() {
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

    let json = cache.serialize_definitions().unwrap();
    assert!(json.contains("cons.1"));
    assert!(json.contains("Test"));
}

#[test]
fn test_cache_deserialization() {
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

    let json = cache.serialize_definitions().unwrap();
    let mut deserialized = CommandCache::deserialize_definitions(&json).unwrap();

    assert_eq!(deserialized.constitutions.len(), 1);
    assert!(deserialized
        .resolve_constitution(&"cons.1".to_string())
        .is_some());
}

#[test]
fn test_cache_round_trip() {
    let mut cache = CommandCache::new();

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test Constitution".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    let law = Law {
        id: "cons1.law1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "Test Law".to_string(),
        description: "A test law".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };
    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test Rule".to_string(),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    };

    cache.store_constitution(constitution).unwrap();
    cache.store_law(law).unwrap();
    cache.store_rule(rule).unwrap();

    let json = cache.serialize_definitions().unwrap();
    let mut restored = CommandCache::deserialize_definitions(&json).unwrap();

    assert_eq!(restored.constitutions.len(), 1);
    assert_eq!(restored.laws.len(), 1);
    assert_eq!(restored.rules.len(), 1);

    assert!(restored
        .resolve_constitution(&"cons.1".to_string())
        .is_some());
    assert!(restored.resolve_law(&"cons1.law1".to_string()).is_some());
    assert!(restored.resolve_rule(&"rule.1".to_string()).is_some());
}

#[test]
fn test_lru_eviction_with_small_cache() {
    let mut cache = CommandCache::with_capacity(3);

    for i in 1..=3 {
        let rule = Rule {
            id: format!("rule.{}", i),
            name: format!("Rule {}", i),
            conditions: vec![],
            actions: vec![Action::Allow],
            priority: 10,
        };
        cache.store_rule(rule).unwrap();
    }

    assert_eq!(cache.total_items(), 3);
    assert!(cache.resolve_rule(&"rule.1".to_string()).is_some());
    assert!(cache.resolve_rule(&"rule.2".to_string()).is_some());
    assert!(cache.resolve_rule(&"rule.3".to_string()).is_some());

    let rule4 = Rule {
        id: "rule.4".to_string(),
        name: "Rule 4".to_string(),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    };
    cache.store_rule(rule4).unwrap();

    assert_eq!(cache.total_items(), 3);
    assert!(cache.rules.get(&"rule.1".to_string()).is_none()); // Evicted
    assert!(cache.resolve_rule(&"rule.2".to_string()).is_some());
    assert!(cache.resolve_rule(&"rule.3".to_string()).is_some());
    assert!(cache.resolve_rule(&"rule.4".to_string()).is_some());
}

#[test]
fn test_lru_updates_on_access() {
    let mut cache = CommandCache::with_capacity(3);

    for i in 1..=3 {
        let rule = Rule {
            id: format!("rule.{}", i),
            name: format!("Rule {}", i),
            conditions: vec![],
            actions: vec![Action::Allow],
            priority: 10,
        };
        cache.store_rule(rule).unwrap();
    }

    // Access rule.1 to make it most recently used
    cache.resolve_rule(&"rule.1".to_string());

    let rule4 = Rule {
        id: "rule.4".to_string(),
        name: "Rule 4".to_string(),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    };
    cache.store_rule(rule4).unwrap();

    assert!(cache.resolve_rule(&"rule.1".to_string()).is_some()); // Still present
    assert!(cache.rules.get(&"rule.2".to_string()).is_none()); // Evicted
    assert!(cache.resolve_rule(&"rule.3".to_string()).is_some());
    assert!(cache.resolve_rule(&"rule.4".to_string()).is_some());
}

#[test]
fn test_lru_eviction_priority() {
    let mut cache = CommandCache::with_capacity(3);

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test Constitution".to_string(),
        description: None,
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

    assert_eq!(cache.total_items(), 3);

    let rule2 = Rule {
        id: "rule.2".to_string(),
        name: "Test Rule 2".to_string(),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    };
    cache.store_rule(rule2).unwrap();

    assert_eq!(cache.total_items(), 3);
    assert!(cache.rules.get(&"rule.1".to_string()).is_none()); // Rule evicted first
    assert!(cache.resolve_constitution(&"cons.1".to_string()).is_some());
    assert!(cache.resolve_law(&"cons1.law1".to_string()).is_some());
}

#[test]
fn test_default_cache_size() {
    let cache = CommandCache::new();
    assert_eq!(cache.max_size, 1000);
}

#[test]
fn test_custom_cache_size() {
    let cache = CommandCache::with_capacity(500);
    assert_eq!(cache.max_size, 500);
}

#[test]
fn test_managed_cache_store_and_resolve() {
    let mut cache = ManagedCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();

    let resolved = cache.resolve_rule(&"rule.1".to_string());
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name, "Rule rule.1");
}

#[test]
fn test_managed_cache_statistics() {
    let mut cache = ManagedCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();
    cache.store_rule(make_rule("rule.2")).unwrap();

    let stats = cache.statistics();
    assert_eq!(stats.rule_count, 2);
    assert_eq!(stats.total_items, 2);
    assert!(stats.total_size_bytes > 0);
}

#[test]
fn test_managed_cache_prune() {
    let config = ManagedCacheConfig {
        max_items: 100,
        eviction_policy: EvictionPolicy::Lru,
        quota_config: QuotaConfig::default(),
        enable_dedup: true,
    };
    let mut cache = ManagedCache::with_config(config);

    cache.store_rule(make_rule("rule.1")).unwrap();
    cache.store_rule(make_rule("rule.2")).unwrap();
    cache.store_rule(make_rule("rule.3")).unwrap();

    let result = cache.prune(1);
    assert_eq!(result.entries_removed, 1);
    assert_eq!(result.removed_keys.len(), 1);
    assert_eq!(result.removed_keys[0], "rule.1");

    assert_eq!(cache.statistics().total_items, 2);
}

#[test]
fn test_managed_cache_clear() {
    let mut cache = ManagedCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();
    cache.store_rule(make_rule("rule.2")).unwrap();

    cache.clear();

    let stats = cache.statistics();
    assert_eq!(stats.total_items, 0);
    assert_eq!(stats.total_size_bytes, 0);
}

#[test]
fn test_managed_cache_eviction_on_capacity() {
    let config = ManagedCacheConfig {
        max_items: 3,
        eviction_policy: EvictionPolicy::Lru,
        quota_config: QuotaConfig::default(),
        enable_dedup: false,
    };
    let mut cache = ManagedCache::with_config(config);

    cache.store_rule(make_rule("rule.1")).unwrap();
    cache.store_rule(make_rule("rule.2")).unwrap();
    cache.store_rule(make_rule("rule.3")).unwrap();

    cache.store_rule(make_rule("rule.4")).unwrap();

    assert_eq!(cache.statistics().total_items, 3);
    assert!(cache.resolve_rule(&"rule.1".to_string()).is_none());
    assert!(cache.resolve_rule(&"rule.4".to_string()).is_some());
}

#[test]
fn test_managed_cache_lfu_eviction() {
    let config = ManagedCacheConfig {
        max_items: 3,
        eviction_policy: EvictionPolicy::Lfu,
        quota_config: QuotaConfig::default(),
        enable_dedup: false,
    };
    let mut cache = ManagedCache::with_config(config);

    cache.store_rule(make_rule("rule.1")).unwrap();
    cache.store_rule(make_rule("rule.2")).unwrap();
    cache.store_rule(make_rule("rule.3")).unwrap();

    cache.resolve_rule(&"rule.1".to_string());
    cache.resolve_rule(&"rule.1".to_string());
    cache.resolve_rule(&"rule.3".to_string());

    cache.store_rule(make_rule("rule.4")).unwrap();

    assert!(cache.resolve_rule(&"rule.2".to_string()).is_none());
    assert!(cache.resolve_rule(&"rule.1".to_string()).is_some());
}

#[test]
fn test_managed_cache_dedup_tracking() {
    let mut cache = ManagedCache::new();

    cache.store_rule(make_rule("rule.1")).unwrap();
    cache.store_rule(make_rule("rule.2")).unwrap();

    let stats = cache.statistics();
    assert_eq!(stats.unique_contents, 2);
}

#[test]
fn test_managed_cache_index_tracks_types() {
    let mut cache = ManagedCache::new();

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();
    cache.store_rule(make_rule("rule.1")).unwrap();

    let summary = cache.index().summary();
    assert_eq!(summary.constitutions, 1);
    assert_eq!(summary.rules, 1);
    assert_eq!(summary.total_entries, 2);
}

#[test]
fn test_managed_cache_quota_alerts() {
    let config = ManagedCacheConfig {
        max_items: 1000,
        eviction_policy: EvictionPolicy::Lru,
        quota_config: QuotaConfig {
            quota_bytes: 100,
            warning_threshold: 0.80,
            critical_threshold: 0.95,
        },
        enable_dedup: false,
    };
    let mut cache = ManagedCache::with_config(config);

    cache.store_rule(make_rule("rule.1")).unwrap();

    let alerts = cache.drain_alerts();
    assert!(!alerts.is_empty());
}

#[test]
fn test_managed_cache_default_trait() {
    let cache = ManagedCache::default();
    assert_eq!(cache.statistics().total_items, 0);
}

#[test]
fn test_managed_cache_inner_accessor() {
    let mut cache = ManagedCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();
    let inner = cache.inner();
    assert_eq!(inner.rules.len(), 1);
}

#[test]
fn test_managed_cache_eviction_accessor() {
    let cache = ManagedCache::new();
    assert_eq!(cache.eviction().policy(), EvictionPolicy::Lru);
}

#[test]
fn test_managed_cache_quota_accessor() {
    let cache = ManagedCache::new();
    assert_eq!(cache.quota().current_bytes(), 0);
}

#[test]
fn test_managed_cache_dedup_store_accessor() {
    let cache = ManagedCache::new();
    assert!(cache.dedup_store().is_empty());
}

#[test]
fn test_managed_cache_drain_alerts_empty() {
    let mut cache = ManagedCache::new();
    let alerts = cache.drain_alerts();
    assert_eq!(alerts.len(), 0);
}

#[test]
fn test_managed_cache_store_constitution() {
    let mut cache = ManagedCache::new();
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    let id = cache.store_constitution(constitution).unwrap();
    assert_eq!(id, "cons.1");
    assert!(cache.resolve_constitution(&"cons.1".to_string()).is_some());
}

#[test]
fn test_managed_cache_store_law() {
    let mut cache = ManagedCache::new();
    let law = Law {
        id: "law.1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "Test Law".to_string(),
        description: "desc".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };
    let id = cache.store_law(law).unwrap();
    assert_eq!(id, "law.1");
    assert!(cache.resolve_law(&"law.1".to_string()).is_some());
}

#[test]
fn test_managed_cache_resolve_nonexistent() {
    let mut cache = ManagedCache::new();
    assert!(cache
        .resolve_constitution(&"cons.999".to_string())
        .is_none());
    assert!(cache.resolve_law(&"law.999".to_string()).is_none());
    assert!(cache.resolve_rule(&"rule.999".to_string()).is_none());
}

#[test]
fn test_managed_cache_statistics_empty() {
    let cache = ManagedCache::new();
    let stats = cache.statistics();
    assert_eq!(stats.constitution_count, 0);
    assert_eq!(stats.law_count, 0);
    assert_eq!(stats.rule_count, 0);
    assert_eq!(stats.total_items, 0);
    assert_eq!(stats.max_capacity, 1000);
    assert_eq!(stats.total_size_bytes, 0);
    assert_eq!(stats.unique_contents, 0);
    assert_eq!(stats.duplicate_count, 0);
    assert_eq!(stats.dedup_savings_bytes, 0);
    assert_eq!(stats.eviction_policy, EvictionPolicy::Lru);
}

#[test]
fn test_managed_cache_prune_removes_from_dedup() {
    let mut cache = ManagedCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();
    cache.store_rule(make_rule("rule.2")).unwrap();

    let before = cache.dedup_store().total_key_count();
    assert_eq!(before, 2);

    cache.prune(1);
    assert_eq!(cache.dedup_store().total_key_count(), 1);
}

#[test]
fn test_managed_cache_prune_zero_items() {
    let mut cache = ManagedCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();
    let result = cache.prune(0);
    assert_eq!(result.entries_removed, 0);
    assert_eq!(result.removed_keys.len(), 0);
    assert_eq!(result.bytes_reclaimed, 0);
}

#[test]
fn test_managed_cache_clear_resets_all_subsystems() {
    let mut cache = ManagedCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();
    cache.store_rule(make_rule("rule.2")).unwrap();

    cache.clear();

    assert!(cache.eviction().is_empty());
    assert!(cache.index().is_empty());
    assert!(cache.dedup_store().is_empty());
    assert_eq!(cache.statistics().total_items, 0);
}

#[test]
fn test_managed_cache_config_default() {
    let config = ManagedCacheConfig::default();
    assert_eq!(config.max_items, 1000);
    assert_eq!(config.eviction_policy, EvictionPolicy::Lru);
    assert!(config.enable_dedup);
}

#[test]
fn test_cache_error_display_constitution_not_found() {
    let err = CacheError::ConstitutionNotFound("cons.99".to_string());
    assert!(err.to_string().contains("Constitution not found: cons.99"));
}

#[test]
fn test_cache_error_display_law_not_found() {
    let err = CacheError::LawNotFound("law.99".to_string());
    assert!(err.to_string().contains("Law not found: law.99"));
}

#[test]
fn test_cache_error_display_rule_not_found() {
    let err = CacheError::RuleNotFound("rule.99".to_string());
    assert!(err.to_string().contains("Rule not found: rule.99"));
}

#[test]
fn test_cache_error_display_quota_exceeded() {
    let err = CacheError::QuotaExceeded("too big".to_string());
    assert!(err.to_string().contains("Quota exceeded: too big"));
}

#[test]
fn test_cache_error_display_duplicate_content() {
    let err = CacheError::DuplicateContent {
        key: "rule.2".to_string(),
        existing: "rule.1".to_string(),
    };
    assert!(err
        .to_string()
        .contains("Duplicate content detected for key rule.2: already exists as rule.1"));
}

#[test]
fn test_store_duplicate_law_fails() {
    let mut cache = CommandCache::new();
    let law = Law {
        id: "law.1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "Law".to_string(),
        description: "desc".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };
    cache.store_law(law.clone()).unwrap();
    let result = cache.store_law(law);
    assert!(matches!(result.unwrap_err(), CacheError::IdCollision(_)));
}

#[test]
fn test_store_duplicate_rule_fails() {
    let mut cache = CommandCache::new();
    let rule = make_rule("rule.1");
    cache.store_rule(rule.clone()).unwrap();
    let result = cache.store_rule(rule);
    assert!(matches!(result.unwrap_err(), CacheError::IdCollision(_)));
}

#[test]
fn test_lru_eviction_laws_before_constitutions() {
    let mut cache = CommandCache::with_capacity(2);

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "C".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();

    let law = Law {
        id: "law.1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "L".to_string(),
        description: "d".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };
    cache.store_law(law).unwrap();

    let law2 = Law {
        id: "law.2".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "L2".to_string(),
        description: "d".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };
    cache.store_law(law2).unwrap();

    assert_eq!(cache.total_items(), 2);
    assert!(cache.laws.get("law.1").is_none());
    assert!(cache.laws.get("law.2").is_some());
    assert!(cache.constitutions.get("cons.1").is_some());
}

#[test]
fn test_deserialization_invalid_json() {
    let result = CommandCache::deserialize_definitions("not json");
    assert!(matches!(
        result.unwrap_err(),
        CacheError::DeserializationError(_)
    ));
}

#[test]
fn test_cache_error_display_serialization_error() {
    let err = CacheError::SerializationError("bad format".to_string());
    assert!(err.to_string().contains("Serialization error: bad format"));
}

#[test]
fn test_cache_error_display_deserialization_error() {
    let err = CacheError::DeserializationError("invalid json".to_string());
    assert!(err
        .to_string()
        .contains("Deserialization error: invalid json"));
}

#[test]
fn test_cache_error_display_id_collision() {
    let err = CacheError::IdCollision("cons.1".to_string());
    assert!(err.to_string().contains("Cache ID collision: cons.1"));
}

#[test]
fn test_lru_constitution_eviction() {
    let mut cache = CommandCache::with_capacity(1);
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "C1".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution).unwrap();

    let constitution2 = Constitution {
        id: "cons.2".to_string(),
        name: "C2".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };
    cache.store_constitution(constitution2).unwrap();

    assert_eq!(cache.total_items(), 1);
    assert!(cache.constitutions.get("cons.1").is_none());
    assert!(cache.constitutions.get("cons.2").is_some());
}
