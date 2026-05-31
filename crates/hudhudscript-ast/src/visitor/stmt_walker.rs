//! Statement child walker.

use super::api::{walk_decl, walk_expr, walk_stmt, walk_stmts_check};
use super::helpers::{walk_catch_clause, walk_class_member, walk_match_arm, walk_switch_case};
use super::{AstVisitor, VisitControl};
use crate::Stmt;

/// Recurse into child nodes of a statement.
pub(crate) fn walk_stmt_children(visitor: &mut impl AstVisitor, stmt: &Stmt) -> VisitControl {
    match stmt {
        Stmt::Decl(decl) => {
            if walk_decl(visitor, decl) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::McpServer(mcp) => {
            for (_key, expr) in &mcp.fields {
                if walk_expr(visitor, expr) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            for tool_def in &mcp.tools {
                if walk_stmts_check(visitor, &tool_def.body) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Stmt::VarDecl(var_decl) => {
            if let Some(init) = &var_decl.initializer {
                if walk_expr(visitor, init) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Stmt::Let { value, .. } | Stmt::Const { value, .. } => {
            if walk_expr(visitor, value) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Assignment { target, value, .. } => {
            if walk_expr(visitor, target) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_expr(visitor, value) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            if walk_expr(visitor, condition) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_stmt(visitor, then_branch) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if let Some(else_br) = else_branch {
                if walk_stmt(visitor, else_br) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Stmt::While {
            condition, body, ..
        } => {
            if walk_expr(visitor, condition) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_stmt(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::For { iterable, body, .. } => {
            if walk_expr(visitor, iterable) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_stmt(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::ForCStyle {
            init,
            condition,
            update,
            body,
            ..
        } => {
            if let Some(init_stmt) = init {
                if walk_stmt(visitor, init_stmt) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            if let Some(cond) = condition {
                if walk_expr(visitor, cond) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            if let Some(upd) = update {
                if walk_stmt(visitor, upd) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            if walk_stmt(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::ForRange {
            start,
            stop,
            step,
            body,
            ..
        } => {
            if walk_expr(visitor, start) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_expr(visitor, stop) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if let Some(s) = step {
                if walk_expr(visitor, s) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            if walk_stmt(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Block { statements, .. } => {
            if walk_stmts_check(visitor, statements) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Return { value, .. } => {
            if let Some(val) = value {
                if walk_expr(visitor, val) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Stmt::Break { .. } | Stmt::Continue { .. } => {
            // Leaf nodes — no children to walk.
        }

        Stmt::Switch {
            value,
            cases,
            default,
            ..
        } => {
            if walk_expr(visitor, value) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            for case in cases {
                if walk_switch_case(visitor, case) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            if let Some(default_stmts) = default {
                if walk_stmts_check(visitor, default_stmts) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Stmt::Try {
            try_block,
            catch_clause,
            finally_block,
            ..
        } => {
            if walk_stmt(visitor, try_block) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if let Some(catch) = catch_clause {
                if walk_catch_clause(visitor, catch) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            if let Some(finally) = finally_block {
                if walk_stmt(visitor, finally) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Stmt::Throw { value, .. } => {
            if walk_expr(visitor, value) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Expr(expr) => {
            if walk_expr(visitor, expr) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Import { .. } => {
            // No child expressions to walk (path and imports are data, not AST nodes).
        }

        Stmt::Export { item, .. } => {
            if walk_stmt(visitor, item) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Function { body, .. } => {
            if walk_stmts_check(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Trait { .. } => {
            // Trait method signatures have no bodies — nothing to recurse into.
        }

        Stmt::Destructure { value, .. } => {
            if walk_expr(visitor, value) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Class(class_decl) => {
            for member in &class_decl.members {
                if walk_class_member(visitor, member) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Stmt::Match { value, arms, .. } => {
            if walk_expr(visitor, value) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            for arm in arms {
                if walk_match_arm(visitor, arm) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Stmt::EnumDecl { .. } => {
            // Enum variants have no child expressions to walk.
        }

        // ── SOP statements ──────────────────────────────────────────────
        Stmt::Spawn { args, .. } => {
            for arg in args {
                if walk_expr(visitor, arg) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }
        Stmt::Despawn { .. } => {}

        Stmt::Send {
            message, target, ..
        } => {
            if walk_expr(visitor, message) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_expr(visitor, target) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Receive { source, .. } => {
            if walk_expr(visitor, source) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Require { condition, .. } => {
            if walk_expr(visitor, condition) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Perform { action, .. } => {
            if walk_expr(visitor, action) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        // ── RAG statements ──────────────────────────────────────────────
        Stmt::Remember { content, .. } => {
            if walk_expr(visitor, content) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Recall { query, .. } => {
            if walk_expr(visitor, query) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Stmt::Forget { target, .. } => {
            if walk_expr(visitor, target) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }
    }
    VisitControl::Continue
}
