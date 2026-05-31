//! Batch API dispatch — queue non-interactive requests for 50% cost reduction

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Dispatch mode for a request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DispatchMode {
    /// Real-time streaming, full price
    Stream,
    /// Concurrent async, full price
    Async,
    /// Queued batch, 50% discount
    Batch,
}

/// A pending batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    pub id: Uuid,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
    pub enqueued_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Batch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub request_id: Uuid,
    pub content: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub cost_usd: f64,
    pub completed_at: DateTime<Utc>,
}

/// Status of a submitted batch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    Queued,
    Submitted,
    Processing,
    Completed,
    Failed(String),
}

/// A batch of requests
#[derive(Debug, Clone)]
pub struct Batch {
    pub id: Uuid,
    pub requests: Vec<BatchRequest>,
    pub status: BatchStatus,
    pub created_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub results: Vec<BatchResult>,
}

/// Batch queue manager
pub struct BatchQueue {
    queue: VecDeque<BatchRequest>,
    pub batches: Vec<Batch>,
    max_batch_size: usize,
    flush_interval_seconds: u64,
    auto_promote: bool,
    pub last_flush: DateTime<Utc>,
}

impl BatchQueue {
    pub fn new(max_batch_size: usize, flush_interval_seconds: u64, auto_promote: bool) -> Self {
        Self {
            queue: VecDeque::new(),
            batches: Vec::new(),
            max_batch_size,
            flush_interval_seconds,
            auto_promote,
            last_flush: Utc::now(),
        }
    }

    /// Decide dispatch mode for a request
    pub fn classify_dispatch(
        &self,
        has_stream_callback: bool,
        is_interactive: bool,
        cost_optimize: bool,
    ) -> DispatchMode {
        if has_stream_callback || is_interactive {
            return DispatchMode::Stream;
        }
        if self.auto_promote || cost_optimize {
            return DispatchMode::Batch;
        }
        DispatchMode::Async
    }

    /// Enqueue a request for batch processing
    pub fn enqueue(&mut self, request: BatchRequest) {
        self.queue.push_back(request);
    }

    /// Check if the queue should be flushed
    pub fn should_flush(&self) -> bool {
        if self.queue.len() >= self.max_batch_size {
            return true;
        }
        let elapsed = Utc::now().signed_duration_since(self.last_flush);
        if !self.queue.is_empty() && elapsed.num_seconds() as u64 >= self.flush_interval_seconds {
            return true;
        }
        false
    }

    /// Flush the queue into a batch
    pub fn flush(&mut self) -> Option<Batch> {
        if self.queue.is_empty() {
            return None;
        }

        let count = self.queue.len().min(self.max_batch_size);
        let requests: Vec<BatchRequest> = self.queue.drain(..count).collect();
        let batch = Batch {
            id: Uuid::new_v4(),
            requests,
            status: BatchStatus::Queued,
            created_at: Utc::now(),
            submitted_at: None,
            completed_at: None,
            results: Vec::new(),
        };
        self.batches.push(batch.clone());
        self.last_flush = Utc::now();
        Some(batch)
    }

    /// Get queue length
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Get batch count
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Get pending batches
    pub fn pending_batches(&self) -> Vec<&Batch> {
        self.batches
            .iter()
            .filter(|b| {
                b.status == BatchStatus::Queued
                    || b.status == BatchStatus::Submitted
                    || b.status == BatchStatus::Processing
            })
            .collect()
    }

    /// Calculate savings from batching
    pub fn total_savings(&self, pricing_fn: impl Fn(&BatchResult) -> f64) -> f64 {
        self.batches
            .iter()
            .filter(|b| b.status == BatchStatus::Completed)
            .flat_map(|b| b.results.iter())
            .map(|r| pricing_fn(r) * 0.50) // 50% discount
            .sum()
    }
}
