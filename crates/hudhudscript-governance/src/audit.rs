//! Audit logging for governance operations
//!
//! This module provides comprehensive audit logging for the governance system,
//! tracking enforcement decisions, constitution modifications, agent role changes,
//! and cache access patterns. All logs include timestamps and can be exported to JSON.
//!
//! **Validates Requirements:** 6.5, 14.1, 14.2, 14.3, 14.4, 14.5

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::types::{AgentId, AgentRole, ConstitutionId, LawId};

/// Type of audit event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    /// Enforcement decision was made
    EnforcementDecision,
    /// Constitution was modified
    ConstitutionModified,
    /// Agent role was changed
    AgentRoleChanged,
    /// Cache was accessed
    CacheAccess,
}

/// Audit event entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Type of event
    pub event_type: AuditEventType,
    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,
    /// Event-specific data
    pub data: AuditEventData,
}

/// Event-specific audit data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum AuditEventData {
    /// Enforcement decision details
    Enforcement {
        constitution_id: ConstitutionId,
        action_description: String,
        allowed: bool,
        violations: Vec<LawId>,
        advisory_violations: Vec<LawId>,
    },
    /// Constitution modification details
    ConstitutionModification {
        constitution_id: ConstitutionId,
        modification_type: String,
        old_version: u32,
        new_version: u32,
        description: String,
    },
    /// Agent role change details
    RoleChange {
        agent_id: AgentId,
        old_role: Option<AgentRole>,
        new_role: AgentRole,
        council_id: String,
    },
    /// Cache access details
    CacheAccess {
        operation: String,
        cache_key: String,
        hit: bool,
    },
}

/// Audit logger for governance operations
#[derive(Debug, Clone)]
pub struct AuditLogger {
    events: Arc<Mutex<Vec<AuditEvent>>>,
    next_id: Arc<Mutex<u64>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Generate next event ID
    fn next_event_id(&self) -> String {
        let mut id = self.next_id.lock().unwrap();
        let event_id = format!("audit.{}", *id);
        *id += 1;
        event_id
    }

    /// Log an enforcement decision
    pub fn log_enforcement_decision(
        &self,
        constitution_id: ConstitutionId,
        action_description: String,
        allowed: bool,
        violations: Vec<LawId>,
        advisory_violations: Vec<LawId>,
    ) {
        let event = AuditEvent {
            id: self.next_event_id(),
            event_type: AuditEventType::EnforcementDecision,
            timestamp: Utc::now(),
            data: AuditEventData::Enforcement {
                constitution_id,
                action_description,
                allowed,
                violations,
                advisory_violations,
            },
        };

        self.events.lock().unwrap().push(event);
    }

    /// Log a constitution modification
    pub fn log_constitution_modification(
        &self,
        constitution_id: ConstitutionId,
        modification_type: String,
        old_version: u32,
        new_version: u32,
        description: String,
    ) {
        let event = AuditEvent {
            id: self.next_event_id(),
            event_type: AuditEventType::ConstitutionModified,
            timestamp: Utc::now(),
            data: AuditEventData::ConstitutionModification {
                constitution_id,
                modification_type,
                old_version,
                new_version,
                description,
            },
        };

        self.events.lock().unwrap().push(event);
    }

    /// Log an agent role change
    pub fn log_agent_role_change(
        &self,
        agent_id: AgentId,
        old_role: Option<AgentRole>,
        new_role: AgentRole,
        council_id: String,
    ) {
        let event = AuditEvent {
            id: self.next_event_id(),
            event_type: AuditEventType::AgentRoleChanged,
            timestamp: Utc::now(),
            data: AuditEventData::RoleChange {
                agent_id,
                old_role,
                new_role,
                council_id,
            },
        };

        self.events.lock().unwrap().push(event);
    }

    /// Log a cache access
    pub fn log_cache_access(&self, operation: String, cache_key: String, hit: bool) {
        let event = AuditEvent {
            id: self.next_event_id(),
            event_type: AuditEventType::CacheAccess,
            timestamp: Utc::now(),
            data: AuditEventData::CacheAccess {
                operation,
                cache_key,
                hit,
            },
        };

        self.events.lock().unwrap().push(event);
    }

    /// Get all audit events
    pub fn get_events(&self) -> Vec<AuditEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Get events filtered by type
    pub fn get_events_by_type(&self, event_type: AuditEventType) -> Vec<AuditEvent> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get events within a time range
    pub fn get_events_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<AuditEvent> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Export audit logs to JSON
    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        let events = self.events.lock().unwrap();
        serde_json::to_string_pretty(&*events)
    }

    /// Export audit logs to compact JSON
    pub fn export_to_json_compact(&self) -> Result<String, serde_json::Error> {
        let events = self.events.lock().unwrap();
        serde_json::to_string(&*events)
    }

    /// Clear all audit logs
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }

    /// Get the number of audit events
    pub fn count(&self) -> usize {
        self.events.lock().unwrap().len()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
