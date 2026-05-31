//! Public entry points for AST traversal.

use super::decl_walker::walk_decl_children;
use super::expr_walker::walk_expr_children;
use super::stmt_walker::walk_stmt_children;
use super::{AstVisitor, VisitControl};
use crate::{Decl, Expr, Stmt};

/// Walk a slice of statements, visiting each and recursing into children.
pub fn walk_stmts(visitor: &mut impl AstVisitor, stmts: &[Stmt]) {
    for stmt in stmts {
        if walk_stmt(visitor, stmt) == VisitControl::Stop {
            return;
        }
    }
}

/// Walk a single statement: visit it, then recurse into children.
///
/// Returns `Stop` if the traversal should halt, otherwise `Continue`.
pub fn walk_stmt(visitor: &mut impl AstVisitor, stmt: &Stmt) -> VisitControl {
    match visitor.visit_stmt(stmt) {
        VisitControl::Stop => return VisitControl::Stop,
        VisitControl::SkipChildren => return VisitControl::Continue,
        VisitControl::Continue => {}
    }
    let ctrl = walk_stmt_children(visitor, stmt);
    if ctrl == VisitControl::Stop {
        return VisitControl::Stop;
    }
    visitor.leave_stmt(stmt);
    VisitControl::Continue
}

/// Walk a single expression: visit it, then recurse into children.
///
/// Returns `Stop` if the traversal should halt, otherwise `Continue`.
pub fn walk_expr(visitor: &mut impl AstVisitor, expr: &Expr) -> VisitControl {
    match visitor.visit_expr(expr) {
        VisitControl::Stop => return VisitControl::Stop,
        VisitControl::SkipChildren => return VisitControl::Continue,
        VisitControl::Continue => {}
    }
    let ctrl = walk_expr_children(visitor, expr);
    if ctrl == VisitControl::Stop {
        return VisitControl::Stop;
    }
    visitor.leave_expr(expr);
    VisitControl::Continue
}

/// Walk a single declaration: visit it, then recurse into children.
///
/// Returns `Stop` if the traversal should halt, otherwise `Continue`.
pub fn walk_decl(visitor: &mut impl AstVisitor, decl: &Decl) -> VisitControl {
    match visitor.visit_decl(decl) {
        VisitControl::Stop => return VisitControl::Stop,
        VisitControl::SkipChildren => return VisitControl::Continue,
        VisitControl::Continue => {}
    }
    let ctrl = walk_decl_children(visitor, decl);
    if ctrl == VisitControl::Stop {
        return VisitControl::Stop;
    }
    visitor.leave_decl(decl);
    VisitControl::Continue
}

/// Walk a slice of statements, returning `Stop` if any visit halts traversal.
pub(crate) fn walk_stmts_check(visitor: &mut impl AstVisitor, stmts: &[Stmt]) -> VisitControl {
    for stmt in stmts {
        if walk_stmt(visitor, stmt) == VisitControl::Stop {
            return VisitControl::Stop;
        }
    }
    VisitControl::Continue
}
