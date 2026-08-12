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
//! use hudhudscript_orchestration::{
//!     CouncilExecutor, EventBus, Layer, LayerExecutor, Network, NetworkExecutor,
//!     OrchestrationEngine, Workflow,
//! };
//! use hudhudscript_orchestration::orchestration::types::{
//!     StepConfig, StepType, WorkflowConfig, WorkflowStep,
//! };
//! use std::sync::Arc;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!     // The engine is composed from the event bus and the three executors it
//!     // drives; the network executor runs layers, so it borrows that one.
//!     let event_bus = Arc::new(EventBus::new());
//!     let layer_executor = Arc::new(LayerExecutor::new());
//!     let network_executor = Arc::new(NetworkExecutor::new(layer_executor.clone()));
//!     let council_executor = Arc::new(CouncilExecutor::new(event_bus.clone()));
//!     let engine = OrchestrationEngine::new(
//!         event_bus,
//!         layer_executor.clone(),
//!         network_executor,
//!         council_executor,
//!     );
//!
//!     // A layer names the agents that run together. Layers are registered on
//!     // the layer executor, not on the engine.
//!     let mut layer = Layer::new("processing_layer");
//!     layer.agents.push("agent1".to_string());
//!     let layer_id = layer.id;
//!     layer_executor.register_layer(layer).await.unwrap();
//!
//!     // A network chains layers together.
//!     let mut network = Network::new("pipeline".to_string());
//!     network.add_layer(layer_id);
//!     let network_id = network.id;
//!
//!     // A workflow is a sequence of steps; one step can run a whole network.
//!     let workflow = Workflow {
//!         id: Uuid::new_v4(),
//!         name: "main_workflow".to_string(),
//!         steps: vec![WorkflowStep {
//!             name: "run_pipeline".to_string(),
//!             step_type: StepType::Network { network_id },
//!             config: StepConfig::default(),
//!         }],
//!         config: WorkflowConfig::default(),
//!     };
//!     let _workflow_id = engine.register_workflow(workflow).await.unwrap();
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
