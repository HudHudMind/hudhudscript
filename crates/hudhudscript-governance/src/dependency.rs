//! Circular dependency detection for constitutions
//!
//! This module provides functionality to detect circular dependencies when constitutions
//! reference each other. It ensures that constitution hierarchies remain acyclic and
//! supports arbitrary depth validation.

use crate::types::{Constitution, ConstitutionId};
use std::collections::{HashMap, HashSet};

/// Result type for dependency operations
pub type DependencyResult<T> = Result<T, CircularDependencyError>;

/// Error indicating a circular dependency was detected
#[derive(Debug, Clone, PartialEq)]
pub struct CircularDependencyError {
    /// The chain of constitution IDs that form the circular dependency
    pub dependency_chain: Vec<ConstitutionId>,
}

impl CircularDependencyError {
    /// Create a new circular dependency error with the given chain
    pub fn new(dependency_chain: Vec<ConstitutionId>) -> Self {
        Self { dependency_chain }
    }

    /// Get a formatted error message showing the circular dependency
    pub fn message(&self) -> String {
        format!(
            "Circular dependency detected: {}",
            self.dependency_chain.join(" -> ")
        )
    }
}

impl std::fmt::Display for CircularDependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for CircularDependencyError {}

/// Dependency graph for constitutions
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Map of constitution ID to its direct dependencies (constitutions it references)
    dependencies: HashMap<ConstitutionId, Vec<ConstitutionId>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    /// Add a constitution with its dependencies to the graph
    ///
    /// # Arguments
    /// * `constitution_id` - The ID of the constitution
    /// * `references` - List of constitution IDs that this constitution references
    pub fn add_constitution(
        &mut self,
        constitution_id: ConstitutionId,
        references: Vec<ConstitutionId>,
    ) {
        self.dependencies.insert(constitution_id, references);
    }

    /// Check if adding a new constitution with the given references would create a cycle
    ///
    /// # Arguments
    /// * `constitution_id` - The ID of the constitution to add
    /// * `references` - List of constitution IDs that this constitution would reference
    ///
    /// # Returns
    /// * `Ok(())` if no circular dependency would be created
    /// * `Err(CircularDependencyError)` with the dependency chain if a cycle is detected
    pub fn validate_no_cycle(
        &self,
        constitution_id: &ConstitutionId,
        references: &[ConstitutionId],
    ) -> DependencyResult<()> {
        // Create a temporary graph with the new constitution added
        let mut temp_graph = self.clone();
        temp_graph.add_constitution(constitution_id.clone(), references.to_vec());

        // Check for cycles starting from the new constitution
        temp_graph.detect_cycle_from(constitution_id)
    }

    /// Detect if there's a cycle starting from the given constitution
    ///
    /// Uses depth-first search with a visited set to detect cycles.
    ///
    /// # Arguments
    /// * `start_id` - The constitution ID to start the search from
    ///
    /// # Returns
    /// * `Ok(())` if no cycle is detected
    /// * `Err(CircularDependencyError)` with the dependency chain if a cycle is found
    fn detect_cycle_from(&self, start_id: &ConstitutionId) -> DependencyResult<()> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        self.dfs_cycle_detection(start_id, &mut visited, &mut path)
    }

    /// Depth-first search for cycle detection
    ///
    /// # Arguments
    /// * `current_id` - Current constitution being visited
    /// * `visited` - Set of constitutions visited in the current path
    /// * `path` - Current path of constitution IDs
    ///
    /// # Returns
    /// * `Ok(())` if no cycle is detected
    /// * `Err(CircularDependencyError)` if a cycle is found
    fn dfs_cycle_detection(
        &self,
        current_id: &ConstitutionId,
        visited: &mut HashSet<ConstitutionId>,
        path: &mut Vec<ConstitutionId>,
    ) -> DependencyResult<()> {
        // If we've already visited this node in the current path, we have a cycle
        if visited.contains(current_id) {
            // Find where the cycle starts in the path
            let cycle_start = path.iter().position(|id| id == current_id).unwrap_or(0);
            let mut cycle_chain = path[cycle_start..].to_vec();
            cycle_chain.push(current_id.clone());
            return Err(CircularDependencyError::new(cycle_chain));
        }

        // Mark as visited and add to path
        visited.insert(current_id.clone());
        path.push(current_id.clone());

        // Visit all dependencies
        if let Some(deps) = self.dependencies.get(current_id) {
            for dep_id in deps {
                self.dfs_cycle_detection(dep_id, visited, path)?;
            }
        }

        // Remove from visited set and path when backtracking
        visited.remove(current_id);
        path.pop();

        Ok(())
    }

    /// Validate the entire graph for any cycles
    ///
    /// This checks all constitutions in the graph, not just from a single starting point.
    ///
    /// # Returns
    /// * `Ok(())` if no cycles are detected
    /// * `Err(CircularDependencyError)` if any cycle is found
    pub fn validate_all(&self) -> DependencyResult<()> {
        let all_ids: Vec<ConstitutionId> = self.dependencies.keys().cloned().collect();

        for id in all_ids {
            self.detect_cycle_from(&id)?;
        }

        Ok(())
    }

    /// Get the depth of the dependency hierarchy for a given constitution
    ///
    /// Returns the maximum depth of dependencies (0 if no dependencies).
    ///
    /// # Arguments
    /// * `constitution_id` - The constitution to check
    ///
    /// # Returns
    /// * Maximum depth of the dependency tree
    pub fn get_depth(&self, constitution_id: &ConstitutionId) -> usize {
        self.calculate_depth(constitution_id, &mut HashSet::new())
    }

    /// Recursively calculate the depth of dependencies
    fn calculate_depth(
        &self,
        constitution_id: &ConstitutionId,
        visited: &mut HashSet<ConstitutionId>,
    ) -> usize {
        // Prevent infinite recursion in case of cycles
        if visited.contains(constitution_id) {
            return 0;
        }

        visited.insert(constitution_id.clone());

        let max_child_depth = self
            .dependencies
            .get(constitution_id)
            .map(|deps| {
                deps.iter()
                    .map(|dep_id| self.calculate_depth(dep_id, visited))
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        visited.remove(constitution_id);

        if max_child_depth > 0 {
            max_child_depth + 1
        } else if self
            .dependencies
            .get(constitution_id)
            .is_some_and(|deps| !deps.is_empty())
        {
            1
        } else {
            0
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract constitution references from a constitution.
///
/// Walks the constitution's description and laws to find identifiers that
/// reference other constitutions. A reference is detected when:
///
/// 1. The description contains `extends <ConstitutionId>` or
///    `includes <ConstitutionId>`.
/// 2. A law's description contains `extends <ConstitutionId>` or
///    `includes <ConstitutionId>`.
/// 3. A law's condition fields reference constitution IDs
///    (fields with names ending in `_constitution` or `_constitution_id`).
///
/// # Arguments
/// * `constitution` - The constitution to extract references from
///
/// # Returns
/// * Vector of constitution IDs that this constitution references
pub fn extract_constitution_references(constitution: &Constitution) -> Vec<ConstitutionId> {
    let mut refs = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Helper: extract "extends <id>" and "includes <id>" from text
    let extract_from_text =
        |text: &str,
         refs: &mut Vec<ConstitutionId>,
         seen: &mut std::collections::HashSet<ConstitutionId>| {
            for keyword in &["extends", "includes"] {
                let parts: Vec<&str> = text.split(keyword).collect();
                // Skip the first part (text before the keyword); process parts after each keyword occurrence
                for part in parts.iter().skip(1) {
                    // Take the next whitespace-delimited token after the keyword
                    if let Some(id) = part.split_whitespace().next() {
                        // Validate it looks like a constitution ID (e.g., "cons.1", alphanumeric with dots)
                        let trimmed =
                            id.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '_');
                        if !trimmed.is_empty()
                            && trimmed != constitution.id
                            && seen.insert(trimmed.to_string())
                        {
                            refs.push(trimmed.to_string());
                        }
                    }
                }
            }
        };

    // 1. Check the description for extends/includes references
    if let Some(desc) = &constitution.description {
        extract_from_text(desc, &mut refs, &mut seen);
    }

    // 2. Walk all laws
    for law in constitution.laws.values() {
        // Check law description
        extract_from_text(&law.description, &mut refs, &mut seen);

        // 3. Check law conditions for constitution ID references
        for condition in &law.conditions {
            extract_condition_refs(condition, &constitution.id, &mut refs, &mut seen);
        }
    }

    refs
}

/// Recursively extract constitution references from a Condition tree.
fn extract_condition_refs(
    condition: &crate::types::Condition,
    self_id: &ConstitutionId,
    refs: &mut Vec<ConstitutionId>,
    seen: &mut std::collections::HashSet<ConstitutionId>,
) {
    use crate::types::Condition;
    match condition {
        Condition::Equals { field, value } | Condition::NotEquals { field, value } => {
            // If the field name suggests a constitution reference, extract the value
            if field.contains("constitution") {
                if let Some(id) = value.as_str() {
                    if id != self_id && seen.insert(id.to_string()) {
                        refs.push(id.to_string());
                    }
                }
            }
        }
        Condition::In { field, values } => {
            if field.contains("constitution") {
                for v in values {
                    if let Some(id) = v.as_str() {
                        if id != self_id && seen.insert(id.to_string()) {
                            refs.push(id.to_string());
                        }
                    }
                }
            }
        }
        Condition::And(children) | Condition::Or(children) => {
            for child in children {
                extract_condition_refs(child, self_id, refs, seen);
            }
        }
        Condition::Not(child) => {
            extract_condition_refs(child, self_id, refs, seen);
        }
        // GreaterThan, LessThan, Between don't carry string IDs
        _ => {}
    }
}

/// Validate that a constitution doesn't create circular dependencies
///
/// This should be called before storing a constitution in the cache.
///
/// # Arguments
/// * `graph` - The current dependency graph
/// * `constitution` - The constitution to validate
///
/// # Returns
/// * `Ok(())` if no circular dependency would be created
/// * `Err(CircularDependencyError)` if adding this constitution would create a cycle
pub fn validate_constitution_dependencies(
    graph: &DependencyGraph,
    constitution: &Constitution,
) -> DependencyResult<()> {
    let references = extract_constitution_references(constitution);
    graph.validate_no_cycle(&constitution.id, &references)
}
