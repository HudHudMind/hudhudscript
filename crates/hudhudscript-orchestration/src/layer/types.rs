use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Layer ID
pub type LayerId = Uuid;

/// Agent ID
pub type AgentId = String;

/// Layer - A group of agents executed together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub agents: Vec<AgentId>,
    pub config: LayerConfig,
}

impl Layer {
    /// Create a new layer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            agents: Vec::new(),
            config: LayerConfig::default(),
        }
    }

    /// Add an agent to the layer
    pub fn add_agent(&mut self, agent_id: impl Into<String>) {
        self.agents.push(agent_id.into());
    }

    /// Add multiple agents
    pub fn add_agents(&mut self, agent_ids: impl IntoIterator<Item = impl Into<String>>) {
        self.agents.extend(agent_ids.into_iter().map(Into::into));
    }

    /// Set execution mode
    pub fn with_mode(mut self, mode: ExecutionMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set failure strategy
    pub fn with_failure_strategy(mut self, strategy: FailureStrategy) -> Self {
        self.config.failure_strategy = strategy;
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Set dependency layers
    pub fn with_dependencies(mut self, dependencies: Vec<LayerId>) -> Self {
        self.config.dependencies = dependencies;
        self
    }
}

/// Layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    /// Execution mode
    pub mode: ExecutionMode,
    /// Timeout in seconds
    pub timeout: u64,
    /// Failure strategy
    pub failure_strategy: FailureStrategy,
    /// Max retries for failed agents
    pub max_retries: u32,
    /// Dependency layer IDs
    pub dependencies: Vec<LayerId>,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Parallel,
            timeout: 300,
            failure_strategy: FailureStrategy::Stop,
            max_retries: 0,
            dependencies: Vec::new(),
        }
    }
}

/// Execution mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Execute agents in parallel
    Parallel,
    /// Execute agents sequentially
    Sequential,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::Parallel
    }
}

/// Failure strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureStrategy {
    /// Stop on first failure
    Stop,
    /// Continue despite failures
    Continue,
    /// Retry failed agents
    Retry,
}

impl Default for FailureStrategy {
    fn default() -> Self {
        FailureStrategy::Stop
    }
}

/// Layer input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInput {
    pub data: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Agent result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub agent_id: AgentId,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Layer output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerOutput {
    pub layer_id: LayerId,
    pub agent_results: Vec<AgentResult>,
    pub aggregated_output: serde_json::Value,
    pub duration_ms: u64,
}
