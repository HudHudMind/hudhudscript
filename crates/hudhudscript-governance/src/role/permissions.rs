//! Role permission queries and custom role registration.

use crate::types::{AgentRole, PermissionStr};

use super::RoleManager;

impl RoleManager {
    /// Get default permissions for a role
    pub fn get_default_permissions(&self, role: &AgentRole) -> Vec<PermissionStr> {
        let role_key = match role {
            AgentRole::Prosecutor => "Prosecutor",
            AgentRole::Judge => "Judge",
            AgentRole::Executor => "Executor",
            AgentRole::Member => "Member",
            AgentRole::Custom(_) => return Vec::new(),
        };

        self.role_permissions
            .get(role_key)
            .cloned()
            .unwrap_or_default()
    }

    /// Set default permissions for a custom role
    pub fn set_custom_role_permissions(
        &mut self,
        role_name: String,
        permissions: Vec<PermissionStr>,
    ) {
        self.role_permissions.insert(role_name, permissions);
    }

    /// Get permissions for a custom role
    pub fn get_custom_role_permissions(&self, role_name: &str) -> Vec<PermissionStr> {
        self.role_permissions
            .get(role_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Check if a role has a specific permission
    pub fn has_permission(&self, role: &AgentRole, permission: &str) -> bool {
        let permissions = match role {
            AgentRole::Custom(name) => self.get_custom_role_permissions(name),
            _ => self.get_default_permissions(role),
        };

        permissions.iter().any(|p| p == permission)
    }
}
