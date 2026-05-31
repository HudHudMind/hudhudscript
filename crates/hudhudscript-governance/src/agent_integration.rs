//! Agent Integration
//!
//! This module provides functionality for integrating governance structures
//! with the agent runtime system, including agent validation and membership queries.

use crate::types::{AgentId, Community, Council, CouncilId, Swarm};
use std::collections::{HashMap, HashSet};

/// Agent registry for governance-domain validation.
///
/// **Status**: This is a complete in-memory registry suitable for governance
/// validation use cases (checking if an agent ID is registered, listing
/// registered agents). It is intentionally decoupled from the agent runtime
/// system to keep the governance crate dependency-free.
///
/// To wire this to the live runtime: synchronize with the runtime's
/// `AgentRegistry` (in `hudhudscript-runtime`) at agent spawn/despawn time
/// using a callback or event listener. The governance side only needs to
/// know agent IDs, not the actual agent state.
///
/// (v0.4.47.9 — Issue #800: clarified intent)
#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    agents: HashSet<AgentId>,
}

impl AgentRegistry {
    /// Create a new agent registry
    pub fn new() -> Self {
        Self {
            agents: HashSet::new(),
        }
    }

    /// Register an agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to register
    ///
    /// # Returns
    /// * `Ok(())` if registration succeeded
    /// * `Err(String)` if agent already exists
    pub fn register_agent(&mut self, agent_id: AgentId) -> Result<(), String> {
        if self.agents.contains(&agent_id) {
            return Err(format!("Agent {} already registered", agent_id));
        }
        self.agents.insert(agent_id);
        Ok(())
    }

    /// Unregister an agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to unregister
    ///
    /// # Returns
    /// * `Ok(())` if unregistration succeeded
    /// * `Err(String)` if agent not found
    pub fn unregister_agent(&mut self, agent_id: &AgentId) -> Result<(), String> {
        if !self.agents.remove(agent_id) {
            return Err(format!("Agent {} not found", agent_id));
        }
        Ok(())
    }

    /// Check if an agent exists
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to check
    ///
    /// # Returns
    /// * `true` if agent exists, `false` otherwise
    pub fn agent_exists(&self, agent_id: &AgentId) -> bool {
        self.agents.contains(agent_id)
    }

    /// Validate a list of agent IDs
    ///
    /// # Arguments
    /// * `agent_ids` - List of agent IDs to validate
    ///
    /// # Returns
    /// * `Ok(())` if all agents exist
    /// * `Err(String)` with list of missing agent IDs
    pub fn validate_agents(&self, agent_ids: &[AgentId]) -> Result<(), String> {
        let missing: Vec<_> = agent_ids
            .iter()
            .filter(|id| !self.agents.contains(*id))
            .collect();

        if !missing.is_empty() {
            return Err(format!("Missing agents: {:?}", missing));
        }

        Ok(())
    }

    /// Get all registered agents
    ///
    /// # Returns
    /// * Vector of all registered agent IDs
    pub fn get_all_agents(&self) -> Vec<AgentId> {
        self.agents.iter().cloned().collect()
    }
}

/// Agent membership tracker
///
/// Tracks which councils, swarms, and communities an agent belongs to.
#[derive(Debug, Clone, Default)]
pub struct AgentMembershipTracker {
    council_memberships: HashMap<AgentId, Vec<CouncilId>>,
    swarm_memberships: HashMap<AgentId, Vec<String>>, // SwarmId
    community_memberships: HashMap<AgentId, Vec<String>>, // CommunityId
}

impl AgentMembershipTracker {
    /// Create a new membership tracker
    pub fn new() -> Self {
        Self {
            council_memberships: HashMap::new(),
            swarm_memberships: HashMap::new(),
            community_memberships: HashMap::new(),
        }
    }

