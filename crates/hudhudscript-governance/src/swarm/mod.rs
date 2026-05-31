//! Swarm coordination module.
//!
//! Implements the Swarm struct for coordinating multiple agents
//! working together. Swarms support different coordination strategies
//! (Parallel, Sequential, Competitive, Collaborative) and maintain shared
//! state accessible to all members.

pub mod agents;
pub mod error;
pub mod state;
pub mod strategy;

pub use error::*;

use crate::types::{AgentId, CoordinationStrategy, Swarm, SwarmId};
use std::collections::HashMap;

impl Swarm {
    /// Create a new swarm with the specified coordination strategy
    pub fn new(
        id: SwarmId,
        name: String,
        agents: Vec<AgentId>,
        coordination_strategy: CoordinationStrategy,
    ) -> Self {
        Self {
            id,
            name,
            agents,
            coordination_strategy,
            shared_state: HashMap::new(),
        }
    }
}
