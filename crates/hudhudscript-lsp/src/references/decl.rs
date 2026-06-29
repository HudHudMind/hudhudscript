use super::expr::collect_expr;
use super::push_if_match;
use super::stmt::collect_stmt;
use hudhudscript_ast::*;
use tower_lsp::lsp_types::{Location, Url};

pub(crate) fn collect_decl(decl: &Decl, name: &str, uri: &Url, out: &mut Vec<Location>) {
    match decl {
        Decl::Agent {
            name: n,
            fields,
            span,
            ..
        }
        | Decl::Action {
            name: n,
            fields,
            span,
            ..
        }
        | Decl::Tool {
            name: n,
            fields,
            span,
            ..
        }
        | Decl::Resource {
            name: n,
            fields,
            span,
            ..
        }
        | Decl::Provider {
            name: n,
            fields,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for (key, val) in fields {
                push_if_match(key, name, *span, uri, out);
                collect_expr(val, name, uri, out);
            }
        }

        Decl::Import {
            module,
            alias,
            span,
            ..
        } => {
            push_if_match(module, name, *span, uri, out);
            if let Some(a) = alias.as_deref() {
                push_if_match(a, name, *span, uri, out);
            }
        }

        Decl::Constitution {
            name: n,
            laws,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for law in laws {
                push_if_match(&law.name, name, law.span, uri, out);
            }
        }

        Decl::Law { name: n, span, .. } => {
            push_if_match(n, name, *span, uri, out);
            // Issue #474: rules are now Expr nodes — identifier extraction
            // for references is handled by the expression visitor
        }

        Decl::Council {
            name: n,
            constitution,
            members,
            rules,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            push_if_match(constitution, name, *span, uri, out);
            for m in members {
                push_if_match(&m.agent_id, name, m.span, uri, out);
            }
            for r in rules {
                push_if_match(r, name, *span, uri, out);
            }
        }

        Decl::Rule {
            name: n,
            conditions,
            actions,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for c in conditions {
                collect_expr(&c.value, name, uri, out);
            }
            for a in actions {
                for (_, v) in &a.params {
                    collect_expr(v, name, uri, out);
                }
            }
        }

        Decl::Swarm {
            name: n,
            agents,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for a in agents {
                push_if_match(a, name, *span, uri, out);
            }
        }

        Decl::Community {
            name: n,
            members,
            councils,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for m in members {
                push_if_match(m, name, *span, uri, out);
            }
            for c in councils {
                push_if_match(c, name, *span, uri, out);
            }
        }

        Decl::Protocol {
            name: n,
            governance,
            session,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            if let Some(g) = governance.as_deref() {
                push_if_match(g, name, *span, uri, out);
            }
            for (_, expr) in session {
                collect_expr(expr, name, uri, out);
            }
        }

        Decl::Governance {
            name: n,
            fields,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for (key, val) in fields {
                push_if_match(key, name, *span, uri, out);
                collect_expr(val, name, uri, out);
            }
        }

        Decl::Subject {
            name: n,
            roles,
            capabilities,
            intents,
            uses,
            states,
            memory,
            perception,
            fields,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for r in roles {
                push_if_match(r, name, *span, uri, out);
            }
            for c in capabilities {
                push_if_match(c, name, *span, uri, out);
            }
            for i in intents {
                push_if_match(i, name, *span, uri, out);
            }
            for (provider, _channel) in uses {
                push_if_match(provider, name, *span, uri, out);
            }
            for (sn, sv) in states {
                push_if_match(sn, name, *span, uri, out);
                collect_expr(sv, name, uri, out);
            }
            for (mn, mv) in memory {
                push_if_match(mn, name, *span, uri, out);
                collect_expr(mv, name, uri, out);
            }
            for (pn, pv) in perception {
                push_if_match(pn, name, *span, uri, out);
                collect_expr(pv, name, uri, out);
            }
            for (fn_, fv) in fields {
                push_if_match(fn_, name, *span, uri, out);
                collect_expr(fv, name, uri, out);
            }
        }

        Decl::Relation {
            subject_a,
            subject_b,
            fields,
            span,
            ..
        } => {
            push_if_match(subject_a, name, *span, uri, out);
            push_if_match(subject_b, name, *span, uri, out);
            for (key, val) in fields {
                push_if_match(key, name, *span, uri, out);
                collect_expr(val, name, uri, out);
            }
        }

        Decl::Effect {
            event_name,
            body,
            span,
            ..
        } => {
            push_if_match(event_name, name, *span, uri, out);
            for s in body {
                collect_stmt(s, name, uri, out);
            }
        }

        Decl::Role {
            name: n,
            capabilities,
            fields,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for c in capabilities {
                push_if_match(c, name, *span, uri, out);
            }
            for (key, val) in fields {
                push_if_match(key, name, *span, uri, out);
                collect_expr(val, name, uri, out);
            }
        }

        Decl::Compose { base_subject, span, .. } => {
            push_if_match(base_subject, name, *span, uri, out);
        }

        Decl::Store {
            name: n,
            fields,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for (key, val) in fields {
                push_if_match(key, name, *span, uri, out);
                collect_expr(val, name, uri, out);
            }
        }

        Decl::Strategy {
            name: n,
            governance,
            session,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            if let Some(g) = governance.as_deref() {
                push_if_match(g, name, *span, uri, out);
            }
            for (_, expr) in session {
                collect_expr(expr, name, uri, out);
            }
        }

        Decl::Entity {
            name: n,
            fields,
            span,
            ..
        }
        | Decl::StateMachine {
            name: n,
            fields,
            span,
            ..
        }
        | Decl::Event {
            name: n,
            fields,
            span,
            ..
        }
        | Decl::Contract {
            name: n,
            fields,
            span,
            ..
        }
        | Decl::Treaty {
            name: n,
            fields,
            span,
            ..
        }
        | Decl::Music {
            name: n,
            fields,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for (key, val) in fields {
                push_if_match(key, name, *span, uri, out);
                collect_expr(val, name, uri, out);
            }
        }

        Decl::UiApp { name: n, span, .. } => {
            push_if_match(n, name, *span, uri, out);
        }
        Decl::AgentAction {
            name: n,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
        }
        Decl::Ability {
            name: n,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
        }
        Decl::Deploy {
            name: n,
            fields,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for (key, val) in fields {
                push_if_match(key, name, *span, uri, out);
                collect_expr(val, name, uri, out);
            }
        }
        _ => {}
    }
}

// ── Expression walker ───────────────────────────────────────────────────────
