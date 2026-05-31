//! Shared scope management trait.
//!
//! Defines the common interface for variable scope tracking across
//! Interpreter, VM, Compiler, Linter, and TypeChecker.

/// Information about a declared symbol.
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub is_mutable: bool,
    pub declared_at_depth: usize,
}

/// Common scope management interface.
///
/// Each pass implements this differently but the conceptual operations are the same.
pub trait ScopeManager {
    /// Enter a new scope level (block, function, loop body).
    fn push_scope(&mut self);

    /// Exit the current scope level, discarding local declarations.
    fn pop_scope(&mut self);

    /// Declare a new variable in the current scope.
    fn declare(&mut self, name: &str, mutable: bool);

    /// Look up a variable by name across all visible scopes.
    fn resolve(&self, name: &str) -> Option<&SymbolInfo>;

    /// Check if a name is declared in the current (innermost) scope only.
    fn is_declared_in_current(&self, name: &str) -> bool;

    /// Current scope depth (0 = global).
    fn depth(&self) -> usize;
}
