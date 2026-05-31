//! Swarm Execution — swarm coordination (Issue #15)
//!
//! Starts all agents in the swarm in parallel, applies consensus mechanism,
//! shares swarm-level state, and provides fault tolerance.

use crate::agent_executor::{default_agent_executor, AgentExecutor, AgentTask};
use crate::events::{AgentEvent, EventBus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Swarm consensus strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum ConsensusStrategy {
    /// Pick the majority result
    #[default]
    Majority,
    /// Pick the highest confidence result
    HighestConfidence,
    /// Aggregate all results
    Aggregate,
    /// Take the first successful result
    FirstSuccess,
}

/// Swarm agent result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmAgentResult {
    pub agent_id: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub confidence: f64,
    pub error: Option<String>,
}

/// Swarm execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmResult {
    pub swarm_id: String,
    pub agent_results: Vec<SwarmAgentResult>,
    pub consensus_output: serde_json::Value,
    pub success: bool,
    pub failed_agents: Vec<String>,
}

/// Swarm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub consensus: ConsensusStrategy,
    pub timeout_secs: u64,
    /// Minimum successful agent count (0 = at least 1)
    pub min_success: usize,
    /// Fault tolerance: continue if an agent fails
    pub fault_tolerant: bool,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            consensus: ConsensusStrategy::Majority,
            timeout_secs: 60,
            min_success: 1,
            fault_tolerant: true,
        }
    }
}

/// Swarm-level shared state
#[derive(Debug, Clone, Default)]
pub struct SwarmState {
    pub variables: HashMap<String, serde_json::Value>,
}

impl SwarmState {
    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.variables.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }
}

/// Swarm executor
pub struct SwarmExecutor {
    event_bus: Arc<EventBus>,
    configs: Arc<RwLock<HashMap<String, SwarmConfig>>>,
    /// Swarm-level shared state
    states: Arc<RwLock<HashMap<String, SwarmState>>>,
    /// Agent executor used to dispatch work to individual agents
    agent_executor: Arc<dyn AgentExecutor>,
}

