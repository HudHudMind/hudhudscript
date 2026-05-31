use hudhudscript_cache::*;
use hudhudscript_governance::*;
use std::collections::HashMap;

fn make_constitution(id: &str) -> Constitution {
    Constitution {
        id: id.to_string(),
        name: format!("Constitution {}", id),
        description: None,
        laws: HashMap::new(),
        created_at: chrono::Utc::now(),
        version: 1,
    }
}

fn make_law(id: &str, cons_id: &str) -> Law {
    Law {
        id: id.to_string(),
        constitution_id: cons_id.to_string(),
        name: format!("Law {}", id),
        description: "test law".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    }
}

fn make_rule(id: &str) -> Rule {
    Rule {
        id: id.to_string(),
        name: format!("Rule {}", id),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    }
}

fn make_index_entry(key: &str, entry_type: CacheEntryType, size: u64) -> IndexEntry {
    IndexEntry {
        key: key.to_string(),
        entry_type,
        size_bytes: size,
        created_at: chrono::Utc::now(),
        content_hash: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CommandCache — new / default / with_capacity
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn command_cache_new_empty() {
    let cache = CommandCache::new();
    assert_eq!(cache.constitutions.len(), 0);
    assert_eq!(cache.laws.len(), 0);
    assert_eq!(cache.rules.len(), 0);
}

#[test]
fn command_cache_default_empty() {
    let cache = CommandCache::default();
    assert_eq!(cache.constitutions.len(), 0);
}

#[test]
fn command_cache_with_capacity() {
    let cache = CommandCache::with_capacity(50);
    assert_eq!(cache.constitutions.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CommandCache — store / resolve constitution
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn store_constitution_returns_id() {
    let mut cache = CommandCache::new();
    let id = cache
        .store_constitution(make_constitution("cons.1"))
        .unwrap();
    assert_eq!(id, "cons.1");
}

#[test]
fn resolve_constitution_found() {
    let mut cache = CommandCache::new();
    cache
        .store_constitution(make_constitution("cons.1"))
        .unwrap();
    let resolved = cache.resolve_constitution(&"cons.1".to_string());
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name, "Constitution cons.1");
}

#[test]
fn resolve_constitution_not_found() {
    let mut cache = CommandCache::new();
    assert!(cache.resolve_constitution(&"cons.99".to_string()).is_none());
}

#[test]
fn store_constitution_id_collision() {
    let mut cache = CommandCache::new();
    cache
        .store_constitution(make_constitution("cons.1"))
        .unwrap();
    let result = cache.store_constitution(make_constitution("cons.1"));
    assert!(result.is_err());
}

#[test]
fn store_multiple_constitutions() {
    let mut cache = CommandCache::new();
    cache
        .store_constitution(make_constitution("cons.1"))
        .unwrap();
    cache
        .store_constitution(make_constitution("cons.2"))
        .unwrap();
    assert_eq!(cache.constitutions.len(), 2);
}

#[test]
fn resolve_constitution_updates_lru() {
    let mut cache = CommandCache::new();
    cache
        .store_constitution(make_constitution("cons.1"))
        .unwrap();
    cache
        .store_constitution(make_constitution("cons.2"))
        .unwrap();
    // Access cons.1 again so it becomes most recently used
    let resolved = cache.resolve_constitution(&"cons.1".to_string());
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name, "Constitution cons.1");
}

// ═══════════════════════════════════════════════════════════════════════════════
// CommandCache — store / resolve law
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn store_law_returns_id() {
    let mut cache = CommandCache::new();
    let id = cache.store_law(make_law("cons1.law1", "cons.1")).unwrap();
    assert_eq!(id, "cons1.law1");
}

#[test]
fn resolve_law_found() {
    let mut cache = CommandCache::new();
    cache.store_law(make_law("cons1.law1", "cons.1")).unwrap();
    let resolved = cache.resolve_law(&"cons1.law1".to_string());
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name, "Law cons1.law1");
}

#[test]
fn resolve_law_not_found() {
    let mut cache = CommandCache::new();
    assert!(cache.resolve_law(&"law.99".to_string()).is_none());
}

#[test]
fn store_law_id_collision() {
    let mut cache = CommandCache::new();
    cache.store_law(make_law("cons1.law1", "cons.1")).unwrap();
    let result = cache.store_law(make_law("cons1.law1", "cons.1"));
    assert!(result.is_err());
}

#[test]
fn store_multiple_laws() {
    let mut cache = CommandCache::new();
    cache.store_law(make_law("law.1", "cons.1")).unwrap();
    cache.store_law(make_law("law.2", "cons.1")).unwrap();
    assert_eq!(cache.laws.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CommandCache — store / resolve rule
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn store_rule_returns_id() {
    let mut cache = CommandCache::new();
    let id = cache.store_rule(make_rule("rule.1")).unwrap();
    assert_eq!(id, "rule.1");
}

#[test]
fn resolve_rule_found() {
    let mut cache = CommandCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();
    let resolved = cache.resolve_rule(&"rule.1".to_string());
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name, "Rule rule.1");
}

#[test]
fn resolve_rule_not_found() {
    let mut cache = CommandCache::new();
    assert!(cache.resolve_rule(&"rule.99".to_string()).is_none());
}

#[test]
fn store_rule_id_collision() {
    let mut cache = CommandCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();
    let result = cache.store_rule(make_rule("rule.1"));
    assert!(result.is_err());
}

#[test]
fn store_multiple_rules() {
    let mut cache = CommandCache::new();
    cache.store_rule(make_rule("rule.1")).unwrap();
    cache.store_rule(make_rule("rule.2")).unwrap();
    cache.store_rule(make_rule("rule.3")).unwrap();
    assert_eq!(cache.rules.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CommandCache — eviction on capacity
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn cache_evicts_rules_first_when_full() {
    let mut cache = CommandCache::with_capacity(2);
    cache.store_rule(make_rule("rule.1")).unwrap();
    cache
        .store_constitution(make_constitution("cons.1"))
        .unwrap();
    // Cache is full (2 items). Storing another should evict the rule.
    cache.store_law(make_law("law.1", "cons.1")).unwrap();
    assert!(cache.rules.is_empty());
    assert_eq!(cache.constitutions.len(), 1);
    assert_eq!(cache.laws.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CommandCache — serialize / deserialize
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn serialize_deserialize_roundtrip() {
    let mut cache = CommandCache::new();
    cache
        .store_constitution(make_constitution("cons.1"))
        .unwrap();
    cache.store_law(make_law("cons1.law1", "cons.1")).unwrap();
    cache.store_rule(make_rule("rule.1")).unwrap();

    let json = cache.serialize_definitions().unwrap();
    let mut restored = CommandCache::deserialize_definitions(&json).unwrap();

    assert_eq!(restored.constitutions.len(), 1);
    assert_eq!(restored.laws.len(), 1);
    assert_eq!(restored.rules.len(), 1);
    assert!(restored
        .resolve_constitution(&"cons.1".to_string())
        .is_some());
}

#[test]
fn serialize_empty_cache() {
    let cache = CommandCache::new();
    let json = cache.serialize_definitions().unwrap();
    let restored = CommandCache::deserialize_definitions(&json).unwrap();
    assert_eq!(restored.constitutions.len(), 0);
}

#[test]
fn deserialize_invalid_json_fails() {
    let result = CommandCache::deserialize_definitions("not valid json");
    assert!(result.is_err());
}

#[test]
fn serialize_deserialize_preserves_names() {
    let mut cache = CommandCache::new();
    cache.store_constitution(make_constitution("c1")).unwrap();
    let json = cache.serialize_definitions().unwrap();
    let mut restored = CommandCache::deserialize_definitions(&json).unwrap();
    let c = restored.resolve_constitution(&"c1".to_string()).unwrap();
    assert_eq!(c.name, "Constitution c1");
}

#[test]
fn serialize_deserialize_law_content() {
    let mut cache = CommandCache::new();
    cache.store_law(make_law("law.1", "cons.1")).unwrap();
    let json = cache.serialize_definitions().unwrap();
    let mut restored = CommandCache::deserialize_definitions(&json).unwrap();
    let law = restored.resolve_law(&"law.1".to_string()).unwrap();
    assert_eq!(law.constitution_id, "cons.1");
}

// ═══════════════════════════════════════════════════════════════════════════════
// CacheError — Display
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn cache_error_constitution_not_found_display() {
    let err = CacheError::ConstitutionNotFound("cons.99".to_string());
    assert!(err.to_string().contains("Constitution not found: cons.99"));
}

#[test]
fn cache_error_law_not_found_display() {
    let err = CacheError::LawNotFound("law.99".to_string());
    assert!(err.to_string().contains("Law not found: law.99"));
}

#[test]
fn cache_error_rule_not_found_display() {
    let err = CacheError::RuleNotFound("rule.99".to_string());
    assert!(err.to_string().contains("Rule not found: rule.99"));
}

#[test]
fn cache_error_id_collision_display() {
    let err = CacheError::IdCollision("cons.1".to_string());
    assert!(err.to_string().contains("Cache ID collision: cons.1"));
}

#[test]
fn cache_error_serialization_display() {
    let err = CacheError::SerializationError("bad data".to_string());
    assert!(err.to_string().contains("Serialization error: bad data"));
}

#[test]
fn cache_error_deserialization_display() {
    let err = CacheError::DeserializationError("corrupt".to_string());
    assert!(err.to_string().contains("Deserialization error: corrupt"));
}

#[test]
fn cache_error_quota_exceeded_display() {
    let err = CacheError::QuotaExceeded("over limit".to_string());
    assert!(err.to_string().contains("Quota exceeded: over limit"));
}

#[test]
fn cache_error_duplicate_content_display() {
    let err = CacheError::DuplicateContent {
        key: "k1".to_string(),
        existing: "k2".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("k1"));
    assert!(msg.contains("k2"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvictionEngine — LRU
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn eviction_lru_basic_victim() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("c");
    assert_eq!(engine.select_victim(), Some("a"));
}

#[test]
fn eviction_lru_reaccess_changes_victim() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("a"); // a is now most recent
    assert_eq!(engine.select_victim(), Some("b"));
}

#[test]
fn eviction_lru_empty_no_victim() {
    let engine = EvictionEngine::new(EvictionPolicy::Lru);
    assert!(engine.select_victim().is_none());
}

#[test]
fn eviction_lru_single_entry() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("only");
    assert_eq!(engine.select_victim(), Some("only"));
}

#[test]
fn eviction_lru_select_victims_multiple() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("c");
    engine.record_access("d");
    let victims = engine.select_victims(2);
    assert_eq!(victims, vec!["a", "b"]);
}

#[test]
fn eviction_lru_select_victims_more_than_available() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.record_access("b");
    let victims = engine.select_victims(5);
    assert_eq!(victims.len(), 2);
}

#[test]
fn eviction_lru_select_victims_zero() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    let victims = engine.select_victims(0);
    assert!(victims.is_empty());
}

#[test]
fn eviction_lru_empty_select_victims() {
    let engine = EvictionEngine::new(EvictionPolicy::Lru);
    let victims = engine.select_victims(5);
    assert!(victims.is_empty());
}

#[test]
fn eviction_lru_multiple_reaccesses() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("c");
    engine.record_access("b"); // b moves to back
    engine.record_access("a"); // a moves to back
                               // Order should be: c (least recent), b, a (most recent)
    assert_eq!(engine.select_victim(), Some("c"));
    let victims = engine.select_victims(3);
    assert_eq!(victims, vec!["c", "b", "a"]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvictionEngine — LFU
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn eviction_lfu_basic_victim() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.record_access("a");
    engine.record_access("a");
    engine.record_access("b");
    assert_eq!(engine.select_victim(), Some("b"));
}

#[test]
fn eviction_lfu_equal_frequency_uses_tick() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.record_access("b");
    // Both have count=1, but "a" has lower tick -> victim
    let victim = engine.select_victim().unwrap();
    assert_eq!(victim, "a");
}

#[test]
fn eviction_lfu_select_victims_sorted() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    // a=3, b=1, c=2
    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("c");
    engine.record_access("a");
    engine.record_access("c");
    engine.record_access("a");
    let victims = engine.select_victims(3);
    assert_eq!(victims[0], "b"); // count=1
    assert_eq!(victims[1], "c"); // count=2
    assert_eq!(victims[2], "a"); // count=3
}

#[test]
fn eviction_lfu_empty_no_victim() {
    let engine = EvictionEngine::new(EvictionPolicy::Lfu);
    assert!(engine.select_victim().is_none());
}

#[test]
fn eviction_lfu_empty_select_victims() {
    let engine = EvictionEngine::new(EvictionPolicy::Lfu);
    assert!(engine.select_victims(5).is_empty());
}

#[test]
fn eviction_lfu_select_victims_more_than_available() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("x");
    let victims = engine.select_victims(10);
    assert_eq!(victims.len(), 1);
    assert_eq!(victims[0], "x");
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvictionEngine — common operations
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn eviction_remove_lru() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.record_access("b");
    engine.remove("a");
    assert_eq!(engine.len(), 1);
    assert_eq!(engine.select_victim(), Some("b"));
}

