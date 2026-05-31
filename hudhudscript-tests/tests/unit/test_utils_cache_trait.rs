use hudhudscript_utils::cache_trait::CacheStats;

#[test]
fn cache_stats_hit_rate_empty() {
    let stats = CacheStats::default();
    assert_eq!(stats.hit_rate(), 0.0);
}

#[test]
fn cache_stats_hit_rate_all_hits() {
    let stats = CacheStats {
        hits: 10,
        misses: 0,
        ..Default::default()
    };
    assert_eq!(stats.hit_rate(), 1.0);
}

#[test]
fn cache_stats_hit_rate_mixed() {
    let stats = CacheStats {
        hits: 3,
        misses: 7,
        ..Default::default()
    };
    assert!((stats.hit_rate() - 0.3).abs() < f64::EPSILON);
}

#[test]
fn cache_stats_display() {
    let stats = CacheStats {
        hits: 100,
        misses: 50,
        evictions: 10,
        size: 42,
        capacity: 100,
    };
    let s = format!("{}", stats);
    assert!(s.contains("hits: 100"));
    assert!(s.contains("66.7%"));
}
