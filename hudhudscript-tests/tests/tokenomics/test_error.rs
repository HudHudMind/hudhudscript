//! Tests for tokenomics::error — all TokenomicsError variants, is_recoverable, should_fallback_to_rules

use hudhudscript_tokenomics::error::*;

// ---------------------------------------------------------------------------
// is_recoverable — true cases
// ---------------------------------------------------------------------------

#[test]
fn test_cache_error_is_recoverable() {
    assert!(TokenomicsError::CacheError("timeout".into()).is_recoverable());
}

#[test]
fn test_cold_start_is_recoverable() {
    assert!(TokenomicsError::ColdStart.is_recoverable());
}

#[test]
fn test_prediction_failed_is_recoverable() {
    assert!(TokenomicsError::PredictionFailed("nan".into()).is_recoverable());
}

// ---------------------------------------------------------------------------
// is_recoverable — false cases (one test per variant)
// ---------------------------------------------------------------------------

#[test]
fn test_insufficient_budget_not_recoverable() {
    assert!(!TokenomicsError::InsufficientBudget {
        needed: 100,
        available: 50
    }
    .is_recoverable());
}

#[test]
fn test_budget_not_found_not_recoverable() {
    assert!(!TokenomicsError::BudgetNotFound("proj-1".into()).is_recoverable());
}

#[test]
fn test_invalid_budget_not_recoverable() {
    assert!(!TokenomicsError::InvalidBudget(0).is_recoverable());
}

#[test]
fn test_model_error_not_recoverable() {
    assert!(!TokenomicsError::ModelError("bad weights".into()).is_recoverable());
}

#[test]
fn test_storage_error_not_recoverable() {
    assert!(!TokenomicsError::StorageError("disk full".into()).is_recoverable());
}

#[test]
fn test_config_error_not_recoverable() {
    assert!(!TokenomicsError::ConfigError("missing field".into()).is_recoverable());
}

#[test]
fn test_federated_error_not_recoverable() {
    assert!(!TokenomicsError::FederatedError("sync".into()).is_recoverable());
}

#[test]
fn test_reinforcement_error_not_recoverable() {
    assert!(!TokenomicsError::ReinforcementError("diverged".into()).is_recoverable());
}

#[test]
fn test_model_drift_not_recoverable() {
    assert!(!TokenomicsError::ModelDrift.is_recoverable());
}

#[test]
fn test_overfitting_not_recoverable() {
    assert!(!TokenomicsError::Overfitting.is_recoverable());
}

#[test]
fn test_unknown_not_recoverable() {
    assert!(!TokenomicsError::Unknown("???".into()).is_recoverable());
}

// ---------------------------------------------------------------------------
// should_fallback_to_rules — true cases
// ---------------------------------------------------------------------------

#[test]
fn test_cold_start_should_fallback() {
    assert!(TokenomicsError::ColdStart.should_fallback_to_rules());
}

#[test]
fn test_model_drift_should_fallback() {
    assert!(TokenomicsError::ModelDrift.should_fallback_to_rules());
}

#[test]
fn test_model_error_should_fallback() {
    assert!(TokenomicsError::ModelError("crash".into()).should_fallback_to_rules());
}

#[test]
fn test_prediction_failed_should_fallback() {
    assert!(TokenomicsError::PredictionFailed("timeout".into()).should_fallback_to_rules());
}

// ---------------------------------------------------------------------------
// should_fallback_to_rules — false cases (one test per variant)
// ---------------------------------------------------------------------------

#[test]
fn test_insufficient_budget_no_fallback() {
    assert!(!TokenomicsError::InsufficientBudget {
        needed: 10,
        available: 0
    }
    .should_fallback_to_rules());
}

#[test]
fn test_budget_not_found_no_fallback() {
    assert!(!TokenomicsError::BudgetNotFound("x".into()).should_fallback_to_rules());
}

#[test]
fn test_invalid_budget_no_fallback() {
    assert!(!TokenomicsError::InvalidBudget(0).should_fallback_to_rules());
}