#[test]
fn eviction_remove_lfu() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.record_access("b");
    engine.remove("a");
    assert_eq!(engine.len(), 1);
    assert_eq!(engine.select_victim(), Some("b"));
}

#[test]
fn eviction_remove_nonexistent_lru() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.remove("nonexistent"); // should not panic
    assert_eq!(engine.len(), 1);
}

#[test]
fn eviction_remove_nonexistent_lfu() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.remove("nonexistent"); // should not panic
    assert_eq!(engine.len(), 1);
}

#[test]
fn eviction_clear() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.record_access("b");
    engine.clear();
    assert!(engine.is_empty());
    assert!(engine.select_victim().is_none());
}

#[test]
fn eviction_clear_then_reuse() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.record_access("b");
    engine.clear();
    engine.record_access("c");
    assert_eq!(engine.len(), 1);
    assert_eq!(engine.select_victim(), Some("c"));
}

#[test]
fn eviction_len() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    assert_eq!(engine.len(), 0);
    engine.record_access("a");
    assert_eq!(engine.len(), 1);
    engine.record_access("b");
    assert_eq!(engine.len(), 2);
    engine.record_access("a"); // reaccess, no new entry
    assert_eq!(engine.len(), 2);
}

#[test]
fn eviction_is_empty() {
    let engine = EvictionEngine::new(EvictionPolicy::Lru);
    assert!(engine.is_empty());
}

