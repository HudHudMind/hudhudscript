//! Public API tests for tokenomics::metrics

use chrono::Utc;
use hudhudscript_tokenomics::metrics::MetricsCollector;
use hudhudscript_tokenomics::types::Metrics;

#[tokio::test]
async fn test_new() {
    let mc = MetricsCollector::new();
    let output = mc.export_prometheus();
    assert!(output.contains("tokenomics_total_users"));
}

#[test]
fn test_default_impl() {
    let mc = MetricsCollector::default();
    let output = mc.export_prometheus();
    assert!(output.contains("tokenomics_total_users"));
}

#[tokio::test]
async fn test_set_gauge() {
    let mc = MetricsCollector::new();
    mc.set_gauge("tokenomics_total_users", &[], 42.0).await;
    let output = mc.export_prometheus();
    assert!(output.contains("tokenomics_total_users 42"));
}

#[tokio::test]
async fn test_inc_counter() {
    let mc = MetricsCollector::new();
    mc.inc_counter("tokenomics_total_usage_tokens", &[], 100.0)
        .await;
    mc.inc_counter("tokenomics_total_usage_tokens", &[], 50.0)
        .await;
    let output = mc.export_prometheus();
    assert!(output.contains("tokenomics_total_usage_tokens 150"));
}

#[tokio::test]
async fn test_register_and_use() {
    let mc = MetricsCollector::new();
    mc.register("http_requests_total", "counter", "Total HTTP requests")
        .await;
    mc.inc_counter("http_requests_total", &[("method", "GET")], 10.0)
        .await;
    let output = mc.export_prometheus();
    assert!(output.contains("http_requests_total{method=\"GET\"} 10"));
}

#[tokio::test]
async fn test_histogram() {
    let mc = MetricsCollector::new();
    mc.register("request_duration", "histogram", "Duration")
        .await;
    for &v in &[5.0, 15.0, 80.0, 200.0, 1500.0] {
        mc.observe_histogram("request_duration", v).await;
    }
    let output = mc.export_prometheus();
    assert!(output.contains("request_duration_bucket{le=\"10\"} 1"));
    assert!(output.contains("request_duration_bucket{le=\"+Inf\"} 5"));
    assert!(output.contains("request_duration_count 5"));
}

#[tokio::test]
async fn test_collect_round_trip() {
    let mc = MetricsCollector::new();
    let m = Metrics {
        total_users: 5,
        total_usage: 1000,
        average_usage: 200.0,
        peak_usage: 500,
        prediction_accuracy: 0.85,
        timestamp: Utc::now(),
    };
    mc.update_from_metrics(&m).await;
    let collected = mc.collect().await;
    assert_eq!(collected.total_users, 5);
    assert_eq!(collected.total_usage, 1000);
    assert_eq!(collected.peak_usage, 500);
}

#[tokio::test]
async fn test_register_gauge_default_kind() {
    let mc = MetricsCollector::new();
    mc.register("custom_metric", "something_unknown", "A custom gauge")
        .await;
    mc.set_gauge("custom_metric", &[], 42.0).await;
    let output = mc.export_prometheus();
    assert!(output.contains("# TYPE custom_metric gauge"));
    assert!(output.contains("custom_metric 42"));
}

#[test]
fn test_export_empty_values() {
    let mc = MetricsCollector::new();
    let output = mc.export_prometheus();
    assert!(output.contains("tokenomics_total_users 0"));
}

#[tokio::test]
async fn test_export_contains_help_and_type() {
    let mc = MetricsCollector::new();
    mc.set_gauge("tokenomics_total_users", &[], 42.0).await;
    let output = mc.export_prometheus();
    assert!(output.contains("# HELP tokenomics_total_users"));
    assert!(output.contains("# TYPE tokenomics_total_users gauge"));
}
