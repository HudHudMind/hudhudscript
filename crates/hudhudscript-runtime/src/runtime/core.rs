//! Agent Runtime core implementation.

use crate::agent::{Agent, AgentId, AgentState, ExecutionRecord, ExecutionStatus, StateValue};
use crate::provider::ProviderRegistry;
use crate::runtime::config::{RuntimeConfig, RuntimeStatistics};
use crate::runtime::error::RuntimeError;
use crate::runtime::executor::execute_ast_task;
use hudhudscript_mcp::client::McpClient;
use hudhudscript_resources::ResourceManager;
use hudhudscript_tools::ToolRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent Runtime for managing agent lifecycle and execution
pub struct AgentRuntime {
    /// Registered agents
    agents: Arc<RwLock<HashMap<AgentId, Agent>>>,

    /// Agent states (isolated per agent)
    states: Arc<RwLock<HashMap<AgentId, AgentState>>>,

    /// Execution history
    executions: Arc<RwLock<Vec<ExecutionRecord>>>,

    /// Tool registry — stored for future task-level tool dispatch.
    _tool_registry: Arc<ToolRegistry>,

    /// Resource manager — stored for future resource-aware execution.
    _resource_manager: Arc<ResourceManager>,

    /// MCP client — stored for future MCP-integrated task execution.
    _mcp_client: Arc<McpClient>,

    /// Provider registry
    provider_registry: Arc<ProviderRegistry>,

    /// Runtime configuration — stored for future concurrency/history limits.
    _config: RuntimeConfig,
}