#[test]
fn eviction_access_count_tracking() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.record_access("a");
    engine.record_access("a");
    assert_eq!(engine.access_count("a"), Some(3));
    assert_eq!(engine.access_count("b"), None);
}

#[test]
fn eviction_access_count_increments() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("x");
    assert_eq!(engine.access_count("x"), Some(1));
    engine.record_access("x");
    assert_eq!(engine.access_count("x"), Some(2));
}

#[test]
fn eviction_policy_accessor() {
    let engine = EvictionEngine::new(EvictionPolicy::Lru);
    assert_eq!(engine.policy(), EvictionPolicy::Lru);
    let engine2 = EvictionEngine::new(EvictionPolicy::Lfu);
    assert_eq!(engine2.policy(), EvictionPolicy::Lfu);
}

#[test]
fn eviction_policy_default() {
    let policy = EvictionPolicy::default();
    assert_eq!(policy, EvictionPolicy::Lru);
}

// ═══════════════════════════════════════════════════════════════════════════════
// QuotaMonitor
// ═══════════════════════════════════════════════════════════════════════════════

fn monitor_with_quota(quota_bytes: u64) -> QuotaMonitor {
    QuotaMonitor::with_config(QuotaConfig {
        quota_bytes,
        warning_threshold: 0.80,
        critical_threshold: 0.95,
    })
}

