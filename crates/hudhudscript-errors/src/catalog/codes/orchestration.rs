use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum OrchestrationErrorCode {
    /// E0159 — Workflow definition failed validation
    OrchestrationInvalidWorkflow = 159,
    /// E0160 — Orchestration encountered a layer-level error
    OrchestrationLayerError = 160,
    /// E0161 — Orchestration encountered a network-level error
    OrchestrationNetworkError = 161,
    /// E0162 — Network execution failed during orchestration
    OrchestrationNetworkExecutionFailed = 162,
    /// E0163 — Workflow references an unknown network
    OrchestrationNetworkNotFound = 163,
    /// E0164 — Workflow with this name is already registered
    OrchestrationWorkflowAlreadyExists = 164,
    /// E0165 — Referenced workflow is not registered
    OrchestrationWorkflowNotFound = 165,
    /// E0166 — Workflow execution exceeded its overall timeout
    OrchestrationWorkflowTimedOut = 166,
}
