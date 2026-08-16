//! Declaration child walker.

use super::api::{walk_expr, walk_stmts_check};
use super::helpers::walk_ui_nodes;
use super::{AstVisitor, VisitControl};
use crate::Decl;

/// Recurse into child nodes of a declaration.
pub(crate) fn walk_decl_children(visitor: &mut impl AstVisitor, decl: &Decl) -> VisitControl {
    match decl {
        // Declarations with Vec<(String, Expr)> fields pattern
        Decl::Agent { fields, .. }
        | Decl::Action { fields, .. }
        | Decl::Tool { fields, .. }
        | Decl::Resource { fields, .. }
        | Decl::Provider { fields, .. }
        | Decl::Governance { fields, .. }
        | Decl::Entity { fields, .. }
        | Decl::StateMachine { fields, .. }
        | Decl::Event { fields, .. }
        | Decl::Contract { fields, .. }
        | Decl::Treaty { fields, .. }
        | Decl::Music { fields, .. }
        | Decl::Store { fields, .. } => {
            for (_key, expr) in fields {
                if walk_expr(visitor, expr) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Decl::Import { .. } => {
            // No child expressions.
        }

        Decl::Constitution { laws, .. } => {
            for law in laws {
                for rule_expr in &law.rules {
                    if walk_expr(visitor, rule_expr) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
            }
        }

        Decl::Law { rules, .. } => {
            for rule_expr in rules {
                if walk_expr(visitor, rule_expr) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Decl::Council { .. } => {
            // Members and rules are strings, no expressions to walk.
        }

        Decl::Rule {
            conditions,
            actions,
            ..
        } => {
            for cond in conditions {
                if walk_expr(visitor, &cond.value) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            for action in actions {
                for (_key, expr) in &action.params {
                    if walk_expr(visitor, expr) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
            }
        }

        Decl::Swarm { .. } => {
            // Agents and strategy are strings, no expressions.
        }

        Decl::Community { .. } => {
            // Members, councils are strings; culture has no expressions.
        }

        Decl::Protocol { session, .. } | Decl::Strategy { session, .. } => {
            for (_key, expr) in session {
                if walk_expr(visitor, expr) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Decl::Subject {
            states,
            memory,
            perception,
            fields,
            ..
        } => {
            for field_list in [states, memory, perception, fields] {
                for (_key, expr) in field_list {
                    if walk_expr(visitor, expr) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
            }
        }

        Decl::Relation { fields, .. } => {
            for (_key, expr) in fields {
                if walk_expr(visitor, expr) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Decl::Effect { body, .. } => {
            if walk_stmts_check(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Decl::Role { fields, .. } => {
            for (_key, expr) in fields {
                if walk_expr(visitor, expr) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Decl::Compose { .. } => {}

        Decl::UiApp {
            screens,
            components,
            ..
        } => {
            for screen in screens {
                if walk_ui_nodes(visitor, &screen.body) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
            for component in components {
                // Walk default prop values
                for (_name, default_val) in &component.props {
                    if let Some(val) = default_val {
                        if walk_expr(visitor, val) == VisitControl::Stop {
                            return VisitControl::Stop;
                        }
                    }
                }
                if walk_ui_nodes(visitor, &component.body) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }

        Decl::AgentAction { body, .. } => {
            if walk_stmts_check(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Decl::Ability { body, .. } => {
            if walk_stmts_check(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
        }

        Decl::Deploy {
            targets,
            providers,
            fields,
            ..
        } => {
            for target in targets {
                for (_key, expr) in &target.fields {
                    if walk_expr(visitor, expr) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
            }
            for provider in providers {
                for (_key, expr) in &provider.fields {
                    if walk_expr(visitor, expr) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
            }
            for (_key, expr) in fields {
                if walk_expr(visitor, expr) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }
        Decl::Loop { items, .. } => {
            for item in items {
                match item {
                    crate::stmt::decl::LoopItemAst::InlineStep(s) => {
                        if walk_decl_children(visitor, s) == VisitControl::Stop {
                            return VisitControl::Stop;
                        }
                    }
                    _ => {}
                }
            }
        }
        Decl::Step { body, gate, .. } => {
            if walk_stmts_check(visitor, body) == VisitControl::Stop {
                return VisitControl::Stop;
            }
            if let Some(ref g) = gate {
                for b in &g.branches {
                    if walk_expr(visitor, &b.cond) == VisitControl::Stop {
                        return VisitControl::Stop;
                    }
                }
            }
        }
        Decl::Gate { branches, .. } => {
            for branch in branches {
                if walk_expr(visitor, &branch.cond) == VisitControl::Stop {
                    return VisitControl::Stop;
                }
            }
        }
        Decl::Chain { .. }
        | Decl::AttachStep { .. }
        | Decl::AttachLoop { .. }
        | Decl::RunLoop { .. }
        | Decl::RunChain { .. } => {}
    }
    VisitControl::Continue
}
