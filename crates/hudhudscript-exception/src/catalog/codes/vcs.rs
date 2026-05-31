use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum VcsExceptionCode {
    /// E0319 — Branch name is already in use
    VcsBranchAlreadyExists = 319,
    /// E0320 — Branch does not exist in repository
    VcsBranchNotFound = 320,
    /// E0321 — VCS operation is not allowed in current state
    VcsInvalidOperation = 321,
    /// E0322 — Merge produced conflicts requiring resolution
    VcsMergeConflict = 322,
    /// E0323 — Failed to parse VCS output
    VcsParseError = 323,
}
