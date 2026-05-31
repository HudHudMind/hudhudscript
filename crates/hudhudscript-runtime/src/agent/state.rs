//! Agent state and state values.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::agent::entity::AgentId;

/// Agent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Agent ID
    pub agent_id: AgentId,

    /// State variables
    pub variables: HashMap<String, StateValue>,

    /// State version (for optimistic locking)
    pub version: u64,

    /// Last updated timestamp
    pub updated_at: SystemTime,
}

impl AgentState {
    /// Create new agent state
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            variables: HashMap::new(),
            version: 0,
            updated_at: SystemTime::now(),
        }
    }

    /// Set variable
    pub fn set(&mut self, name: String, value: StateValue) {
        self.variables.insert(name, value);
        self.version += 1;
        self.updated_at = SystemTime::now();
    }

    /// Get variable
    pub fn get(&self, name: &str) -> Option<&StateValue> {
        self.variables.get(name)
    }

    /// Remove variable
    pub fn remove(&mut self, name: &str) -> Option<StateValue> {
        self.version += 1;
        self.updated_at = SystemTime::now();
        self.variables.remove(name)
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.variables.clear();
        self.version += 1;
        self.updated_at = SystemTime::now();
    }
}

/// State value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateValue {
    /// String value
    String(String),

    /// Number value
    Number(f64),

    /// Boolean value
    Boolean(bool),

    /// Null value
    Null,

    /// Array value
    Array(Vec<StateValue>),

    /// Object value
    Object(HashMap<String, StateValue>),
}
