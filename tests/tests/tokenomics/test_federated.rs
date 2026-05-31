//! Public API tests for tokenomics::federated

use chrono::Utc;
use hudhudscript_tokenomics::federated::FederatedLearning;
use hudhudscript_tokenomics::types::FederatedUpdate;
use uuid::Uuid;

fn make_update(gradients: Vec<f64>, samples: usize) -> FederatedUpdate {
    FederatedUpdate {
        id: Uuid::new_v4(),
        model_version: "v1".into(),
        gradients,
        sample_count: samples,
        timestamp: Utc::now(),
    }
}

// ── Construction ────────────────────────────────────────────────────

#[tokio::test]
async fn test_new_enabled() {
    let fl = FederatedLearning::new(true);
    assert_eq!(fl.model_version().await, 0);
    assert!(fl.global_weights().await.is_empty());
}

#[tokio::test]
async fn test_new_disabled() {
    let fl = FederatedLearning::new(false);
    assert_eq!(fl.model_version().await, 0);
}

#[tokio::test]
async fn test_with_min_updates() {
    let fl = FederatedLearning::with_min_updates(true, 3);
    // min_updates=3, so 2 updates should not be enough
    fl.submit_update(make_update(vec![1.0], 10)).await.unwrap();
    fl.submit_update(make_update(vec![2.0], 10)).await.unwrap();
    let result = fl.run_round().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_with_min_updates_zero_clamps_to_one() {
    let fl = FederatedLearning::with_min_updates(false, 0);
    fl.submit_update(make_update(vec![5.0], 10)).await.unwrap();
    let result = fl.run_round().await.unwrap();
    assert!((result[0] - 5.0).abs() < 1e-9);
}

// ── init_global_model ───────────────────────────────────────────────

#[tokio::test]
async fn test_init_global_model() {
    let fl = FederatedLearning::new(true);
    fl.init_global_model(vec![1.0, 2.0, 3.0]).await;
    let w = fl.global_weights().await;
    assert_eq!(w, vec![1.0, 2.0, 3.0]);
    assert_eq!(fl.model_version().await, 0);
}

#[tokio::test]
async fn test_init_global_model_overwrites() {
    let fl = FederatedLearning::new(true);
    fl.init_global_model(vec![1.0, 2.0]).await;
    fl.init_global_model(vec![10.0, 20.0, 30.0]).await;
    let w = fl.global_weights().await;
    assert_eq!(w, vec![10.0, 20.0, 30.0]);
}

// ── submit_update ───────────────────────────────────────────────────

#[tokio::test]
async fn test_submit_update_ok() {
    let fl = FederatedLearning::new(true);
    assert!(fl
        .submit_update(make_update(vec![1.0, 2.0], 10))
        .await
        .is_ok());
}

#[tokio::test]
async fn test_submit_update_empty_gradients() {
    let fl = FederatedLearning::new(true);
    let result = fl.submit_update(make_update(vec![], 10)).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Federated learning error: empty gradient vector"));
}

// ── aggregate_updates ───────────────────────────────────────────────

#[tokio::test]
async fn test_fedavg_uniform_weights() {
    let fl = FederatedLearning::new(true);
    let updates = vec![
        make_update(vec![1.0, 2.0, 3.0], 100),
        make_update(vec![3.0, 4.0, 5.0], 100),
    ];
    let avg = fl.aggregate_updates(updates).await.unwrap();
    assert!((avg[0] - 2.0).abs() < 1e-9);
    assert!((avg[1] - 3.0).abs() < 1e-9);
    assert!((avg[2] - 4.0).abs() < 1e-9);
}

#[tokio::test]
async fn test_fedavg_weighted() {
    let fl = FederatedLearning::new(true);
    let updates = vec![make_update(vec![10.0], 900), make_update(vec![0.0], 100)];
    let avg = fl.aggregate_updates(updates).await.unwrap();
    assert!((avg[0] - 9.0).abs() < 1e-9);
}

#[tokio::test]
async fn test_aggregate_updates_empty() {
    let fl = FederatedLearning::new(true);
    let result = fl.aggregate_updates(vec![]).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Federated learning error: no updates to aggregate"));
}

#[tokio::test]
async fn test_aggregate_updates_dimension_mismatch() {
    let fl = FederatedLearning::new(true);
    let updates = vec![make_update(vec![1.0, 2.0], 50), make_update(vec![1.0], 50)];
    assert!(fl.aggregate_updates(updates).await.is_err());
}

#[tokio::test]
async fn test_aggregate_updates_zero_sample_count() {
    let fl = FederatedLearning::new(true);
    let updates = vec![
        make_update(vec![1.0, 2.0], 0),
        make_update(vec![3.0, 4.0], 0),
    ];
    let result = fl.aggregate_updates(updates).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Federated learning error: total sample count is zero"));
}

#[tokio::test]
async fn test_aggregate_single_update() {
    let fl = FederatedLearning::new(true);
    let updates = vec![make_update(vec![7.0, 8.0], 50)];
    let avg = fl.aggregate_updates(updates).await.unwrap();
    assert!((avg[0] - 7.0).abs() < 1e-9);
    assert!((avg[1] - 8.0).abs() < 1e-9);
}

#[tokio::test]
async fn test_aggregate_three_clients() {
    let fl = FederatedLearning::new(true);
    let updates = vec![
        make_update(vec![3.0], 100),
        make_update(vec![6.0], 100),
        make_update(vec![9.0], 100),
    ];
    let avg = fl.aggregate_updates(updates).await.unwrap();
    assert!((avg[0] - 6.0).abs() < 1e-9);
}

// ── apply_update ────────────────────────────────────────────────────

#[tokio::test]
async fn test_apply_update_initializes_model() {
    let fl = FederatedLearning::new(true);
    fl.apply_update(vec![1.0, 2.0]).await.unwrap();
    let w = fl.global_weights().await;
    assert_eq!(w, vec![1.0, 2.0]);
    assert_eq!(fl.model_version().await, 1);
}

#[tokio::test]
async fn test_apply_update_adds_gradients() {
    let fl = FederatedLearning::new(true);
    fl.init_global_model(vec![1.0, 1.0]).await;
    fl.apply_update(vec![0.5, -0.5]).await.unwrap();
    let w = fl.global_weights().await;
    assert!((w[0] - 1.5).abs() < 1e-9);
    assert!((w[1] - 0.5).abs() < 1e-9);
}

#[tokio::test]
async fn test_apply_update_dimension_mismatch() {
    let fl = FederatedLearning::new(true);
    fl.init_global_model(vec![1.0, 2.0]).await;
    let result = fl.apply_update(vec![0.1, 0.2, 0.3]).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("gradient dimension 3 does not match model dimension 2"));
}

