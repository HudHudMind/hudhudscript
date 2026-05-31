//! AST Visitor trait — single traversal pattern for all passes.
//!
//! Formatter, linter, type-checker, LSP, and other passes should implement
//! this trait instead of writing their own match blocks.
//!
//! Design: a simple 3-method trait (`visit_stmt`, `visit_expr`, `visit_decl`)
//! plus free `walk_*` functions that handle recursion into child nodes.
//! Visitors only see each node — the walk functions do the structural traversal.

mod api;
mod control;
mod decl_walker;
mod expr_walker;
mod helpers;
mod stmt_walker;
mod trait_def;

pub use api::{walk_decl, walk_expr, walk_stmt, walk_stmts};
pub use control::VisitControl;
pub use trait_def::AstVisitor;
