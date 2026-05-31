//! Governance types (Constitution, Law, Council)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    AgentId, Condition, ConstitutionId, CouncilId, GovernanceModel, LawId, PermissionStr, RuleId,
};

/// Constitution with integer ID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constitution {
    pub id: ConstitutionId,
    pub name: String,
    pub description: Option<String>,
    pub laws: HashMap<LawId, Law>,
    pub created_at: DateTime<Utc>,
    pub version: u32,
}

/// Law with integer ID within constitution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Law {
    pub id: LawId,
    pub constitution_id: ConstitutionId,
    pub name: String,
    pub description: String,
    pub enforcement_level: EnforcementLevel,
    pub conditions: Vec<Condition>,
}

/// Enforcement level for laws
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnforcementLevel {
    /// Must be followed
    Mandatory,
    /// Should be followed
    Advisory,
    /// May be followed
    Optional,
}

/// Council composed of agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Council {
    pub id: CouncilId,
    pub name: String,
    pub constitution_id: ConstitutionId,
    pub members: Vec<AgentMember>,
    pub rules: Vec<RuleId>,
    pub state: CouncilState,
    /// Governance model defining how rules are enforced (optional)
    pub governance_model: Option<GovernanceModel>,
}

/// Agent member with role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentMember {
    pub agent_id: AgentId,
    pub role: AgentRole,
    pub joined_at: DateTime<Utc>,
    pub permissions: Vec<PermissionStr>,
}

/// Agent role within a council
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentRole {
    /// Proposes actions
    Prosecutor,
    /// Evaluates compliance
    Judge,
    /// Executes decisions
    Executor,
    /// General member
    Member,
    /// Custom role with name
    Custom(String),
}

/// Council state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CouncilState {
    pub active: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}