#[test]
fn quota_monitor_new_defaults() {
    let monitor = QuotaMonitor::new();
    assert_eq!(monitor.current_bytes(), 0);
    assert!(!monitor.is_exceeded());
}

#[test]
fn quota_monitor_default() {
    let monitor = QuotaMonitor::default();
    assert_eq!(monitor.current_bytes(), 0);
}

#[test]
fn quota_monitor_no_alert_below_warning() {
    let mut monitor = monitor_with_quota(1000);
    let alert = monitor.record_addition(500);
    assert!(alert.is_none());
}

#[test]
fn quota_monitor_warning_alert() {
    let mut monitor = monitor_with_quota(1000);
    let alert = monitor.record_addition(850);
    assert!(alert.is_some());
    assert_eq!(alert.unwrap().level, QuotaAlertLevel::Warning);
}

#[test]
fn quota_monitor_critical_alert() {
    let mut monitor = monitor_with_quota(1000);
    let alert = monitor.record_addition(960);
    assert!(alert.is_some());
    assert_eq!(alert.unwrap().level, QuotaAlertLevel::Critical);
}

#[test]
fn quota_monitor_exceeded_alert() {
    let mut monitor = monitor_with_quota(1000);
    let alert = monitor.record_addition(1100);
    assert!(alert.is_some());
    assert_eq!(alert.unwrap().level, QuotaAlertLevel::Exceeded);
    assert!(monitor.is_exceeded());
}

