//! Public API tests for tokenomics::forecasting_ml

use chrono::Utc;
use hudhudscript_tokenomics::forecasting::*;
use hudhudscript_tokenomics::types::TimeSeriesPoint;

fn make_series(values: &[f64]) -> Vec<TimeSeriesPoint> {
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| TimeSeriesPoint {
            timestamp: Utc::now() + chrono::Duration::hours(i as i64),
            value: v,
            metadata: serde_json::json!({}),
        })
        .collect()
}

#[test]
fn test_holt_forecast() {
    let data = make_series(&[100.0, 110.0, 120.0, 130.0, 140.0, 150.0]);
    let forecaster = AdvancedForecaster::new(ForecastMethod::Holt);
    let result = forecaster.forecast(&data, 3).unwrap();
    assert_eq!(result.predicted_values.len(), 3);
    assert!(result.predicted_values[0] > 140.0);
    assert!(result.predicted_values[2] > result.predicted_values[0]);
    assert_eq!(result.method, "holt");
    assert_eq!(result.confidence, 0.7);
}

#[test]
fn test_confidence_intervals() {
    let data = make_series(&[100.0, 108.0, 103.0, 115.0, 120.0]);
    let forecaster = AdvancedForecaster::new(ForecastMethod::Holt);
    let result = forecaster.forecast(&data, 5).unwrap();
    for i in 0..5 {
        assert!(result.lower_bound[i] <= result.predicted_values[i]);
        assert!(result.predicted_values[i] <= result.upper_bound[i]);
    }
    // Intervals widen over time
    let interval_1 = result.upper_bound[0] - result.lower_bound[0];
    let interval_5 = result.upper_bound[4] - result.lower_bound[4];
    assert!(interval_5 > interval_1);
}

#[test]
fn test_cold_start() {
    let data = make_series(&[100.0]);
    let forecaster = AdvancedForecaster::new(ForecastMethod::Holt);
    assert!(forecaster.forecast(&data, 3).is_err());
}

#[test]
fn test_cold_start_empty() {
    let forecaster = AdvancedForecaster::new(ForecastMethod::Holt);
    assert!(forecaster.forecast(&[], 3).is_err());
}

#[test]
fn test_cold_start_two_points() {
    let data = make_series(&[100.0, 110.0]);
    let forecaster = AdvancedForecaster::new(ForecastMethod::Holt);
    assert!(forecaster.forecast(&data, 3).is_err());
}

#[test]
fn test_method_availability_holt() {
    assert!(AdvancedForecaster::new(ForecastMethod::Holt).is_method_available());
}

#[test]
fn test_method_availability_arima() {
    assert!(!AdvancedForecaster::new(ForecastMethod::Arima).is_method_available());
}

#[test]
fn test_prophet_fallback() {
    let data = make_series(&[100.0, 110.0, 120.0, 130.0]);
    let forecaster = AdvancedForecaster::new(ForecastMethod::Prophet);
    let result = forecaster.forecast(&data, 2).unwrap();
    assert_eq!(result.predicted_values.len(), 2);
}

#[test]
fn test_arima_fallback() {
    let data = make_series(&[100.0, 110.0, 120.0, 130.0]);
    let forecaster = AdvancedForecaster::new(ForecastMethod::Arima);
    let result = forecaster.forecast(&data, 3).unwrap();
    assert_eq!(result.predicted_values.len(), 3);
    assert_eq!(result.method, "arima(1,0,0)");
}

#[test]
fn test_stationary_series() {
    let data = make_series(&[50.0, 50.0, 50.0, 50.0, 50.0, 50.0]);
    let forecaster = AdvancedForecaster::new(ForecastMethod::Holt);
    let result = forecaster.forecast(&data, 3).unwrap();
    for pred in &result.predicted_values {
        assert!((*pred - 50.0).abs() < 5.0);
    }
}

#[test]
fn test_cluster_users() {
    let data = vec![
        ("user1".into(), vec![100.0, 200.0]),
        ("user2".into(), vec![150.0, 250.0]),
        ("user3".into(), vec![999.0, 1.0]),
    ];
    let clusters = UsageClusterer::cluster_users(&data, 3);
    assert_eq!(clusters.len(), 3);
    assert_eq!(clusters[0].0, "user1");
    assert_eq!(clusters[1].0, "user2");
    assert_eq!(clusters[2].0, "user3");
}

#[test]
fn test_cost_regressor() {
    let cost = CostRegressor::predict(&[10.0, 20.0, 30.0]);
    // OLS regression: slope=10, intercept=10, predict at x=3 → 40.0
    assert_eq!(cost, 40.0);
}

#[test]
fn test_cost_regressor_empty() {
    assert_eq!(CostRegressor::predict(&[]), 0.0);
}

#[test]
fn test_cost_regressor_single() {
    assert_eq!(CostRegressor::predict(&[42.0]), 42.0);
}

#[test]
fn test_forecast_method_eq() {
    assert_eq!(ForecastMethod::Holt, ForecastMethod::Holt);
    assert_ne!(ForecastMethod::Holt, ForecastMethod::Prophet);
}
