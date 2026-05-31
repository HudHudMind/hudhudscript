//! Main tokenomics optimization engine

use crate::error::{Result, TokenomicsError};
use crate::ml::{FeatureExtractor, TokenPredictionModel};
use crate::types::{Budget, OptimizationStrategy, Prediction, TimeSeriesPoint, TokenUsageRecord};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Main tokenomics engine
pub struct TokenomicsEngine {
    /// ML model for predictions
    model: Arc<RwLock<TokenPredictionModel>>,

    /// User budgets
    budgets: Arc<RwLock<HashMap<String, Budget>>>,

    /// Historical usage data
    usage_history: Arc<RwLock<Vec<TokenUsageRecord>>>,

    /// Optimization strategy
    strategy: OptimizationStrategy,

    /// Fallback to rule-based system
    fallback_enabled: bool,
}

impl TokenomicsEngine {
    /// Create a new tokenomics engine
    pub fn new(strategy: OptimizationStrategy) -> Self {
        info!(
            "Initializing TokenomicsEngine with strategy: {:?}",
            strategy
        );

        Self {
            model: Arc::new(RwLock::new(TokenPredictionModel::new(10))),
            budgets: Arc::new(RwLock::new(HashMap::new())),
            usage_history: Arc::new(RwLock::new(Vec::new())),
            strategy,
            fallback_enabled: true,
        }
    }

    /// Create or get budget for a user
    pub async fn get_or_create_budget(&self, user_id: &str, initial_budget: u64) -> Result<Budget> {
        let mut budgets = self.budgets.write().await;

        if let Some(budget) = budgets.get(user_id) {
            Ok(budget.clone())
        } else {
            let budget = Budget::new(user_id.to_string(), initial_budget);
            budgets.insert(user_id.to_string(), budget.clone());
            info!(
                "Created new budget for user {}: {} tokens",
                user_id, initial_budget
            );
            Ok(budget)
        }
    }

    /// Consume tokens from user budget
    pub async fn consume_tokens(&self, user_id: &str, amount: u64, operation: &str) -> Result<()> {
        let mut budgets = self.budgets.write().await;

        let budget = budgets
            .get_mut(user_id)
            .ok_or_else(|| TokenomicsError::BudgetNotFound(user_id.to_string()))?;

        if amount > budget.remaining {
            return Err(TokenomicsError::InsufficientBudget {
                needed: amount,
                available: budget.remaining,
            });
        }

        budget.consume(amount).map_err(TokenomicsError::Unknown)?;

        // Record usage
        let usage = TokenUsageRecord::new(user_id.to_string(), operation.to_string(), amount);
        let mut history = self.usage_history.write().await;
        history.push(usage);

        debug!(
            "User {} consumed {} tokens for {}",
            user_id, amount, operation
        );
        Ok(())
    }

    /// Predict future token usage.
    ///
    /// Returns a Prediction whose `model_version` field indicates the source:
    /// - "ml-v1+": real ML model output
    /// - "fallback-v1": rule-based heuristic (used when ML fails or insufficient data)
    ///
    /// Callers MUST inspect `model_version` to know whether the result is from
    /// the real model or a degraded fallback. Use `predict_usage_strict()` to
    /// reject fallback results.
    pub async fn predict_usage(&self, user_id: &str, horizon_seconds: u64) -> Result<Prediction> {
        let history = self.usage_history.read().await;

        // Filter user's history
        let user_history: Vec<_> = history.iter().filter(|u| u.user_id == user_id).collect();

        if user_history.len() < 10 {
            if self.fallback_enabled {
                warn!(
                    "Insufficient data for user {}, using fallback (model_version='fallback-v1')",
                    user_id
                );
                return self.fallback_prediction(user_id, horizon_seconds).await;
            } else {
                return Err(TokenomicsError::ColdStart);
            }
        }

        // Convert to time series
        let time_series: Vec<TimeSeriesPoint> = user_history
            .iter()
            .map(|u| TimeSeriesPoint {
                timestamp: u.timestamp,
                value: u.tokens_used as f64,
                metadata: serde_json::json!({"operation": u.operation}),
            })
            .collect();

        // Extract features
        let features = FeatureExtractor::extract(&time_series);

        // Predict
        let model = self.model.read().await;
        match model.predict(&features, horizon_seconds) {
            Ok(prediction) => {
                info!(
                    "Predicted {} tokens for user {} over {}s (confidence: {:.2}%)",
                    prediction.predicted_usage,
                    user_id,
                    horizon_seconds,
                    prediction.confidence * 100.0
                );
                Ok(prediction)
            }
            Err(e) if e.should_fallback_to_rules() && self.fallback_enabled => {
                warn!(
                    "ML prediction failed: {} — using fallback (model_version='fallback-v1')",
                    e
                );
                self.fallback_prediction(user_id, horizon_seconds).await
            }
            Err(e) => Err(e),
        }
    }

