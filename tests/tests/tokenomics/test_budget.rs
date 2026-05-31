//! Public API tests for tokenomics::budget

use hudhudscript_tokenomics::budget::*;
use hudhudscript_tokenomics::types::Budget;

#[test]
fn test_is_low_false() {
    let budget = Budget::new("user1".to_string(), 10000);
    assert!(!BudgetManager::is_low(&budget, 5000));
}

#[test]
fn test_is_low_true() {
    let mut budget = Budget::new("user2".to_string(), 1000);
    budget.consume(900).unwrap();
    assert!(BudgetManager::is_low(&budget, 500));
}

#[test]
fn test_health_status_healthy() {
    let budget = Budget::new("user1".to_string(), 1000);
    assert_eq!(BudgetManager::health_status(&budget), BudgetHealth::Healthy);
}

#[test]
fn test_health_status_warning() {
    let mut budget = Budget::new("user1".to_string(), 1000);
    budget.consume(600).unwrap();
    assert_eq!(BudgetManager::health_status(&budget), BudgetHealth::Warning);
}

#[test]
fn test_health_status_critical() {
    let mut budget = Budget::new("user1".to_string(), 1000);
    budget.consume(850).unwrap();
    assert_eq!(
        BudgetManager::health_status(&budget),
        BudgetHealth::Critical
    );
}

#[test]
fn test_health_status_depleted() {
    let mut budget = Budget::new("u".to_string(), 1000);
    budget.consume(950).unwrap();
    assert_eq!(
        BudgetManager::health_status(&budget),
        BudgetHealth::Depleted
    );
}

#[test]
fn test_calculate_refill_no_deficit() {
    let mut budget = Budget::new("user1".to_string(), 1000);
    budget.consume(200).unwrap();
    // remaining = 800, predicted = 500 => deficit = 0, buffer = 100
    let refill = BudgetManager::calculate_refill(&budget, 500);
    assert_eq!(refill, 100);
}

#[test]
fn test_calculate_refill_with_deficit() {
    let mut budget = Budget::new("user1".to_string(), 1000);
    budget.consume(800).unwrap();
    // remaining = 200, predicted = 500 => deficit = 300, buffer = 100
    let refill = BudgetManager::calculate_refill(&budget, 500);
    assert_eq!(refill, 400);
}

#[test]
fn test_validate_operation_ok() {
    let budget = Budget::new("user1".to_string(), 1000);
    assert!(BudgetManager::validate_operation(&budget, 500).is_ok());
}

#[test]
fn test_validate_operation_zero_amount() {
    let budget = Budget::new("user1".to_string(), 1000);
    let result = BudgetManager::validate_operation(&budget, 0);
    assert!(result.is_err());
}

#[test]
fn test_validate_operation_exceeds_remaining() {
    let mut budget = Budget::new("user1".to_string(), 1000);
    budget.consume(900).unwrap();
    let result = BudgetManager::validate_operation(&budget, 200);
    assert!(result.is_err());
}

#[test]
fn test_health_boundary_50_pct() {
    let mut budget = Budget::new("u".to_string(), 1000);
    budget.consume(500).unwrap();
    assert_eq!(BudgetManager::health_status(&budget), BudgetHealth::Warning);
}

#[test]
fn test_health_boundary_80_pct() {
    let mut budget = Budget::new("u".to_string(), 1000);
    budget.consume(800).unwrap();
    assert_eq!(
        BudgetManager::health_status(&budget),
        BudgetHealth::Critical
    );
}

#[test]
fn test_health_boundary_95_pct() {
    let mut budget = Budget::new("u".to_string(), 1000);
    budget.consume(950).unwrap();
    assert_eq!(
        BudgetManager::health_status(&budget),
        BudgetHealth::Depleted
    );
}
