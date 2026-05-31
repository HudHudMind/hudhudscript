//! Cost attribution — per-user, per-session, per-feature cost tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A single cost event with attribution tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub feature_tag: Option<String>,
    pub environment: Option<String>,
    pub prompt_version: Option<String>,
    pub model: String,
    pub provider: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub thinking_tokens: usize,
    pub cached_tokens: usize,
    pub total_cost_usd: f64,
}

/// Per-feature budget status
#[derive(Debug, Clone)]
pub struct FeatureBudgetStatus {
    pub feature: String,
    pub spent_today: f64,
    pub budget: f64,
    pub remaining: f64,
    pub exceeded: bool,
}

/// Cost attribution aggregator
pub struct CostAttributor {
    events: Vec<CostEvent>,
    feature_budgets: HashMap<String, f64>,
}

impl Default for CostAttributor {
    fn default() -> Self {
        Self::new()
    }
}

impl CostAttributor {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            feature_budgets: HashMap::new(),
        }
    }

    pub fn record_event(&mut self, event: CostEvent) {
        self.events.push(event);
    }

    pub fn set_feature_budget(&mut self, feature: String, max_usd_per_day: f64) {
        self.feature_budgets.insert(feature, max_usd_per_day);
    }

    pub fn check_feature_budget(&self, feature: &str) -> Option<FeatureBudgetStatus> {
        let budget = self.feature_budgets.get(feature)?;
        let today = Utc::now().date_naive();
        let spent: f64 = self
            .events
            .iter()
            .filter(|e| e.feature_tag.as_deref() == Some(feature))
            .filter(|e| e.timestamp.date_naive() == today)
            .map(|e| e.total_cost_usd)
            .sum();
        Some(FeatureBudgetStatus {
            feature: feature.to_string(),
            spent_today: spent,
            budget: *budget,
            remaining: (*budget - spent).max(0.0),
            exceeded: spent >= *budget,
        })
    }

    pub fn cost_by_feature(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        for e in &self.events {
            if let Some(ref feat) = e.feature_tag {
                *map.entry(feat.clone()).or_insert(0.0) += e.total_cost_usd;
            }
        }
        map
    }

    pub fn cost_by_user(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        for e in &self.events {
            if let Some(ref uid) = e.user_id {
                *map.entry(uid.clone()).or_insert(0.0) += e.total_cost_usd;
            }
        }
        map
    }

    pub fn cost_by_session(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        for e in &self.events {
            if let Some(ref sid) = e.session_id {
                *map.entry(sid.clone()).or_insert(0.0) += e.total_cost_usd;
            }
        }
        map
    }

    pub fn cost_by_model(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        for e in &self.events {
            *map.entry(e.model.clone()).or_insert(0.0) += e.total_cost_usd;
        }
        map
    }

    pub fn total_cost_in_range(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> f64 {
        self.events
            .iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .map(|e| e.total_cost_usd)
            .sum()
    }

    pub fn total_events(&self) -> usize {
        self.events.len()
    }
    pub fn total_cost(&self) -> f64 {
        self.events.iter().map(|e| e.total_cost_usd).sum()
    }
}