    /// Track council membership
    ///
    /// # Arguments
    /// * `council` - Council to track
    pub fn track_council(&mut self, council: &Council) {
        for member in &council.members {
            self.council_memberships
                .entry(member.agent_id.clone())
                .or_default()
                .push(council.id.clone());
        }
    }

    /// Track swarm membership
    ///
    /// # Arguments
    /// * `swarm` - Swarm to track
    pub fn track_swarm(&mut self, swarm: &Swarm) {
        for agent_id in &swarm.agents {
            self.swarm_memberships
                .entry(agent_id.clone())
                .or_default()
                .push(swarm.id.clone());
        }
    }

    /// Track community membership
    ///
    /// # Arguments
    /// * `community` - Community to track
    pub fn track_community(&mut self, community: &Community) {
        for agent_id in &community.members {
            self.community_memberships
                .entry(agent_id.clone())
                .or_default()
                .push(community.id.clone());
        }
    }

    /// Get councils an agent belongs to
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to query
    ///
    /// # Returns
    /// * Vector of council IDs the agent belongs to
    pub fn get_agent_councils(&self, agent_id: &AgentId) -> Vec<CouncilId> {
        self.council_memberships
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get swarms an agent belongs to
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to query
    ///
    /// # Returns
    /// * Vector of swarm IDs the agent belongs to
    pub fn get_agent_swarms(&self, agent_id: &AgentId) -> Vec<String> {
        self.swarm_memberships
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get communities an agent belongs to
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to query
    ///
    /// # Returns
    /// * Vector of community IDs the agent belongs to
    pub fn get_agent_communities(&self, agent_id: &AgentId) -> Vec<String> {
        self.community_memberships
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get agent's role in a specific council
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to query
    /// * `council_id` - Council ID to query
    /// * `council` - Council instance to check
    ///
    /// # Returns
    /// * `Some(role)` if agent is a member, `None` otherwise
    pub fn get_agent_role_in_council(
        &self,
        agent_id: &AgentId,
        council_id: &CouncilId,
        council: &Council,
    ) -> Option<String> {
        if &council.id != council_id {
            return None;
        }

        council
            .members
            .iter()
            .find(|m| &m.agent_id == agent_id)
            .map(|m| format!("{:?}", m.role))
    }

    /// Check if agent is a member of any governance structure
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to check
    ///
    /// # Returns
    /// * `true` if agent is a member of any council, swarm, or community
    pub fn is_member_of_any(&self, agent_id: &AgentId) -> bool {
        self.council_memberships.contains_key(agent_id)
            || self.swarm_memberships.contains_key(agent_id)
            || self.community_memberships.contains_key(agent_id)
    }
}

/// Validate council agent references
///
/// # Arguments
/// * `council` - Council to validate
/// * `registry` - Agent registry for validation
///
/// # Returns
/// * `Ok(())` if all agent references are valid
/// * `Err(String)` if any agent references are invalid
pub fn validate_council_agents(council: &Council, registry: &AgentRegistry) -> Result<(), String> {
    let agent_ids: Vec<_> = council.members.iter().map(|m| m.agent_id.clone()).collect();
    registry.validate_agents(&agent_ids)
}

/// Validate swarm agent references
///
/// # Arguments
/// * `swarm` - Swarm to validate
/// * `registry` - Agent registry for validation
///
/// # Returns
/// * `Ok(())` if all agent references are valid
/// * `Err(String)` if any agent references are invalid
pub fn validate_swarm_agents(swarm: &Swarm, registry: &AgentRegistry) -> Result<(), String> {
    registry.validate_agents(&swarm.agents)
}

/// Validate community agent references
///
/// # Arguments
/// * `community` - Community to validate
/// * `registry` - Agent registry for validation
///
/// # Returns
/// * `Ok(())` if all agent references are valid
/// * `Err(String)` if any agent references are invalid
pub fn validate_community_agents(
    community: &Community,
    registry: &AgentRegistry,
) -> Result<(), String> {
    registry.validate_agents(&community.members)
}
