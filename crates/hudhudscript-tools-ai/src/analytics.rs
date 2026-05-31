//! Usage Analytics and Reporting (Issue #607)
//!
//! Aggregates data from [`CostTracker`] into structured reports: per-provider
//! summaries, per-model breakdowns, time-series usage, and overall statistics.

use crate::cost::{AiTokenUsage, CostTracker, Provider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

/// High-level usage summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    /// Total number of API requests.
    pub total_requests: usize,
    /// Total input tokens across all requests.
    pub total_input_tokens: usize,
    /// Total output tokens across all requests.
    pub total_output_tokens: usize,
    /// Total tokens (input + output).
    pub total_tokens: usize,
    /// Total cost in USD.
    pub total_cost_usd: f64,
    /// Average cost per request (USD).
    pub avg_cost_per_request: f64,
    /// Average tokens per request.
    pub avg_tokens_per_request: f64,
}

/// Per-provider usage breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummary {
    pub provider: Provider,
    pub requests: usize,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub cost_usd: f64,
    /// Fraction of total cost attributed to this provider (0.0 – 1.0).
    pub cost_share: f64,
}

/// Per-model usage breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSummary {
    pub model: String,
    pub provider: Provider,
    pub requests: usize,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub cost_usd: f64,
}

/// A complete analytics report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    /// Overall summary.
    pub summary: UsageSummary,
    /// Breakdown by provider.
    pub by_provider: Vec<ProviderSummary>,
    /// Breakdown by model.
    pub by_model: Vec<ModelSummary>,
    /// Unix timestamp when this report was generated.
    pub generated_at: u64,
}

// ---------------------------------------------------------------------------
// Analytics engine
// ---------------------------------------------------------------------------

/// Generates analytics reports from a [`CostTracker`].
pub struct Analytics {
    tracker: CostTracker,
}

impl Analytics {
    /// Create a new analytics engine backed by the given tracker.
    pub fn new(tracker: CostTracker) -> Self {
        Self { tracker }
    }

    /// Build a summary from the current tracker state.
    pub fn summary(&self) -> UsageSummary {
        let history = self.tracker.history();
        let total_requests = history.len();
        let total_input: usize = history.iter().map(|u| u.input_tokens).sum();
        let total_output: usize = history.iter().map(|u| u.output_tokens).sum();
        let total_tokens = total_input + total_output;
        let total_cost = self.tracker.total_cost();

        UsageSummary {
            total_requests,
            total_input_tokens: total_input,
            total_output_tokens: total_output,
            total_tokens,
            total_cost_usd: total_cost,
            avg_cost_per_request: if total_requests > 0 {
                total_cost / total_requests as f64
            } else {
                0.0
            },
            avg_tokens_per_request: if total_requests > 0 {
                total_tokens as f64 / total_requests as f64
            } else {
                0.0
            },
        }
    }

    /// Breakdown by provider.
    pub fn by_provider(&self) -> Vec<ProviderSummary> {
        let history = self.tracker.history();
        let total_cost = self.tracker.total_cost();

        let mut map: HashMap<Provider, (usize, usize, usize, f64)> = HashMap::new();
        for u in &history {
            let entry = map.entry(u.provider).or_default();
            entry.0 += 1; // requests
            entry.1 += u.input_tokens;
            entry.2 += u.output_tokens;
            entry.3 += u.cost_usd;
        }

        let mut summaries: Vec<ProviderSummary> = map
            .into_iter()
            .map(
                |(provider, (requests, input, output, cost))| ProviderSummary {
                    provider,
                    requests,
                    input_tokens: input,
                    output_tokens: output,
                    total_tokens: input + output,
                    cost_usd: cost,
                    cost_share: if total_cost > 0.0 {
                        cost / total_cost
                    } else {
                        0.0
                    },
                },
            )
            .collect();

        summaries.sort_by(|a, b| {
            b.cost_usd
                .partial_cmp(&a.cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        summaries
    }

    /// Breakdown by model.
    pub fn by_model(&self) -> Vec<ModelSummary> {
        let history = self.tracker.history();

        let mut map: HashMap<String, (Provider, usize, usize, usize, f64)> = HashMap::new();
        for u in &history {
            let entry = map
                .entry(u.model.clone())
                .or_insert((u.provider, 0, 0, 0, 0.0));
            entry.1 += 1; // requests
            entry.2 += u.input_tokens;
            entry.3 += u.output_tokens;
            entry.4 += u.cost_usd;
        }

        let mut summaries: Vec<ModelSummary> = map
            .into_iter()
            .map(
                |(model, (provider, requests, input, output, cost))| ModelSummary {
                    model,
                    provider,
                    requests,
                    input_tokens: input,
                    output_tokens: output,
                    total_tokens: input + output,
                    cost_usd: cost,
                },
            )
            .collect();

        summaries.sort_by(|a, b| {
            b.cost_usd
                .partial_cmp(&a.cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        summaries
    }

    /// Filter usage history to a specific time range (Unix seconds).
    pub fn history_in_range(&self, from: u64, to: u64) -> Vec<AiTokenUsage> {
        self.tracker
            .history()
            .into_iter()
            .filter(|u| u.timestamp >= from && u.timestamp <= to)
            .collect()
    }

    /// Generate a complete [`AnalyticsReport`].
    pub fn report(&self) -> AnalyticsReport {
        AnalyticsReport {
            summary: self.summary(),
            by_provider: self.by_provider(),
            by_model: self.by_model(),
            generated_at: unix_now(),
        }
    }

    /// Serialize the report to a JSON string.
    pub fn report_json(&self) -> Result<String, serde_json::Error> {
        let report = self.report();
        serde_json::to_string_pretty(&report)
    }

    /// Top-N most expensive models.
    pub fn top_models_by_cost(&self, n: usize) -> Vec<ModelSummary> {
        let mut models = self.by_model();
        models.truncate(n);
        models
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
