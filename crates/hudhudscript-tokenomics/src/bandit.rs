//! Contextual bandit for learned model routing
//!
//! Uses Thompson sampling to learn which model gives the best cost/quality
//! trade-off for each task type, building on the existing ReinforcementAgent.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Arm = a model that can be selected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArm {
    pub model: String,
    pub provider: String,
    pub cost_per_1k: f64,
}

/// Observed reward after using a model
#[derive(Debug, Clone)]
pub struct BanditReward {
    pub quality_score: f64, // 0.0-1.0 (user satisfaction, quality gate score)
    pub cost_usd: f64,      // actual cost
    pub latency_ms: u64,    // response latency
}

/// Beta distribution parameters for Thompson sampling
#[derive(Debug, Clone)]
struct BetaParams {
    alpha: f64, // successes + prior
    beta: f64,  // failures + prior
}

impl BetaParams {
    fn new() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
        } // uniform prior
    }

    /// Sample from Beta distribution using Jitter method approximation
    fn sample(&self) -> f64 {
        // Simple approximation: use mean + noise scaled by variance
        let mean = self.alpha / (self.alpha + self.beta);
        let variance = (self.alpha * self.beta)
            / ((self.alpha + self.beta).powi(2) * (self.alpha + self.beta + 1.0));
        let noise = (rand::random::<f64>() - 0.5) * 2.0 * variance.sqrt() * 3.0;
        (mean + noise).clamp(0.0, 1.0)
    }

    fn update(&mut self, reward: f64) {
        // reward in [0,1]: treat as Bernoulli-like
        if reward >= 0.5 {
            self.alpha += reward;
        } else {
            self.beta += 1.0 - reward;
        }
    }

    fn mean(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    fn total_observations(&self) -> f64 {
        self.alpha + self.beta - 2.0 // subtract priors
    }
}

/// Context key for bandit (task type + complexity)
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BanditContext {
    pub task_type: String,  // "chat", "code", "analysis", etc.
    pub complexity: String, // "simple", "medium", "hard"
}

/// Contextual bandit for model selection
pub struct ContextualBandit {
    arms: Vec<ModelArm>,
    /// Per-context, per-arm beta parameters
    params: HashMap<BanditContext, HashMap<String, BetaParams>>,
    /// Cost weight in reward computation (0.0-1.0)
    cost_weight: f64,
    /// Quality weight in reward computation (0.0-1.0)
    quality_weight: f64,
    /// Latency penalty weight
    latency_weight: f64,
    /// Total selections made
    total_selections: usize,
}

impl ContextualBandit {
    pub fn new(
        arms: Vec<ModelArm>,
        cost_weight: f64,
        quality_weight: f64,
        latency_weight: f64,
    ) -> Self {
        Self {
            arms,
            params: HashMap::new(),
            cost_weight: cost_weight.clamp(0.0, 1.0),
            quality_weight: quality_weight.clamp(0.0, 1.0),
            latency_weight: latency_weight.clamp(0.0, 1.0),
            total_selections: 0,
        }
    }

    /// Create with default model arms for testing and backward compatibility.
    /// Production use should build arms from `PricingRegistry` instead.
    pub fn with_defaults() -> Self {
        let arms = vec![
            ModelArm {
                model: "claude-haiku-3.5".into(),
                provider: "anthropic".into(),
                cost_per_1k: 0.0024,
            },
            ModelArm {
                model: "claude-sonnet-4".into(),
                provider: "anthropic".into(),
                cost_per_1k: 0.009,
            },
            ModelArm {
                model: "claude-opus-4".into(),
                provider: "anthropic".into(),
                cost_per_1k: 0.045,
            },
            ModelArm {
                model: "gpt-4o-mini".into(),
                provider: "openai".into(),
                cost_per_1k: 0.000375,
            },
            ModelArm {
                model: "gpt-4o".into(),
                provider: "openai".into(),
                cost_per_1k: 0.00625,
            },
            ModelArm {
                model: "deepseek-v3".into(),
                provider: "deepseek".into(),
                cost_per_1k: 0.000685,
            },
        ];
        Self::new(arms, 0.3, 0.6, 0.1)
    }

    /// Select best model for a context using Thompson sampling
    pub fn select(&mut self, context: &BanditContext) -> &ModelArm {
        self.total_selections += 1;

        let ctx_params = self.params.entry(context.clone()).or_insert_with(|| {
            let mut map = HashMap::new();
            for arm in &self.arms {
                map.insert(arm.model.clone(), BetaParams::new());
            }
            map
        });

        let mut best_score = f64::NEG_INFINITY;
        let mut best_idx = 0;

        for (i, arm) in self.arms.iter().enumerate() {
            let params = ctx_params.get(&arm.model).unwrap();
            let sample = params.sample();
            // Adjust by cost: cheaper models get a bonus
            let max_cost = self
                .arms
                .iter()
                .map(|a| a.cost_per_1k)
                .fold(0.0f64, f64::max);
            let cost_bonus = if max_cost > 0.0 {
                1.0 - arm.cost_per_1k / max_cost
            } else {
                0.0
            };
            let score = sample * self.quality_weight + cost_bonus * self.cost_weight;

            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        &self.arms[best_idx]
    }

    /// Update model performance after observing a reward
    pub fn update(&mut self, context: &BanditContext, model: &str, reward: &BanditReward) {
        // Compute composite reward in [0, 1]
        let quality_component = reward.quality_score * self.quality_weight;

        // Cost component: lower is better, normalize by max arm cost
        let max_cost = self
            .arms
            .iter()
            .map(|a| a.cost_per_1k)
            .fold(0.01f64, f64::max);
        let cost_component = (1.0 - (reward.cost_usd / max_cost).min(1.0)) * self.cost_weight;

        // Latency component: lower is better, normalize (assume 10s max)
        let latency_component =
            (1.0 - (reward.latency_ms as f64 / 10000.0).min(1.0)) * self.latency_weight;

        let composite = (quality_component + cost_component + latency_component).clamp(0.0, 1.0);

        if let Some(ctx_params) = self.params.get_mut(context) {
            if let Some(params) = ctx_params.get_mut(model) {
                params.update(composite);
            }
        }
    }

    /// Get estimated performance for each model in a context
    pub fn model_estimates(&self, context: &BanditContext) -> Vec<(String, f64)> {
        match self.params.get(context) {
            Some(ctx_params) => {
                let mut estimates: Vec<(String, f64)> = ctx_params
                    .iter()
                    .map(|(model, params)| (model.clone(), params.mean()))
                    .collect();
                estimates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                estimates
            }
            None => self.arms.iter().map(|a| (a.model.clone(), 0.5)).collect(),
        }
    }

    /// Get total number of observations for a context
    pub fn observations(&self, context: &BanditContext) -> f64 {
        self.params
            .get(context)
            .map(|ctx| ctx.values().map(|p| p.total_observations()).sum())
            .unwrap_or(0.0)
    }

    pub fn total_selections(&self) -> usize {
        self.total_selections
    }
    pub fn arm_count(&self) -> usize {
        self.arms.len()
    }
}
