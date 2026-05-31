use crate::branch::Branch;
use crate::conflict::{Conflict, ConflictType};
use crate::merge::{MergeResult, MergeStrategy};
use crate::state_tree::core::StateTree;
use crate::state_tree::error::VcsError;

impl StateTree {
    pub fn merge(
        &self,
        source_name: &str,
        target_name: &str,
        strategy: MergeStrategy,
    ) -> Result<MergeResult, VcsError> {
        let (source_id, target_id) = {
            let names = self.names.read().expect("internal: mutex poisoned");
            let source = *names
                .get(source_name)
                .ok_or_else(|| VcsError::BranchNotFound(source_name.to_string()))?;
            let target = *names
                .get(target_name)
                .ok_or_else(|| VcsError::BranchNotFound(target_name.to_string()))?;
            (source, target)
        };

        let (source, target) = {
            let branches = self.branches.read().expect("internal: mutex poisoned");
            let s = branches
                .get(&source_id)
                .ok_or(VcsError::BranchNotFound(source_name.to_string()))?
                .clone();
            let t = branches
                .get(&target_id)
                .ok_or(VcsError::BranchNotFound(target_name.to_string()))?
                .clone();
            (s, t)
        };

        match strategy {
            MergeStrategy::FastForward => {
                let source_change_keys: Vec<String> = source
                    .changes
                    .iter()
                    .map(|c| serde_json::to_string(c).unwrap_or_default())
                    .collect();
                let target_diverged: Vec<_> = target
                    .changes
                    .iter()
                    .filter(|c| {
                        let key = serde_json::to_string(c).unwrap_or_default();
                        !source_change_keys.contains(&key)
                    })
                    .collect();
                if !target_diverged.is_empty() {
                    let conflicts = vec![Conflict::new(
                        ConflictType::EntityModified,
                        format!(
                            "fast-forward blocked: target '{}' has {} divergent change(s)",
                            target_name,
                            target_diverged.len()
                        ),
                        None,
                        None,
                    )];
                    return Ok(MergeResult::failure(conflicts));
                }

                let target_change_keys: Vec<String> = target
                    .changes
                    .iter()
                    .map(|c| serde_json::to_string(c).unwrap_or_default())
                    .collect();
                let new_changes: Vec<_> = source
                    .changes
                    .iter()
                    .filter(|c| {
                        let key = serde_json::to_string(c).unwrap_or_default();
                        !target_change_keys.contains(&key)
                    })
                    .cloned()
                    .collect();

                {
                    let mut branches = self.branches.write().expect("internal: mutex poisoned");
                    let target_mut = branches
                        .get_mut(&target_id)
                        .ok_or(VcsError::BranchNotFound(target_name.to_string()))?;
                    for change in &new_changes {
                        target_mut.add_change(change.clone());
                    }
                }

                Ok(MergeResult::success(new_changes))
            }

            MergeStrategy::ThreeWay => {
                let conflicts = self.detect_conflicts(&source, &target);
                if !conflicts.is_empty() {
                    return Ok(MergeResult::failure(conflicts));
                }

                let target_change_keys: Vec<String> = target
                    .changes
                    .iter()
                    .map(|c| serde_json::to_string(c).unwrap_or_default())
                    .collect();
                let new_changes: Vec<_> = source
                    .changes
                    .iter()
                    .filter(|c| {
                        let key = serde_json::to_string(c).unwrap_or_default();
                        !target_change_keys.contains(&key)
                    })
                    .cloned()
                    .collect();

                {
                    let mut branches = self.branches.write().expect("internal: mutex poisoned");
                    let target_mut = branches
                        .get_mut(&target_id)
                        .ok_or(VcsError::BranchNotFound(target_name.to_string()))?;
                    for change in &new_changes {
                        target_mut.add_change(change.clone());
                    }
                }

                Ok(MergeResult::success(new_changes))
            }

            MergeStrategy::Consensus { council, quorum } => {
                let pending_msg = format!(
                    "Consensus merge from '{}' into '{}' requires council '{}' approval ({:?})",
                    source_name, target_name, council, quorum
                );
                Err(VcsError::MergeConflict(pending_msg))
            }
        }
    }

    fn detect_conflicts(&self, source: &Branch, target: &Branch) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        for (name, source_entity) in &source.state.entities {
            if let Some(target_entity) = target.state.entities.get(name) {
                if source_entity != target_entity {
                    conflicts.push(Conflict::new(
                        ConflictType::EntityModified,
                        name.clone(),
                        Some(format!("{:?}", source_entity)),
                        Some(format!("{:?}", target_entity)),
                    ));
                }
            }
        }

        for (name, source_agent) in &source.state.agents {
            if let Some(target_agent) = target.state.agents.get(name) {
                if source_agent != target_agent {
                    conflicts.push(Conflict::new(
                        ConflictType::AgentModified,
                        name.clone(),
                        Some(format!("{:?}", source_agent)),
                        Some(format!("{:?}", target_agent)),
                    ));
                }
            }
        }

        for (key, source_val) in &source.state.config {
            if let Some(target_val) = target.state.config.get(key) {
                if source_val != target_val {
                    conflicts.push(Conflict::new(
                        ConflictType::ConfigModified,
                        key.clone(),
                        Some(format!("{:?}", source_val)),
                        Some(format!("{:?}", target_val)),
                    ));
                }
            }
        }

        conflicts
    }
}
