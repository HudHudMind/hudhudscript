//! Module data structures

use hudhudscript_ast::Stmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Module ID (file path)
pub type ModuleId = PathBuf;

/// Export kind
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportKind {
    /// Named export: export { foo, bar }
    Named(Vec<String>),
    /// Default export: export default value
    Default(String),
    /// Wildcard export: export * from "./module"
    Wildcard,
}

/// Export declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Export {
    /// Export kind
    pub kind: ExportKind,
    /// Source module (for re-exports)
    pub source: Option<String>,
}

/// Module - represents a loaded HudHudScript module
#[derive(Debug, Clone)]
pub struct Module {
    /// Module ID (file path)
    pub id: ModuleId,

    /// Module source code
    pub source: String,

    /// Parsed AST
    pub ast: Vec<Stmt>,

    /// Exports from this module
    pub exports: HashMap<String, Export>,

    /// Imports in this module (module path -> imported names)
    pub imports: HashMap<String, Vec<String>>,

    /// Whether module has been executed
    pub executed: bool,

    /// Exported values (after execution)
    pub exported_values: HashMap<String, ModuleValue>,
}

/// Module value - values that can be exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleValue {
    /// Function declaration
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    /// Variable value
    Variable {
        name: String,
        value: String, // Serialized value
    },
    /// Agent declaration
    Agent {
        name: String,
        decl: String, // Serialized AgentDecl
    },
    /// Tool declaration
    Tool {
        name: String,
        decl: String, // Serialized ToolDecl
    },
}

impl Module {
    /// Create new module
    pub fn new(id: ModuleId, source: String, ast: Vec<Stmt>) -> Self {
        Self {
            id,
            source,
            ast,
            exports: HashMap::new(),
            imports: HashMap::new(),
            executed: false,
            exported_values: HashMap::new(),
        }
    }

    /// Add export
    pub fn add_export(&mut self, name: String, export: Export) {
        self.exports.insert(name, export);
    }

    /// Add import
    pub fn add_import(&mut self, module_path: String, names: Vec<String>) {
        self.imports.insert(module_path, names);
    }

    /// Get export by name
    pub fn get_export(&self, name: &str) -> Option<&Export> {
        self.exports.get(name)
    }

    /// Get all export names
    pub fn export_names(&self) -> Vec<String> {
        self.exports.keys().cloned().collect()
    }

    /// Get all import module paths
    pub fn import_paths(&self) -> Vec<String> {
        self.imports.keys().cloned().collect()
    }

    /// Mark as executed
    pub fn mark_executed(&mut self) {
        self.executed = true;
    }

    /// Add exported value
    pub fn add_exported_value(&mut self, name: String, value: ModuleValue) {
        self.exported_values.insert(name, value);
    }

    /// Get exported value
    pub fn get_exported_value(&self, name: &str) -> Option<&ModuleValue> {
        self.exported_values.get(name)
    }
}
