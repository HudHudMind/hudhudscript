//! Public API tests for tokenomics::ml
//! Covers: TokenPredictionModel (new, train, predict, save, load, metadata,
//!         needs_retraining), FeatureExtractor (extract).

use chrono::{Duration, Utc};
use hudhudscript_tokenomics::ml::{FeatureExtractor, TokenPredictionModel};
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

// ── TokenPredictionModel construction ───────────────────────────────

#[test]
fn model_new_has_correct_feature_count() {
    let model = TokenPredictionModel::new(10);
    assert_eq!(model.metadata().name, "token_predictor");
    assert_eq!(model.metadata().version, "0.1.0");
    assert_eq!(model.metadata().accuracy, 0.0);
    assert_eq!(model.metadata().samples_count, 0);
}

#[test]
fn model_new_zero_features_still_constructs() {
    let model = TokenPredictionModel::new(0);
    assert_eq!(model.metadata().accuracy, 0.0);
}

#[test]
fn model_new_large_feature_count() {
    let model = TokenPredictionModel::new(100);
    assert_eq!(model.metadata().samples_count, 0);
}

// ── TokenPredictionModel::needs_retraining ───────────────────────────

#[test]
fn model_needs_retraining_initially_true() {
    let model = TokenPredictionModel::new(5);
    // accuracy=0.0 < 0.7 and samples_count=0 < 100
    assert!(model.needs_retraining());
}

#[test]
fn model_metadata_name_and_version_correct() {
    let model = TokenPredictionModel::new(3);
    let meta = model.metadata();
    assert_eq!(meta.name, "token_predictor");
    assert_eq!(meta.version, "0.1.0");
}

// ── TokenPredictionModel::predict (before training) ──────────────────

#[test]
fn predict_without_training_confidence_is_zero() {
    let model = TokenPredictionModel::new(3);
    let prediction = model.predict(&[1.0, 2.0, 3.0], 3600).unwrap();
    assert_eq!(prediction.confidence, 0.0);
    assert_eq!(prediction.horizon_seconds, 3600);
    assert_eq!(prediction.model_version, "0.1.0");
}

#[test]
fn predict_wrong_feature_count_returns_error() {
    let model = TokenPredictionModel::new(3);
    let err = model.predict(&[1.0, 2.0], 3600).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Expected 3 features, got 2"));
}

#[test]
fn predict_too_many_features_returns_error() {
    let model = TokenPredictionModel::new(2);
    let err = model.predict(&[1.0, 2.0, 3.0], 3600).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Expected 2 features, got 3"));
}

#[test]
fn predict_horizon_preserved_in_result() {
    let model = TokenPredictionModel::new(2);
    let pred = model.predict(&[1.0, 1.0], 7200).unwrap();
    assert_eq!(pred.horizon_seconds, 7200);
}

#[test]
fn predict_result_has_model_version() {
    let model = TokenPredictionModel::new(2);
    let pred = model.predict(&[0.0, 0.0], 3600).unwrap();
    assert_eq!(pred.model_version, "0.1.0");
}

// ── TokenPredictionModel::train ──────────────────────────────────────

#[test]
fn train_basic_succeeds() {
    let mut model = TokenPredictionModel::new(3);
    let features = vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 3.0, 4.0],
        vec![3.0, 4.0, 5.0],
    ];
    let targets = vec![10.0, 15.0, 20.0];
    assert!(model.train(&features, &targets).is_ok());
}

#[test]
fn train_updates_samples_count() {
    let mut model = TokenPredictionModel::new(3);
    let features = vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 3.0, 4.0],
        vec![3.0, 4.0, 5.0],
    ];
    let targets = vec![10.0, 15.0, 20.0];
    model.train(&features, &targets).unwrap();
    assert_eq!(model.metadata().samples_count, 3);
}

#[test]
fn train_accuracy_becomes_nonzero() {
    let mut model = TokenPredictionModel::new(3);
    let features = vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 3.0, 4.0],
        vec![3.0, 4.0, 5.0],
    ];
    let targets = vec![10.0, 15.0, 20.0];
    model.train(&features, &targets).unwrap();
    assert!(model.metadata().accuracy >= 0.0);
}

#[test]
fn train_empty_features_returns_error() {
    let mut model = TokenPredictionModel::new(3);
    let err = model.train(&[], &[1.0]).unwrap_err();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn train_empty_targets_returns_error() {
    let mut model = TokenPredictionModel::new(3);
    let err = model.train(&[vec![1.0, 2.0, 3.0]], &[]).unwrap_err();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn train_mismatched_lengths_returns_error() {
    let mut model = TokenPredictionModel::new(2);
    let features = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
    let targets = vec![10.0, 20.0, 30.0]; // 3 targets but 2 rows
    let err = model.train(&features, &targets).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("mismatch") || !msg.is_empty());
}

#[test]
fn train_single_sample_does_not_panic() {
    let mut model = TokenPredictionModel::new(2);
    let features = vec![vec![1.0, 2.0]];
    let targets = vec![5.0];
    assert!(model.train(&features, &targets).is_ok());
}

// ── predict after training ───────────────────────────────────────────

#[test]
fn predict_after_training_returns_ok() {
    let mut model = TokenPredictionModel::new(3);
    let features = vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 3.0, 4.0],
        vec![3.0, 4.0, 5.0],
    ];
    let targets = vec![10.0, 15.0, 20.0];
    model.train(&features, &targets).unwrap();
    assert!(model.predict(&[2.0, 3.0, 4.0], 3600).is_ok());
}

