//! AST Visitor trait definition.

use super::VisitControl;
use crate::{Decl, Expr, Stmt};

/// Visitor trait for AST traversal.
///
/// Implement only the methods you care about — defaults return `Continue`.
/// The `walk_*` free functions handle recursion into child nodes.
pub trait AstVisitor {
    /// Called for every `Stmt` node before walking its children.
    fn visit_stmt(&mut self, _stmt: &Stmt) -> VisitControl {
        VisitControl::Continue
    }

    /// Called for every `Stmt` node after walking its children.
    /// Only called when `visit_stmt` returned `Continue` (not `SkipChildren` or `Stop`).
    fn leave_stmt(&mut self, _stmt: &Stmt) {}

    /// Called for every `Expr` node before walking its children.
    fn visit_expr(&mut self, _expr: &Expr) -> VisitControl {
        VisitControl::Continue
    }

    /// Called for every `Expr` node after walking its children.
    /// Only called when `visit_expr` returned `Continue` (not `SkipChildren` or `Stop`).
    fn leave_expr(&mut self, _expr: &Expr) {}

    /// Called for every `Decl` node before walking its children.
    fn visit_decl(&mut self, _decl: &Decl) -> VisitControl {
        VisitControl::Continue
    }

    /// Called for every `Decl` node after walking its children.
    /// Only called when `visit_decl` returned `Continue` (not `SkipChildren` or `Stop`).
    fn leave_decl(&mut self, _decl: &Decl) {}
}
