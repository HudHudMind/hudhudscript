//! Budget enforcement engine with configurable alert actions

use crate::budget::BudgetHealth;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// What to do when a threshold is hit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertAction {
    Log,
    Notify,
    Pause,
    Block,
}

impl AlertAction {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "notify" => Self::Notify,
            "pause" => Self::Pause,
            "block" => Self::Block,
            _ => Self::Log,
        }
    }
}

/// A triggered alert
#[derive(Debug, Clone)]
pub struct Alert {
    pub level: BudgetHealth,
    pub action: AlertAction,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub budget_usage_pct: f64,
}

/// Enforcement decision returned by check_request
#[derive(Debug, PartialEq)]
pub enum EnforcementDecision {
    Allowed,
    Warning(String),
    Blocked(String),
}

/// Usage summary
#[derive(Debug, Clone)]
pub struct UsageSummary {
    pub daily_usage: usize,
    pub daily_limit: usize,
    pub daily_pct: f64,
    pub monthly_usage: usize,
    pub monthly_limit: usize,
    pub monthly_pct: f64,
    pub health: BudgetHealth,
}

/// Budget enforcement engine
pub struct BudgetEnforcer {
    max_tokens_per_call: usize,
    max_tokens_per_day: usize,
    max_tokens_per_month: usize,
    alert_threshold: f64,
    on_warning: AlertAction,
    on_critical: AlertAction,
    on_depleted: AlertAction,
    daily_usage: usize,
    monthly_usage: usize,
    alerts: Vec<Alert>,
    pub last_daily_reset: DateTime<Utc>,
    pub last_monthly_reset: DateTime<Utc>,
}

impl BudgetEnforcer {
    pub fn new(
        max_tokens_per_call: usize,
        max_tokens_per_day: usize,
        max_tokens_per_month: usize,
        alert_threshold: f64,
        on_warning: AlertAction,
        on_critical: AlertAction,
        on_depleted: AlertAction,
    ) -> Self {
        let now = Utc::now();
        Self {
            max_tokens_per_call,
            max_tokens_per_day,
            max_tokens_per_month,
            alert_threshold,
            on_warning,
            on_critical,
            on_depleted,
            daily_usage: 0,
            monthly_usage: 0,
            alerts: Vec::new(),
            last_daily_reset: now,
            last_monthly_reset: now,
        }
    }

    /// Check if a request should be allowed
    pub fn check_request(&mut self, estimated_tokens: usize) -> EnforcementDecision {
        self.check_and_reset();

        // Per-call limit
        if estimated_tokens > self.max_tokens_per_call {
            return EnforcementDecision::Blocked(format!(
                "Request exceeds per-call limit: {} > {}",
                estimated_tokens, self.max_tokens_per_call
            ));
        }

        // Daily limit
        if self.daily_usage + estimated_tokens > self.max_tokens_per_day {
            self.emit_alert(BudgetHealth::Depleted);
            if self.on_depleted == AlertAction::Block {
                return EnforcementDecision::Blocked(format!(
                    "Daily budget would be exceeded: {} + {} > {}",
                    self.daily_usage, estimated_tokens, self.max_tokens_per_day
                ));
            }
        }

        // Monthly limit
        if self.monthly_usage + estimated_tokens > self.max_tokens_per_month {
            self.emit_alert(BudgetHealth::Depleted);
            if self.on_depleted == AlertAction::Block {
                return EnforcementDecision::Blocked(format!(
                    "Monthly budget would be exceeded: {} + {} > {}",
                    self.monthly_usage, estimated_tokens, self.max_tokens_per_month
                ));
            }
        }

        // Check health thresholds
        let health = self.evaluate_health();
        match health {
            BudgetHealth::Warning => {
                self.emit_alert(health);
                EnforcementDecision::Warning(format!(
                    "Budget usage at {:.0}% of daily limit",
                    self.daily_pct() * 100.0
                ))
            }
            BudgetHealth::Critical => {
                self.emit_alert(health);
                if self.on_critical == AlertAction::Block {
                    EnforcementDecision::Blocked("Budget critically low".into())
                } else {
                    EnforcementDecision::Warning(format!(
                        "Budget critically low at {:.0}%",
                        self.daily_pct() * 100.0
                    ))
                }
            }
            _ => EnforcementDecision::Allowed,
        }
    }

    /// Record actual token usage
    pub fn record_usage(&mut self, tokens: usize) {
        self.check_and_reset();
        self.daily_usage += tokens;
        self.monthly_usage += tokens;
    }

    pub fn alerts(&self) -> &[Alert] {
        &self.alerts
    }

    /// Drain and return all pending alerts.
    pub fn drain_alerts(&mut self) -> Vec<Alert> {
        std::mem::take(&mut self.alerts)
    }

    pub fn usage_summary(&self) -> UsageSummary {
        UsageSummary {
            daily_usage: self.daily_usage,
            daily_limit: self.max_tokens_per_day,
            daily_pct: self.daily_pct(),
            monthly_usage: self.monthly_usage,
            monthly_limit: self.max_tokens_per_month,
            monthly_pct: self.monthly_pct(),
            health: self.evaluate_health(),
        }
    }

    fn daily_pct(&self) -> f64 {
        if self.max_tokens_per_day == 0 {
            return 0.0;
        }
        self.daily_usage as f64 / self.max_tokens_per_day as f64
    }

    fn monthly_pct(&self) -> f64 {
        if self.max_tokens_per_month == 0 {
            return 0.0;
        }
        self.monthly_usage as f64 / self.max_tokens_per_month as f64
    }

    fn evaluate_health(&self) -> BudgetHealth {
        let pct = self.daily_pct();
        if pct >= 0.95 {
            BudgetHealth::Depleted
        } else if pct >= self.alert_threshold + 0.10 {
            BudgetHealth::Critical
        } else if pct >= self.alert_threshold {
            BudgetHealth::Warning
        } else {
            BudgetHealth::Healthy
        }
    }

    fn emit_alert(&mut self, health: BudgetHealth) {
        let action = match health {
            BudgetHealth::Warning => self.on_warning.clone(),
            BudgetHealth::Critical => self.on_critical.clone(),
            BudgetHealth::Depleted => self.on_depleted.clone(),
            BudgetHealth::Healthy => return,
        };

        let msg = format!(
            "Budget health: {:?} (daily {:.0}%, monthly {:.0}%)",
            health,
            self.daily_pct() * 100.0,
            self.monthly_pct() * 100.0
        );

        match action {
            AlertAction::Log => {
                eprintln!("tokenomics alert: {}", msg);
            }
            AlertAction::Notify => {
                eprintln!("tokenomics alert (notify): {}", msg);
            }
            AlertAction::Pause => {
                eprintln!("tokenomics alert (pause): {}", msg);
            }
            AlertAction::Block => {
                eprintln!("tokenomics alert (block): {}", msg);
            }
        }

        self.alerts.push(Alert {
            level: health,
            action,
            message: msg,
            timestamp: Utc::now(),
            budget_usage_pct: self.daily_pct(),
        });
    }

    fn check_and_reset(&mut self) {
        let now = Utc::now();
        let daily_elapsed = now.signed_duration_since(self.last_daily_reset);
        if daily_elapsed.num_seconds() > 86400 {
            self.daily_usage = 0;
            self.last_daily_reset = now;
        }
        let monthly_elapsed = now.signed_duration_since(self.last_monthly_reset);
        if monthly_elapsed.num_seconds() > 2592000 {
            self.monthly_usage = 0;
            self.last_monthly_reset = now;
        }
    }
}
