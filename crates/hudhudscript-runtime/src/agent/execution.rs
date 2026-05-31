//! Execution records and status.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::agent::entity::{AgentId, TaskId};
use crate::agent::state::StateValue;

/// Execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Execution ID
    pub id: String,

    /// Agent ID
    pub agent_id: AgentId,

    /// Task ID
    pub task_id: TaskId,

    /// Input parameters
    pub input: HashMap<String, StateValue>,

    /// Output result
    pub output: Option<StateValue>,

    /// Execution status
    pub status: ExecutionStatus,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Start time
    pub started_at: SystemTime,

    /// End time
    pub ended_at: Option<SystemTime>,

    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
}

impl ExecutionRecord {
    /// Create new execution record
    pub fn new(agent_id: AgentId, task_id: TaskId, input: HashMap<String, StateValue>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id,
            task_id,
            input,
            output: None,
            status: ExecutionStatus::Running,
            error: None,
            started_at: SystemTime::now(),
            ended_at: None,
            duration_ms: None,
        }
    }

    /// Mark as completed
    pub fn complete(&mut self, output: StateValue) {
        self.output = Some(output);
        self.status = ExecutionStatus::Completed;
        self.ended_at = Some(SystemTime::now());
        self.calculate_duration();
    }

    /// Mark as failed
    pub fn fail(&mut self, error: String) {
        self.error = Some(error);
        self.status = ExecutionStatus::Failed;
        self.ended_at = Some(SystemTime::now());
        self.calculate_duration();
    }

    /// Calculate duration
    fn calculate_duration(&mut self) {
        if let Some(ended) = self.ended_at {
            if let Ok(duration) = ended.duration_since(self.started_at) {
                self.duration_ms = Some(duration.as_millis() as u64);
            }
        }
    }
}

/// Execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Task is running
    Running,

    /// Task completed successfully
    Completed,

    /// Task failed
    Failed,

    /// Task was cancelled
    Cancelled,
}