impl SwarmExecutor {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            configs: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            agent_executor: default_agent_executor(),
        }
    }

    /// Create a swarm executor with a custom agent executor.
    pub fn with_agent_executor(
        event_bus: Arc<EventBus>,
        agent_executor: Arc<dyn AgentExecutor>,
    ) -> Self {
        Self {
            event_bus,
            configs: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            agent_executor,
        }
    }

    /// Register a swarm
    pub async fn register(&self, swarm_id: String, config: SwarmConfig) {
        self.configs.write().await.insert(swarm_id.clone(), config);
        self.states
            .write()
            .await
            .insert(swarm_id, SwarmState::default());
    }

    /// Write a value to swarm state
    pub async fn set_state(
        &self,
        swarm_id: &str,
        key: impl Into<String>,
        value: serde_json::Value,
    ) {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(swarm_id) {
            state.set(key, value);
        }
    }

    /// Read a value from swarm state
    pub async fn get_state(&self, swarm_id: &str, key: &str) -> Option<serde_json::Value> {
        let states = self.states.read().await;
        states.get(swarm_id)?.get(key).cloned()
    }

    /// Execute the swarm
    pub async fn execute(
        &self,
        swarm_id: &str,
        agent_ids: Vec<String>,
        task: serde_json::Value,
    ) -> Result<SwarmResult, SwarmError> {
        if agent_ids.is_empty() {
            return Err(SwarmError::NoAgents);
        }

        let config = {
            let configs = self.configs.read().await;
            configs.get(swarm_id).cloned().unwrap_or_default()
        };

        let timeout_duration = std::time::Duration::from_secs(config.timeout_secs);

        // Start all agents in parallel via the agent executor
        let handles: Vec<_> = agent_ids
            .iter()
            .map(|id| {
                let agent_id = id.clone();
                let task_data = task.clone();
                let exec = self.agent_executor.clone();
                tokio::spawn(async move {
                    let agent_task = AgentTask {
                        data: task_data,
                        metadata: HashMap::new(),
                    };
                    let r = exec.execute(&agent_id, agent_task).await;
                    SwarmAgentResult {
                        agent_id,
                        success: r.success,
                        output: r.output,
                        confidence: r.confidence,
                        error: r.error,
                    }
                })
            })
            .collect();

        let join_all = futures::future::join_all(handles);
        let raw_results = tokio::time::timeout(timeout_duration, join_all)
            .await
            .map_err(|_| SwarmError::Timeout)?;

        // Separate successful and failed results
        let mut agent_results: Vec<SwarmAgentResult> = Vec::new();
        let mut failed_agents: Vec<String> = Vec::new();

        for (i, res) in raw_results.into_iter().enumerate() {
            match res {
                Ok(r) => {
                    if r.success {
                        agent_results.push(r);
                    } else {
                        let agent_id = agent_ids[i].clone();
                        failed_agents.push(agent_id.clone());
                        if !config.fault_tolerant {
                            return Err(SwarmError::AgentFailed(agent_id));
                        }
                        agent_results.push(r);
                    }
                }
                Err(e) => {
                    let agent_id = agent_ids[i].clone();
                    failed_agents.push(agent_id.clone());
                    if !config.fault_tolerant {
                        return Err(SwarmError::AgentFailed(format!("{}: {}", agent_id, e)));
                    }
                }
            }
        }

        // Minimum success check
        let success_count = agent_results.iter().filter(|r| r.success).count();
        let min = if config.min_success == 0 {
            1
        } else {
            config.min_success
        };
        if success_count < min {
            return Err(SwarmError::InsufficientSuccess {
                required: min,
                got: success_count,
            });
        }

        // Apply consensus
        let consensus_output = Self::apply_consensus(&agent_results, &config.consensus);

        // Update swarm state
        {
            let mut states = self.states.write().await;
            if let Some(state) = states.get_mut(swarm_id) {
                state.set("last_result", consensus_output.clone());
                state.set("success_count", serde_json::json!(success_count));
                state.set("failed_count", serde_json::json!(failed_agents.len()));
            }
        }

        // Emit SwarmConsensus event
        let _ = self
            .event_bus
            .emit(AgentEvent::SwarmConsensus {
                swarm_id: swarm_id.to_string(),
                result: consensus_output.clone(),
            })
            .await;

        Ok(SwarmResult {
            swarm_id: swarm_id.to_string(),
            agent_results,
            consensus_output,
            success: true,
            failed_agents,
        })
    }

    fn apply_consensus(
        results: &[SwarmAgentResult],
        strategy: &ConsensusStrategy,
    ) -> serde_json::Value {
        let successful: Vec<_> = results.iter().filter(|r| r.success).collect();
        if successful.is_empty() {
            return serde_json::Value::Null;
        }

        match strategy {
            ConsensusStrategy::Majority => {
                // Pick the most frequently occurring output (simple: string comparison)
                let mut counts: HashMap<String, usize> = HashMap::new();
                for r in &successful {
                    let key = r.output.to_string();
                    *counts.entry(key).or_insert(0) += 1;
                }
                let best = counts.into_iter().max_by_key(|(_, c)| *c).map(|(k, _)| k);
                best.and_then(|k| serde_json::from_str(&k).ok())
                    .unwrap_or_else(|| successful[0].output.clone())
            }
            ConsensusStrategy::HighestConfidence => successful
                .iter()
                .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                .map(|r| r.output.clone())
                .unwrap_or(serde_json::Value::Null),
            ConsensusStrategy::Aggregate => {
                serde_json::Value::Array(successful.iter().map(|r| r.output.clone()).collect())
            }
            ConsensusStrategy::FirstSuccess => successful[0].output.clone(),
        }
    }
}

/// Swarm errors for the orchestration layer.
///
/// Note: there is a separate `SwarmError` in `hudhudscript-governance` that
/// covers governance-domain swarm errors (agent registration, state lookup).
/// This type covers orchestration-time errors (no agents, insufficient success).
/// They are intentionally separate — see Issue #825 / #849 for the rationale.
#[derive(Debug)]
pub enum SwarmError {
    NoAgents,
    AgentFailed(String),
    InsufficientSuccess { required: usize, got: usize },
    Timeout,
}

impl std::fmt::Display for SwarmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            SwarmError::NoAgents => write!(f, "No agents in swarm"),
            SwarmError::AgentFailed(s) => write!(f, "Agent failed: {}", s),
            SwarmError::InsufficientSuccess { required, got } => write!(
                f,
                "Insufficient success: required {}, got {}",
                required, got
            ),
            SwarmError::Timeout => write!(f, "Timeout"),
        }
    }
}

impl std::error::Error for SwarmError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl SwarmError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            SwarmError::AgentFailed(..) => hudhudscript_errors::ErrorCode::SwarmAgentFailed,
            SwarmError::InsufficientSuccess { .. } => {
                hudhudscript_errors::ErrorCode::SwarmInsufficientSuccess
            }
            SwarmError::NoAgents => hudhudscript_errors::ErrorCode::SwarmNoAgents,
            SwarmError::Timeout => hudhudscript_errors::ErrorCode::SwarmTimeout,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<SwarmError> for hudhudscript_errors::Error {
    fn from(e: SwarmError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
