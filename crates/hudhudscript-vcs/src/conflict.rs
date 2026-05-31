//! Conflict detection and resolution

use serde::{Deserialize, Serialize};

/// Conflict between branches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Conflict {
    /// Type of conflict
    pub conflict_type: ConflictType,

    /// Path to conflicting item
    pub path: String,

    /// Source value
    pub source_value: Option<String>,

    /// Target value
    pub target_value: Option<String>,
}

impl Conflict {
    /// Create new conflict
    pub fn new(
        conflict_type: ConflictType,
        path: String,
        source_value: Option<String>,
        target_value: Option<String>,
    ) -> Self {
        Self {
            conflict_type,
            path,
            source_value,
            target_value,
        }
    }

    /// Check if conflict can be auto-resolved
    pub fn can_auto_resolve(&self) -> bool {
        match self.conflict_type {
            ConflictType::EntityModified => false,
            ConflictType::AgentModified => false,
            ConflictType::ConfigModified => true,
        }
    }
}

/// Type of conflict
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictType {
    /// Entity definition conflict
    EntityModified,

    /// Agent definition conflict
    AgentModified,

    /// Configuration conflict
    ConfigModified,
}
