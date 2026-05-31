//! Budget management utilities

use crate::error::{Result, TokenomicsError};
use crate::types::Budget;

/// Budget manager
pub struct BudgetManager;

impl BudgetManager {
    /// Check if budget is low
    pub fn is_low(budget: &Budget, threshold: u64) -> bool {
        budget.remaining < threshold
    }

    /// Calculate refill amount based on usage pattern
    pub fn calculate_refill(budget: &Budget, predicted_usage: u64) -> u64 {
        let current_deficit = predicted_usage.saturating_sub(budget.remaining);
        let buffer = (predicted_usage as f64 * 0.2) as u64; // 20% buffer
        current_deficit + buffer
    }

    /// Validate budget operation
    pub fn validate_operation(budget: &Budget, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(TokenomicsError::InvalidBudget(amount));
        }

        if amount > budget.remaining {
            return Err(TokenomicsError::InsufficientBudget {
                needed: amount,
                available: budget.remaining,
            });
        }

        Ok(())
    }

    /// Get budget health status
    pub fn health_status(budget: &Budget) -> BudgetHealth {
        let usage_pct = budget.usage_percentage();

        if usage_pct < 50.0 {
            BudgetHealth::Healthy
        } else if usage_pct < 80.0 {
            BudgetHealth::Warning
        } else if usage_pct < 95.0 {
            BudgetHealth::Critical
        } else {
            BudgetHealth::Depleted
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetHealth {
    Healthy,
    Warning,
    Critical,
    Depleted,
}
