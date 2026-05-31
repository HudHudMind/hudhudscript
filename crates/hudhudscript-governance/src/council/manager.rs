//! Council manager — runtime member and rule management.

use chrono::Utc;
use std::collections::HashSet;

use crate::types::{AgentId, AgentMember, AgentRole, Council, PermissionStr, RuleId};

use super::{CouncilError, CouncilResult};

/// Council manager for managing council operations
#[derive(Debug, Clone)]
pub struct CouncilManager {
    council: Council,
}

impl CouncilManager {
    /// Create a new council manager from a council
    pub fn new(council: Council) -> Self {
        Self { council }
    }

    /// Get the current council
    pub fn council(&self) -> &Council {
        &self.council
    }

    /// Get a mutable reference to the council
    pub fn council_mut(&mut self) -> &mut Council {
        &mut self.council
    }

    /// Add a member to the council
    pub fn add_member(
        &mut self,
        agent_id: AgentId,
        role: AgentRole,
        permissions: Vec<PermissionStr>,
    ) -> CouncilResult<()> {
        if self.council.members.iter().any(|m| m.agent_id == agent_id) {
            return Err(CouncilError::DuplicateAgent(agent_id));
        }

        self.council.members.push(AgentMember {
            agent_id,
            role,
            joined_at: Utc::now(),
            permissions,
        });

        Ok(())
    }

    /// Remove a member from the council
    pub fn remove_member(&mut self, agent_id: &AgentId) -> CouncilResult<()> {
        let initial_len = self.council.members.len();
        self.council.members.retain(|m| &m.agent_id != agent_id);

        if self.council.members.len() == initial_len {
            return Err(CouncilError::AgentNotFound(agent_id.clone()));
        }

        Ok(())
    }

    /// Update a member's role
    pub fn update_member_role(
        &mut self,
        agent_id: &AgentId,
        new_role: AgentRole,
    ) -> CouncilResult<()> {
        let member = self
            .council
            .members
            .iter_mut()
            .find(|m| &m.agent_id == agent_id)
            .ok_or_else(|| CouncilError::AgentNotFound(agent_id.clone()))?;

        member.role = new_role;
        Ok(())
    }

    /// Update a member's permissions
    pub fn update_member_permissions(
        &mut self,
        agent_id: &AgentId,
        permissions: Vec<PermissionStr>,
    ) -> CouncilResult<()> {
        let member = self
            .council
            .members
            .iter_mut()
            .find(|m| &m.agent_id == agent_id)
            .ok_or_else(|| CouncilError::AgentNotFound(agent_id.clone()))?;

        member.permissions = permissions;
        Ok(())
    }

    /// Get a member by agent ID
    pub fn get_member(&self, agent_id: &AgentId) -> Option<&AgentMember> {
        self.council
            .members
            .iter()
            .find(|m| &m.agent_id == agent_id)
    }

    /// Check if an agent is a member of the council
    pub fn has_member(&self, agent_id: &AgentId) -> bool {
        self.council.members.iter().any(|m| &m.agent_id == agent_id)
    }

    /// Get all members with a specific role
    pub fn get_members_by_role(&self, role: &AgentRole) -> Vec<&AgentMember> {
        self.council
            .members
            .iter()
            .filter(|m| &m.role == role)
            .collect()
    }

    /// Add a rule to the council
    pub fn add_rule(&mut self, rule_id: RuleId) {
        self.council.rules.push(rule_id);
    }

    /// Remove a rule from the council
    pub fn remove_rule(&mut self, rule_id: &RuleId) {
        self.council.rules.retain(|r| r != rule_id);
    }

    /// Set council active state
    pub fn set_active(&mut self, active: bool) {
        self.council.state.active = active;
    }

    /// Check if council is active
    pub fn is_active(&self) -> bool {
        self.council.state.active
    }

    /// Validate that all agent IDs are unique
    pub fn validate_unique_agents(&self) -> Result<(), CouncilError> {
        let agent_ids: HashSet<&AgentId> =
            self.council.members.iter().map(|m| &m.agent_id).collect();

        if agent_ids.len() != self.council.members.len() {
            let mut seen = HashSet::new();
            for member in &self.council.members {
                if !seen.insert(&member.agent_id) {
                    return Err(CouncilError::DuplicateAgent(member.agent_id.clone()));
                }
            }
        }

        Ok(())
    }
}
