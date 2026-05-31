//! Analytics, reporting, and z-score anomaly detection

use crate::types::TokenUsageRecord;
use chrono::Timelike;
use std::collections::HashMap;

/// Analytics engine
pub struct AnalyticsEngine;

impl AnalyticsEngine {
    /// Analyze usage patterns
    pub fn analyze_patterns(usage: &[TokenUsageRecord]) -> UsagePatterns {
        let mut by_operation: HashMap<String, u64> = HashMap::new();
        let mut by_hour: HashMap<u8, u64> = HashMap::new();

        for u in usage {
            *by_operation.entry(u.operation.clone()).or_insert(0) += u.tokens_used;
            let hour = u.timestamp.hour() as u8;
            *by_hour.entry(hour).or_insert(0) += u.tokens_used;
        }

        UsagePatterns {
            by_operation,
            by_hour,
            total_operations: usage.len(),
        }
    }

    /// Detect anomalies in usage via z-score analysis.
    ///
    /// Each usage record's `tokens_used` is compared against the population
    /// mean/stddev. Records whose z-score exceeds 2.0 are flagged as anomalies.
    pub fn detect_anomalies(usage: &[TokenUsageRecord]) -> Vec<Anomaly> {
        if usage.len() < 2 {
            return Vec::new();
        }

        const Z_THRESHOLD: f64 = 2.0;

        let values: Vec<f64> = usage.iter().map(|u| u.tokens_used as f64).collect();
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();

        if stddev < f64::EPSILON {
            return Vec::new();
        }

        let mut anomalies = Vec::new();
        for (u, &val) in usage.iter().zip(values.iter()) {
            let z = (val - mean) / stddev;
            let abs_z = z.abs();
            if abs_z >= Z_THRESHOLD {
                let severity = ((abs_z - Z_THRESHOLD) / 3.0).clamp(0.0, 1.0);
                let direction = if z > 0.0 { "above" } else { "below" };
                anomalies.push(Anomaly {
                    timestamp: u.timestamp,
                    severity,
                    description: format!(
                        "Token usage {} (z={:.2}) is {direction} the mean ({:.0}) by {:.1} standard deviations",
                        u.tokens_used, z, mean, abs_z,
                    ),
                });
            }
        }

        anomalies
    }
}

#[derive(Debug)]
pub struct UsagePatterns {
    pub by_operation: HashMap<String, u64>,
    pub by_hour: HashMap<u8, u64>,
    pub total_operations: usize,
}

#[derive(Debug)]
pub struct Anomaly {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: f64,
    pub description: String,
}
