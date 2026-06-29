//! Task definitions.

use serde::{Deserialize, Serialize};

use crate::agent::entity::TaskId;

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task unique identifier
    pub id: TaskId,

    /// Task name
    pub name: String,

    /// Task description
    pub description: Option<String>,

    /// Task parameters
    pub parameters: Vec<TaskParameter>,

    /// Task return type
    pub return_type: Option<String>,

    /// Task implementation (bytecode or AST reference)
    pub implementation: TaskImplementation,
}

impl Task {
    /// Create a new task
    pub fn new(id: TaskId, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            parameters: Vec::new(),
            return_type: None,
            implementation: TaskImplementation::Native,
        }
    }

    /// Add parameter
    pub fn add_parameter(&mut self, param: TaskParameter) {
        self.parameters.push(param);
    }
}

/// Task parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskParameter {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub param_type: Option<String>,

    /// Is parameter optional
    pub optional: bool,

    /// Default value
    pub default: Option<String>,
}

/// Task implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskImplementation {
    /// Native Rust implementation
    Native,

    /// HudHudScript bytecode
    Bytecode(Vec<u8>),

    /// AST reference (DEPRECATED — deserialized but always returns error)
    /// Use compile+VM pipeline instead.
    Ast(String),
}
