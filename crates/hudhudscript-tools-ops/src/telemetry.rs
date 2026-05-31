//! Tool Telemetry, Tracing, and Logging (Issue #124)
//!
//! Wraps tool execution with standardised observability:
//! - structured tracing spans with tool name, duration, status
//! - input/output size recording
//! - per-tool and aggregate statistics via `TelemetryCollector`

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};

// ---------------------------------------------------------------------------
// Execution outcome
// ---------------------------------------------------------------------------

/// Whether a tool call succeeded or failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failure,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Success => write!(f, "success"),
            ExecutionStatus::Failure => write!(f, "failure"),
        }
    }
}

// ---------------------------------------------------------------------------
// Telemetry record
// ---------------------------------------------------------------------------

/// A single tool execution telemetry record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTelemetryRecord {
    /// Name of the tool that was called.
    pub tool_name: String,
    /// Wall-clock duration of the execution.
    pub duration: Duration,
    /// Whether the call succeeded or failed.
    pub status: ExecutionStatus,
    /// Size (in bytes) of the serialised input arguments.
    pub input_size_bytes: usize,
    /// Size (in bytes) of the serialised output.
    pub output_size_bytes: usize,
    /// Optional error message (populated on failure).
    pub error: Option<String>,
    /// Estimated token count of the output (4 chars ≈ 1 token heuristic).
    pub output_tokens_estimated: usize,
}

// ---------------------------------------------------------------------------
// Per-tool aggregated statistics
// ---------------------------------------------------------------------------

/// Aggregated statistics for a single tool across multiple calls.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolStats {
    /// Total number of calls.
    pub call_count: u64,
    /// Number of successful calls.
    pub success_count: u64,
    /// Number of failed calls.
    pub failure_count: u64,
    /// Total execution time.
    pub total_duration: Duration,
    /// Minimum observed execution time.
    pub min_duration: Option<Duration>,
    /// Maximum observed execution time.
    pub max_duration: Option<Duration>,
    /// Total bytes of input processed.
    pub total_input_bytes: u64,
    /// Total bytes of output produced.
    pub total_output_bytes: u64,
}

impl ToolStats {
    fn record(&mut self, record: &ToolTelemetryRecord) {
        self.call_count += 1;
        match record.status {
            ExecutionStatus::Success => self.success_count += 1,
            ExecutionStatus::Failure => self.failure_count += 1,
        }
        self.total_duration += record.duration;
        self.min_duration = Some(
            self.min_duration
                .map_or(record.duration, |m: Duration| m.min(record.duration)),
        );
        self.max_duration = Some(
            self.max_duration
                .map_or(record.duration, |m: Duration| m.max(record.duration)),
        );
        self.total_input_bytes += record.input_size_bytes as u64;
        self.total_output_bytes += record.output_size_bytes as u64;
    }

    /// Average duration per call.
    pub fn avg_duration(&self) -> Option<Duration> {
        if self.call_count == 0 {
            None
        } else {
            Some(self.total_duration / self.call_count as u32)
        }
    }

    /// Success rate as a fraction `[0.0, 1.0]`.
    pub fn success_rate(&self) -> f64 {
        if self.call_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.call_count as f64
        }
    }
}

// ---------------------------------------------------------------------------
// Telemetry collector
// ---------------------------------------------------------------------------

/// Thread-safe telemetry collector that stores records and aggregates stats.
#[derive(Clone)]
pub struct TelemetryCollector {
    records: Arc<RwLock<Vec<ToolTelemetryRecord>>>,
    stats: Arc<RwLock<HashMap<String, ToolStats>>>,
    /// Maximum number of raw records retained in memory (ring-buffer behaviour).
    max_records: usize,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl TelemetryCollector {
    /// Create a collector that retains at most `max_records` raw records.
    pub fn new(max_records: usize) -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(HashMap::new())),
            max_records,
        }
    }

    /// Record a tool execution telemetry entry.
    pub fn record(&self, entry: ToolTelemetryRecord) {
        // Update per-tool stats
        {
            let mut stats = self.stats.write().unwrap();
            stats
                .entry(entry.tool_name.clone())
                .or_default()
                .record(&entry);
        }

        // Append to raw records (capped)
        {
            let mut records = self.records.write().unwrap();
            if records.len() >= self.max_records {
                records.remove(0); // drop oldest
            }
            records.push(entry);
        }
    }

    /// Return a snapshot of all raw records.
    pub fn all_records(&self) -> Vec<ToolTelemetryRecord> {
        self.records.read().unwrap().clone()
    }

    /// Return stats for a specific tool, or `None` if it has never been called.
    pub fn stats_for(&self, tool_name: &str) -> Option<ToolStats> {
        self.stats.read().unwrap().get(tool_name).cloned()
    }

    /// Return stats for all tools.
    pub fn all_stats(&self) -> HashMap<String, ToolStats> {
        self.stats.read().unwrap().clone()
    }

    /// Clear all collected data.
    pub fn clear(&self) {
        self.records.write().unwrap().clear();
        self.stats.write().unwrap().clear();
    }

    /// Total number of raw records stored.
    pub fn record_count(&self) -> usize {
        self.records.read().unwrap().len()
    }
}

