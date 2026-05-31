//! Agent entity and configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::agent::task::Task;

/// Agent identifier
pub type AgentId = String;

/// Task identifier
pub type TaskId = String;

/// Agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Agent unique identifier
    pub id: AgentId,

    /// Agent name
    pub name: String,

    /// Agent description
    pub description: Option<String>,

    /// Provider name (optional)
    pub provider: Option<String>,

    /// Tasks this agent can perform
    pub tasks: HashMap<TaskId, Task>,

    /// Tools this agent has access to
    pub tools: Vec<String>,

    /// Resources this agent has access to
    pub resources: Vec<String>,

    /// Agent configuration
    pub config: AgentConfig,

    /// Agent metadata
    pub metadata: AgentMetadata,
}

impl Agent {
    /// Create a new agent
    pub fn new(id: AgentId, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            provider: None,
            tasks: HashMap::new(),
            tools: Vec::new(),
            resources: Vec::new(),
            config: AgentConfig::default(),
            metadata: AgentMetadata::new(),
        }
    }

    /// Set provider for this agent
    pub fn set_provider(&mut self, provider: String) {
        self.provider = Some(provider);
    }

    /// Get provider name
    pub fn get_provider(&self) -> Option<&str> {
        self.provider.as_deref()
    }

    /// Add a task to the agent
    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id.clone(), task);
    }

    /// Add a tool to the agent
    pub fn add_tool(&mut self, tool: String) {
        if !self.tools.contains(&tool) {
            self.tools.push(tool);
        }
    }

    /// Add a resource to the agent
    pub fn add_resource(&mut self, resource: String) {
        if !self.resources.contains(&resource) {
            self.resources.push(resource);
        }
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,

    /// Task timeout in seconds
    pub task_timeout: u64,

    /// Enable state persistence
    pub persist_state: bool,

    /// Enable monitoring
    pub monitoring: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            task_timeout: 300, // 5 minutes
            persist_state: true,
            monitoring: true,
        }
    }
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// When was this agent created
    pub created_at: SystemTime,

    /// Last execution time
    pub last_executed: Option<SystemTime>,

    /// Total executions
    pub execution_count: u64,

    /// Total successful executions
    pub success_count: u64,

    /// Total failed executions
    pub failure_count: u64,

    /// Tags for categorization
    pub tags: Vec<String>,
}

impl Default for AgentMetadata {
    fn default() -> Self {
        Self {
            created_at: SystemTime::now(),
            last_executed: None,
            execution_count: 0,
            success_count: 0,
            failure_count: 0,
            tags: Vec::new(),
        }
    }
}

impl AgentMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Record execution
    pub fn record_execution(&mut self, success: bool) {
        self.execution_count += 1;
        self.last_executed = Some(SystemTime::now());

        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.execution_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.execution_count as f64
        }
    }
}
