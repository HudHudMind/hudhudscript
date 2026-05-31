//! Swarm types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{AgentId, SwarmId};

/// Swarm: Coordinated group of agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Swarm {
    pub id: SwarmId,
    pub name: String,
    pub agents: Vec<AgentId>,
    pub coordination_strategy: CoordinationStrategy,
    pub shared_state: HashMap<String, serde_json::Value>,
}

/// Coordination strategy for swarms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CoordinationStrategy {
    /// All agents work simultaneously
    Parallel,
    /// Agents work in order
    Sequential,
    /// Agents compete for tasks
    Competitive,
    /// Agents share workload
    Collaborative,
}
