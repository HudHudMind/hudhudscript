//! Public API tests for tokenomics::enforcement

use hudhudscript_tokenomics::enforcement::*;

#[test]
fn test_new_enforcer() {
    let e = BudgetEnforcer::new(
        4000,
        100000,
        3000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    let summary = e.usage_summary();
    assert_eq!(summary.daily_usage, 0);
    assert_eq!(summary.monthly_usage, 0);
}

#[test]
fn test_check_request_allowed() {
    let mut e = BudgetEnforcer::new(
        4000,
        100000,
        3000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    assert_eq!(e.check_request(1000), EnforcementDecision::Allowed);
}

#[test]
fn test_per_call_blocked() {
    let mut e = BudgetEnforcer::new(
        4000,
        100000,
        3000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    match e.check_request(5000) {
        EnforcementDecision::Blocked(msg) => assert!(msg.contains("per-call")),
        _ => panic!("Expected blocked"),
    }
}

#[test]
fn test_daily_budget_exceeded() {
    let mut e = BudgetEnforcer::new(
        4000,
        100000,
        3000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    e.record_usage(99000);
    match e.check_request(2000) {
        EnforcementDecision::Blocked(msg) => assert!(msg.contains("Daily")),
        _ => panic!("Expected blocked"),
    }
}

#[test]
fn test_warning_threshold() {
    let mut e = BudgetEnforcer::new(
        4000,
        100000,
        3000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    e.record_usage(81000);
    match e.check_request(100) {
        EnforcementDecision::Warning(_) => {}
        other => panic!("Expected warning, got {:?}", other),
    }
}

#[test]
fn test_record_usage() {
    let mut e = BudgetEnforcer::new(
        4000,
        100000,
        3000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    e.record_usage(5000);
    let summary = e.usage_summary();
    assert_eq!(summary.daily_usage, 5000);
    assert_eq!(summary.monthly_usage, 5000);
}

#[test]
fn test_alerts_recorded() {
    let mut e = BudgetEnforcer::new(
        4000,
        100000,
        3000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    e.record_usage(85000);
    e.check_request(100);
    assert_eq!(e.alerts().len(), 1);
    assert_eq!(e.alerts()[0].action, AlertAction::Log);
}

#[test]
fn test_monthly_budget_blocking() {
    let mut e = BudgetEnforcer::new(
        10000,
        1000000,
        5000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    e.record_usage(4000);
    match e.check_request(2000) {
        EnforcementDecision::Blocked(msg) => assert!(msg.contains("Monthly")),
        other => panic!("Expected Blocked, got {:?}", other),
    }
}

#[test]
fn test_critical_health_block() {
    let mut e = BudgetEnforcer::new(
        10000,
        100000,
        10000000,
        0.80,
        AlertAction::Log,
        AlertAction::Block,
        AlertAction::Block,
    );
    e.record_usage(91000);
    match e.check_request(100) {
        EnforcementDecision::Blocked(msg) => assert!(msg.contains("critically")),
        other => panic!("Expected Blocked, got {:?}", other),
    }
}

#[test]
fn test_critical_health_warning() {
    let mut e = BudgetEnforcer::new(
        10000,
        100000,
        10000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    e.record_usage(91000);
    match e.check_request(100) {
        EnforcementDecision::Warning(msg) => assert!(msg.contains("critically")),
        other => panic!("Expected Warning, got {:?}", other),
    }
}

#[test]
fn test_alert_action_from_str() {
    assert_eq!(AlertAction::from_str("notify"), AlertAction::Notify);
    assert_eq!(AlertAction::from_str("pause"), AlertAction::Pause);
    assert_eq!(AlertAction::from_str("block"), AlertAction::Block);
    assert_eq!(AlertAction::from_str("anything"), AlertAction::Log);
}

#[test]
fn test_usage_summary_health() {
    let mut e = BudgetEnforcer::new(
        4000,
        100000,
        3000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    e.record_usage(50000);
    let summary = e.usage_summary();
    assert_eq!(summary.daily_usage, 50000);
    assert_eq!(summary.daily_limit, 100000);
    assert_eq!(summary.monthly_limit, 3000000);
}

#[test]
fn test_daily_and_monthly_reset() {
    let mut e = BudgetEnforcer::new(
        10000,
        100000,
        3000000,
        0.80,
        AlertAction::Log,
        AlertAction::Log,
        AlertAction::Block,
    );
    e.record_usage(50000);
    assert_eq!(e.usage_summary().daily_usage, 50000);
    assert_eq!(e.usage_summary().monthly_usage, 50000);

    // Simulate daily reset by backdating last_daily_reset
    e.last_daily_reset = chrono::Utc::now() - chrono::Duration::seconds(86401);
    e.record_usage(0); // triggers check_and_reset
    assert_eq!(e.usage_summary().daily_usage, 0);

    // Monthly still holds (only daily was reset)
    // Now simulate monthly reset
    e.record_usage(1000);
    e.last_monthly_reset = chrono::Utc::now() - chrono::Duration::seconds(2592001);
    e.record_usage(0); // triggers check_and_reset
    assert_eq!(e.usage_summary().monthly_usage, 0);
}
