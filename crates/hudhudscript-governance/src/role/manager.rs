//! Role manager — holds default permission tables.

use std::collections::HashMap;

use crate::types::{AgentRole, PermissionStr};

/// Role manager for managing agent roles and permissions
#[derive(Debug, Clone)]
pub struct RoleManager {
    /// Default permissions for predefined roles
    pub(crate) role_permissions: HashMap<String, Vec<PermissionStr>>,
}

impl Default for RoleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RoleManager {
    /// Create a new role manager with default permissions
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();

        role_permissions.insert(
            "Prosecutor".to_string(),
            vec![
                "propose_action".to_string(),
                "read_constitution".to_string(),
                "read_laws".to_string(),
            ],
        );

        role_permissions.insert(
            "Judge".to_string(),
            vec![
                "evaluate_compliance".to_string(),
                "read_constitution".to_string(),
                "read_laws".to_string(),
                "make_decision".to_string(),
            ],
        );

        role_permissions.insert(
            "Executor".to_string(),
            vec![
                "execute_decision".to_string(),
                "read_constitution".to_string(),
                "read_laws".to_string(),
            ],
        );

        role_permissions.insert(
            "Member".to_string(),
            vec!["read_constitution".to_string(), "read_laws".to_string()],
        );

        Self { role_permissions }
    }
}
