//! Agent Executor trait — abstraction for dispatching work to agents.
//!
//! The orchestration modules (council, layer, swarm) use this trait to call
//! into the actual agent runtime. The interpreter (or any other host) provides
//! an implementation via `Arc<dyn AgentExecutor>`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A unit of work to be executed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// The input data / prompt for the agent.
    pub data: serde_json::Value,
    /// Optional metadata carried alongside the task.
    pub metadata: std::collections::HashMap<String, String>,
}

/// Result returned by an agent after executing a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTaskResult {
    /// Whether the agent completed successfully.
    pub success: bool,
    /// The output produced by the agent.
    pub output: serde_json::Value,
    /// An optional confidence score (used by swarm consensus).
    pub confidence: f64,
    /// An optional vote (used by council voting).
    pub vote: Option<bool>,
    /// Error message, if the agent failed.
    pub error: Option<String>,
}

/// Trait that the host (interpreter / runtime) implements to let the
/// orchestration engine dispatch work to individual agents.
#[async_trait]
pub trait AgentExecutor: Send + Sync {
    /// Execute a task on the given agent and return the result.
    async fn execute(&self, agent_id: &str, task: AgentTask) -> AgentTaskResult;
}

/// Default executor that reports failure when no real runtime is configured.
///
/// Returns an honest error instead of simulated success. Callers must plug in a
/// real AgentExecutor implementation (backed by the interpreter or VM) for
/// actual task execution.
pub struct DefaultAgentExecutor;

#[async_trait]
impl AgentExecutor for DefaultAgentExecutor {
    async fn execute(&self, agent_id: &str, task: AgentTask) -> AgentTaskResult {
        AgentTaskResult {
            success: false,
            output: serde_json::json!({
                "error": "No agent runtime configured",
                "agent": agent_id,
                "task_data": task.data,
                "hint": "Plug in a real AgentExecutor implementation backed by the interpreter or VM"
            }),
            confidence: 0.0,
            vote: None,
            error: Some(format!(
                "DefaultAgentExecutor: no runtime configured for agent '{}'",
                agent_id
            )),
        }
    }
}

/// Helper to build a default executor wrapped in an `Arc`.
pub fn default_agent_executor() -> Arc<dyn AgentExecutor> {
    Arc::new(DefaultAgentExecutor)
}
