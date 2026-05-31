//! Import graph for circular dependency detection

use crate::module::ModuleId;
use std::collections::{HashMap, HashSet};

/// Graph error
#[derive(Debug)]
pub enum GraphError {
    CircularDependency(String),
    ModuleNotFound(String),
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            GraphError::CircularDependency(s) => write!(f, "Circular dependency detected: {}", s),
            GraphError::ModuleNotFound(s) => write!(f, "Module not found in graph: {}", s),
        }
    }
}

impl std::error::Error for GraphError {}

/// Import graph - tracks module dependencies
pub struct ImportGraph {
    /// Adjacency list (module -> dependencies)
    pub edges: HashMap<ModuleId, Vec<ModuleId>>,
}

impl ImportGraph {
    /// Create new import graph
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    /// Add module
    pub fn add_module(&mut self, module_id: ModuleId) {
        self.edges.entry(module_id).or_default();
    }

    /// Add dependency
    pub fn add_dependency(&mut self, from: ModuleId, to: ModuleId) {
        self.edges.entry(from).or_default().push(to);
    }

    /// Check for circular dependencies
    pub fn check_cycles(&self) -> Result<(), GraphError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for module_id in self.edges.keys() {
            if !visited.contains(module_id)
                && self.has_cycle_util(module_id, &mut visited, &mut rec_stack)?
            {
                return Err(GraphError::CircularDependency(format!("{:?}", module_id)));
            }
        }

        Ok(())
    }

    /// Topological sort
    pub fn topological_sort(&self) -> Result<Vec<ModuleId>, GraphError> {
        self.check_cycles()?;

        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for module_id in self.edges.keys() {
            if !visited.contains(module_id) {
                self.topo_sort_util(module_id, &mut visited, &mut stack);
            }
        }

        stack.reverse();
        Ok(stack)
    }

    fn has_cycle_util(
        &self,
        module_id: &ModuleId,
        visited: &mut HashSet<ModuleId>,
        rec_stack: &mut HashSet<ModuleId>,
    ) -> Result<bool, GraphError> {
        visited.insert(module_id.clone());
        rec_stack.insert(module_id.clone());

        if let Some(deps) = self.edges.get(module_id) {
            for dep in deps {
                if !visited.contains(dep) {
                    if self.has_cycle_util(dep, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(dep) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(module_id);
        Ok(false)
    }

    fn topo_sort_util(
        &self,
        module_id: &ModuleId,
        visited: &mut HashSet<ModuleId>,
        stack: &mut Vec<ModuleId>,
    ) {
        visited.insert(module_id.clone());

        if let Some(deps) = self.edges.get(module_id) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.topo_sort_util(dep, visited, stack);
                }
            }
        }

        stack.push(module_id.clone());
    }
}

impl Default for ImportGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl GraphError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            GraphError::CircularDependency(..) => {
                hudhudscript_errors::ErrorCode::GraphCircularDependency
            }
            GraphError::ModuleNotFound(..) => hudhudscript_errors::ErrorCode::GraphModuleNotFound,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<GraphError> for hudhudscript_errors::Error {
    fn from(e: GraphError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
