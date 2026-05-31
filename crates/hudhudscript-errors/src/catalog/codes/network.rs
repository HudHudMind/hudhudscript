use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum NetworkErrorCode {
    /// E0113 — Layer references an unknown dependency
    LayerDependencyNotFound = 113,
    /// E0114 — Layer execution returned a failure
    LayerExecutionFailed = 114,
    /// E0115 — Layer with this name is already registered
    LayerLayerAlreadyExists = 115,
    /// E0116 — Referenced layer does not exist
    LayerLayerNotFound = 116,
    /// E0117 — Layer exceeded its execution timeout
    LayerTimeoutExceeded = 117,
    /// E0148 — Cycle detected in network dependency graph
    NetworkCyclicDependency = 148,
    /// E0149 — Network topology configuration is invalid
    NetworkInvalidTopology = 149,
    /// E0150 — Network-scoped layer execution failed
    NetworkLayerExecutionFailed = 150,
    /// E0151 — Layer not found in network
    NetworkLayerNotFound = 151,
    /// E0152 — Network with this name is already registered
    NetworkNetworkAlreadyExists = 152,
    /// E0153 — Referenced network does not exist
    NetworkNetworkNotFound = 153,
    /// E0154 — Network execution exceeded its overall timeout
    NetworkTimeoutExceeded = 154,
}
