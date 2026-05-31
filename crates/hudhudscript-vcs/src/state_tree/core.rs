use crate::branch::{Branch, BranchId, StateChange};
use crate::state_tree::error::VcsError;
use crate::state_tree::info::BranchInfo;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// State Tree - Root of the version control system
pub struct StateTree {
    pub(crate) branches: Arc<RwLock<HashMap<BranchId, Branch>>>,
    pub(crate) current: Arc<RwLock<BranchId>>,
    pub(crate) names: Arc<RwLock<HashMap<String, BranchId>>>,
}

impl StateTree {
    pub fn from_parts(
        branches: HashMap<BranchId, Branch>,
        current: BranchId,
        names: HashMap<String, BranchId>,
    ) -> Self {
        Self {
            branches: Arc::new(RwLock::new(branches)),
            current: Arc::new(RwLock::new(current)),
            names: Arc::new(RwLock::new(names)),
        }
    }

    pub(crate) fn branches_read(
        &self,
    ) -> std::sync::RwLockReadGuard<'_, HashMap<BranchId, Branch>> {
        self.branches.read().expect("internal: mutex poisoned")
    }

    pub(crate) fn current_read(&self) -> std::sync::RwLockReadGuard<'_, BranchId> {
        self.current.read().expect("internal: mutex poisoned")
    }

    pub(crate) fn names_read(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, BranchId>> {
        self.names.read().expect("internal: mutex poisoned")
    }

    pub fn new() -> Self {
        let root_id = Uuid::new_v4();
        let root = Branch::new(root_id, "level_1".to_string(), None);
        let mut branches = HashMap::new();
        branches.insert(root_id, root);
        let mut names = HashMap::new();
        names.insert("level_1".to_string(), root_id);
        Self {
            branches: Arc::new(RwLock::new(branches)),
            current: Arc::new(RwLock::new(root_id)),
            names: Arc::new(RwLock::new(names)),
        }
    }

    pub fn create_branch(
        &self,
        name: String,
        parent: Option<BranchId>,
    ) -> Result<BranchId, VcsError> {
        {
            let names = self.names.read().expect("internal: mutex poisoned");
            if names.contains_key(&name) {
                return Err(VcsError::BranchAlreadyExists(name));
            }
        }
        let parent_id =
            parent.unwrap_or_else(|| *self.current.read().expect("internal: mutex poisoned"));
        let parent_branch = {
            let branches = self.branches.read().expect("internal: mutex poisoned");
            branches
                .get(&parent_id)
                .ok_or(VcsError::BranchNotFound(parent_id.to_string()))?
                .clone()
        };
        let new_id = Uuid::new_v4();
        let mut new_branch = Branch::new(new_id, name.clone(), Some(parent_id));
        new_branch.state = parent_branch.state.clone();
        self.branches
            .write()
            .expect("internal: mutex poisoned")
            .insert(new_id, new_branch);
        self.names
            .write()
            .expect("internal: mutex poisoned")
            .insert(name, new_id);
        Ok(new_id)
    }

    pub fn checkout(&self, branch_name: &str) -> Result<(), VcsError> {
        let branch_id = {
            let names = self.names.read().expect("internal: mutex poisoned");
            *names
                .get(branch_name)
                .ok_or_else(|| VcsError::BranchNotFound(branch_name.to_string()))?
        };
        {
            let branches = self.branches.read().expect("internal: mutex poisoned");
            if !branches.contains_key(&branch_id) {
                return Err(VcsError::BranchNotFound(branch_name.to_string()));
            }
        }
        *self.current.write().expect("internal: mutex poisoned") = branch_id;
        Ok(())
    }

    pub fn current_branch(&self) -> Result<Branch, VcsError> {
        let current_id = *self.current.read().expect("internal: mutex poisoned");
        let branches = self.branches.read().expect("internal: mutex poisoned");
        branches
            .get(&current_id)
            .cloned()
            .ok_or_else(|| VcsError::BranchNotFound(current_id.to_string()))
    }

    pub fn get_branch(&self, name: &str) -> Option<Branch> {
        let names = self.names.read().expect("internal: mutex poisoned");
        let id = names.get(name)?;
        let branches = self.branches.read().expect("internal: mutex poisoned");
        branches.get(id).cloned()
    }

    pub fn record_change(&self, change: StateChange) -> Result<(), VcsError> {
        let current_id = *self.current.read().expect("internal: mutex poisoned");
        let mut branches = self.branches.write().expect("internal: mutex poisoned");
        let branch = branches
            .get_mut(&current_id)
            .ok_or(VcsError::BranchNotFound(current_id.to_string()))?;
        branch.add_change(change);
        Ok(())
    }

    pub fn list_branches(&self) -> Vec<BranchInfo> {
        let branches = self.branches.read().expect("internal: mutex poisoned");
        branches
            .values()
            .map(|b| BranchInfo {
                id: b.id,
                name: b.name.clone(),
                parent: b.parent,
                version: b.version,
                change_count: b.metadata.change_count,
            })
            .collect()
    }

    pub fn delete_branch(&self, name: &str) -> Result<(), VcsError> {
        if name == "level_1" {
            return Err(VcsError::InvalidOperation(
                "Cannot delete root branch".to_string(),
            ));
        }
        let current_name = self.current_branch()?.name;
        if name == current_name {
            return Err(VcsError::InvalidOperation(
                "Cannot delete current branch".to_string(),
            ));
        }
        let branch_id = {
            let names = self.names.read().expect("internal: mutex poisoned");
            *names
                .get(name)
                .ok_or_else(|| VcsError::BranchNotFound(name.to_string()))?
        };
        self.branches
            .write()
            .expect("internal: mutex poisoned")
            .remove(&branch_id);
        self.names
            .write()
            .expect("internal: mutex poisoned")
            .remove(name);
        Ok(())
    }
}

impl Default for StateTree {
    fn default() -> Self {
        Self::new()
    }
}
