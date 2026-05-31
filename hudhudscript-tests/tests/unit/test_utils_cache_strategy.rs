use hudhudscript_utils::cache_strategy::*;
use std::time::Duration;

#[test]
fn test_default_strategy() {
    let s = CacheStrategy::default();
    assert_eq!(s.lookup, LookupMode::Exact);
    assert_eq!(s.max_entries, 1000);
    assert_eq!(s.ttl, Duration::from_secs(300));
    assert!(!s.uses_embeddings());
}

#[test]
fn test_semantic_strategy() {
    let s = CacheStrategy::semantic(500, Duration::from_secs(60));
    assert_eq!(s.lookup, LookupMode::Semantic);
    assert!(s.uses_embeddings());
}

#[test]
fn test_hybrid_strategy() {
    let s = CacheStrategy::hybrid(2000, Duration::from_secs(600));
    assert_eq!(s.lookup, LookupMode::Hybrid);
    assert!(s.uses_embeddings());
    assert_eq!(s.max_entries, 2000);
}

#[test]
fn test_exact_no_embeddings() {
    let s = CacheStrategy::exact(100, Duration::from_secs(30));
    assert!(!s.uses_embeddings());
}
