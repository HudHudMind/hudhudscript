//! Role metadata queries — names, predefined checks, listing.

use crate::types::AgentRole;

use super::RoleManager;

impl RoleManager {
    /// Get a human-readable name for a role
    pub fn role_name(&self, role: &AgentRole) -> String {
        match role {
            AgentRole::Prosecutor => "Prosecutor".to_string(),
            AgentRole::Judge => "Judge".to_string(),
            AgentRole::Executor => "Executor".to_string(),
            AgentRole::Member => "Member".to_string(),
            AgentRole::Custom(name) => name.clone(),
        }
    }

    /// Check if a role is a predefined role
    pub fn is_predefined_role(&self, role: &AgentRole) -> bool {
        matches!(
            role,
            AgentRole::Prosecutor | AgentRole::Judge | AgentRole::Executor | AgentRole::Member
        )
    }

    /// List all predefined roles
    pub fn list_predefined_roles(&self) -> Vec<AgentRole> {
        vec![
            AgentRole::Prosecutor,
            AgentRole::Judge,
            AgentRole::Executor,
            AgentRole::Member,
        ]
    }

    /// List all custom roles that have been defined
    pub fn list_custom_roles(&self) -> Vec<String> {
        self.role_permissions
            .keys()
            .filter(|k| !["Prosecutor", "Judge", "Executor", "Member"].contains(&k.as_str()))
            .cloned()
            .collect()
    }
}
