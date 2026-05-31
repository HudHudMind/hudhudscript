//! CLI display helpers for tokenomics status, reports, and budget info

use crate::budget::BudgetHealth;
use crate::enforcement::UsageSummary;
use std::collections::HashMap;

/// Format usage summary for CLI display
pub fn format_usage_summary(summary: &UsageSummary) -> String {
    let health_icon = match summary.health {
        BudgetHealth::Healthy => "[OK]",
        BudgetHealth::Warning => "[WARN]",
        BudgetHealth::Critical => "[CRIT]",
        BudgetHealth::Depleted => "[STOP]",
    };

    format!(
        "Tokenomics Status {health}\n\
         ─────────────────────────────────\n\
         Daily:   {daily:>8} / {daily_limit:>8} tokens ({daily_pct:.1}%)\n\
         Monthly: {monthly:>8} / {monthly_limit:>8} tokens ({monthly_pct:.1}%)\n\
         Health:  {health_str:?}",
        health = health_icon,
        daily = summary.daily_usage,
        daily_limit = summary.daily_limit,
        daily_pct = summary.daily_pct * 100.0,
        monthly = summary.monthly_usage,
        monthly_limit = summary.monthly_limit,
        monthly_pct = summary.monthly_pct * 100.0,
        health_str = summary.health,
    )
}

/// Format cost breakdown by dimension (feature, user, model, etc.)
pub fn format_cost_breakdown(title: &str, costs: &HashMap<String, f64>) -> String {
    if costs.is_empty() {
        return format!("{}: (no data)\n", title);
    }

    let mut sorted: Vec<(&String, &f64)> = costs.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    let total: f64 = sorted.iter().map(|(_, v)| **v).sum();

    let mut lines = vec![format!("{} (total: ${:.4})", title, total)];
    lines.push("─────────────────────────────────".into());
    for (key, cost) in &sorted {
        let pct = if total > 0.0 {
            *cost / total * 100.0
        } else {
            0.0
        };
        lines.push(format!("  {:<20} ${:.4} ({:.1}%)", key, cost, pct));
    }
    lines.join("\n")
}

/// Format a short budget status line for REPL prompt
pub fn format_repl_budget(daily_usage: usize, daily_limit: usize, health: BudgetHealth) -> String {
    let icon = match health {
        BudgetHealth::Healthy => "●",
        BudgetHealth::Warning => "◐",
        BudgetHealth::Critical => "○",
        BudgetHealth::Depleted => "✕",
    };
    let pct = if daily_limit > 0 {
        daily_usage as f64 / daily_limit as f64 * 100.0
    } else {
        0.0
    };
    format!("{} {:.0}%", icon, pct)
}

/// Available CLI subcommands
#[derive(Debug, Clone, PartialEq)]
pub enum TokenomicsCommand {
    Status,
    Report,
    Reset,
}

impl TokenomicsCommand {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "status" => Some(Self::Status),
            "report" => Some(Self::Report),
            "reset" => Some(Self::Reset),
            _ => None,
        }
    }

    pub fn help() -> &'static str {
        "Tokenomics commands:\n\
         \x20 tokenomics status  — show daily/monthly usage and budget health\n\
         \x20 tokenomics report  — show cost breakdown by feature, user, model\n\
         \x20 tokenomics reset   — reset daily/monthly counters"
    }
}
