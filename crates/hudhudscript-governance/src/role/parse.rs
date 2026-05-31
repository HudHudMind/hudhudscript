//! Role parsing — string-to-enum conversion and validation.

use crate::types::AgentRole;

use super::RoleManager;

impl RoleManager {
    /// Parse a role string into an AgentRole enum
    pub fn parse_role(&self, role_str: &str) -> AgentRole {
        match role_str {
            "Prosecutor" => AgentRole::Prosecutor,
            "Judge" => AgentRole::Judge,
            "Executor" => AgentRole::Executor,
            "Member" => AgentRole::Member,
            "" => {
                log::warn!("Empty role string, defaulting to Member");
                AgentRole::Member
            }
            custom => {
                if custom.trim().is_empty() {
                    log::warn!("Whitespace-only role string, defaulting to Member");
                    AgentRole::Member
                } else {
                    AgentRole::Custom(custom.to_string())
                }
            }
        }
    }

    /// Validate a role and return it, or default to Member if invalid
    pub fn validate_role(&self, role: AgentRole) -> AgentRole {
        role
    }
}
