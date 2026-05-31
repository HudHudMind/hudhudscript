//! Access control and security for the governance system

use crate::error::{FormatValidationError, GovernanceError};
use crate::id_validator::{
    sanitize_id, validate_constitution_id, validate_law_id, validate_rule_id,
};
use std::collections::HashSet;

/// Permission types for governance operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Create new constitutions
    CreateConstitution,

    /// Modify existing constitutions
    ModifyConstitution,

    /// Delete constitutions
    DeleteConstitution,

    /// Create councils
    CreateCouncil,

    /// Modify councils
    ModifyCouncil,

    /// Delete councils
    DeleteCouncil,

    /// Create rules
    CreateRule,

    /// Modify rules
    ModifyRule,

    /// Delete rules
    DeleteRule,

    /// Create swarms
    CreateSwarm,

    /// Modify swarms
    ModifySwarm,

    /// Delete swarms
    DeleteSwarm,

    /// Create communities
    CreateCommunity,

    /// Modify communities
    ModifyCommunity,

    /// Delete communities
    DeleteCommunity,

    /// View governance structures
    ViewGovernance,

    /// Enforce constitutions
    EnforceConstitution,

    /// Audit governance operations
    AuditGovernance,
}

/// Access control manager
pub struct AccessControl {
    /// Agent permissions
    agent_permissions: std::collections::HashMap<String, HashSet<Permission>>,

    /// Maximum cache size (for DoS prevention)
    max_cache_size: usize,

    /// Enable strict validation
    pub strict_validation: bool,
}

impl AccessControl {
    /// Create new access control manager
    pub fn new() -> Self {
        Self {
            agent_permissions: std::collections::HashMap::new(),
            max_cache_size: 10000,
            strict_validation: true,
        }
    }

    /// Create with custom cache size limit
    pub fn with_cache_limit(max_cache_size: usize) -> Self {
        Self {
            agent_permissions: std::collections::HashMap::new(),
            max_cache_size,
            strict_validation: true,
        }
    }

    /// Set strict validation mode
    pub fn set_strict_validation(&mut self, strict: bool) {
        self.strict_validation = strict;
    }

    /// Get maximum cache size
    pub fn max_cache_size(&self) -> usize {
        self.max_cache_size
    }

    /// Grant permission to an agent
    pub fn grant_permission(&mut self, agent_id: String, permission: Permission) {
        self.agent_permissions
            .entry(agent_id)
            .or_default()
            .insert(permission);
    }

    /// Revoke permission from an agent
    pub fn revoke_permission(&mut self, agent_id: &str, permission: &Permission) -> bool {
        if let Some(permissions) = self.agent_permissions.get_mut(agent_id) {
            permissions.remove(permission)
        } else {
            false
        }
    }

    /// Check if agent has permission
    pub fn has_permission(&self, agent_id: &str, permission: &Permission) -> bool {
        self.agent_permissions
            .get(agent_id)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }

    /// Get all permissions for an agent
    pub fn get_permissions(&self, agent_id: &str) -> Vec<Permission> {
        self.agent_permissions
            .get(agent_id)
            .map(|perms| perms.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Verify agent has required permission
    pub fn verify_permission(
        &self,
        agent_id: &str,
        permission: &Permission,
    ) -> Result<(), GovernanceError> {
        if self.has_permission(agent_id, permission) {
            Ok(())
        } else {
            Err(GovernanceError::InvalidConfiguration(format!(
                "Agent {} does not have permission {:?}",
                agent_id, permission
            )))
        }
    }

    /// Sanitize and optionally validate an ID against a predicate.
    ///
    /// When `strict_validation` is enabled, `validator` is called on the
    /// sanitized value; if it returns `false` a `FormatValidationError` is
    /// returned. This consolidates the repeated sanitize-then-validate pattern
    /// used by the three ID-type helpers below.
    fn sanitize_and_validate<F>(
        &self,
        raw: &str,
        validator: F,
        expected_pattern: &str,
        field_name: &str,
    ) -> Result<String, GovernanceError>
    where
        F: Fn(&str) -> bool,
    {
        let sanitized = sanitize_id(raw);
        if self.strict_validation && !validator(&sanitized) {
            return Err(FormatValidationError::with_field(
                raw.to_string(),
                expected_pattern.to_string(),
                field_name.to_string(),
            )
            .into());
        }
        Ok(sanitized)
    }

    /// Validate and sanitize constitution ID
    pub fn validate_constitution_id(&self, id: &str) -> Result<String, GovernanceError> {
        self.sanitize_and_validate(id, validate_constitution_id, "cons.\\d+", "constitution_id")
    }

    /// Validate and sanitize law ID
    pub fn validate_law_id(&self, id: &str) -> Result<String, GovernanceError> {
        self.sanitize_and_validate(id, validate_law_id, "cons\\d+.law\\d+", "law_id")
    }

    /// Validate and sanitize rule ID
    pub fn validate_rule_id(&self, id: &str) -> Result<String, GovernanceError> {
        self.sanitize_and_validate(id, validate_rule_id, "rule.\\d+", "rule_id")
    }

    /// Validate cache size doesn't exceed limit
    pub fn validate_cache_size(&self, current_size: usize) -> Result<(), GovernanceError> {
        if current_size >= self.max_cache_size {
            Err(GovernanceError::InvalidConfiguration(format!(
                "Cache size {} exceeds maximum allowed size {}",
                current_size, self.max_cache_size
            )))
        } else {
            Ok(())
        }
    }

    /// Clear all permissions for an agent
    pub fn clear_agent_permissions(&mut self, agent_id: &str) {
        self.agent_permissions.remove(agent_id);
    }

    /// Clear all permissions
    pub fn clear_all_permissions(&mut self) {
        self.agent_permissions.clear();
    }

    /// Get number of agents with permissions
    pub fn agent_count(&self) -> usize {
        self.agent_permissions.len()
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::new()
    }
}
