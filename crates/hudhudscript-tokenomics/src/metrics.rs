//! Prometheus-compatible metrics collection and export

use crate::types::Metrics;
use chrono::Utc;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct MetricEntry {
    kind: MetricKind,
    help: String,
    values: BTreeMap<Vec<(String, String)>, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricKind {
    Counter,
    Gauge,
    Histogram,
}

impl MetricKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
        }
    }
}

/// Metrics collector
pub struct MetricsCollector {
    entries: Arc<RwLock<BTreeMap<String, MetricEntry>>>,
    histogram_buckets: Vec<f64>,
    histogram_obs: Arc<RwLock<BTreeMap<String, Vec<f64>>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        // Pre-register well-known tokenomics metrics.
        let mut map = BTreeMap::new();
        for (name, kind, help) in [
            (
                "tokenomics_total_users",
                MetricKind::Gauge,
                "Total number of tracked users",
            ),
            (
                "tokenomics_total_usage_tokens",
                MetricKind::Counter,
                "Cumulative token usage",
            ),
            (
                "tokenomics_average_usage_tokens",
                MetricKind::Gauge,
                "Average token usage per user",
            ),
            (
                "tokenomics_peak_usage_tokens",
                MetricKind::Gauge,
                "Peak token usage observed",
            ),
            (
                "tokenomics_prediction_accuracy",
                MetricKind::Gauge,
                "Current prediction model accuracy (0-1)",
            ),
        ] {
            map.insert(
                name.to_string(),
                MetricEntry {
                    kind,
                    help: help.to_string(),
                    values: BTreeMap::new(),
                },
            );
        }

        Self {
            entries: Arc::new(RwLock::new(map)),
            histogram_buckets: vec![10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0],
            histogram_obs: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Register a new metric.
    pub async fn register(&self, name: &str, kind: &str, help: &str) {
        let kind = match kind {
            "counter" => MetricKind::Counter,
            "histogram" => MetricKind::Histogram,
            _ => MetricKind::Gauge,
        };
        let mut map = self.entries.write().await;
        map.entry(name.to_string()).or_insert_with(|| MetricEntry {
            kind,
            help: help.to_string(),
            values: BTreeMap::new(),
        });
    }

    /// Set a gauge value.
    pub async fn set_gauge(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        let label_vec: Vec<(String, String)> = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let mut map = self.entries.write().await;
        if let Some(entry) = map.get_mut(name) {
            entry.values.insert(label_vec, value);
        }
    }

    /// Increment a counter.
    pub async fn inc_counter(&self, name: &str, labels: &[(&str, &str)], delta: f64) {
        let label_vec: Vec<(String, String)> = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let mut map = self.entries.write().await;
        if let Some(entry) = map.get_mut(name) {
            let val = entry.values.entry(label_vec).or_insert(0.0);
            *val += delta;
        }
    }

    /// Record a histogram observation.
    pub async fn observe_histogram(&self, name: &str, value: f64) {
        let mut obs = self.histogram_obs.write().await;
        obs.entry(name.to_string()).or_default().push(value);
    }

    /// Update built-in metrics from a Metrics snapshot.
    pub async fn update_from_metrics(&self, m: &Metrics) {
        self.set_gauge("tokenomics_total_users", &[], m.total_users as f64)
            .await;
        {
            let mut map = self.entries.write().await;
            if let Some(entry) = map.get_mut("tokenomics_total_usage_tokens") {
                entry.values.insert(vec![], m.total_usage as f64);
            }
        }
        self.set_gauge("tokenomics_average_usage_tokens", &[], m.average_usage)
            .await;
        self.set_gauge("tokenomics_peak_usage_tokens", &[], m.peak_usage as f64)
            .await;
        self.set_gauge("tokenomics_prediction_accuracy", &[], m.prediction_accuracy)
            .await;
    }

    /// Collect current metrics
    pub async fn collect(&self) -> Metrics {
        let map = self.entries.read().await;
        let get = |name: &str| -> f64 {
            map.get(name)
                .and_then(|e| e.values.get(&vec![]))
                .copied()
                .unwrap_or(0.0)
        };

        Metrics {
            total_users: get("tokenomics_total_users") as usize,
            total_usage: get("tokenomics_total_usage_tokens") as u64,
            average_usage: get("tokenomics_average_usage_tokens"),
            peak_usage: get("tokenomics_peak_usage_tokens") as u64,
            prediction_accuracy: get("tokenomics_prediction_accuracy"),
            timestamp: Utc::now(),
        }
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let map = match self.entries.try_read() {
            Ok(m) => m,
            Err(_) => return String::new(),
        };
        let histogram_obs = self.histogram_obs.try_read().ok();

        let mut out = String::new();

        for (name, entry) in map.iter() {
            out.push_str(&format!("# HELP {} {}\n", name, entry.help));
            out.push_str(&format!("# TYPE {} {}\n", name, entry.kind.as_str()));

            if entry.kind == MetricKind::Histogram {
                if let Some(ref obs_map) = histogram_obs {
                    if let Some(observations) = obs_map.get(name) {
                        let count = observations.len();
                        let sum: f64 = observations.iter().sum();
                        for &bound in &self.histogram_buckets {
                            let bucket_count = observations.iter().filter(|&&v| v <= bound).count();
                            out.push_str(&format!(
                                "{}_bucket{{le=\"{}\"}} {}\n",
                                name, bound, bucket_count
                            ));
                        }
                        out.push_str(&format!("{}_bucket{{le=\"+Inf\"}} {}\n", name, count));
                        out.push_str(&format!("{}_sum {}\n", name, sum));
                        out.push_str(&format!("{}_count {}\n", name, count));
                    }
                }
            } else if entry.values.is_empty() {
                out.push_str(&format!("{} 0\n", name));
            } else {
                for (labels, value) in &entry.values {
                    if labels.is_empty() {
                        out.push_str(&format!("{} {}\n", name, value));
                    } else {
                        let label_str: String = labels
                            .iter()
                            .map(|(k, v)| format!("{}=\"{}\"", k, v))
                            .collect::<Vec<_>>()
                            .join(",");
                        out.push_str(&format!("{}{{{}}} {}\n", name, label_str, value));
                    }
                }
            }
            out.push('\n');
        }

        out
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