#[test]
fn quota_monitor_remaining_bytes() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(300);
    assert_eq!(monitor.remaining_bytes(), 700);
}

#[test]
fn quota_monitor_remaining_bytes_exceeded() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(1500);
    assert_eq!(monitor.remaining_bytes(), 0);
}

#[test]
fn quota_monitor_usage_percent() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(500);
    assert!((monitor.usage_percent() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn quota_monitor_usage_fraction() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(500);
    assert!((monitor.usage_fraction() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn quota_monitor_recovery_alert() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(900); // Warning
    let alert = monitor.record_removal(800); // Back to Normal
    assert!(alert.is_some());
    assert_eq!(alert.unwrap().level, QuotaAlertLevel::Normal);
}

#[test]
fn quota_monitor_no_duplicate_alert() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(850); // Warning
    let alert = monitor.record_addition(10); // Still warning, no new alert
    assert!(alert.is_none());
}

#[test]
fn quota_monitor_current_alert_level_boundaries() {
    let mut monitor = monitor_with_quota(1000);
    assert_eq!(monitor.current_alert_level(), QuotaAlertLevel::Normal);

    monitor.set_current_bytes(800); // exactly at warning threshold
    assert_eq!(monitor.current_alert_level(), QuotaAlertLevel::Warning);

    monitor.set_current_bytes(950); // exactly at critical threshold
    assert_eq!(monitor.current_alert_level(), QuotaAlertLevel::Critical);

    monitor.set_current_bytes(1000); // at quota
    assert_eq!(monitor.current_alert_level(), QuotaAlertLevel::Critical);

    monitor.set_current_bytes(1001); // over quota
    assert_eq!(monitor.current_alert_level(), QuotaAlertLevel::Exceeded);
}

#[test]
fn quota_monitor_set_current_bytes() {
    let mut monitor = QuotaMonitor::new();
    monitor.set_current_bytes(12345);
    assert_eq!(monitor.current_bytes(), 12345);
}

#[test]
fn quota_monitor_config_accessor() {
    let config = QuotaConfig {
        quota_bytes: 2000,
        warning_threshold: 0.70,
        critical_threshold: 0.90,
    };
    let monitor = QuotaMonitor::with_config(config);
    assert_eq!(monitor.config().quota_bytes, 2000);
    assert_eq!(monitor.config().warning_threshold, 0.70);
}

#[test]
fn quota_monitor_zero_quota_with_usage() {
    let mut monitor = QuotaMonitor::with_config(QuotaConfig {
        quota_bytes: 0,
        warning_threshold: 0.80,
        critical_threshold: 0.95,
    });
    monitor.set_current_bytes(100);
    assert!(monitor.usage_fraction().is_infinite());
}

#[test]
fn quota_monitor_zero_quota_zero_usage() {
    let monitor = QuotaMonitor::with_config(QuotaConfig {
        quota_bytes: 0,
        warning_threshold: 0.80,
        critical_threshold: 0.95,
    });
    assert!((monitor.usage_fraction() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn quota_monitor_saturating_addition() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(u64::MAX - 10);
    monitor.record_addition(100);
    assert_eq!(monitor.current_bytes(), u64::MAX);
}

#[test]
fn quota_monitor_saturating_removal() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(50);
    monitor.record_removal(100);
    assert_eq!(monitor.current_bytes(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// QuotaAlertLevel
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn quota_alert_level_display() {
    assert_eq!(QuotaAlertLevel::Normal.to_string(), "NORMAL");
    assert_eq!(QuotaAlertLevel::Warning.to_string(), "WARNING");
    assert_eq!(QuotaAlertLevel::Critical.to_string(), "CRITICAL");
    assert_eq!(QuotaAlertLevel::Exceeded.to_string(), "EXCEEDED");
}

#[test]
fn quota_alert_level_ordering() {
    assert!(QuotaAlertLevel::Normal < QuotaAlertLevel::Warning);
    assert!(QuotaAlertLevel::Warning < QuotaAlertLevel::Critical);
    assert!(QuotaAlertLevel::Critical < QuotaAlertLevel::Exceeded);
}

#[test]
fn quota_config_default() {
    let config = QuotaConfig::default();
    assert_eq!(config.quota_bytes, 100 * 1024 * 1024);
    assert_eq!(config.warning_threshold, 0.80);
    assert_eq!(config.critical_threshold, 0.95);
}

// ═══════════════════════════════════════════════════════════════════════════════
// DedupStore
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn dedup_store_new_empty() {
    let store = DedupStore::new();
    assert!(store.is_empty());
    assert_eq!(store.unique_content_count(), 0);
    assert_eq!(store.total_key_count(), 0);
}

#[test]
fn dedup_store_register_unique() {
    let mut store = DedupStore::new();
    let result = store.register("key1", b"content1");
    assert!(matches!(result, DedupResult::Unique { .. }));
    assert_eq!(store.total_key_count(), 1);
    assert_eq!(store.unique_content_count(), 1);
}

#[test]
fn dedup_store_register_duplicate() {
    let mut store = DedupStore::new();
    store.register("key1", b"same content");
    let result = store.register("key2", b"same content");
    assert!(matches!(result, DedupResult::Duplicate { .. }));
    assert_eq!(store.total_key_count(), 2);
    assert_eq!(store.unique_content_count(), 1);
}

#[test]
fn dedup_store_check_unique() {
    let store = DedupStore::new();
    let result = store.check(b"anything");
    assert!(matches!(result, DedupResult::Unique { .. }));
}

#[test]
fn dedup_store_check_duplicate() {
    let mut store = DedupStore::new();
    store.register("key1", b"content");
    let result = store.check(b"content");
    assert!(matches!(result, DedupResult::Duplicate { .. }));
}

#[test]
fn dedup_store_check_does_not_register() {
    let mut store = DedupStore::new();
    store.register("key1", b"content");
    store.check(b"new content"); // should not register
    assert_eq!(store.total_key_count(), 1);
}

#[test]
fn dedup_store_hash_for_key() {
    let mut store = DedupStore::new();
    store.register("key1", b"data");
    assert!(store.hash_for_key("key1").is_some());
    assert!(store.hash_for_key("missing").is_none());
}

#[test]
fn dedup_store_hash_for_key_matches_compute() {
    let mut store = DedupStore::new();
    store.register("k1", b"data");
    let hash = store.hash_for_key("k1").unwrap();
    let expected = DedupStore::compute_hash(b"data");
    assert_eq!(hash, &expected);
}

#[test]
fn dedup_store_keys_for_hash() {
    let mut store = DedupStore::new();
    store.register("key1", b"same");
    store.register("key2", b"same");
    let hash = store.hash_for_key("key1").unwrap().clone();
    let keys = store.keys_for_hash(&hash).unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains("key1"));
    assert!(keys.contains("key2"));
}

#[test]
fn dedup_store_keys_for_hash_nonexistent() {
    let store = DedupStore::new();
    assert!(store.keys_for_hash("nonexistent").is_none());
}

#[test]
fn dedup_store_remove() {
    let mut store = DedupStore::new();
    store.register("key1", b"data");
    store.remove("key1");
    assert!(store.is_empty());
    assert!(store.hash_for_key("key1").is_none());
}

#[test]
fn dedup_store_remove_one_of_two() {
    let mut store = DedupStore::new();
    store.register("key1", b"content");
    store.register("key2", b"content");
    store.remove("key1");
    assert_eq!(store.total_key_count(), 1);
    assert!(store.hash_for_key("key2").is_some());
    // Hash entry should still exist with one key
    assert_eq!(store.unique_content_count(), 1);
}

#[test]
fn dedup_store_remove_nonexistent() {
    let mut store = DedupStore::new();
    store.remove("nonexistent"); // should not panic
    assert!(store.is_empty());
}

#[test]
fn dedup_store_duplicate_key_count() {
    let mut store = DedupStore::new();
    store.register("k1", b"same");
    store.register("k2", b"same");
    store.register("k3", b"different");
    assert_eq!(store.duplicate_key_count(), 1); // k2 is duplicate of k1
}

#[test]
fn dedup_store_duplicate_key_count_no_duplicates() {
    let mut store = DedupStore::new();
    store.register("k1", b"a");
    store.register("k2", b"b");
    assert_eq!(store.duplicate_key_count(), 0);
}

#[test]
fn dedup_store_estimated_savings() {
    let mut store = DedupStore::new();
    store.register("k1", b"same");
    store.register("k2", b"same");
    assert_eq!(store.estimated_savings(100), 100); // 1 duplicate * 100 bytes
}

#[test]
fn dedup_store_estimated_savings_no_duplicates() {
    let mut store = DedupStore::new();
    store.register("k1", b"a");
    store.register("k2", b"b");
    assert_eq!(store.estimated_savings(100), 0);
}

#[test]
fn dedup_store_clear() {
    let mut store = DedupStore::new();
    store.register("k1", b"data");
    store.register("k2", b"data");
    store.clear();
    assert!(store.is_empty());
    assert_eq!(store.unique_content_count(), 0);
    assert_eq!(store.total_key_count(), 0);
}

#[test]
fn dedup_store_compute_hash_deterministic() {
    let h1 = DedupStore::compute_hash(b"hello");
    let h2 = DedupStore::compute_hash(b"hello");
    assert_eq!(h1, h2);
}

#[test]
fn dedup_store_compute_hash_different_content() {
    let h1 = DedupStore::compute_hash(b"hello");
    let h2 = DedupStore::compute_hash(b"world");
    assert_ne!(h1, h2);
}

#[test]
fn dedup_store_compute_hash_empty() {
    let hash = DedupStore::compute_hash(b"");
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn dedup_store_reregister_different_content() {
    let mut store = DedupStore::new();
    store.register("k1", b"content A");
    let hash_a = store.hash_for_key("k1").cloned().unwrap();
    store.register("k1", b"content B");
    let hash_b = store.hash_for_key("k1").cloned().unwrap();
    assert_ne!(hash_a, hash_b);
    assert_eq!(store.total_key_count(), 1);
    // Old hash entry should be cleaned up
    assert!(store.keys_for_hash(&hash_a).is_none());
}

#[test]
fn dedup_store_reregister_same_content() {
    let mut store = DedupStore::new();
    store.register("k1", b"data");
    let result = store.register("k1", b"data");
    assert!(matches!(result, DedupResult::Unique { .. }));
    assert_eq!(store.total_key_count(), 1);
}

#[test]
fn dedup_store_three_duplicates_savings() {
    let mut store = DedupStore::new();
    store.register("k1", b"same");
    store.register("k2", b"same");
    store.register("k3", b"same");
    store.register("k4", b"different");
    assert_eq!(store.duplicate_key_count(), 2);
    assert_eq!(store.estimated_savings(100), 200);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CacheIndex
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn cache_index_new_empty() {
    let index = CacheIndex::new();
    assert!(index.is_empty());
    assert_eq!(index.len(), 0);
}

#[test]
fn cache_index_insert_and_get() {
    let mut index = CacheIndex::new();
    index.insert(make_index_entry("k1", CacheEntryType::Constitution, 100));
    assert_eq!(index.len(), 1);
    assert!(index.contains("k1"));
    let entry = index.get("k1").unwrap();
    assert_eq!(entry.entry_type, CacheEntryType::Constitution);
    assert_eq!(entry.size_bytes, 100);
}

#[test]
fn cache_index_get_nonexistent() {
    let index = CacheIndex::new();
    assert!(index.get("missing").is_none());
}

#[test]
fn cache_index_contains_false_for_missing() {
    let index = CacheIndex::new();
    assert!(!index.contains("missing"));
}

#[test]
fn cache_index_remove() {
    let mut index = CacheIndex::new();
    index.insert(make_index_entry("k1", CacheEntryType::Law, 50));
    let removed = index.remove("k1");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().size_bytes, 50);
    assert!(index.is_empty());
}

#[test]
fn cache_index_remove_nonexistent() {
    let mut index = CacheIndex::new();
    assert!(index.remove("missing").is_none());
}

#[test]
fn cache_index_total_size_bytes() {
    let mut index = CacheIndex::new();
    index.insert(make_index_entry("k1", CacheEntryType::Constitution, 100));
    index.insert(make_index_entry("k2", CacheEntryType::Law, 200));
    index.insert(make_index_entry("k3", CacheEntryType::Rule, 50));
    assert_eq!(index.total_size_bytes(), 350);
}

#[test]
fn cache_index_total_size_bytes_empty() {
    let index = CacheIndex::new();
    assert_eq!(index.total_size_bytes(), 0);
}

#[test]
fn cache_index_count_by_type() {
    let mut index = CacheIndex::new();
    index.insert(make_index_entry("c1", CacheEntryType::Constitution, 100));
    index.insert(make_index_entry("c2", CacheEntryType::Constitution, 100));
    index.insert(make_index_entry("l1", CacheEntryType::Law, 50));
    index.insert(make_index_entry("r1", CacheEntryType::Rule, 30));
    assert_eq!(index.count_by_type(CacheEntryType::Constitution), 2);
    assert_eq!(index.count_by_type(CacheEntryType::Law), 1);
    assert_eq!(index.count_by_type(CacheEntryType::Rule), 1);
}

#[test]
fn cache_index_count_by_type_empty() {
    let index = CacheIndex::new();
    assert_eq!(index.count_by_type(CacheEntryType::Constitution), 0);
}

#[test]
fn cache_index_summary() {
    let mut index = CacheIndex::new();
    index.insert(make_index_entry("c1", CacheEntryType::Constitution, 100));
    index.insert(make_index_entry("l1", CacheEntryType::Law, 200));
    index.insert(make_index_entry("r1", CacheEntryType::Rule, 50));
    let summary = index.summary();
    assert_eq!(summary.total_entries, 3);
    assert_eq!(summary.constitutions, 1);
    assert_eq!(summary.laws, 1);
    assert_eq!(summary.rules, 1);
    assert_eq!(summary.total_size_bytes, 350);
}

#[test]
fn cache_index_summary_empty() {
    let index = CacheIndex::new();
    let summary = index.summary();
    assert_eq!(summary.total_entries, 0);
    assert_eq!(summary.total_size_bytes, 0);
}

#[test]
fn cache_index_clear() {
    let mut index = CacheIndex::new();
    index.insert(make_index_entry("k1", CacheEntryType::Constitution, 100));
    index.insert(make_index_entry("k2", CacheEntryType::Rule, 50));
    index.clear();
    assert!(index.is_empty());
    assert_eq!(index.total_size_bytes(), 0);
}

#[test]
fn cache_index_keys_with_hash() {
    let mut index = CacheIndex::new();
    let mut e1 = make_index_entry("k1", CacheEntryType::Constitution, 100);
    e1.content_hash = Some("abc123".to_string());
    index.insert(e1);
    let mut e2 = make_index_entry("k2", CacheEntryType::Law, 50);
    e2.content_hash = Some("abc123".to_string());
    index.insert(e2);
    let mut e3 = make_index_entry("k3", CacheEntryType::Rule, 30);
    e3.content_hash = Some("def456".to_string());
    index.insert(e3);
    let mut matches = index.keys_with_hash("abc123");
    matches.sort();
    assert_eq!(matches, vec!["k1", "k2"]);
}

#[test]
fn cache_index_keys_with_hash_no_matches() {
    let index = CacheIndex::new();
    assert!(index.keys_with_hash("nonexistent").is_empty());
}

#[test]
fn cache_index_insert_overwrites() {
    let mut index = CacheIndex::new();
    index.insert(make_index_entry("k1", CacheEntryType::Rule, 100));
    index.insert(make_index_entry("k1", CacheEntryType::Law, 200));
    assert_eq!(index.len(), 1);
    let entry = index.get("k1").unwrap();
    assert_eq!(entry.entry_type, CacheEntryType::Law);
    assert_eq!(entry.size_bytes, 200);
}

#[test]
fn cache_index_keys_iterator() {
    let mut index = CacheIndex::new();
    index.insert(make_index_entry("x", CacheEntryType::Rule, 10));
    index.insert(make_index_entry("y", CacheEntryType::Law, 20));
    let mut keys: Vec<&String> = index.keys().collect();
    keys.sort();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0], "x");
    assert_eq!(keys[1], "y");
}

#[test]
fn cache_index_iter() {
    let mut index = CacheIndex::new();
    index.insert(make_index_entry("a", CacheEntryType::Constitution, 50));
    index.insert(make_index_entry("b", CacheEntryType::Rule, 60));
    let entries: Vec<_> = index.iter().collect();
    assert_eq!(entries.len(), 2);
}

#[test]
fn cache_entry_type_display() {
    assert_eq!(CacheEntryType::Constitution.to_string(), "constitution");
    assert_eq!(CacheEntryType::Law.to_string(), "law");
    assert_eq!(CacheEntryType::Rule.to_string(), "rule");
}