#[tokio::test]
async fn test_apply_update_increments_version() {
    let fl = FederatedLearning::new(true);
    fl.init_global_model(vec![0.0]).await;
    assert_eq!(fl.model_version().await, 0);
    fl.apply_update(vec![1.0]).await.unwrap();
    assert_eq!(fl.model_version().await, 1);
    fl.apply_update(vec![1.0]).await.unwrap();
    assert_eq!(fl.model_version().await, 2);
}

// ── run_round ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_run_round_basic() {
    let fl = FederatedLearning::new(true);
    fl.init_global_model(vec![0.0, 0.0]).await;
    fl.submit_update(make_update(vec![1.0, 2.0], 50))
        .await
        .unwrap();
    fl.submit_update(make_update(vec![3.0, 4.0], 50))
        .await
        .unwrap();
    let result = fl.run_round().await.unwrap();
    assert!((result[0] - 2.0).abs() < 1e-9);
    assert!((result[1] - 3.0).abs() < 1e-9);
}

#[tokio::test]
async fn test_run_round_insufficient_updates() {
    let fl = FederatedLearning::new(true);
    let result = fl.run_round().await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("need at least 1 updates, have 0"));
}

#[tokio::test]
async fn test_run_round_drains_pending() {
    let fl = FederatedLearning::new(true);
    fl.init_global_model(vec![0.0]).await;
    fl.submit_update(make_update(vec![5.0], 10)).await.unwrap();
    fl.run_round().await.unwrap();
    // Pending should be drained; running again should fail
    let result = fl.run_round().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_round_updates_global_weights() {
    let fl = FederatedLearning::new(true);
    fl.init_global_model(vec![10.0]).await;
    fl.submit_update(make_update(vec![5.0], 100)).await.unwrap();
    fl.run_round().await.unwrap();
    let w = fl.global_weights().await;
    // 10.0 + 5.0 = 15.0
    assert!((w[0] - 15.0).abs() < 1e-9);
}
