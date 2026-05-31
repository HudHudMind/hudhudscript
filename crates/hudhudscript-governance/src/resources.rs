//! Resource Sharing Management
//!
//! This module provides functionality for managing shared resources within communities,
//! including access control, usage tracking, and allocation policies.

use crate::types::{AgentId, ResourceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resource allocation policy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AllocationPolicy {
    /// First-come, first-served allocation
    FirstComeFirstServed,
    /// Priority-based allocation
    PriorityBased,
    /// Fair-share allocation
    FairShare,
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    pub id: ResourceId,
    pub name: String,
    pub description: Option<String>,
    pub capacity: u32,
    pub available: u32,
    pub allocation_policy: AllocationPolicy,
}

/// Resource usage record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceUsage {
    pub resource_id: ResourceId,
    pub agent_id: AgentId,
    pub amount: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Resource manager for community resource sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManager {
    resources: HashMap<ResourceId, Resource>,
    usage_records: Vec<ResourceUsage>,
    access_control: HashMap<ResourceId, Vec<AgentId>>,
}

impl Resource {
    /// Create a new resource
    ///
    /// # Arguments
    /// * `id` - Unique resource identifier
    /// * `name` - Human-readable resource name
    /// * `description` - Optional resource description
    /// * `capacity` - Total resource capacity
    /// * `allocation_policy` - Allocation policy for the resource
    ///
    /// # Returns
    /// * New `Resource` instance
    pub fn new(
        id: ResourceId,
        name: String,
        description: Option<String>,
        capacity: u32,
        allocation_policy: AllocationPolicy,
    ) -> Self {
        Self {
            id,
            name,
            description,
            capacity,
            available: capacity,
            allocation_policy,
        }
    }

    /// Allocate resource units
    ///
    /// # Arguments
    /// * `amount` - Amount to allocate
    ///
    /// # Returns
    /// * `Ok(())` if allocation succeeded
    /// * `Err(String)` if insufficient resources available
    pub fn allocate(&mut self, amount: u32) -> Result<(), String> {
        if amount > self.available {
            return Err(format!(
                "Insufficient resources: requested {}, available {}",
                amount, self.available
            ));
        }
        self.available -= amount;
        Ok(())
    }

    /// Release resource units
    ///
    /// # Arguments
    /// * `amount` - Amount to release
    ///
    /// # Returns
    /// * `Ok(())` if release succeeded
    /// * `Err(String)` if release would exceed capacity
    pub fn release(&mut self, amount: u32) -> Result<(), String> {
        if self.available + amount > self.capacity {
            return Err(format!(
                "Release would exceed capacity: current {}, release {}, capacity {}",
                self.available, amount, self.capacity
            ));
        }
        self.available += amount;
        Ok(())
    }

    /// Check if resource is available
    ///
    /// # Arguments
    /// * `amount` - Amount to check
    ///
    /// # Returns
    /// * `true` if amount is available, `false` otherwise
    pub fn is_available(&self, amount: u32) -> bool {
        amount <= self.available
    }
}

impl ResourceManager {
    /// Create a new resource manager
    ///
    /// # Returns
    /// * New `ResourceManager` instance
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            usage_records: Vec::new(),
            access_control: HashMap::new(),
        }
    }

    /// Register a new resource
    ///
    /// # Arguments
    /// * `resource` - Resource to register
    ///
    /// # Returns
    /// * `Ok(())` if registration succeeded
    /// * `Err(String)` if resource ID already exists
    pub fn register_resource(&mut self, resource: Resource) -> Result<(), String> {
        if self.resources.contains_key(&resource.id) {
            return Err(format!("Resource {} already exists", resource.id));
        }
        self.resources.insert(resource.id.clone(), resource);
        Ok(())
    }

    /// Grant access to a resource for an agent
    ///
    /// # Arguments
    /// * `resource_id` - Resource ID
    /// * `agent_id` - Agent ID to grant access
    ///
    /// # Returns
    /// * `Ok(())` if access granted successfully
    /// * `Err(String)` if resource not found or agent already has access
    pub fn grant_access(
        &mut self,
        resource_id: &ResourceId,
        agent_id: AgentId,
    ) -> Result<(), String> {
        if !self.resources.contains_key(resource_id) {
            return Err(format!("Resource {} not found", resource_id));
        }

        let access_list = self.access_control.entry(resource_id.clone()).or_default();
        if access_list.contains(&agent_id) {
            return Err(format!(
                "Agent {} already has access to resource {}",
                agent_id, resource_id
            ));
        }
        access_list.push(agent_id);
        Ok(())
    }

    /// Revoke access to a resource for an agent
    ///
    /// # Arguments
    /// * `resource_id` - Resource ID
    /// * `agent_id` - Agent ID to revoke access
    ///
    /// # Returns
    /// * `Ok(())` if access revoked successfully
    /// * `Err(String)` if resource not found or agent doesn't have access
    pub fn revoke_access(
        &mut self,
        resource_id: &ResourceId,
        agent_id: &AgentId,
    ) -> Result<(), String> {
        if !self.resources.contains_key(resource_id) {
            return Err(format!("Resource {} not found", resource_id));
        }

        let access_list = self
            .access_control
            .get_mut(resource_id)
            .ok_or_else(|| format!("No access control list for resource {}", resource_id))?;

        let initial_len = access_list.len();
        access_list.retain(|id| id != agent_id);
        if access_list.len() == initial_len {
            return Err(format!(
                "Agent {} doesn't have access to resource {}",
                agent_id, resource_id
            ));
        }
        Ok(())
    }

    /// Check if an agent has access to a resource
    ///
    /// # Arguments
    /// * `resource_id` - Resource ID
    /// * `agent_id` - Agent ID to check
    ///
    /// # Returns
    /// * `true` if agent has access, `false` otherwise
    pub fn has_access(&self, resource_id: &ResourceId, agent_id: &AgentId) -> bool {
        self.access_control
            .get(resource_id)
            .map(|list| list.contains(agent_id))
            .unwrap_or(false)
    }

    /// Allocate resource to an agent
    ///
    /// # Arguments
    /// * `resource_id` - Resource ID
    /// * `agent_id` - Agent ID requesting allocation
    /// * `amount` - Amount to allocate
    ///
    /// # Returns
    /// * `Ok(())` if allocation succeeded
    /// * `Err(String)` if resource not found, access denied, or insufficient resources
    pub fn allocate_resource(
        &mut self,
        resource_id: &ResourceId,
        agent_id: &AgentId,
        amount: u32,
    ) -> Result<(), String> {
        // Check access control
        if !self.has_access(resource_id, agent_id) {
            return Err(format!(
                "Agent {} doesn't have access to resource {}",
                agent_id, resource_id
            ));
        }

        // Get resource and allocate
        let resource = self
            .resources
            .get_mut(resource_id)
            .ok_or_else(|| format!("Resource {} not found", resource_id))?;

        resource.allocate(amount)?;

        // Record usage
        self.usage_records.push(ResourceUsage {
            resource_id: resource_id.clone(),
            agent_id: agent_id.clone(),
            amount,
            timestamp: chrono::Utc::now(),
        });

        Ok(())
    }

    /// Release resource from an agent
    ///
    /// # Arguments
    /// * `resource_id` - Resource ID
    /// * `agent_id` - Agent ID releasing resource
    /// * `amount` - Amount to release
    ///
    /// # Returns
    /// * `Ok(())` if release succeeded
    /// * `Err(String)` if resource not found or release failed
    pub fn release_resource(
        &mut self,
        resource_id: &ResourceId,
        agent_id: &AgentId,
        amount: u32,
    ) -> Result<(), String> {
        let resource = self
            .resources
            .get_mut(resource_id)
            .ok_or_else(|| format!("Resource {} not found", resource_id))?;

        resource.release(amount)?;

        // Record the release event with the actual amount returned
        self.usage_records.push(ResourceUsage {
            resource_id: resource_id.clone(),
            agent_id: agent_id.clone(),
            amount,
            timestamp: chrono::Utc::now(),
        });

        Ok(())
    }

    /// Get usage records for a specific agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to query
    ///
    /// # Returns
    /// * Vector of usage records for the agent
    pub fn get_agent_usage(&self, agent_id: &AgentId) -> Vec<&ResourceUsage> {
        self.usage_records
            .iter()
            .filter(|record| &record.agent_id == agent_id)
            .collect()
    }

    /// Get usage records for a specific resource
    ///
    /// # Arguments
    /// * `resource_id` - Resource ID to query
    ///
    /// # Returns
    /// * Vector of usage records for the resource
    pub fn get_resource_usage(&self, resource_id: &ResourceId) -> Vec<&ResourceUsage> {
        self.usage_records
            .iter()
            .filter(|record| &record.resource_id == resource_id)
            .collect()
    }

    /// Get total usage by an agent for a specific resource
    ///
    /// # Arguments
    /// * `resource_id` - Resource ID
    /// * `agent_id` - Agent ID
    ///
    /// # Returns
    /// * Total amount used by the agent
    pub fn get_total_usage(&self, resource_id: &ResourceId, agent_id: &AgentId) -> u32 {
        self.usage_records
            .iter()
            .filter(|record| &record.resource_id == resource_id && &record.agent_id == agent_id)
            .map(|record| record.amount)
            .sum()
    }

    /// Validate that all resource IDs exist
    ///
    /// # Arguments
    /// * `resource_ids` - List of resource IDs to validate
    ///
    /// # Returns
    /// * `Ok(())` if all resources exist
    /// * `Err(String)` with list of missing resource IDs
    pub fn validate_resources(&self, resource_ids: &[ResourceId]) -> Result<(), String> {
        let missing: Vec<_> = resource_ids
            .iter()
            .filter(|id| !self.resources.contains_key(*id))
            .collect();

        if !missing.is_empty() {
            return Err(format!("Missing resources: {:?}", missing));
        }

        Ok(())
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