    /// Strict version of predict_usage that REJECTS fallback results.
    ///
    /// Returns an error if the underlying prediction came from the rule-based
    /// fallback instead of a real ML model. Use this when callers cannot
    /// tolerate degraded predictions (e.g., billing decisions).
    pub async fn predict_usage_strict(
        &self,
        user_id: &str,
        horizon_seconds: u64,
    ) -> Result<Prediction> {
        let pred = self.predict_usage(user_id, horizon_seconds).await?;
        if pred.model_version.starts_with("fallback") {
            return Err(TokenomicsError::PredictionFailed(format!(
                "Strict mode: refused fallback prediction (model_version='{}')",
                pred.model_version
            )));
        }
        Ok(pred)
    }

    /// Fallback prediction using rule-based system
    async fn fallback_prediction(&self, user_id: &str, horizon_seconds: u64) -> Result<Prediction> {
        let history = self.usage_history.read().await;

        let user_history: Vec<_> = history.iter().filter(|u| u.user_id == user_id).collect();

        let avg_usage = if user_history.is_empty() {
            1000.0 // Default assumption
        } else {
            let total: u64 = user_history.iter().map(|u| u.tokens_used).sum();
            total as f64 / user_history.len() as f64
        };

        // Simple linear extrapolation
        let predicted = (avg_usage * horizon_seconds as f64 / 3600.0) as u64;

        Ok(Prediction {
            predicted_usage: predicted,
            confidence: 0.5, // Lower confidence for fallback
            horizon_seconds,
            timestamp: Utc::now(),
            model_version: "fallback-v1".to_string(),
        })
    }

    /// Optimize token allocation based on strategy
    pub async fn optimize_allocation(&self, user_id: &str) -> Result<u64> {
        let prediction = self.predict_usage(user_id, 3600).await?; // 1 hour ahead

        let optimized = match &self.strategy {
            OptimizationStrategy::Conservative => {
                // Allocate 80% of predicted usage
                (prediction.predicted_usage as f64 * 0.8) as u64
            }
            OptimizationStrategy::Balanced => {
                // Allocate 100% of predicted usage
                prediction.predicted_usage
            }
            OptimizationStrategy::Aggressive => {
                // Allocate 120% of predicted usage
                (prediction.predicted_usage as f64 * 1.2) as u64
            }
            OptimizationStrategy::Custom {
                performance_weight,
                cost_weight,
            } => {
                // Custom formula
                let base = prediction.predicted_usage as f64;
                let factor = performance_weight / (performance_weight + cost_weight);
                (base * (0.8 + factor * 0.4)) as u64
            }
        };

        info!(
            "Optimized allocation for user {}: {} tokens (strategy: {:?})",
            user_id, optimized, self.strategy
        );

        Ok(optimized)
    }

    /// Train the ML model with accumulated data
    pub async fn train_model(&self) -> Result<()> {
        let history = self.usage_history.read().await;

        if history.len() < 100 {
            return Err(TokenomicsError::ColdStart);
        }

        info!("Training model with {} samples", history.len());

        // Group by user and create training data
        let mut user_data: HashMap<String, Vec<&TokenUsageRecord>> = HashMap::new();
        for usage in history.iter() {
            user_data
                .entry(usage.user_id.clone())
                .or_default()
                .push(usage);
        }

        let mut all_features = Vec::new();
        let mut all_targets = Vec::new();

        for (_, usages) in user_data.iter() {
            if usages.len() < 10 {
                continue;
            }

            // Create sliding windows
            for window in usages.windows(11) {
                let time_series: Vec<TimeSeriesPoint> = window[..10]
                    .iter()
                    .map(|u| TimeSeriesPoint {
                        timestamp: u.timestamp,
                        value: u.tokens_used as f64,
                        metadata: serde_json::json!({}),
                    })
                    .collect();

                let features = FeatureExtractor::extract(&time_series);
                let target = window[10].tokens_used as f64;

                all_features.push(features);
                all_targets.push(target);
            }
        }

        if all_features.is_empty() {
            return Err(TokenomicsError::ColdStart);
        }

        let mut model = self.model.write().await;
        model.train(&all_features, &all_targets)?;

        info!(
            "Model trained successfully. Accuracy: {:.2}%",
            model.metadata().accuracy * 100.0
        );

        Ok(())
    }

    /// Get usage statistics
    pub async fn get_statistics(&self, user_id: &str) -> Result<UsageStatistics> {
        let history = self.usage_history.read().await;
        let budget = self.budgets.read().await;

        let user_history: Vec<_> = history.iter().filter(|u| u.user_id == user_id).collect();

        let total_usage: u64 = user_history.iter().map(|u| u.tokens_used).sum();
        let avg_usage = if user_history.is_empty() {
            0.0
        } else {
            total_usage as f64 / user_history.len() as f64
        };

        let current_budget = budget.get(user_id).cloned();

        Ok(UsageStatistics {
            total_usage,
            average_usage: avg_usage,
            operation_count: user_history.len(),
            current_budget,
        })
    }

    /// Check if model needs retraining
    pub async fn needs_retraining(&self) -> bool {
        let model = self.model.read().await;
        model.needs_retraining()
    }
}

/// Usage statistics for a user
#[derive(Debug, Clone)]
pub struct UsageStatistics {
    pub total_usage: u64,
    pub average_usage: f64,
    pub operation_count: usize,
    pub current_budget: Option<Budget>,
}
