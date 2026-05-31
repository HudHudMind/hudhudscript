use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::layer::{LayerExecutor, LayerId, LayerInput, LayerOutput};

use super::{Network, NetworkError, NetworkId, NetworkInput, NetworkOutput};

/// Network executor
pub struct NetworkExecutor {
    /// Layer executor
    layer_executor: Arc<LayerExecutor>,
    /// Networks indexed by ID
    pub networks: Arc<RwLock<HashMap<NetworkId, Network>>>,
    /// Network name to ID mapping
    names: Arc<RwLock<HashMap<String, NetworkId>>>,
}

impl NetworkExecutor {
    /// Create new network executor
    pub fn new(layer_executor: Arc<LayerExecutor>) -> Self {
        Self {
            layer_executor,
            networks: Arc::new(RwLock::new(HashMap::new())),
            names: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a network
    pub async fn register_network(&self, network: Network) -> Result<NetworkId, NetworkError> {
        let network_id = network.id;
        let network_name = network.name.clone();

        {
            let names = self.names.read().await;
            if names.contains_key(&network_name) {
                return Err(NetworkError::NetworkAlreadyExists(network_name));
            }
        }

        self.validate_topology(&network).await?;

        self.networks.write().await.insert(network_id, network);
        self.names.write().await.insert(network_name, network_id);

        Ok(network_id)
    }

    /// Get network by ID
    pub async fn get_network(&self, network_id: NetworkId) -> Option<Network> {
        self.networks.read().await.get(&network_id).cloned()
    }

    /// Get network by name
    pub async fn get_network_by_name(&self, name: &str) -> Option<Network> {
        let names = self.names.read().await;
        let network_id = names.get(name)?;
        self.networks.read().await.get(network_id).cloned()
    }

    /// Execute a network
    pub async fn execute_network(
        &self,
        network_id: NetworkId,
        input: NetworkInput,
    ) -> Result<NetworkOutput, NetworkError> {
        let network = self
            .get_network(network_id)
            .await
            .ok_or(NetworkError::NetworkNotFound(network_id))?;

        let timeout_duration = std::time::Duration::from_secs(network.config.timeout);

        let layer_executor = self.layer_executor.clone();
        let execution_order = self.topological_sort(&network)?;

        let network_work = async move {
            let mut layer_outputs: HashMap<LayerId, LayerOutput> = HashMap::new();
            let mut current_data = input.data;

            for layer_id in execution_order {
                let layer_input = LayerInput {
                    data: current_data.clone(),
                    metadata: input.metadata.clone(),
                };

                let layer_output = layer_executor
                    .execute_layer(layer_id, layer_input)
                    .await
                    .map_err(|e| NetworkError::LayerExecutionFailed(layer_id, e.to_string()))?;

                current_data = Self::aggregate_layer_output_static(&layer_output);
                layer_outputs.insert(layer_id, layer_output);
            }

            Ok::<NetworkOutput, NetworkError>(NetworkOutput {
                network_id,
                layer_outputs,
                final_data: current_data,
                success: true,
            })
        };

        tokio::time::timeout(timeout_duration, network_work)
            .await
            .map_err(|_| NetworkError::TimeoutExceeded(network_id))?
    }

    /// Validate network topology
    async fn validate_topology(&self, network: &Network) -> Result<(), NetworkError> {
        for layer_id in &network.layers {
            if self.layer_executor.get_layer(*layer_id).await.is_none() {
                return Err(NetworkError::LayerNotFound(*layer_id));
            }
        }

        if self.has_cycle(network)? {
            return Err(NetworkError::CyclicDependency);
        }

        Ok(())
    }

    /// Check if network has cycles
    fn has_cycle(&self, network: &Network) -> Result<bool, NetworkError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &layer_id in &network.layers {
            if !visited.contains(&layer_id)
                && self.has_cycle_util(layer_id, &network.topology, &mut visited, &mut rec_stack)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn has_cycle_util(
        &self,
        layer_id: LayerId,
        topology: &super::NetworkTopology,
        visited: &mut HashSet<LayerId>,
        rec_stack: &mut HashSet<LayerId>,
    ) -> bool {
        visited.insert(layer_id);
        rec_stack.insert(layer_id);

        for edge in topology.outgoing_edges(layer_id) {
            if !visited.contains(&edge.to) {
                if self.has_cycle_util(edge.to, topology, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&edge.to) {
                return true;
            }
        }

        rec_stack.remove(&layer_id);
        false
    }

    /// Topological sort to get execution order
    pub fn topological_sort(&self, network: &Network) -> Result<Vec<LayerId>, NetworkError> {
        let mut in_degree: HashMap<LayerId, usize> = HashMap::new();

        for &layer_id in &network.layers {
            in_degree.insert(layer_id, 0);
        }

        for edge in &network.topology.edges {
            *in_degree.entry(edge.to).or_insert(0) += 1;
        }

        let mut queue: VecDeque<LayerId> = VecDeque::new();
        for (&layer_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(layer_id);
            }
        }

        let mut result = Vec::new();

        while let Some(layer_id) = queue.pop_front() {
            result.push(layer_id);

            for edge in network.topology.outgoing_edges(layer_id) {
                let degree = in_degree.get_mut(&edge.to).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(edge.to);
                }
            }
        }

        if result.len() != network.layers.len() {
            return Err(NetworkError::CyclicDependency);
        }

        Ok(result)
    }

    /// Aggregate layer output for next layer
    fn aggregate_layer_output_static(output: &LayerOutput) -> serde_json::Value {
        let results: Vec<_> = output
            .agent_results
            .iter()
            .filter_map(|r| r.output.clone())
            .collect();

        serde_json::json!({
            "results": results,
            "layer_id": output.layer_id.to_string(),
        })
    }
}
