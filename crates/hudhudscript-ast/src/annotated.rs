//! Annotated AST — carries type information from TypeChecker to Compiler.
//!
//! The TypeChecker analyzes the AST and produces type annotations for each node.
//! The Compiler can then use these annotations for type-specific optimizations.

use crate::{Span, Stmt};
use std::collections::HashMap;

/// A unique identifier for an AST node, used to look up type information.
pub type NodeId = u64;

/// Type information for a single AST node.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// The resolved type of this node.
    pub resolved_type: String, // Using String for now; will be replaced with proper Type enum
    /// Whether this expression can be null.
    pub is_nullable: bool,
    /// Whether this is a constant (known at compile time).
    pub is_constant: bool,
    /// Source span for error reporting.
    pub span: Option<Span>,
}

/// An AST program annotated with type information from the TypeChecker.
///
/// This is the bridge between TypeChecker and Compiler — the TypeChecker produces it,
/// the Compiler consumes it for type-aware bytecode generation.
#[derive(Debug, Clone)]
pub struct AnnotatedProgram {
    /// The original AST statements.
    pub stmts: Vec<Stmt>,
    /// Type annotations keyed by NodeId.
    pub type_info: HashMap<NodeId, TypeInfo>,
    /// Resolved symbol information (variable types, function signatures).
    pub symbols: HashMap<String, SymbolType>,
    /// Diagnostics collected during type analysis (warnings, suggestions).
    pub diagnostics: Vec<AnnotationDiagnostic>,
}

/// Type information for a named symbol (variable, function, class).
#[derive(Debug, Clone)]
pub struct SymbolType {
    pub name: String,
    pub type_name: String,
    pub is_mutable: bool,
    pub is_exported: bool,
}

/// A diagnostic produced during type annotation.
#[derive(Debug, Clone)]
pub struct AnnotationDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiagnosticLevel {
    /// Type mismatch treated as a hard error (strict mode).
    Error,
    Warning,
    Info,
    Hint,
}

impl AnnotatedProgram {
    /// Create a new annotated program from raw AST (no type info yet).
    pub fn from_ast(stmts: Vec<Stmt>) -> Self {
        Self {
            stmts,
            type_info: HashMap::new(),
            symbols: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Look up type info for a node.
    pub fn get_type(&self, node_id: NodeId) -> Option<&TypeInfo> {
        self.type_info.get(&node_id)
    }

    /// Look up a symbol's type.
    pub fn get_symbol_type(&self, name: &str) -> Option<&SymbolType> {
        self.symbols.get(name)
    }
}
