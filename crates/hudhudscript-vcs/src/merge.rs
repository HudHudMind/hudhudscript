//! Merge strategies and operations

use crate::branch::StateChange;
use crate::conflict::Conflict;
use serde::{Deserialize, Serialize};

/// Merge Strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MergeStrategy {
    /// Fast-forward merge (no conflicts possible)
    FastForward,

    /// Three-way merge
    ThreeWay,

    /// Consensus-based merge (requires council approval)
    Consensus { council: String, quorum: QuorumType },
}

/// Quorum Type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuorumType {
    /// Simple majority (>50%)
    Majority,

    /// Unanimous (100%)
    Unanimous,

    /// Custom threshold (n out of m)
    Threshold(usize, usize),
}

impl QuorumType {
    /// Check if votes meet quorum
    pub fn meets_quorum(&self, votes: usize, total: usize) -> bool {
        match self {
            QuorumType::Majority => votes > total / 2,
            QuorumType::Unanimous => votes == total,
            QuorumType::Threshold(required, _) => votes >= *required,
        }
    }
}

/// Merge Result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MergeResult {
    /// Was merge successful
    pub success: bool,

    /// Conflicts found
    pub conflicts: Vec<Conflict>,

    /// Changes that were merged
    pub merged_changes: Vec<StateChange>,
}

impl MergeResult {
    /// Create successful merge result
    pub fn success(merged_changes: Vec<StateChange>) -> Self {
        Self {
            success: true,
            conflicts: Vec::new(),
            merged_changes,
        }
    }

    /// Create failed merge result with conflicts
    pub fn failure(conflicts: Vec<Conflict>) -> Self {
        Self {
            success: false,
            conflicts,
            merged_changes: Vec::new(),
        }
    }

    /// Check if merge has conflicts
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }
}
