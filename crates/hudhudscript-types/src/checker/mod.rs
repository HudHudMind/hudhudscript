//! Type checker implementation

use crate::{type_codes, Ownership, Type, TypeError};
use hudhudscript_ast::{
    annotated::{AnnotatedProgram, AnnotationDiagnostic, DiagnosticLevel, SymbolType},
    ArrowFunctionBody, BinaryOp, ClassDecl, ClassMember, Expr, GenericParam, Literal,
    OwnershipMode, Stmt, UnaryOp, VarDecl,
};
use hudhudscript_errors::ScopeManager;
use std::collections::{HashMap, HashSet};

/// Metadata stored alongside each symbol in the symbol table (Issue #330).
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// The type of the symbol.
    pub ty: Type,
    /// Whether the binding is mutable (`let`) or immutable (`const`).
    pub mutable: bool,
    /// The ownership mode of the binding.
    pub ownership: Ownership,
    /// Lightweight projection for `ScopeManager::resolve` (Issue #895).
    scope_info: hudhudscript_errors::SymbolInfo,
}

impl SymbolInfo {
    /// Create a mutable, owned symbol (default for `let`).
    pub fn mutable_owned(ty: Type) -> Self {
        Self {
            ty,
            mutable: true,
            ownership: Ownership::OwnedValue,
            scope_info: hudhudscript_errors::SymbolInfo {
                name: String::new(),
                is_mutable: true,
                declared_at_depth: 0,
            },
        }
    }

    /// Create an immutable, owned symbol (default for `const`).
    pub fn immutable_owned(ty: Type) -> Self {
        Self {
            ty,
            mutable: false,
            ownership: Ownership::OwnedValue,
            scope_info: hudhudscript_errors::SymbolInfo {
                name: String::new(),
                is_mutable: false,
                declared_at_depth: 0,
            },
        }
    }

    /// Create a symbol with explicit mutability and ownership.
    pub fn new(ty: Type, mutable: bool, ownership: Ownership) -> Self {
        Self {
            ty,
            mutable,
            ownership,
            scope_info: hudhudscript_errors::SymbolInfo {
                name: String::new(),
                is_mutable: mutable,
                declared_at_depth: 0,
            },
        }
    }
}

/// Symbol table for variable and function types
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolInfo>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a variable in the current scope
    pub fn define(&mut self, name: String, ty: Type) -> Result<(), String> {
        self.define_with_info(name, SymbolInfo::mutable_owned(ty))
    }

    /// Define a variable with full symbol info (mutability + ownership).
    pub fn define_with_info(&mut self, name: String, mut info: SymbolInfo) -> Result<(), String> {
        let depth = self.scopes.len().saturating_sub(1);
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                return Err(format!("Variable '{}' already defined in this scope", name));
            }
            // Populate the lightweight scope_info projection (Issue #895).
            info.scope_info.name = name.clone();
            info.scope_info.declared_at_depth = depth;
            scope.insert(name, info);
            Ok(())
        } else {
            Err("No scope available".to_string())
        }
    }

    /// Look up a variable type (backward-compatible convenience method).
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.lookup_info(name).map(|info| &info.ty)
    }

    /// Look up full symbol info (type + mutability + ownership).
    pub fn lookup_info(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Iterate all symbols across all scopes (innermost scopes shadow outer ones).
    ///
    /// Returns deduplicated `(name, &SymbolInfo)` pairs — if a name appears in
    /// multiple scopes, only the innermost (most recent) binding is yielded.
    pub fn all_symbols(&self) -> Vec<(&String, &SymbolInfo)> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for scope in self.scopes.iter().rev() {
            for (name, info) in scope {
                if seen.insert(name) {
                    result.push((name, info));
                }
            }
        }
        result
    }
}

/// `ScopeManager` implementation for `SymbolTable` (Issue #895).
///
/// This bridges the shared `ScopeManager` trait (from `hudhudscript-errors`)
/// with the TypeChecker's richer `SymbolInfo` (which also carries `Type` and
/// `Ownership`).  The trait's `SymbolInfo` is a lightweight projection.
impl ScopeManager for SymbolTable {
    fn push_scope(&mut self) {
        self.enter_scope();
    }

    fn pop_scope(&mut self) {
        self.exit_scope();
    }

    fn declare(&mut self, name: &str, mutable: bool) {
        // Use `Type::Any` and `Ownership::OwnedValue` as defaults when the
        // caller only has the name + mutability (the trait-level API).
        let info = SymbolInfo::new(Type::Any, mutable, Ownership::OwnedValue);
        let _ = self.define_with_info(name.to_string(), info);
    }

    fn resolve(&self, name: &str) -> Option<&hudhudscript_errors::SymbolInfo> {
        self.lookup_info(name).map(|info| &info.scope_info)
    }

    fn is_declared_in_current(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.contains_key(name))
            .unwrap_or(false)
    }

    fn depth(&self) -> usize {
        self.scopes.len().saturating_sub(1)
    }
}

/// Type checker
pub struct TypeChecker {
    symbol_table: SymbolTable,
    errors: Vec<TypeError>,
    /// Non-fatal warnings (e.g. non-exhaustive match on union types) — Issue #1018.
    warnings: Vec<TypeError>,
    /// Whether we are currently inside an `atomically()` block (Issue #443).
    in_atomically_block: bool,
    /// The name of the class currently being checked, if any (Issue #689).
    pub current_class: Option<String>,
    /// Issue #866 TYPE-001: When true, type annotations are enforced at check time.
    /// In strict mode, assigning a value of the wrong type to an annotated variable
    /// produces an error instead of a warning.
    strict: bool,
    /// Issue #1009: Generic type parameter constraints.
    /// Maps generic param name (e.g. "T") to its constraint type name (e.g. "Number").
    /// Populated when entering a scope with type_params, cleared when exiting.
    generic_constraints: HashMap<String, String>,
    /// BOLEM-A: Shadowing redeclare policy
    redeclare_policy: RedeclarePolicy,
}

/// BOLEM-A: Redeclaration severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedeclarePolicy { Allow, Warn, Error }

impl Default for RedeclarePolicy {
    fn default() -> Self { RedeclarePolicy::Warn }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

mod constructor;
mod statements;
mod declarations;
mod expressions;
mod operators;
mod patterns;

pub use constructor::*;
pub use statements::*;
pub use declarations::*;
pub use expressions::*;
pub use operators::*;
pub use patterns::*;
