//! Tool call cost tracking — MCP, HTTP, and other external tool calls

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Type of tool call being tracked
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCallType {
    /// MCP tool call
    Mcp,
    /// HTTP/REST tool call
    Http,
    /// Database query
    Database,
    /// File system operation
    FileSystem,
    /// Custom tool
    Custom(String),
}

/// A single tracked tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedToolCall {
    pub id: Uuid,
    pub call_type: ToolCallType,
    pub tool_name: String,
    pub server_name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub input_size_bytes: usize,
    pub output_size_bytes: usize,
    pub success: bool,
    pub estimated_cost_usd: f64,
    pub metadata: serde_json::Value,
}

/// Aggregated tool usage statistics
#[derive(Debug, Clone, Default)]
pub struct ToolUsageStats {
    pub total_calls: usize,
    pub successful_calls: usize,
    pub failed_calls: usize,
    pub total_cost_usd: f64,
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub total_input_bytes: usize,
    pub total_output_bytes: usize,
}

/// Tool call tracker — records and aggregates all non-LLM tool invocations
pub struct ToolCallTracker {
    calls: Vec<TrackedToolCall>,
    /// Per-tool cost rates (tool_name -> cost_per_call)
    cost_rates: HashMap<String, f64>,
}

impl Default for ToolCallTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolCallTracker {
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            cost_rates: HashMap::new(),
        }
    }

    /// Set cost rate for a specific tool
    pub fn set_cost_rate(&mut self, tool_name: String, cost_per_call: f64) {
        self.cost_rates.insert(tool_name, cost_per_call);
    }

    /// Start tracking a tool call (returns ID)
    pub fn start_call(
        &mut self,
        call_type: ToolCallType,
        tool_name: &str,
        server_name: Option<&str>,
        input_size_bytes: usize,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let estimated_cost = self.cost_rates.get(tool_name).copied().unwrap_or(0.0);
        self.calls.push(TrackedToolCall {
            id,
            call_type,
            tool_name: tool_name.to_string(),
            server_name: server_name.map(String::from),
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            input_size_bytes,
            output_size_bytes: 0,
            success: false,
            estimated_cost_usd: estimated_cost,
            metadata: serde_json::json!({}),
        });
        id
    }

    /// Complete a tracked tool call
    pub fn complete_call(&mut self, id: Uuid, output_size_bytes: usize, success: bool) {
        if let Some(call) = self.calls.iter_mut().find(|c| c.id == id) {
            let now = Utc::now();
            call.completed_at = Some(now);
            call.duration_ms = Some(
                now.signed_duration_since(call.started_at)
                    .num_milliseconds()
                    .max(0) as u64,
            );
            call.output_size_bytes = output_size_bytes;
            call.success = success;
        }
    }

    /// Record a completed call in one step
    pub fn record_call(
        &mut self,
        call_type: ToolCallType,
        tool_name: &str,
        server_name: Option<&str>,
        duration_ms: u64,
        input_size_bytes: usize,
        output_size_bytes: usize,
        success: bool,
    ) {
        let estimated_cost = self.cost_rates.get(tool_name).copied().unwrap_or(0.0);
        let now = Utc::now();
        self.calls.push(TrackedToolCall {
            id: Uuid::new_v4(),
            call_type,
            tool_name: tool_name.to_string(),
            server_name: server_name.map(String::from),
            started_at: now,
            completed_at: Some(now),
            duration_ms: Some(duration_ms),
            input_size_bytes,
            output_size_bytes,
            success,
            estimated_cost_usd: estimated_cost,
            metadata: serde_json::json!({}),
        });
    }

    /// Get stats by tool type
    pub fn stats_by_type(&self) -> HashMap<ToolCallType, ToolUsageStats> {
        let mut map: HashMap<ToolCallType, ToolUsageStats> = HashMap::new();
        for call in &self.calls {
            let stats = map.entry(call.call_type.clone()).or_default();
            stats.total_calls += 1;
            if call.success {
                stats.successful_calls += 1;
            } else {
                stats.failed_calls += 1;
            }
            stats.total_cost_usd += call.estimated_cost_usd;
            stats.total_duration_ms += call.duration_ms.unwrap_or(0);
            stats.total_input_bytes += call.input_size_bytes;
            stats.total_output_bytes += call.output_size_bytes;
        }
        for stats in map.values_mut() {
            stats.avg_duration_ms = if stats.total_calls > 0 {
                stats.total_duration_ms as f64 / stats.total_calls as f64
            } else {
                0.0
            };
        }
        map
    }

    /// Get stats by tool name
    pub fn stats_by_name(&self) -> HashMap<String, ToolUsageStats> {
        let mut map: HashMap<String, ToolUsageStats> = HashMap::new();
        for call in &self.calls {
            let stats = map.entry(call.tool_name.clone()).or_default();
            stats.total_calls += 1;
            if call.success {
                stats.successful_calls += 1;
            } else {
                stats.failed_calls += 1;
            }
            stats.total_cost_usd += call.estimated_cost_usd;
            stats.total_duration_ms += call.duration_ms.unwrap_or(0);
            stats.total_input_bytes += call.input_size_bytes;
            stats.total_output_bytes += call.output_size_bytes;
        }
        for stats in map.values_mut() {
            stats.avg_duration_ms = if stats.total_calls > 0 {
                stats.total_duration_ms as f64 / stats.total_calls as f64
            } else {
                0.0
            };
        }
        map
    }

    /// Total cost across all tool calls
    pub fn total_cost(&self) -> f64 {
        self.calls.iter().map(|c| c.estimated_cost_usd).sum()
    }

    /// Total number of calls
    pub fn total_calls(&self) -> usize {
        self.calls.len()
    }

    /// Get all calls
    pub fn calls(&self) -> &[TrackedToolCall] {
        &self.calls
    }
}
