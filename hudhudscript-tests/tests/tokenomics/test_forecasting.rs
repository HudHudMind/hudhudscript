//! Public API tests for tokenomics::forecasting
//! Covers: ForecastingEngine (new, default, with_params, forecast).

use chrono::{Duration, Utc};
use hudhudscript_tokenomics::forecasting::engine::ForecastingEngine;
use hudhudscript_tokenomics::types::TimeSeriesPoint;

// ── helpers ─────────────────────────────────────────────────────────

fn make_series(values: &[f64]) -> Vec<TimeSeriesPoint> {
    let base = Utc::now() - Duration::hours(values.len() as i64);
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| TimeSeriesPoint {
            timestamp: base + Duration::hours(i as i64),
            value: v,
            metadata: serde_json::json!({}),
        })
        .collect()
}

// ── ForecastingEngine construction ──────────────────────────────────

#[test]
fn forecasting_engine_default_constructs() {
    let _engine = ForecastingEngine::default();
}

#[test]
fn forecasting_engine_new_constructs() {
    let _engine = ForecastingEngine::new();
}

#[test]
fn forecasting_engine_with_params_constructs() {
    let _engine = ForecastingEngine::with_params(0.5, 0.3);
}

#[test]
fn forecasting_engine_with_params_clamps_above_max() {
    // alpha and beta are clamped to 0.99 — should not panic
    let _engine = ForecastingEngine::with_params(5.0, 10.0);
}

#[test]
fn forecasting_engine_with_params_clamps_below_min() {
    // negative values clamped to 0.01 — should not panic
    let _engine = ForecastingEngine::with_params(-2.0, -0.5);
}

// ── ForecastingEngine::forecast — empty data ─────────────────────────

#[tokio::test]
async fn forecast_empty_data_predicted_usage_zero() {
    let engine = ForecastingEngine::new();
    let pred = engine.forecast(&[], 3600).await.unwrap();
    assert_eq!(pred.predicted_usage, 0);
}

#[tokio::test]
async fn forecast_empty_data_confidence_zero() {
    let engine = ForecastingEngine::new();
    let pred = engine.forecast(&[], 3600).await.unwrap();
    assert_eq!(pred.confidence, 0.0);
}

#[tokio::test]
async fn forecast_empty_data_horizon_preserved() {
    let engine = ForecastingEngine::new();
    let pred = engine.forecast(&[], 7200).await.unwrap();
    assert_eq!(pred.horizon_seconds, 7200);
}

#[tokio::test]
async fn forecast_empty_data_model_version_correct() {
    let engine = ForecastingEngine::new();
    let pred = engine.forecast(&[], 3600).await.unwrap();
    assert_eq!(pred.model_version, "holt-ses-v1");
}

// ── ForecastingEngine::forecast — single point ───────────────────────

#[tokio::test]
async fn forecast_single_point_confidence_is_0_3() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[100.0]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert_eq!(pred.confidence, 0.3);
}

#[tokio::test]
async fn forecast_single_point_produces_nonzero_usage_for_positive_value() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[200.0]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert!(pred.predicted_usage > 0);
}

#[tokio::test]
async fn forecast_single_point_model_version_correct() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[100.0]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert_eq!(pred.model_version, "holt-ses-v1");
}

#[tokio::test]
async fn forecast_single_point_horizon_preserved() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[50.0]);
    let pred = engine.forecast(&data, 1800).await.unwrap();
    assert_eq!(pred.horizon_seconds, 1800);
}

#[tokio::test]
async fn forecast_single_point_exact_calculation() {
    // value=3600 with horizon=3600s: predicted = 3600.0 * 3600 / 3600 = 3600
    let engine = ForecastingEngine::new();
    let data = make_series(&[3600.0]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert_eq!(pred.predicted_usage, 3600);
}

// ── ForecastingEngine::forecast — multiple points ────────────────────

#[tokio::test]
async fn forecast_constant_series_stays_near_value() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[100.0; 10]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    let diff = (pred.predicted_usage as f64 - 100.0).abs();
    assert!(diff < 20.0, "Expected ~100, got {}", pred.predicted_usage);
}

