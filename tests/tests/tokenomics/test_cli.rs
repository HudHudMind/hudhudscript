//! Public API tests for tokenomics::cli

use hudhudscript_tokenomics::budget::BudgetHealth;
use hudhudscript_tokenomics::cli::*;
use hudhudscript_tokenomics::enforcement::UsageSummary;
use std::collections::HashMap;

#[test]
fn test_format_usage_summary_healthy() {
    let summary = UsageSummary {
        daily_usage: 50000,
        daily_limit: 100000,
        daily_pct: 0.5,
        monthly_usage: 500000,
        monthly_limit: 3000000,
        monthly_pct: 0.1667,
        health: BudgetHealth::Healthy,
    };
    let output = format_usage_summary(&summary);
    assert!(output.contains("[OK]"));
    assert!(output.contains("50000"));
    assert!(output.contains("100000"));
}

#[test]
fn test_format_usage_summary_warning() {
    let summary = UsageSummary {
        daily_usage: 81000,
        daily_limit: 100000,
        daily_pct: 0.81,
        monthly_usage: 81000,
        monthly_limit: 3000000,
        monthly_pct: 0.027,
        health: BudgetHealth::Warning,
    };
    let output = format_usage_summary(&summary);
    assert!(output.contains("[WARN]"));
}

#[test]
fn test_format_usage_summary_critical() {
    let summary = UsageSummary {
        daily_usage: 91000,
        daily_limit: 100000,
        daily_pct: 0.91,
        monthly_usage: 91000,
        monthly_limit: 3000000,
        monthly_pct: 0.03,
        health: BudgetHealth::Critical,
    };
    let output = format_usage_summary(&summary);
    assert!(output.contains("[CRIT]"));
}

#[test]
fn test_format_usage_summary_depleted() {
    let summary = UsageSummary {
        daily_usage: 99000,
        daily_limit: 100000,
        daily_pct: 0.99,
        monthly_usage: 99000,
        monthly_limit: 3000000,
        monthly_pct: 0.033,
        health: BudgetHealth::Depleted,
    };
    let output = format_usage_summary(&summary);
    assert!(output.contains("[STOP]"));
}

#[test]
fn test_format_cost_breakdown() {
    let mut costs = HashMap::new();
    costs.insert("chat".into(), 0.50);
    costs.insert("search".into(), 0.30);
    costs.insert("code".into(), 0.20);
    let output = format_cost_breakdown("Cost by Feature", &costs);
    assert!(output.contains("chat"));
    assert!(output.contains("$1.0000")); // total
}

#[test]
fn test_format_empty_breakdown() {
    let costs = HashMap::new();
    let output = format_cost_breakdown("Empty", &costs);
    assert!(output.contains("no data"));
}

#[test]
fn test_repl_budget_healthy() {
    let output = format_repl_budget(20000, 100000, BudgetHealth::Healthy);
    assert!(output.contains("20%"));
}

#[test]
fn test_repl_budget_warning() {
    let output = format_repl_budget(81000, 100000, BudgetHealth::Warning);
    assert!(output.contains("81%"));
}

#[test]
fn test_repl_budget_critical() {
    let output = format_repl_budget(90000, 100000, BudgetHealth::Critical);
    assert!(output.contains("90%"));
}

#[test]
fn test_repl_budget_depleted() {
    let output = format_repl_budget(99000, 100000, BudgetHealth::Depleted);
    assert!(output.contains("99%"));
}

#[test]
fn test_repl_budget_zero_limit() {
    let output = format_repl_budget(0, 0, BudgetHealth::Healthy);
    assert!(output.contains("0%"));
}

#[test]
fn test_command_from_str() {
    assert_eq!(
        TokenomicsCommand::from_str("status"),
        Some(TokenomicsCommand::Status)
    );
    assert_eq!(
        TokenomicsCommand::from_str("REPORT"),
        Some(TokenomicsCommand::Report)
    );
    assert_eq!(
        TokenomicsCommand::from_str("reset"),
        Some(TokenomicsCommand::Reset)
    );
    assert_eq!(TokenomicsCommand::from_str("invalid"), None);
}

#[test]
fn test_command_help() {
    let help = TokenomicsCommand::help();
    assert!(help.contains("status"));
    assert!(help.contains("report"));
    assert!(help.contains("reset"));
}

#[test]
fn test_repl_budget_warning_symbol() {
    let output = format_repl_budget(81000, 100000, BudgetHealth::Warning);
    assert!(output.contains("◐"));
}

#[test]
fn test_repl_budget_depleted_symbol() {
    let output = format_repl_budget(99000, 100000, BudgetHealth::Depleted);
    assert!(output.contains("✕"));
}
