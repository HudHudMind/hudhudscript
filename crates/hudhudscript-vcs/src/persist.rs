//! Disk persistence for VCS state
//!
//! Stores the full StateTree (branches, current branch, names) in the
//! `.hudhud/` directory as JSON files.
//!
//! Layout:
//!   .hudhud/
//!     vcs/
//!       state.json      — serialised snapshot (branches + current + names)

use crate::branch::{Branch, BranchId};
use crate::state_tree::{StateTree, VcsError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// On-disk representation of the entire StateTree.
#[derive(Debug, Serialize, Deserialize)]
struct Snapshot {
    /// All branches keyed by UUID string
    branches: HashMap<String, Branch>,
    /// Current active branch ID
    current: BranchId,
    /// Branch name -> ID mapping
    names: HashMap<String, BranchId>,
}

/// Resolve the VCS state file path from a project root.
fn state_path(project_root: &Path) -> PathBuf {
    project_root.join(".hudhud").join("vcs").join("state.json")
}

/// Ensure the `.hudhud/vcs/` directory exists.
fn ensure_dir(project_root: &Path) -> Result<(), VcsError> {
    let dir = project_root.join(".hudhud").join("vcs");
    fs::create_dir_all(&dir).map_err(|e| {
        VcsError::InvalidOperation(format!("Cannot create .hudhud/vcs directory: {}", e))
    })?;
    Ok(())
}

impl StateTree {
    /// Save the current state tree to disk under `project_root/.hudhud/vcs/state.json`.
    pub fn save(&self, project_root: &Path) -> Result<(), VcsError> {
        ensure_dir(project_root)?;

        let branches_guard = self.branches_read();
        let current_guard = self.current_read();
        let names_guard = self.names_read();

        let snapshot = Snapshot {
            branches: branches_guard
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            current: *current_guard,
            names: names_guard.clone(),
        };

        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| VcsError::InvalidOperation(format!("Serialization failed: {}", e)))?;

        fs::write(state_path(project_root), json)
            .map_err(|e| VcsError::InvalidOperation(format!("Cannot write state file: {}", e)))?;

        Ok(())
    }

    /// Load a state tree from disk. Returns `None` if no saved state exists.
    pub fn load(project_root: &Path) -> Result<Option<StateTree>, VcsError> {
        let path = state_path(project_root);
        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&path)
            .map_err(|e| VcsError::InvalidOperation(format!("Cannot read state file: {}", e)))?;

        let snapshot: Snapshot = serde_json::from_str(&json)
            .map_err(|e| VcsError::InvalidOperation(format!("Deserialization failed: {}", e)))?;

        // Rebuild the HashMap<BranchId, Branch> from String keys.
        let mut branches: HashMap<BranchId, Branch> = HashMap::new();
        for (key, branch) in snapshot.branches {
            let id: BranchId = key.parse().map_err(|e| {
                VcsError::InvalidOperation(format!("Invalid branch ID '{}': {}", key, e))
            })?;
            branches.insert(id, branch);
        }

        Ok(Some(StateTree::from_parts(
            branches,
            snapshot.current,
            snapshot.names,
        )))
    }

    /// Load from disk or create a fresh StateTree if nothing is persisted yet.
    pub fn load_or_create(project_root: &Path) -> Result<StateTree, VcsError> {
        match Self::load(project_root)? {
            Some(tree) => Ok(tree),
            None => {
                let tree = StateTree::new();
                tree.save(project_root)?;
                Ok(tree)
            }
        }
    }
}
