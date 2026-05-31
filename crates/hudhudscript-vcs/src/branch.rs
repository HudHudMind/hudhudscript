//! Branch data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Branch ID
pub type BranchId = Uuid;

/// Branch - A snapshot of system state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Branch {
    /// Unique branch ID
    pub id: BranchId,

    /// Branch name
    pub name: String,

    /// Parent branch (None for root)
    pub parent: Option<BranchId>,

    /// Branch state
    pub state: BranchState,

    /// Changes from parent (delta)
    pub changes: Vec<StateChange>,

    /// Metadata
    pub metadata: BranchMetadata,

    /// Version number
    pub version: u64,
}

impl Branch {
    /// Create new branch
    pub fn new(id: BranchId, name: String, parent: Option<BranchId>) -> Self {
        Self {
            id,
            name,
            parent,
            state: BranchState::default(),
            changes: Vec::new(),
            metadata: BranchMetadata::new(),
            version: 0,
        }
    }

    /// Add a change and apply it to the branch state
    pub fn add_change(&mut self, change: StateChange) {
        self.apply_change_to_state(&change);
        self.changes.push(change);
        self.version += 1;
        self.metadata.modified_at = SystemTime::now();
        self.metadata.change_count += 1;
        self.metadata.dirty = true;
    }

    /// Apply a single change to the branch state
    fn apply_change_to_state(&mut self, change: &StateChange) {
        match change {
            StateChange::EntityAdded { name, definition } => {
                self.state.entities.insert(name.clone(), definition.clone());
            }
            StateChange::EntityModified { name, new, .. } => {
                self.state.entities.insert(name.clone(), new.clone());
            }
            StateChange::EntityRemoved { name } => {
                self.state.entities.remove(name);
            }
            StateChange::AgentAdded { name, definition } => {
                self.state.agents.insert(name.clone(), definition.clone());
            }
            StateChange::AgentModified { name, new, .. } => {
                self.state.agents.insert(name.clone(), new.clone());
            }
            StateChange::ConfigChanged { key, new_value, .. } => {
                self.state.config.insert(key.clone(), new_value.clone());
            }
        }
    }

    /// Clear changes
    pub fn clear_changes(&mut self) {
        self.changes.clear();
        self.metadata.dirty = false;
    }
}

/// Branch State - Complete system state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BranchState {
    /// Entity definitions
    pub entities: HashMap<String, EntityDefinition>,

    /// Data definitions
    pub data_types: HashMap<String, DataDefinition>,

    /// Agent definitions
    pub agents: HashMap<String, AgentDefinition>,

    /// Configuration values
    pub config: HashMap<String, ConfigValue>,
}

/// State Change - Delta from parent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateChange {
    /// Entity definition added
    EntityAdded {
        name: String,
        definition: EntityDefinition,
    },

    /// Entity definition modified
    EntityModified {
        name: String,
        old: EntityDefinition,
        new: EntityDefinition,
    },

    /// Entity definition removed
    EntityRemoved { name: String },

    /// Agent added
    AgentAdded {
        name: String,
        definition: AgentDefinition,
    },

    /// Agent modified
    AgentModified {
        name: String,
        old: AgentDefinition,
        new: AgentDefinition,
    },

    /// Configuration changed
    ConfigChanged {
        key: String,
        old_value: ConfigValue,
        new_value: ConfigValue,
    },
}

/// Branch Metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BranchMetadata {
    /// Creation timestamp
    pub created_at: SystemTime,

    /// Last modified timestamp
    pub modified_at: SystemTime,

    /// Description
    pub description: Option<String>,

    /// Is dirty (has uncommitted changes)
    pub dirty: bool,

    /// Number of changes
    pub change_count: usize,
}

impl BranchMetadata {
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            created_at: now,
            modified_at: now,
            description: None,
            dirty: false,
            change_count: 0,
        }
    }
}

impl Default for BranchMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Entity Definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

/// Data Definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

/// Field Definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: String,
}

/// Agent Definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentDefinition {
    pub name: String,
    pub role: Option<String>,
}

/// Config Value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
}