// ---------------------------------------------------------------------------
// InstrumentedToolExecutor
// ---------------------------------------------------------------------------

/// Wraps calls to a [`ToolRegistry`](hudhudscript_tools_schema::ToolRegistry) with
/// automatic telemetry recording and structured tracing spans.
pub struct InstrumentedToolExecutor {
    registry: Arc<hudhudscript_tools_schema::ToolRegistry>,
    collector: TelemetryCollector,
}

impl InstrumentedToolExecutor {
    /// Create a new executor wrapping `registry`, using `collector` for telemetry.
    pub fn new(
        registry: Arc<hudhudscript_tools_schema::ToolRegistry>,
        collector: TelemetryCollector,
    ) -> Self {
        Self {
            registry,
            collector,
        }
    }

    /// Call a tool and automatically emit a tracing span + telemetry record.
    #[instrument(
        name = "tool_call",
        skip(self, arguments),
        fields(tool = %tool_name)
    )]
    pub async fn call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, hudhudscript_tools_schema::RegistryError> {
        let input_size = arguments.to_string().len();
        debug!(tool = tool_name, input_bytes = input_size, "Calling tool");

        let start = Instant::now();
        let result = self.registry.call_tool(tool_name, arguments).await;
        let duration = start.elapsed();

        match &result {
            Ok(output) => {
                let output_str = output.to_string();
                let output_size = output_str.len();
                let output_tokens = output_str.chars().count() / 4;

                info!(
                    tool = tool_name,
                    duration_ms = duration.as_millis(),
                    input_bytes = input_size,
                    output_bytes = output_size,
                    output_tokens_est = output_tokens,
                    status = "success",
                    "Tool call completed"
                );

                self.collector.record(ToolTelemetryRecord {
                    tool_name: tool_name.to_string(),
                    duration,
                    status: ExecutionStatus::Success,
                    input_size_bytes: input_size,
                    output_size_bytes: output_size,
                    error: None,
                    output_tokens_estimated: output_tokens,
                });
            }
            Err(err) => {
                let err_msg = err.to_string();
                error!(
                    tool = tool_name,
                    duration_ms = duration.as_millis(),
                    input_bytes = input_size,
                    error = err_msg.as_str(),
                    status = "failure",
                    "Tool call failed"
                );

                self.collector.record(ToolTelemetryRecord {
                    tool_name: tool_name.to_string(),
                    duration,
                    status: ExecutionStatus::Failure,
                    input_size_bytes: input_size,
                    output_size_bytes: 0,
                    error: Some(err_msg),
                    output_tokens_estimated: 0,
                });
            }
        }

        result
    }

    /// Access the telemetry collector (e.g. to inspect stats).
    pub fn collector(&self) -> &TelemetryCollector {
        &self.collector
    }
}

// ---------------------------------------------------------------------------
// Convenience: emit a standalone telemetry record (no registry required)
// ---------------------------------------------------------------------------

/// Record the telemetry for an already-completed tool call.
///
/// Useful when the tool execution path is external but you still want to
/// capture metrics in the shared collector.
pub fn record_tool_telemetry(
    collector: &TelemetryCollector,
    tool_name: &str,
    duration: Duration,
    status: ExecutionStatus,
    input_size_bytes: usize,
    output_size_bytes: usize,
    error: Option<String>,
) {
    let output_tokens_estimated = output_size_bytes / 4;
    let record = ToolTelemetryRecord {
        tool_name: tool_name.to_string(),
        duration,
        status,
        input_size_bytes,
        output_size_bytes,
        error,
        output_tokens_estimated,
    };

    match &record.status {
        ExecutionStatus::Success => {
            debug!(
                tool = tool_name,
                duration_ms = duration.as_millis(),
                "Recorded successful tool telemetry"
            );
        }
        ExecutionStatus::Failure => {
            warn!(
                tool = tool_name,
                duration_ms = duration.as_millis(),
                error = record.error.as_deref().unwrap_or("unknown"),
                "Recorded failed tool telemetry"
            );
        }
    }

    collector.record(record);
}
