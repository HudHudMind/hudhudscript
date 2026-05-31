use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Workflow ID
pub type WorkflowId = Uuid;

/// Workflow - A sequence of steps to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique workflow ID
    pub id: WorkflowId,
    /// Workflow name
    pub name: String,
    /// Workflow steps
    pub steps: Vec<WorkflowStep>,
    /// Workflow configuration
    pub config: WorkflowConfig,
}

/// Workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step name
    pub name: String,
    /// Step type
    pub step_type: StepType,
    /// Step configuration
    pub config: StepConfig,
}

/// Step type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StepType {
    /// Execute a layer
    Layer {
        /// Layer ID
        layer_id: crate::layer::LayerId,
    },
    /// Execute a network
    Network {
        /// Network ID
        network_id: crate::network::NetworkId,
    },
    /// Execute a council
    Council {
        /// Council ID
        council_id: String,
    },
    /// Execute a function
    Function {
        /// Function name
        function_name: String,
    },
    /// Conditional step
    Conditional {
        /// Condition expression
        condition: String,
        /// True branch steps
        true_branch: Vec<WorkflowStep>,
        /// False branch steps
        false_branch: Vec<WorkflowStep>,
    },
    /// Loop step
    Loop {
        /// Loop type
        loop_type: LoopType,
        /// Loop body
        body: Vec<WorkflowStep>,
    },
}

/// Loop type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LoopType {
    /// Fixed number of iterations
    For {
        /// Number of iterations
        iterations: usize,
    },
    /// While condition is true
    While {
        /// Condition expression
        condition: String,
    },
    /// For each item
    ForEach {
        /// Collection expression
        collection: String,
    },
}

/// Step configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepConfig {
    /// Timeout in seconds
    pub timeout: Option<u64>,
    /// Retry configuration
    pub retry: Option<RetryConfig>,
    /// Error handling
    pub error_handling: Option<ErrorHandling>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Max retries
    pub max_retries: u32,
    /// Retry delay in seconds
    pub delay_secs: u64,
}

/// Error handling strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorHandling {
    /// Stop on error
    Stop,
    /// Continue on error
    Continue,
    /// Retry on error
    Retry,
    /// Skip step on error
    Skip,
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Max concurrent steps
    pub max_concurrent: usize,
    /// Default timeout
    pub default_timeout: u64,
    /// Enable monitoring
    pub monitoring: bool,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            default_timeout: 300,
            monitoring: false,
        }
    }
}

/// Workflow input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInput {
    /// Input data
    pub data: serde_json::Value,
    /// Input metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// Workflow ID
    pub workflow_id: WorkflowId,
    /// Overall success
    pub success: bool,
    /// Step results
    pub step_results: Vec<StepResult>,
    /// Final output
    pub output: serde_json::Value,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Step result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step name
    pub step_name: String,
    /// Success
    pub success: bool,
    /// Output
    pub output: serde_json::Value,
    /// Error message
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}