#[tokio::test]
async fn forecast_constant_series_high_confidence() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[50.0; 20]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert!(
        pred.confidence > 0.5,
        "Expected high confidence, got {}",
        pred.confidence
    );
}

#[tokio::test]
async fn forecast_trending_series_predicts_above_last_value() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert!(
        pred.predicted_usage > 80,
        "Expected >80, got {}",
        pred.predicted_usage
    );
}

#[tokio::test]
async fn forecast_decreasing_series_predicts_below_last_value() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[100.0, 90.0, 80.0, 70.0, 60.0, 50.0, 40.0, 30.0, 20.0]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert!(
        pred.predicted_usage < 20,
        "Expected < 20 for decreasing series, got {}",
        pred.predicted_usage
    );
}

#[tokio::test]
async fn forecast_model_version_always_holt_ses_v1() {
    let engine = ForecastingEngine::new();
    for n in [1, 2, 5, 10] {
        let data = make_series(&vec![100.0; n]);
        let pred = engine.forecast(&data, 3600).await.unwrap();
        assert_eq!(pred.model_version, "holt-ses-v1");
    }
}

#[tokio::test]
async fn forecast_confidence_clamped_to_valid_range() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[10.0, 20.0, 30.0, 40.0, 50.0]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert!(
        pred.confidence >= 0.1,
        "confidence >= 0.1, got {}",
        pred.confidence
    );
    assert!(
        pred.confidence <= 0.99,
        "confidence <= 0.99, got {}",
        pred.confidence
    );
}

// ── with_params variations ───────────────────────────────────────────

#[tokio::test]
async fn forecast_with_high_alpha_beta_trending_series() {
    let engine = ForecastingEngine::with_params(0.5, 0.5);
    let data = make_series(&[10.0, 20.0, 30.0, 40.0, 50.0]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert!(
        pred.predicted_usage > 50,
        "Expected >50 with high alpha/beta, got {}",
        pred.predicted_usage
    );
    assert_eq!(pred.model_version, "holt-ses-v1");
}

#[tokio::test]
async fn forecast_with_clamped_params_does_not_panic() {
    let engine = ForecastingEngine::with_params(5.0, -2.0);
    let data = make_series(&[100.0; 5]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    // Just verify it completes without panic
    let _ = pred.predicted_usage;
}

#[tokio::test]
async fn forecast_with_low_alpha_beta_smooths_more() {
    let engine = ForecastingEngine::with_params(0.01, 0.01);
    let data = make_series(&[100.0; 10]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    // Should still produce a valid prediction near 100
    let diff = (pred.predicted_usage as f64 - 100.0).abs();
    assert!(diff < 50.0);
}

// ── zero-observed APE branch ──────────────────────────────────────────

#[tokio::test]
async fn forecast_all_zero_values_confidence_is_0_5() {
    // When all observed values are 0, ape_count==0 => mape=0.5 => confidence=0.5
    let engine = ForecastingEngine::new();
    let data = make_series(&[0.0, 0.0, 0.0, 0.0]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert_eq!(pred.confidence, 0.5);
}

#[tokio::test]
async fn forecast_all_zero_values_predicted_usage_is_zero() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[0.0; 5]);
    let pred = engine.forecast(&data, 3600).await.unwrap();
    assert_eq!(pred.predicted_usage, 0);
}

// ── horizon variations ───────────────────────────────────────────────

#[tokio::test]
async fn forecast_zero_horizon_returns_ok() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[100.0; 5]);
    let pred = engine.forecast(&data, 0).await.unwrap();
    assert_eq!(pred.horizon_seconds, 0);
}

#[tokio::test]
async fn forecast_large_horizon_returns_ok() {
    let engine = ForecastingEngine::new();
    let data = make_series(&[100.0; 5]);
    let pred = engine.forecast(&data, 86400 * 365).await.unwrap(); // 1 year
    assert!(pred.horizon_seconds > 0);
}
