//! Swarm agent management — add, remove, validate, query agents.

use crate::types::{AgentId, Swarm};

use super::SwarmError;

impl Swarm {
    /// Validate that all referenced agent IDs exist
    pub fn validate_agents(
        &self,
        valid_agents: &std::collections::HashSet<AgentId>,
    ) -> Result<(), SwarmError> {
        for agent_id in &self.agents {
            if !valid_agents.contains(agent_id) {
                return Err(SwarmError::AgentNotFound(agent_id.clone()));
            }
        }
        Ok(())
    }

    /// Add an agent to the swarm
    pub fn add_agent(&mut self, agent_id: AgentId) -> Result<(), SwarmError> {
        if self.agents.contains(&agent_id) {
            return Err(SwarmError::DuplicateAgent(agent_id));
        }
        self.agents.push(agent_id);
        Ok(())
    }

    /// Remove an agent from the swarm
    pub fn remove_agent(&mut self, agent_id: &str) -> Result<(), SwarmError> {
        let initial_len = self.agents.len();
        self.agents.retain(|id| id != agent_id);

        if self.agents.len() == initial_len {
            return Err(SwarmError::AgentNotFound(agent_id.to_string()));
        }
        Ok(())
    }

    /// Get the number of agents in the swarm
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Check if an agent is a member of the swarm
    pub fn has_agent(&self, agent_id: &str) -> bool {
        self.agents.iter().any(|id| id == agent_id)
    }
}
