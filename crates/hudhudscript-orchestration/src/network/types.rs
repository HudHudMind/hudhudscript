use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::layer::{LayerId, LayerOutput};

/// Network ID
pub type NetworkId = Uuid;

/// Network - A collection of layers with execution topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    /// Unique network ID
    pub id: NetworkId,
    /// Network name
    pub name: String,
    /// Layers in this network
    pub layers: Vec<LayerId>,
    /// Network topology (edges between layers)
    pub topology: NetworkTopology,
    /// Network configuration
    pub config: NetworkConfig,
}

impl Network {
    /// Create new network
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            layers: Vec::new(),
            topology: NetworkTopology::default(),
            config: NetworkConfig::default(),
        }
    }

    /// Add layer to network
    pub fn add_layer(&mut self, layer_id: LayerId) {
        if !self.layers.contains(&layer_id) {
            self.layers.push(layer_id);
        }
    }

    /// Add edge between layers
    pub fn add_edge(&mut self, from: LayerId, to: LayerId) {
        self.topology.add_edge(from, to);
    }
}

/// Network topology
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkTopology {
    /// Edges (from -> to)
    pub edges: Vec<Edge>,
}

impl NetworkTopology {
    /// Add edge
    pub fn add_edge(&mut self, from: LayerId, to: LayerId) {
        let edge = Edge {
            from,
            to,
            data_mapping: None,
        };

        if !self.edges.iter().any(|e| e.from == from && e.to == to) {
            self.edges.push(edge);
        }
    }

    /// Get outgoing edges for a layer
    pub fn outgoing_edges(&self, layer_id: LayerId) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.from == layer_id).collect()
    }

    /// Get incoming edges for a layer
    pub fn incoming_edges(&self, layer_id: LayerId) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.to == layer_id).collect()
    }
}

/// Edge between layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source layer
    pub from: LayerId,
    /// Target layer
    pub to: LayerId,
    /// Optional data mapping
    pub data_mapping: Option<DataMapping>,
}

/// Data mapping between layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMapping {
    /// Field mappings (source_field -> target_field)
    pub mappings: std::collections::HashMap<String, String>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable monitoring
    pub monitoring: bool,
    /// Enable detailed logging
    pub detailed_logging: bool,
    /// Timeout in seconds
    pub timeout: u64,
    /// Error strategy
    pub error_strategy: ErrorStrategy,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            monitoring: false,
            detailed_logging: false,
            timeout: 300,
            error_strategy: ErrorStrategy::Stop,
        }
    }
}

/// Error strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorStrategy {
    /// Stop on first error
    Stop,
    /// Continue despite errors
    Continue,
    /// Retry with fallback
    RetryWithFallback,
}

/// Network input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInput {
    /// Input data
    pub data: serde_json::Value,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Network output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOutput {
    /// Network ID
    pub network_id: NetworkId,
    /// Outputs from each layer
    pub layer_outputs: std::collections::HashMap<LayerId, LayerOutput>,
    /// Final aggregated data
    pub final_data: serde_json::Value,
    /// Overall success
    pub success: bool,
}
