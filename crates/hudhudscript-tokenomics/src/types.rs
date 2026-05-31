//! Core types for tokenomics system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Token budget for a user or session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: Uuid,
    pub user_id: String,
    pub total: u64,
    pub used: u64,
    pub remaining: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Budget {
    pub fn new(user_id: String, total: u64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            total,
            used: 0,
            remaining: total,
            created_at: now,
            updated_at: now,
            expires_at: None,
        }
    }

    pub fn consume(&mut self, amount: u64) -> Result<(), String> {
        if amount > self.remaining {
            return Err(format!(
                "Insufficient budget: need {}, have {}",
                amount, self.remaining
            ));
        }
        self.used += amount;
        self.remaining -= amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn refill(&mut self, amount: u64) {
        self.total += amount;
        self.remaining += amount;
        self.updated_at = Utc::now();
    }

    pub fn usage_percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.used as f64 / self.total as f64) * 100.0
    }
}

/// Token usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageRecord {
    pub id: Uuid,
    pub user_id: String,
    pub session_id: Option<String>,
    pub operation: String,
    pub tokens_used: u64,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl TokenUsageRecord {
    pub fn new(user_id: String, operation: String, tokens_used: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            session_id: None,
            operation,
            tokens_used,
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }
}

/// Prediction result from ML models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub predicted_usage: u64,
    pub confidence: f64,
    pub horizon_seconds: u64,
    pub timestamp: DateTime<Utc>,
    pub model_version: String,
}

/// Optimization strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum OptimizationStrategy {
    /// Conservative: minimize token usage
    Conservative,
    /// Balanced: balance performance and cost
    #[default]
    Balanced,
    /// Aggressive: maximize performance
    Aggressive,
    /// Custom: user-defined parameters
    Custom {
        performance_weight: f64,
        cost_weight: f64,
    },
}

/// ML model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub trained_at: DateTime<Utc>,
    pub accuracy: f64,
    pub samples_count: usize,
}

/// Federated learning update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedUpdate {
    pub id: Uuid,
    pub model_version: String,
    pub gradients: Vec<f64>,
    pub sample_count: usize,
    pub timestamp: DateTime<Utc>,
}

/// Reinforcement learning reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    pub action_id: Uuid,
    pub reward_value: f64,
    pub feedback: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub metadata: serde_json::Value,
}

/// Analytics metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub total_users: usize,
    pub total_usage: u64,
    pub average_usage: f64,
    pub peak_usage: u64,
    pub prediction_accuracy: f64,
    pub timestamp: DateTime<Utc>,
}
