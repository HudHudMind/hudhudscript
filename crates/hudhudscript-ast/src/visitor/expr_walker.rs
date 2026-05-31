//! Expression child walker.

use super::api::{walk_expr, walk_stmts_check};
use super::{AstVisitor, VisitControl};
use crate::expr::{ArrowFunctionBody, TemplateStringPart};
use crate::Expr;

/// Recurse into child nodes of an expression.
pub(crate) fn walk_expr_children(visitor: &mut impl AstVisitor, expr: &Expr) -> VisitControl {
    match expr {
        Expr::Literal(_, _) | Expr::Identifier(_, _) | Expr::This(_) => {
            // Leaf nodes — no children.
        }

        Expr::Binary { left, right, .. } => {
            if walk_expr(visitor, left) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_expr(visitor, right) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Expr::Unary { expr: inner, .. } => {
            if walk_expr(visitor, inner) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Expr::Call { callee, args, .. } => {
            if walk_expr(visitor, callee) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            for arg in args {
                if walk_expr(visitor, arg) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Expr::Perform { action, .. } => {
            if walk_expr(visitor, action) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Expr::Member { object, .. } | Expr::OptionalMember { object, .. } => {
            if walk_expr(visitor, object) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Expr::Index { object, index, .. } => {
            if walk_expr(visitor, object) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_expr(visitor, index) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Expr::Ternary { condition, true_expr, false_expr, .. } => {
            if walk_expr(visitor, condition) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_expr(visitor, true_expr) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if walk_expr(visitor, false_expr) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Expr::Array { elements, .. } => {
            for elem in elements {
                if walk_expr(visitor, elem) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Expr::Object { properties, .. } => {
            for (_key, val) in properties {
                if walk_expr(visitor, val) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Expr::TemplateString { parts, .. } => {
            for part in parts {
                if let TemplateStringPart::Interpolation(inner) = part {
                    if walk_expr(visitor, inner) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
            }
        }

        Expr::ArrowFunction { body, .. } => match body {
            ArrowFunctionBody::Expression(expr_body) => {
                if walk_expr(visitor, expr_body) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            ArrowFunctionBody::Block(stmts) => {
                if walk_stmts_check(visitor, stmts) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        },

        Expr::Await { expr: inner, .. } => {
            if walk_expr(visitor, inner) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Expr::New { args, .. } => {
            for arg in args {
                if walk_expr(visitor, arg) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Expr::Spread { expr: inner, .. } => {
            if walk_expr(visitor, inner) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Expr::Yield { value, .. } => {
            if let Some(val) = value {
                if walk_expr(visitor, val) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Expr::Spawn { args, .. } => {
            for arg in args {
                if walk_expr(visitor, arg) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }
        Expr::ViewAs { instance, .. } => {
            if walk_expr(visitor, instance) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }
    }
    VisitControl::Continue
}
