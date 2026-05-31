//! Helper walkers for composite AST nodes.

use super::api::{walk_expr, walk_stmt, walk_stmts_check};
use super::{AstVisitor, VisitControl};
use crate::stmt::{CatchClause, MatchArm, SwitchCase, UiNode};
use crate::ClassMember;

/// Walk a `SwitchCase` — its value expression and body statements.
pub(crate) fn walk_switch_case(visitor: &mut impl AstVisitor, case: &SwitchCase) -> VisitControl {
    if walk_expr(visitor, &case.value) == VisitControl::Stop {
        return VisitControl::Stop;
    }
    walk_stmts_check(visitor, &case.body)
}

/// Walk a `CatchClause` — its body statement.
pub(crate) fn walk_catch_clause(
    visitor: &mut impl AstVisitor,
    catch: &CatchClause,
) -> VisitControl {
    walk_stmt(visitor, &catch.body)
}

/// Walk a `MatchArm` — its optional guard expression and body statements.
pub(crate) fn walk_match_arm(visitor: &mut impl AstVisitor, arm: &MatchArm) -> VisitControl {
    // Walk guard expression if present
    if let Some(guard) = &arm.guard {
        if walk_expr(visitor, guard) == VisitControl::Stop {
            return VisitControl::Stop;
        }
    }
    // Walk pattern expressions (MatchPattern contains no Expr children,
    // but we walk the body statements)
    walk_stmts_check(visitor, &arm.body)
}

/// Walk a `ClassMember` — field initializers and method bodies.
pub(crate) fn walk_class_member(
    visitor: &mut impl AstVisitor,
    member: &ClassMember,
) -> VisitControl {
    match member {
        ClassMember::Field { initializer, .. } => {
            if let Some(init) = initializer {
                if walk_expr(visitor, init) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }
        ClassMember::Method { body, .. } | ClassMember::Constructor { body, .. } => {
            if walk_stmts_check(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }
    }
    VisitControl::Continue
}

/// Walk UI nodes recursively — widgets, vars, events, platform blocks, inline exprs.
pub(crate) fn walk_ui_nodes(visitor: &mut impl AstVisitor, nodes: &[UiNode]) -> VisitControl {
    for node in nodes {
        match node {
            UiNode::Widget {
                label,
                props,
                events,
                children,
                style,
                ..
            } => {
                if let Some(lbl) = label {
                    if walk_expr(visitor, lbl) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
                for (_key, expr) in props {
                    if walk_expr(visitor, expr) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
                for (_key, expr) in events {
                    if walk_expr(visitor, expr) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
                for (_key, expr) in style {
                    if walk_expr(visitor, expr) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
                if walk_ui_nodes(visitor, children) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            UiNode::Var { value, .. } => {
                if walk_expr(visitor, value) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            UiNode::Event { handler, .. } => {
                if walk_expr(visitor, handler) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            UiNode::PlatformBlock { body, .. } => {
                if walk_ui_nodes(visitor, body) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            UiNode::Expr(expr) => {
                if walk_expr(visitor, expr) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }
    }
    VisitControl::Continue
}
