use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{AgentResult, Layer, LayerError, LayerId, LayerInput, LayerOutput};

/// Layer executor
pub struct LayerExecutor {
    /// Layers indexed by ID
    layers: Arc<RwLock<HashMap<LayerId, Layer>>>,
}

impl Default for LayerExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerExecutor {
    /// Create a new layer executor
    pub fn new() -> Self {
        Self {
            layers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a layer
    pub async fn register_layer(&self, layer: Layer) -> Result<LayerId, LayerError> {
        let layer_id = layer.id;

        if self.get_layer(layer_id).await.is_some() {
            return Err(LayerError::LayerAlreadyExists(layer_id));
        }

        self.validate_dependencies(&layer).await?;

        self.layers.write().await.insert(layer_id, layer);

        Ok(layer_id)
    }

    /// Get a layer by ID
    pub async fn get_layer(&self, layer_id: LayerId) -> Option<Layer> {
        self.layers.read().await.get(&layer_id).cloned()
    }

    /// Execute a layer
    pub async fn execute_layer(
        &self,
        layer_id: LayerId,
        input: LayerInput,
    ) -> Result<LayerOutput, LayerError> {
        let layer = self
            .get_layer(layer_id)
            .await
            .ok_or(LayerError::LayerNotFound(layer_id))?;

        self.validate_dependencies(&layer).await?;

        let timeout_duration = std::time::Duration::from_secs(layer.config.timeout);
        let start = std::time::Instant::now();

        let exec_result = async {
            let agent_results = match layer.config.mode {
                super::ExecutionMode::Parallel => self.execute_parallel(&layer, input).await,
                super::ExecutionMode::Sequential => self.execute_sequential(&layer, input).await,
            };

            let aggregated = Self::aggregate_results(&agent_results);
            let duration_ms = start.elapsed().as_millis() as u64;

            Ok::<LayerOutput, LayerError>(LayerOutput {
                layer_id,
                agent_results,
                aggregated_output: aggregated,
                duration_ms,
            })
        };

        tokio::time::timeout(timeout_duration, exec_result)
            .await
            .map_err(|_| LayerError::TimeoutExceeded(layer_id))?
    }

    /// Execute agents in parallel
    async fn execute_parallel(&self, layer: &Layer, input: LayerInput) -> Vec<AgentResult> {
        let handles: Vec<_> = layer
            .agents
            .iter()
            .map(|agent_id| {
                let agent_id = agent_id.clone();
                let input_data = input.clone();
                tokio::spawn(async move {
                    let start = std::time::Instant::now();
                    let result = Self::execute_agent(&agent_id, &input_data.data).await;
                    let (output, error) = match result {
                        Ok(v) => (Some(v), None),
                        Err(e) => (None, Some(e.to_string())),
                    };
                    AgentResult {
                        agent_id,
                        success: error.is_none(),
                        output,
                        error,
                        duration_ms: start.elapsed().as_millis() as u64,
                    }
                })
            })
            .collect();

        futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect()
    }

    /// Execute agents sequentially
    async fn execute_sequential(&self, layer: &Layer, input: LayerInput) -> Vec<AgentResult> {
        let mut results = Vec::new();

        for agent_id in &layer.agents {
            let start = std::time::Instant::now();
            let result = Self::execute_agent(agent_id, &input.data).await;
            let (output, error) = match result {
                Ok(v) => (Some(v), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let success = error.is_none();

            results.push(AgentResult {
                agent_id: agent_id.clone(),
                success,
                output,
                error,
                duration_ms: start.elapsed().as_millis() as u64,
            });

            // Handle failure strategy
            if !success && layer.config.failure_strategy == super::FailureStrategy::Stop {
                break;
            }
        }

        results
    }

    /// Execute a single agent
    async fn execute_agent(
        _agent_id: &str,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, LayerError> {
        // In a real implementation, this would dispatch to an agent executor
        // Currently simulate success
        Ok(data.clone())
    }

    /// Validate layer dependencies
    async fn validate_dependencies(&self, layer: &Layer) -> Result<(), LayerError> {
        for dep_id in &layer.config.dependencies {
            if self.get_layer(*dep_id).await.is_none() {
                return Err(LayerError::DependencyNotFound(*dep_id));
            }
        }
        Ok(())
    }

    /// Aggregate results from multiple agents
    fn aggregate_results(agent_results: &[AgentResult]) -> serde_json::Value {
        let results: Vec<_> = agent_results
            .iter()
            .filter(|r| r.success)
            .filter_map(|r| r.output.clone())
            .collect();

        serde_json::json!({
            "results": results,
            "successful_agents": agent_results.iter().filter(|r| r.success).count(),
            "failed_agents": agent_results.iter().filter(|r| !r.success).count(),
        })
    }
}
