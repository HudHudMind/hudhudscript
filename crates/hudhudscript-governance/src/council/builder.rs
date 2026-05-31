//! Council builder — create and validate councils.

use chrono::Utc;
use std::collections::{HashMap, HashSet};

use crate::types::{
    AgentMember, AgentRole, ConstitutionId, Council, CouncilId, CouncilState, PermissionStr, RuleId,
};

use super::{CouncilError, CouncilResult};

/// Council builder for creating councils with validation
#[allow(clippy::type_complexity)]
pub struct CouncilBuilder {
    id: CouncilId,
    name: String,
    constitution_id: ConstitutionId,
    members: Vec<AgentMember>,
    rules: Vec<RuleId>,
    constitution_validator: Option<Box<dyn Fn(&ConstitutionId) -> bool>>,
}

impl CouncilBuilder {
    /// Create a new council builder
    pub fn new(id: CouncilId, name: String, constitution_id: ConstitutionId) -> Self {
        Self {
            id,
            name,
            constitution_id,
            members: Vec::new(),
            rules: Vec::new(),
            constitution_validator: None,
        }
    }

    /// Set a constitution validator function
    pub fn with_constitution_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&ConstitutionId) -> bool + 'static,
    {
        self.constitution_validator = Some(Box::new(validator));
        self
    }

    /// Add a member to the council
    pub fn add_member(
        mut self,
        agent_id: crate::types::AgentId,
        role: AgentRole,
        permissions: Vec<PermissionStr>,
    ) -> CouncilResult<Self> {
        if self.members.iter().any(|m| m.agent_id == agent_id) {
            return Err(CouncilError::DuplicateAgent(agent_id));
        }

        self.members.push(AgentMember {
            agent_id,
            role,
            joined_at: Utc::now(),
            permissions,
        });

        Ok(self)
    }

    /// Add multiple members to the council
    pub fn add_members(
        mut self,
        members: Vec<(crate::types::AgentId, AgentRole, Vec<PermissionStr>)>,
    ) -> CouncilResult<Self> {
        for (agent_id, role, permissions) in members {
            self = self.add_member(agent_id, role, permissions)?;
        }
        Ok(self)
    }

    /// Add a rule to the council
    pub fn add_rule(mut self, rule_id: RuleId) -> Self {
        self.rules.push(rule_id);
        self
    }

    /// Add multiple rules to the council
    pub fn add_rules(mut self, rule_ids: Vec<RuleId>) -> Self {
        self.rules.extend(rule_ids);
        self
    }

    /// Build the council with validation
    pub fn build(self) -> CouncilResult<Council> {
        if let Some(validator) = &self.constitution_validator {
            if !validator(&self.constitution_id) {
                return Err(CouncilError::ConstitutionNotFound(
                    self.constitution_id.clone(),
                ));
            }
        }

        let agent_ids: HashSet<&crate::types::AgentId> =
            self.members.iter().map(|m| &m.agent_id).collect();
        if agent_ids.len() != self.members.len() {
            let mut seen = HashSet::new();
            for member in &self.members {
                if !seen.insert(&member.agent_id) {
                    return Err(CouncilError::DuplicateAgent(member.agent_id.clone()));
                }
            }
        }

        Ok(Council {
            id: self.id,
            name: self.name,
            constitution_id: self.constitution_id,
            members: self.members,
            rules: self.rules,
            state: CouncilState {
                active: true,
                metadata: HashMap::new(),
            },
            governance_model: None,
        })
    }
}
