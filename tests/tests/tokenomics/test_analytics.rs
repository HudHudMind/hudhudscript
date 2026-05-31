//! Tests for tokenomics::analytics — AnalyticsEngine, UsagePatterns, Anomaly

use hudhudscript_tokenomics::analytics::AnalyticsEngine;
use hudhudscript_tokenomics::types::TokenUsageRecord;

fn make_usage(tokens: u64) -> TokenUsageRecord {
    TokenUsageRecord::new("test_user".into(), "op".into(), tokens)
}

fn make_usage_with_op(op: &str, tokens: u64) -> TokenUsageRecord {
    TokenUsageRecord::new("test_user".into(), op.into(), tokens)
}

// ---------------------------------------------------------------------------
// analyze_patterns — basic
// ---------------------------------------------------------------------------

#[test]
fn test_analyze_patterns_empty() {
    let patterns = AnalyticsEngine::analyze_patterns(&[]);
    assert_eq!(patterns.total_operations, 0);
    assert!(patterns.by_operation.is_empty());
    assert!(patterns.by_hour.is_empty());
}

#[test]
fn test_analyze_patterns_single_record() {
    let data = vec![make_usage(100)];
    let patterns = AnalyticsEngine::analyze_patterns(&data);
    assert_eq!(patterns.total_operations, 1);
    assert_eq!(*patterns.by_operation.get("op").unwrap(), 100);
}

#[test]
fn test_analyze_patterns_multiple_same_op() {
    let data = vec![make_usage(50), make_usage(30), make_usage(20)];
    let patterns = AnalyticsEngine::analyze_patterns(&data);
    assert_eq!(patterns.total_operations, 3);
    assert_eq!(*patterns.by_operation.get("op").unwrap(), 100);
}

#[test]
fn test_analyze_patterns_multiple_ops() {
    let data = vec![
        make_usage_with_op("compile", 50),
        make_usage_with_op("compile", 30),
        make_usage_with_op("run", 20),
    ];
    let patterns = AnalyticsEngine::analyze_patterns(&data);
    assert_eq!(patterns.total_operations, 3);
    assert_eq!(*patterns.by_operation.get("compile").unwrap(), 80);
    assert_eq!(*patterns.by_operation.get("run").unwrap(), 20);
}

#[test]
fn test_analyze_patterns_by_hour_populated() {
    let data = vec![make_usage(100)];
    let patterns = AnalyticsEngine::analyze_patterns(&data);
    assert!(!patterns.by_hour.is_empty());
}

#[test]
fn test_analyze_patterns_by_hour_sum() {
    let data = vec![make_usage(100), make_usage(200)];
    let patterns = AnalyticsEngine::analyze_patterns(&data);
    let total: u64 = patterns.by_hour.values().sum();
    assert_eq!(total, 300);
}

#[test]
fn test_analyze_patterns_by_operation_count() {
    let data = vec![
        make_usage_with_op("a", 10),
        make_usage_with_op("b", 20),
        make_usage_with_op("c", 30),
    ];
    let patterns = AnalyticsEngine::analyze_patterns(&data);
    assert_eq!(patterns.by_operation.len(), 3);
}

#[test]
fn test_analyze_patterns_total_operations_matches_input() {
    let data: Vec<TokenUsageRecord> = (0..15).map(|i| make_usage(i * 10)).collect();
    let patterns = AnalyticsEngine::analyze_patterns(&data);
    assert_eq!(patterns.total_operations, 15);
}

// ---------------------------------------------------------------------------
// detect_anomalies — edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_detect_anomalies_empty() {
    let anomalies = AnalyticsEngine::detect_anomalies(&[]);
    assert!(anomalies.is_empty());
}

#[test]
fn test_detect_anomalies_single_record() {
    let data = vec![make_usage(100)];
    assert!(AnalyticsEngine::detect_anomalies(&data).is_empty());
}

#[test]
fn test_detect_anomalies_two_identical() {
    let data = vec![make_usage(100), make_usage(100)];
    // stddev == 0 => no anomalies
    assert!(AnalyticsEngine::detect_anomalies(&data).is_empty());
}

#[test]
fn test_detect_anomalies_uniform_data() {
    let data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(100)).collect();
    assert!(AnalyticsEngine::detect_anomalies(&data).is_empty());
}

// ---------------------------------------------------------------------------
// detect_anomalies — spike detection
// ---------------------------------------------------------------------------

#[test]
fn test_detect_anomalies_spike_above() {
    let mut data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(100)).collect();
    data.push(make_usage(10_000));
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    assert!(!anomalies.is_empty());
    assert!(anomalies.iter().any(|a| a.description.contains("above")));
}

#[test]
fn test_detect_anomalies_spike_below() {
    let mut data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(10_000)).collect();
    data.push(make_usage(1));
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    let below: Vec<_> = anomalies
        .iter()
        .filter(|a| a.description.contains("below"))
        .collect();
    assert!(!below.is_empty());
}

#[test]
fn test_detect_anomalies_severity_positive() {
    let mut data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(100)).collect();
    data.push(make_usage(10_000));
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    for a in &anomalies {
        assert!(a.severity >= 0.0);
        assert!(a.severity <= 1.0);
    }
}

#[test]
fn test_detect_anomalies_severity_clamped() {
    let mut data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(100)).collect();
    data.push(make_usage(1_000_000)); // extreme outlier
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    assert!(!anomalies.is_empty());
    // Severity is clamped to [0.0, 1.0]
    assert!(anomalies[0].severity <= 1.0);
}

#[test]
fn test_detect_anomalies_description_contains_z_score() {
    let mut data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(100)).collect();
    data.push(make_usage(10_000));
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    assert!(anomalies[0].description.contains("z="));
}

#[test]
fn test_detect_anomalies_description_contains_mean() {
    let mut data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(100)).collect();
    data.push(make_usage(10_000));
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    assert!(anomalies[0].description.contains("mean"));
}

#[test]
fn test_detect_anomalies_timestamp_present() {
    let mut data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(100)).collect();
    data.push(make_usage(10_000));
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    // Timestamp should be from the outlier record
    let _ = anomalies[0].timestamp;
}

// ---------------------------------------------------------------------------
// detect_anomalies — no false positives for gradual variation
// ---------------------------------------------------------------------------

#[test]
fn test_detect_anomalies_gradual_variation() {
    // Values within normal range should not trigger anomalies
    let data: Vec<TokenUsageRecord> = (90..=110).map(|v| make_usage(v)).collect();
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    assert!(anomalies.is_empty());
}

#[test]
fn test_detect_anomalies_multiple_outliers() {
    let mut data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(100)).collect();
    data.push(make_usage(10_000));
    data.push(make_usage(10_000));
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    // Both outliers should be flagged (or neither, depending on how they shift the mean)
    // With 20 x 100 + 2 x 10000, the mean shifts but 10000 should still be anomalous
    assert!(!anomalies.is_empty());
}

// ---------------------------------------------------------------------------
// UsagePatterns fields
// ---------------------------------------------------------------------------

#[test]
fn test_usage_patterns_debug() {
    let patterns = AnalyticsEngine::analyze_patterns(&[make_usage(100)]);
    let debug = format!("{:?}", patterns);
    assert!(debug.contains("UsagePatterns"));
}

#[test]
fn test_anomaly_debug() {
    let mut data: Vec<TokenUsageRecord> = (0..20).map(|_| make_usage(100)).collect();
    data.push(make_usage(10_000));
    let anomalies = AnalyticsEngine::detect_anomalies(&data);
    let debug = format!("{:?}", anomalies[0]);
    assert!(debug.contains("Anomaly"));
}
