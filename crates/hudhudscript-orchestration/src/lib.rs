//! HudHudScript Orchestration Engine
//!
//! This crate provides layer-based orchestration for coordinating multiple agents.
//!
//! ## Architecture
//!
//! - **Layer**: A group of agents that execute in parallel
//! - **Network**: A collection of layers with execution topology
//! - **Workflow**: A collection of networks to execute
//! - **OrchestrationEngine**: Main coordinator for all orchestration
//!
//! ## Example
//!
//! ```rust
//! use hudhudscript_orchestration::{OrchestrationEngine, Layer, Network, Workflow};
//!
//! #[tokio::main]
//! async fn main() {
//!     let engine = OrchestrationEngine::new();
//!     
//!     // Register a layer
//!     let layer = Layer::new("processing_layer".to_string(), vec!["agent1".to_string()]);
//!     let layer_id = engine.register_layer(layer).await.unwrap();
//!     
//!     // Register a network
//!     let mut network = Network::new("pipeline".to_string());
//!     network.add_layer(layer_id);
//!     let network_id = engine.register_network(network).await.unwrap();
//!     
//!     // Register a workflow
//!     let mut workflow = Workflow::new("main_workflow".to_string());
//!     workflow.add_network(network_id);
//!     let workflow_id = engine.register_workflow(workflow).await.unwrap();
//! }
//! ```

pub mod agent_executor;
pub mod council;
pub mod coup;
pub mod cybernetics;
pub mod events;
pub mod layer;
pub mod network;
pub mod orchestration;
pub mod permissions;
pub mod swarm;
pub mod text_stream;

// Re-export main types
pub use agent_executor::{AgentExecutor, AgentTask, AgentTaskResult, DefaultAgentExecutor};

pub use layer::{
    AgentResult, ExecutionMode, FailureStrategy, Layer, LayerConfig, LayerError, LayerExecutor,
    LayerId, LayerInput, LayerOutput,
};

pub use network::{
    DataMapping, Edge, ErrorStrategy, Network, NetworkConfig, NetworkError, NetworkExecutor,
    NetworkId, NetworkInput, NetworkOutput, NetworkTopology,
};

pub use orchestration::{
    OrchestrationEngine, OrchestrationError, Workflow, WorkflowConfig, WorkflowId, WorkflowInput,
    WorkflowResult,
};

pub use council::{
    CouncilConfig, CouncilDecision, CouncilError, CouncilExecutor, CouncilMember, CouncilResult,
    ExecutionStrategy, SessionHooks, VotingAlgorithm,
};
pub use coup::{AgentTrustState, CoupCondition, CoupConfig, CoupError, CoupExecutor, CoupResult};
pub use cybernetics::{
    ActuationResult, Actuator, BangBangPolicy, ControlError, ControlLoop, CyberneticsError,
    FeedbackPolicy, Goal, LoopStats, Observable, Observer,
};
pub use events::{AgentEvent, EventBus, EventBusError, EventStats};
pub use permissions::{
    PermissionConfig, PermissionEngine, PermissionError, PermissionRule, RuleEffect,
};
pub use swarm::{
    ConsensusStrategy, SwarmAgentResult, SwarmConfig, SwarmError, SwarmExecutor, SwarmResult,
    SwarmState,
};
pub use text_stream::{
    agent_pipe, AgentStreamReader, AgentStreamWriter, StreamError, StreamMessage, TextStreamAdapter,
};
