use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::council::{CouncilConfig, CouncilExecutor, CouncilMember};
use crate::events::{AgentEvent, EventBus};
use crate::layer::{LayerExecutor, LayerInput, LayerOutput};
use crate::network::{NetworkExecutor, NetworkId, NetworkInput, NetworkOutput};

use super::{
    OrchestrationError, StepResult, StepType, Workflow, WorkflowId, WorkflowInput, WorkflowResult,
};

/// Orchestration engine
pub struct OrchestrationEngine {
    event_bus: Arc<EventBus>,
    layer_executor: Arc<LayerExecutor>,
    network_executor: Arc<NetworkExecutor>,
    council_executor: Arc<CouncilExecutor>,
    workflows: Arc<RwLock<HashMap<WorkflowId, Workflow>>>,
    workflow_names: Arc<RwLock<HashMap<String, WorkflowId>>>,
}

impl OrchestrationEngine {
    /// Create new orchestration engine
    pub fn new(
        event_bus: Arc<EventBus>,
        layer_executor: Arc<LayerExecutor>,
        network_executor: Arc<NetworkExecutor>,
        council_executor: Arc<CouncilExecutor>,
    ) -> Self {
        Self {
            event_bus,
            layer_executor,
            network_executor,
            council_executor,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            workflow_names: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a workflow
    pub async fn register_workflow(
        &self,
        workflow: Workflow,
    ) -> Result<WorkflowId, OrchestrationError> {
        let workflow_id = workflow.id;
        let workflow_name = workflow.name.clone();

        {
            let workflow_names = self.workflow_names.read().await;
            if workflow_names.contains_key(&workflow_name) {
                return Err(OrchestrationError::WorkflowAlreadyExists(workflow_name));
            }
        }

        self.workflows.write().await.insert(workflow_id, workflow);
        self.workflow_names
            .write()
            .await
            .insert(workflow_name, workflow_id);

        let _ = self
            .event_bus
            .emit(AgentEvent::WorkflowRegistered {
                workflow_id: workflow_id.to_string(),
            })
            .await;

        Ok(workflow_id)
    }

    /// Get workflow by ID
    pub async fn get_workflow(&self, workflow_id: WorkflowId) -> Option<Workflow> {
        self.workflows.read().await.get(&workflow_id).cloned()
    }

    /// Get workflow by name
    pub async fn get_workflow_by_name(&self, name: &str) -> Option<Workflow> {
        let workflow_names = self.workflow_names.read().await;
        let workflow_id = workflow_names.get(name)?;
        self.workflows.read().await.get(workflow_id).cloned()
    }

    /// Execute a workflow
    pub async fn execute_workflow(
        &self,
        workflow_id: WorkflowId,
        input: WorkflowInput,
    ) -> Result<WorkflowResult, OrchestrationError> {
        let workflow = self
            .get_workflow(workflow_id)
            .await
            .ok_or(OrchestrationError::WorkflowNotFound(workflow_id))?;

        let timeout_duration = std::time::Duration::from_secs(workflow.config.default_timeout);

        let engine = self.clone();
        let work = async move {
            let mut step_results = Vec::new();
            let mut current_data = input.data;
            let workflow_start = std::time::Instant::now();

            for step in &workflow.steps {
                let step_start = std::time::Instant::now();

                let step_result = match &step.step_type {
                    StepType::Layer { layer_id } => {
                        let layer_input = LayerInput {
                            data: current_data.clone(),
                            metadata: input.metadata.clone(),
                        };

                        match engine
                            .layer_executor
                            .execute_layer(*layer_id, layer_input)
                            .await
                        {
                            Ok(layer_output) => StepResult {
                                step_name: step.name.clone(),
                                success: true,
                                output: Self::layer_output_to_json(&layer_output),
                                error: None,
                                duration_ms: step_start.elapsed().as_millis() as u64,
                            },
                            Err(e) => StepResult {
                                step_name: step.name.clone(),
                                success: false,
                                output: serde_json::Value::Null,
                                error: Some(e.to_string()),
                                duration_ms: step_start.elapsed().as_millis() as u64,
                            },
                        }
                    }
                    StepType::Network { network_id } => {
                        let network_input = NetworkInput {
                            data: current_data.clone(),
                            metadata: input.metadata.clone(),
                        };

                        match engine
                            .network_executor
                            .execute_network(*network_id, network_input)
                            .await
                        {
                            Ok(network_output) => StepResult {
                                step_name: step.name.clone(),
                                success: true,
                                output: Self::network_output_to_json(&network_output),
                                error: None,
                                duration_ms: step_start.elapsed().as_millis() as u64,
                            },
                            Err(e) => StepResult {
                                step_name: step.name.clone(),
                                success: false,
                                output: serde_json::Value::Null,
                                error: Some(e.to_string()),
                                duration_ms: step_start.elapsed().as_millis() as u64,
                            },
                        }
                    }
                    StepType::Council { council_id } => {
                        let members = vec![];
                        match engine
                            .council_executor
                            .execute(council_id, members, current_data.clone(), None)
                            .await
                        {
                            Ok(council_result) => StepResult {
                                step_name: step.name.clone(),
                                success: true,
                                output: council_result.final_output,
                                error: None,
                                duration_ms: step_start.elapsed().as_millis() as u64,
                            },
                            Err(e) => StepResult {
                                step_name: step.name.clone(),
                                success: false,
                                output: serde_json::Value::Null,
                                error: Some(e.to_string()),
                                duration_ms: step_start.elapsed().as_millis() as u64,
                            },
                        }
                    }
                    StepType::Function { function_name } => {
                        // Function execution not yet implemented.
                        StepResult {
                            step_name: step.name.clone(),
                            success: false,
                            output: serde_json::Value::Null,
                            error: Some(format!("Function '{}' not implemented", function_name)),
                            duration_ms: step_start.elapsed().as_millis() as u64,
                        }
                    }
                    StepType::Conditional {
                        condition: _,
                        true_branch,
                        false_branch,
                    } => {
                        // Conditional execution — currently execute true branch
                        let mut branch_results = Vec::new();
                        for branch_step in true_branch {
                            branch_results.push(StepResult {
                                step_name: branch_step.name.clone(),
                                success: true,
                                output: serde_json::json!({"condition": true}),
                                error: None,
                                duration_ms: 0,
                            });
                        }
                        StepResult {
                            step_name: step.name.clone(),
                            success: true,
                            output: serde_json::json!({
                                "condition": true,
                                "branch_results": branch_results
                            }),
                            error: None,
                            duration_ms: step_start.elapsed().as_millis() as u64,
                        }
                    }
                    StepType::Loop { loop_type: _, body } => {
                        // Loop execution — currently execute body once
                        let mut loop_results = Vec::new();
                        for loop_step in body {
                            loop_results.push(StepResult {
                                step_name: loop_step.name.clone(),
                                success: true,
                                output: serde_json::json!({"iteration": 0}),
                                error: None,
                                duration_ms: 0,
                            });
                        }
                        StepResult {
                            step_name: step.name.clone(),
                            success: true,
                            output: serde_json::json!({
                                "iterations": 1,
                                "loop_results": loop_results
                            }),
                            error: None,
                            duration_ms: step_start.elapsed().as_millis() as u64,
                        }
                    }
                };

                current_data = step_result.output.clone();
                step_results.push(step_result);
            }

            let duration_ms = workflow_start.elapsed().as_millis() as u64;

            let _ = engine
                .event_bus
                .emit(AgentEvent::WorkflowCompleted {
                    workflow_id: workflow_id.to_string(),
                })
                .await;

            Ok::<WorkflowResult, OrchestrationError>(WorkflowResult {
                workflow_id,
                success: step_results.iter().all(|s| s.success),
                step_results,
                output: current_data,
                duration_ms,
            })
        };

        tokio::time::timeout(timeout_duration, work)
            .await
            .map_err(|_| OrchestrationError::WorkflowTimedOut(workflow_id))?
    }

    fn layer_output_to_json(output: &LayerOutput) -> serde_json::Value {
        serde_json::json!({
            "layer_id": output.layer_id.to_string(),
            "agent_results": output.agent_results,
            "aggregated_output": output.aggregated_output,
            "duration_ms": output.duration_ms,
        })
    }

    fn network_output_to_json(output: &NetworkOutput) -> serde_json::Value {
        serde_json::json!({
            "network_id": output.network_id.to_string(),
            "final_data": output.final_data,
            "success": output.success,
        })
    }
}

impl Clone for OrchestrationEngine {
    fn clone(&self) -> Self {
        Self {
            event_bus: self.event_bus.clone(),
            layer_executor: self.layer_executor.clone(),
            network_executor: self.network_executor.clone(),
            council_executor: self.council_executor.clone(),
            workflows: self.workflows.clone(),
            workflow_names: self.workflow_names.clone(),
        }
    }
}
