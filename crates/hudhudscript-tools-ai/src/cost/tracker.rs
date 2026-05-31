use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::warn;

use crate::context::estimate_tokens;

use super::{default_pricing, CostError, ModelPricing, Provider};

/// Records token usage for a single request/response pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTokenUsage {
    /// Model used.
    pub model: String,
    /// Provider that served the request.
    pub provider: Provider,
    /// Tokens in the request (prompt).
    pub input_tokens: usize,
    /// Tokens in the response (completion).
    pub output_tokens: usize,
    /// Computed cost (USD) for this request.
    pub cost_usd: f64,
    /// Unix timestamp (seconds).
    pub timestamp: u64,
}

/// Configurable budget thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Hard spending limit (USD). Requests are rejected once exceeded.
    pub hard_limit_usd: f64,
    /// Warning thresholds as fractions of `hard_limit_usd` (e.g. `[0.5, 0.8]`).
    pub alert_thresholds: Vec<f64>,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            hard_limit_usd: 10.0,
            alert_thresholds: vec![0.5, 0.8, 0.95],
        }
    }
}

/// Thread-safe tracker that accumulates token usage, computes costs and
/// enforces budget limits.
#[derive(Clone)]
pub struct CostTracker {
    pub(crate) inner: Arc<RwLock<CostTrackerInner>>,
}

pub(crate) struct CostTrackerInner {
    pub(crate) pricing: HashMap<String, ModelPricing>,
    pub(crate) budget: BudgetConfig,
    pub(crate) history: Vec<AiTokenUsage>,
    pub(crate) total_cost_usd: f64,
    /// Tracks which alert thresholds have already fired.
    pub(crate) fired_alerts: Vec<bool>,
}

impl CostTracker {
    /// Create a tracker with default pricing and the given budget.
    pub fn new(budget: BudgetConfig) -> Self {
        let fired = vec![false; budget.alert_thresholds.len()];
        Self {
            inner: Arc::new(RwLock::new(CostTrackerInner {
                pricing: default_pricing(),
                budget,
                history: Vec::new(),
                total_cost_usd: 0.0,
                fired_alerts: fired,
            })),
        }
    }

    /// Create a tracker with default budget settings.
    pub fn with_defaults() -> Self {
        Self::new(BudgetConfig::default())
    }

    /// Override or add pricing for a specific model.
    pub fn set_model_pricing(&self, pricing: ModelPricing) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.pricing.insert(pricing.model.clone(), pricing);
    }

    /// Update the budget configuration at runtime.
    pub fn set_budget(&self, budget: BudgetConfig) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.fired_alerts = vec![false; budget.alert_thresholds.len()];
        inner.budget = budget;
    }

    /// Estimate the number of tokens in the given text.
    pub fn count_tokens(text: &str) -> usize {
        estimate_tokens(text)
    }

    /// Look up pricing for a model. Returns `None` if unknown.
    pub fn get_pricing(&self, model: &str) -> Option<ModelPricing> {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        inner.pricing.get(model).cloned()
    }

    /// Compute the cost of a single request given token counts.
    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: usize,
        output_tokens: usize,
    ) -> Result<f64, CostError> {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        let pricing = inner
            .pricing
            .get(model)
            .ok_or_else(|| CostError::UnknownModel(model.to_string()))?;

        let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k;
        Ok(input_cost + output_cost)
    }

    /// Record a completed request/response and enforce budget limits.
    pub fn record_usage(
        &self,
        model: &str,
        input_tokens: usize,
        output_tokens: usize,
    ) -> Result<AiTokenUsage, CostError> {
        let cost = self.calculate_cost(model, input_tokens, output_tokens)?;

        let provider = {
            let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
            inner
                .pricing
                .get(model)
                .map(|p| p.provider)
                .ok_or_else(|| CostError::UnknownModel(model.to_string()))?
        };

        let usage = AiTokenUsage {
            model: model.to_string(),
            provider,
            input_tokens,
            output_tokens,
            cost_usd: cost,
            timestamp: super::unix_now(),
        };

        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.total_cost_usd += cost;
        inner.history.push(usage.clone());

        let fraction = inner.total_cost_usd / inner.budget.hard_limit_usd;
        let thresholds: Vec<(usize, f64)> = inner
            .budget
            .alert_thresholds
            .iter()
            .copied()
            .enumerate()
            .collect();
        for (i, threshold) in thresholds {
            if fraction >= threshold && !inner.fired_alerts[i] {
                inner.fired_alerts[i] = true;
                warn!(
                    total_cost = inner.total_cost_usd,
                    limit = inner.budget.hard_limit_usd,
                    threshold_pct = threshold * 100.0,
                    "Budget alert: {:.0}% of spending limit reached",
                    threshold * 100.0
                );
            }
        }

        if inner.total_cost_usd > inner.budget.hard_limit_usd {
            return Err(CostError::BudgetExceeded {
                spent: inner.total_cost_usd,
                limit: inner.budget.hard_limit_usd,
            });
        }

        Ok(usage)
    }

    /// Record usage by estimating tokens from raw request/response text.
    pub fn record_usage_from_text(
        &self,
        model: &str,
        input_text: &str,
        output_text: &str,
    ) -> Result<AiTokenUsage, CostError> {
        let input_tokens = Self::count_tokens(input_text);
        let output_tokens = Self::count_tokens(output_text);
        self.record_usage(model, input_tokens, output_tokens)
    }

    /// Total cost accumulated so far (USD).
    pub fn total_cost(&self) -> f64 {
        self.inner
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .total_cost_usd
    }

    /// Total tokens (input + output) consumed so far.
    pub fn total_tokens(&self) -> usize {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        inner
            .history
            .iter()
            .map(|u| u.input_tokens + u.output_tokens)
            .sum()
    }

    /// Number of requests recorded.
    pub fn request_count(&self) -> usize {
        self.inner
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .history
            .len()
    }

    /// Remaining budget (USD). Returns 0.0 if over budget.
    pub fn remaining_budget(&self) -> f64 {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        (inner.budget.hard_limit_usd - inner.total_cost_usd).max(0.0)
    }

    /// Clone the full usage history.
    pub fn history(&self) -> Vec<AiTokenUsage> {
        self.inner
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .history
            .clone()
    }

    /// Cost breakdown grouped by provider.
    pub fn cost_by_provider(&self) -> HashMap<Provider, f64> {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        let mut map: HashMap<Provider, f64> = HashMap::new();
        for u in &inner.history {
            *map.entry(u.provider).or_default() += u.cost_usd;
        }
        map
    }

    /// Cost breakdown grouped by model.
    pub fn cost_by_model(&self) -> HashMap<String, f64> {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        let mut map: HashMap<String, f64> = HashMap::new();
        for u in &inner.history {
            *map.entry(u.model.clone()).or_default() += u.cost_usd;
        }
        map
    }

    /// Reset all tracked usage and cost data.
    pub fn reset(&self) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.history.clear();
        inner.total_cost_usd = 0.0;
        inner.fired_alerts = vec![false; inner.budget.alert_thresholds.len()];
    }
}