impl AgentRuntime {
    /// Create new agent runtime
    pub fn new(
        tool_registry: Arc<ToolRegistry>,
        resource_manager: Arc<ResourceManager>,
        mcp_client: Arc<McpClient>,
    ) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(Vec::new())),
            _tool_registry: tool_registry,
            _resource_manager: resource_manager,
            _mcp_client: mcp_client,
            provider_registry: Arc::new(ProviderRegistry::new()),
            _config: RuntimeConfig::default(),
        }
    }

    /// Get provider registry
    pub fn provider_registry(&self) -> Arc<ProviderRegistry> {
        self.provider_registry.clone()
    }

    /// Register an agent
    pub async fn register_agent(&self, agent: Agent) -> Result<(), RuntimeError> {
        let agent_id = agent.id.clone();

        // Register agent
        {
            let mut agents = self.agents.write().await;
            if agents.contains_key(&agent_id) {
                return Err(RuntimeError::AgentAlreadyExists(agent_id));
            }
            agents.insert(agent_id.clone(), agent);
        }

        // Initialize agent state
        {
            let mut states = self.states.write().await;
            states.insert(agent_id.clone(), AgentState::new(agent_id));
        }

        Ok(())
    }

    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<(), RuntimeError> {
        {
            let mut agents = self.agents.write().await;
            agents
                .remove(agent_id)
                .ok_or_else(|| RuntimeError::AgentNotFound(agent_id.to_string()))?;
        }

        {
            let mut states = self.states.write().await;
            states.remove(agent_id);
        }

        Ok(())
    }

    /// Get agent by ID
    pub async fn get_agent(&self, agent_id: &str) -> Option<Agent> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// List all agents
    pub async fn list_agents(&self) -> Vec<Agent> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Get agent state
    pub async fn get_state(&self, agent_id: &str) -> Option<AgentState> {
        let states = self.states.read().await;
        states.get(agent_id).cloned()
    }

    /// Update agent state
    pub async fn update_state(
        &self,
        agent_id: &str,
        updater: impl FnOnce(&mut AgentState),
    ) -> Result<(), RuntimeError> {
        let mut states = self.states.write().await;
        let state = states
            .get_mut(agent_id)
            .ok_or_else(|| RuntimeError::AgentNotFound(agent_id.to_string()))?;

        updater(state);
        Ok(())
    }

    /// Execute a task
    pub async fn execute_task(
        &self,
        agent_id: &str,
        task_id: &str,
        input: HashMap<String, StateValue>,
    ) -> Result<StateValue, RuntimeError> {
        // Get agent
        let agent = self
            .get_agent(agent_id)
            .await
            .ok_or_else(|| RuntimeError::AgentNotFound(agent_id.to_string()))?;

        // Get task
        let task = agent
            .get_task(task_id)
            .ok_or_else(|| RuntimeError::TaskNotFound(task_id.to_string()))?
            .clone();

        // Create execution record
        let mut record =
            ExecutionRecord::new(agent_id.to_string(), task_id.to_string(), input.clone());

        // Execute the task via the AST interpreter or bytecode VM path
        let result = self.execute_task_impl(&agent, &task, input).await;

        // Update record
        match result {
            Ok(output) => {
                record.complete(output.clone());

                // Update agent metadata
                {
                    let mut agents = self.agents.write().await;
                    if let Some(agent) = agents.get_mut(agent_id) {
                        agent.metadata.record_execution(true);
                    }
                }

                // Store execution record
                {
                    let mut executions = self.executions.write().await;
                    executions.push(record);
                }

                Ok(output)
            }
            Err(e) => {
                record.fail(e.to_string());

                // Update agent metadata
                {
                    let mut agents = self.agents.write().await;
                    if let Some(agent) = agents.get_mut(agent_id) {
                        agent.metadata.record_execution(false);
                    }
                }

                // Store execution record
                {
                    let mut executions = self.executions.write().await;
                    executions.push(record);
                }

                Err(e)
            }
        }
    }

    /// Execute task implementation — dispatches to AST interpreter or native handler
    async fn execute_task_impl(
        &self,
        agent: &Agent,
        task: &crate::agent::Task,
        input: HashMap<String, StateValue>,
    ) -> Result<StateValue, RuntimeError> {
        // Validate required parameters
        for param in &task.parameters {
            if !param.optional && param.default.is_none() && !input.contains_key(&param.name) {
                return Err(RuntimeError::ExecutionFailed(format!(
                    "Missing required parameter: {}",
                    param.name
                )));
            }
        }

        match &task.implementation {
            crate::agent::TaskImplementation::Native => {
                // Native tasks return a simple success value
                Ok(StateValue::String(format!("Task '{}' executed", task.name)))
            }

            crate::agent::TaskImplementation::Ast(ast_source) => {
                // Execute AST source via the interpreter
                // Run in a blocking thread to avoid blocking the async runtime
                let source = ast_source.clone();
                let agent_id = agent.id.clone();
                let task_name = task.name.clone();
                let input_clone = input.clone();

                let result = tokio::task::spawn_blocking(move || {
                    execute_ast_task(&source, &agent_id, &task_name, input_clone)
                })
                .await
                .map_err(|e| RuntimeError::ExecutionFailed(format!("Task panicked: {}", e)))?;

                result
            }

            crate::agent::TaskImplementation::Bytecode(_bytes) => {
                // Cannot directly depend on hudhudscript-vm due to cyclic dependency:
                // debug → vm → runtime → debug. The VM must be invoked from the
                // CLI layer (which sits above both runtime and VM) or via a trait
                // object injected at construction time.
                Err(RuntimeError::ExecutionFailed(
                    "Bytecode execution requires VM injection — use AgentRuntime::with_vm_executor() \
                     or execute via the CLI (hudhud run script.hudb). Direct runtime→VM dependency \
                     is blocked by cyclic crate dependency (debug→vm→runtime→debug)."
                        .to_string(),
                ))
            }
        }
    }

    /// Get execution history for an agent
    pub async fn get_execution_history(&self, agent_id: &str) -> Vec<ExecutionRecord> {
        let executions = self.executions.read().await;
        executions
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Get all execution records
    pub async fn get_all_executions(&self) -> Vec<ExecutionRecord> {
        let executions = self.executions.read().await;
        executions.clone()
    }

    /// Clear execution history
    pub async fn clear_execution_history(&self) {
        let mut executions = self.executions.write().await;
        executions.clear();
    }

    /// Get runtime statistics
    pub async fn get_statistics(&self) -> RuntimeStatistics {
        let agents = self.agents.read().await;
        let executions = self.executions.read().await;

        let total_agents = agents.len();
        let total_executions = executions.len();
        let successful_executions = executions
            .iter()
            .filter(|e| e.status == ExecutionStatus::Completed)
            .count();
        let failed_executions = executions
            .iter()
            .filter(|e| e.status == ExecutionStatus::Failed)
            .count();

        RuntimeStatistics {
            total_agents,
            total_executions,
            successful_executions,
            failed_executions,
            success_rate: if total_executions > 0 {
                successful_executions as f64 / total_executions as f64
            } else {
                0.0
            },
        }
    }
}
