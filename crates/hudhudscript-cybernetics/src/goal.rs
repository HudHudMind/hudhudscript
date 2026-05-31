use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;

/// A goal specifies the desired state of a system — the reference signal r(t).
///
/// Goals are intentionally kept simple: they carry a semantic description and
/// a priority.  The concrete *measurement* of whether a goal is achieved is the
/// responsibility of the `Observer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    /// Human-readable name of the goal.
    pub name: String,
    /// Detailed description of the desired outcome.
    pub description: String,
    /// Priority — higher values take precedence when multiple goals conflict.
    pub priority: i32,
    /// Optional deadline by which the goal must be achieved.
    #[serde(skip)]
    pub deadline: Option<Instant>,
}

impl Goal {
    /// Create a new goal.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            priority: 0,
            deadline: None,
        }
    }

    /// Set the priority.
    pub fn with_priority(mut self, p: i32) -> Self {
        self.priority = p;
        self
    }

    /// Set a deadline.
    pub fn with_deadline(mut self, d: Instant) -> Self {
        self.deadline = Some(d);
        self
    }

    /// Returns `true` if the deadline has passed.
    pub fn is_overdue(&self) -> bool {
        self.deadline.map(|d| Instant::now() > d).unwrap_or(false)
    }
}

impl fmt::Display for Goal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Goal({}, priority={})", self.name, self.priority)
    }
}
