use hudhudscript_cache::{EvictionEngine, EvictionPolicy};

#[test]
fn test_lru_basic_eviction_order() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);

    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("c");

    assert_eq!(engine.select_victim(), Some("a"));
}

#[test]
fn test_lru_reaccess_moves_to_back() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);

    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("c");
    engine.record_access("a"); // re-access

    assert_eq!(engine.select_victim(), Some("b"));
}

#[test]
fn test_lru_select_multiple_victims() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);

    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("c");
    engine.record_access("d");

    let victims = engine.select_victims(2);
    assert_eq!(victims, vec!["a", "b"]);
}

#[test]
fn test_lfu_basic_eviction_order() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);

    engine.record_access("a");
    engine.record_access("a");
    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("c");
    engine.record_access("c");

    assert_eq!(engine.select_victim(), Some("b"));
}

#[test]
fn test_lfu_tiebreak_by_recency() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);

    engine.record_access("a");
    engine.record_access("b");

    assert_eq!(engine.select_victim(), Some("a"));
}

#[test]
fn test_lfu_select_multiple_victims() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);

    engine.record_access("a");
    engine.record_access("b");
    engine.record_access("c");
    engine.record_access("a");
    engine.record_access("c");
    engine.record_access("a");

    let victims = engine.select_victims(2);
    assert_eq!(victims, vec!["b", "c"]);
}

#[test]
fn test_remove_key() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);

    engine.record_access("a");
    engine.record_access("b");

    engine.remove("a");
    assert_eq!(engine.len(), 1);
    assert_eq!(engine.select_victim(), Some("b"));
}

#[test]
fn test_empty_engine_returns_none() {
    let engine = EvictionEngine::new(EvictionPolicy::Lru);
    assert!(engine.select_victim().is_none());
    assert!(engine.is_empty());
}

#[test]
fn test_clear() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.record_access("b");
    engine.clear();

    assert!(engine.is_empty());
    assert_eq!(engine.len(), 0);
    assert!(engine.select_victim().is_none());
}

#[test]
fn test_access_count_tracking() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.record_access("a");
    engine.record_access("a");

    assert_eq!(engine.access_count("a"), Some(3));
    assert_eq!(engine.access_count("b"), None);
}

#[test]
fn test_lfu_empty_engine_returns_none() {
    let engine = EvictionEngine::new(EvictionPolicy::Lfu);
    assert!(engine.select_victim().is_none());
    assert!(engine.is_empty());
}

#[test]
fn test_lfu_empty_select_victims() {
    let engine = EvictionEngine::new(EvictionPolicy::Lfu);
    let victims = engine.select_victims(5);
    assert_eq!(victims.len(), 0);
}

#[test]
fn test_lru_empty_select_victims() {
    let engine = EvictionEngine::new(EvictionPolicy::Lru);
    let victims = engine.select_victims(5);
    assert_eq!(victims.len(), 0);
}

#[test]
fn test_remove_nonexistent_key_in_lfu() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.remove("nonexistent"); // should not panic
    assert_eq!(engine.len(), 1);
}

#[test]
fn test_remove_nonexistent_key_in_lru() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.remove("nonexistent"); // should not panic
    assert_eq!(engine.len(), 1);
}

#[test]
fn test_policy_accessor() {
    let lru_engine = EvictionEngine::new(EvictionPolicy::Lru);
    assert_eq!(lru_engine.policy(), EvictionPolicy::Lru);

    let lfu_engine = EvictionEngine::new(EvictionPolicy::Lfu);
    assert_eq!(lfu_engine.policy(), EvictionPolicy::Lfu);
}

#[test]
fn test_eviction_policy_default_is_lru() {
    let policy = EvictionPolicy::default();
    assert_eq!(policy, EvictionPolicy::Lru);
}

#[test]
fn test_lfu_remove_key() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("a");
    engine.record_access("b");
    engine.remove("a");
    assert_eq!(engine.len(), 1);
    assert_eq!(engine.select_victim(), Some("b"));
}

#[test]
fn test_lru_select_victims_more_than_available() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.record_access("b");
    let victims = engine.select_victims(10);
    assert_eq!(victims.len(), 2);
    assert_eq!(victims, vec!["a", "b"]);
}

#[test]
fn test_lfu_select_victims_more_than_available() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lfu);
    engine.record_access("x");
    let victims = engine.select_victims(10);
    assert_eq!(victims.len(), 1);
    assert_eq!(victims[0], "x");
}

#[test]
fn test_clear_resets_tick() {
    let mut engine = EvictionEngine::new(EvictionPolicy::Lru);
    engine.record_access("a");
    engine.record_access("b");
    engine.clear();
    engine.record_access("c");
    assert_eq!(engine.len(), 1);
    assert_eq!(engine.select_victim(), Some("c"));
}