#[test]
fn test_storage_error_no_fallback() {
    assert!(!TokenomicsError::StorageError("err".into()).should_fallback_to_rules());
}

#[test]
fn test_cache_error_no_fallback() {
    assert!(!TokenomicsError::CacheError("miss".into()).should_fallback_to_rules());
}

#[test]
fn test_config_error_no_fallback() {
    assert!(!TokenomicsError::ConfigError("bad".into()).should_fallback_to_rules());
}

#[test]
fn test_federated_error_no_fallback() {
    assert!(!TokenomicsError::FederatedError("err".into()).should_fallback_to_rules());
}

#[test]
fn test_reinforcement_error_no_fallback() {
    assert!(!TokenomicsError::ReinforcementError("err".into()).should_fallback_to_rules());
}

#[test]
fn test_overfitting_no_fallback() {
    assert!(!TokenomicsError::Overfitting.should_fallback_to_rules());
}

#[test]
fn test_unknown_no_fallback() {
    assert!(!TokenomicsError::Unknown("err".into()).should_fallback_to_rules());
}

// ---------------------------------------------------------------------------
// Display messages — one test per variant
// ---------------------------------------------------------------------------

#[test]
fn test_display_insufficient_budget() {
    assert!(TokenomicsError::InsufficientBudget {
        needed: 100,
        available: 50
    }
    .to_string()
    .contains("Insufficient budget: need 100, have 50"));
}

#[test]
fn test_display_budget_not_found() {
    assert!(TokenomicsError::BudgetNotFound("project-alpha".into())
        .to_string()
        .contains("Budget not found: project-alpha"));
}

#[test]
fn test_display_invalid_budget() {
    assert!(TokenomicsError::InvalidBudget(0)
        .to_string()
        .contains("Invalid budget amount: 0"));
}

#[test]
fn test_display_model_error() {
    assert!(TokenomicsError::ModelError("weights corrupted".into())
        .to_string()
        .contains("ML model error: weights corrupted"));
}

#[test]
fn test_display_prediction_failed() {
    assert!(TokenomicsError::PredictionFailed("NaN output".into())
        .to_string()
        .contains("Prediction failed: NaN output"));
}

#[test]
fn test_display_storage_error() {
    assert!(TokenomicsError::StorageError("disk full".into())
        .to_string()
        .contains("Storage error: disk full"));
}

#[test]
fn test_display_cache_error() {
    assert!(TokenomicsError::CacheError("evicted".into())
        .to_string()
        .contains("Cache error: evicted"));
}

#[test]
fn test_display_config_error() {
    assert!(TokenomicsError::ConfigError("missing field".into())
        .to_string()
        .contains("Configuration error: missing field"));
}

#[test]
fn test_display_federated_error() {
    assert!(TokenomicsError::FederatedError("sync timeout".into())
        .to_string()
        .contains("Federated learning error: sync timeout"));
}

#[test]
fn test_display_reinforcement_error() {
    assert!(TokenomicsError::ReinforcementError("diverged".into())
        .to_string()
        .contains("Reinforcement learning error: diverged"));
}

#[test]
fn test_display_cold_start() {
    assert!(TokenomicsError::ColdStart
        .to_string()
        .contains("Cold start: insufficient training data"));
}

#[test]
fn test_display_model_drift() {
    assert!(TokenomicsError::ModelDrift
        .to_string()
        .contains("Model drift detected: accuracy below threshold"));
}

#[test]
fn test_display_overfitting() {
    assert!(TokenomicsError::Overfitting
        .to_string()
        .contains("Overfitting detected: validation loss increasing"));
}

#[test]
fn test_display_unknown() {
    assert!(TokenomicsError::Unknown("mystery".into())
        .to_string()
        .contains("Unknown error: mystery"));
}

// ---------------------------------------------------------------------------
// std::error::Error trait
// ---------------------------------------------------------------------------

#[test]
fn test_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(TokenomicsError::ColdStart);
    assert!(!err.to_string().is_empty());
}