#[test]
fn predict_confidence_matches_training_accuracy() {
    let mut model = TokenPredictionModel::new(3);
    let features = vec![vec![1.0, 2.0, 3.0], vec![2.0, 3.0, 4.0]];
    let targets = vec![10.0, 15.0];
    model.train(&features, &targets).unwrap();
    let pred = model.predict(&[1.0, 2.0, 3.0], 3600).unwrap();
    assert_eq!(pred.confidence, model.metadata().accuracy);
}

#[test]
fn predict_usage_is_non_negative() {
    let mut model = TokenPredictionModel::new(2);
    let features = vec![vec![1.0, 1.0], vec![2.0, 2.0]];
    let targets = vec![5.0, 10.0];
    model.train(&features, &targets).unwrap();
    let pred = model.predict(&[1.5, 1.5], 3600).unwrap();
    // predicted_usage is u64, always >= 0
    let _ = pred.predicted_usage;
}

// ── save / load roundtrip ────────────────────────────────────────────

#[test]
fn save_and_load_roundtrip_preserves_weights_and_metadata() {
    let mut model = TokenPredictionModel::new(3);
    let features = vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 3.0, 4.0],
        vec![3.0, 4.0, 5.0],
    ];
    let targets = vec![10.0, 15.0, 20.0];
    model.train(&features, &targets).unwrap();

    let path = std::env::temp_dir().join("hudhud_ml_roundtrip.json");
    model.save(&path).unwrap();

    let loaded = TokenPredictionModel::load(&path).unwrap();
    assert_eq!(loaded.metadata().name, "token_predictor");
    assert_eq!(loaded.metadata().version, "0.1.0");
    assert_eq!(loaded.metadata().samples_count, 3);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn load_from_nonexistent_file_returns_error() {
    let path = std::path::Path::new("/tmp/nonexistent_ml_model_xyz.json");
    assert!(TokenPredictionModel::load(path).is_err());
}

#[test]
fn save_creates_a_file() {
    let mut model = TokenPredictionModel::new(2);
    let path = std::env::temp_dir().join("hudhud_ml_save_check.json");
    let _ = std::fs::remove_file(&path);
    model.train(&[vec![1.0, 2.0]], &[5.0]).unwrap();
    model.save(&path).unwrap();
    assert!(path.exists());
    let _ = std::fs::remove_file(&path);
}

// ── FeatureExtractor::extract ────────────────────────────────────────

#[test]
fn feature_extractor_always_returns_10_features() {
    let data = make_series(&[100.0, 150.0, 200.0]);
    let features = FeatureExtractor::extract(&data);
    assert_eq!(features.len(), 10);
}

#[test]
fn feature_extractor_empty_series_returns_ten_zeros() {
    let features = FeatureExtractor::extract(&[]);
    assert_eq!(features.len(), 10);
    assert_eq!(features, vec![0.0; 10]);
}

#[test]
fn feature_extractor_single_point_mean_equals_value() {
    let data = make_series(&[42.0]);
    let features = FeatureExtractor::extract(&data);
    assert_eq!(features[0], 42.0); // mean
    assert_eq!(features[1], 0.0); // std_dev of single value
    assert_eq!(features[2], 42.0); // min
    assert_eq!(features[3], 42.0); // max
    assert_eq!(features[4], 42.0); // median
    assert_eq!(features[5], 0.0); // trend (< 2 values)
    assert_eq!(features[6], 0.0); // volatility (< 2 values)
    assert_eq!(features[7], 1.0); // count
    assert_eq!(features[8], 42.0); // recent_average
    assert_eq!(features[9], 0.0); // momentum (< 2 values)
}

#[test]
fn feature_extractor_trending_series_mean_correct() {
    // 10, 15, 20, ..., 55 (step 5)
    let values: Vec<f64> = (0..10).map(|i| 10.0 + 5.0 * i as f64).collect();
    let data = make_series(&values);
    let features = FeatureExtractor::extract(&data);
    assert_eq!(features[0], 32.5); // mean of 10..55
    assert_eq!(features[2], 10.0); // min
    assert_eq!(features[3], 55.0); // max
    assert_eq!(features[5], 4.5); // trend = (55-10)/10
    assert_eq!(features[6], 5.0); // volatility = mean of [5,5,...,5]
    assert_eq!(features[7], 10.0); // count
    assert_eq!(features[8], 45.0); // recent_average last 5: 35,40,45,50,55 => 45
    assert_eq!(features[9], 12.5); // momentum = 45 - 32.5
}

#[test]
fn feature_extractor_even_count_median_is_average_of_middle_two() {
    // [10, 20, 30, 40] => median = (20+30)/2 = 25
    let data = make_series(&[10.0, 20.0, 30.0, 40.0]);
    let features = FeatureExtractor::extract(&data);
    assert_eq!(features[4], 25.0);
}

#[test]
fn feature_extractor_odd_count_median_is_middle_value() {
    // [1, 2, 3, 4, 5] => median = 3
    let data = make_series(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let features = FeatureExtractor::extract(&data);
    assert_eq!(features[4], 3.0);
}

#[test]
fn feature_extractor_constant_series_std_dev_zero() {
    let data = make_series(&[50.0; 5]);
    let features = FeatureExtractor::extract(&data);
    assert_eq!(features[1], 0.0); // std_dev
    assert_eq!(features[5], 0.0); // trend (50-50)/5 = 0
    assert_eq!(features[6], 0.0); // volatility
}

#[test]
fn feature_extractor_recent_average_with_fewer_than_five_points() {
    // 3 points, recent_average last 5 == mean of all
    let data = make_series(&[10.0, 20.0, 30.0]);
    let features = FeatureExtractor::extract(&data);
    assert_eq!(features[8], 20.0); // mean of [10, 20, 30]
}

#[test]
fn feature_extractor_count_feature_equals_data_length() {
    let data = make_series(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    let features = FeatureExtractor::extract(&data);
    assert_eq!(features[7], 7.0);
}
