use super::decl::collect_decl;
use super::expr::collect_expr;
use super::expr::{collect_class_member, collect_match_pattern};
use super::push_if_match;
use hudhudscript_ast::*;
use tower_lsp::lsp_types::{Location, Url};

pub(crate) fn collect_stmt(stmt: &Stmt, name: &str, uri: &Url, out: &mut Vec<Location>) {
    match stmt {
        Stmt::Decl(decl) => collect_decl(decl, name, uri, out),

        Stmt::McpServer(decl) => {
            push_if_match(&decl.name, name, decl.span, uri, out);
        }

        Stmt::VarDecl(decl) => {
            push_if_match(&decl.name, name, decl.span, uri, out);
            if let Some(init) = &decl.initializer {
                collect_expr(init, name, uri, out);
            }
        }

        Stmt::Let {
            name: n,
            value,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            collect_expr(value, name, uri, out);
        }

        Stmt::Const {
            name: n,
            value,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            collect_expr(value, name, uri, out);
        }

        Stmt::Assignment { target, value, .. } => {
            collect_expr(target, name, uri, out);
            collect_expr(value, name, uri, out);
        }

        Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr(condition, name, uri, out);
            collect_stmt(then_branch, name, uri, out);
            if let Some(eb) = else_branch {
                collect_stmt(eb, name, uri, out);
            }
        }

        Stmt::While {
            condition, body, ..
        } => {
            collect_expr(condition, name, uri, out);
            collect_stmt(body, name, uri, out);
        }

        Stmt::For {
            variable,
            iterable,
            body,
            ..
        } => {
            push_if_match(variable, name, stmt.span(), uri, out);
            collect_expr(iterable, name, uri, out);
            collect_stmt(body, name, uri, out);
        }

        Stmt::ForCStyle {
            init,
            condition,
            update,
            body,
            ..
        } => {
            if let Some(i) = init {
                collect_stmt(i, name, uri, out);
            }
            if let Some(c) = condition {
                collect_expr(c, name, uri, out);
            }
            if let Some(u) = update {
                collect_stmt(u, name, uri, out);
            }
            collect_stmt(body, name, uri, out);
        }

        Stmt::ForRange {
            start,
            stop,
            step,
            body,
            ..
        } => {
            collect_expr(start, name, uri, out);
            collect_expr(stop, name, uri, out);
            if let Some(s) = step {
                collect_expr(s, name, uri, out);
            }
            collect_stmt(body, name, uri, out);
        }

        Stmt::Block { statements, .. } => {
            for s in statements {
                collect_stmt(s, name, uri, out);
            }
        }

        Stmt::Return { value, .. } => {
            if let Some(v) = value {
                collect_expr(v, name, uri, out);
            }
        }

        Stmt::Switch {
            value,
            cases,
            default,
            ..
        } => {
            collect_expr(value, name, uri, out);
            for case in cases {
                collect_expr(&case.value, name, uri, out);
                for s in &case.body {
                    collect_stmt(s, name, uri, out);
                }
            }
            if let Some(default_stmts) = default {
                for s in default_stmts {
                    collect_stmt(s, name, uri, out);
                }
            }
        }

        Stmt::Try {
            try_block,
            catch_clause,
            finally_block,
            ..
        } => {
            collect_stmt(try_block, name, uri, out);
            if let Some(cc) = catch_clause {
                push_if_match(&cc.param, name, cc.span, uri, out);
                collect_stmt(&cc.body, name, uri, out);
            }
            if let Some(fb) = finally_block {
                collect_stmt(fb, name, uri, out);
            }
        }

        Stmt::Throw { value, .. } => {
            collect_expr(value, name, uri, out);
        }

        Stmt::Expr(expr) => {
            collect_expr(expr, name, uri, out);
        }

        Stmt::Import { imports, .. } => match imports {
            ImportKind::Named(names) => {
                for n in names {
                    push_if_match(n, name, stmt.span(), uri, out);
                }
            }
            ImportKind::Default(n) | ImportKind::Wildcard(n) => {
                push_if_match(n, name, stmt.span(), uri, out);
            }
        },

        Stmt::Export { item, .. } => {
            collect_stmt(item, name, uri, out);
        }

        Stmt::Function {
            name: n,
            params,
            body,
            ..
        } => {
            push_if_match(n, name, stmt.span(), uri, out);
            for p in params {
                push_if_match(p, name, stmt.span(), uri, out);
            }
            for s in body {
                collect_stmt(s, name, uri, out);
            }
        }

        Stmt::Class(decl) => {
            push_if_match(&decl.name, name, decl.span, uri, out);
            if let Some(parent) = &decl.parent {
                push_if_match(parent, name, decl.span, uri, out);
            }
            for member in &decl.members {
                collect_class_member(member, name, uri, out);
            }
        }

        Stmt::Match { value, arms, .. } => {
            collect_expr(value, name, uri, out);
            for arm in arms {
                collect_match_pattern(&arm.pattern, name, arm.span, uri, out);
                for s in &arm.body {
                    collect_stmt(s, name, uri, out);
                }
            }
        }

        Stmt::EnumDecl {
            name: n,
            variants,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for v in variants {
                push_if_match(&v.name, name, v.span, uri, out);
            }
        }

        Stmt::Spawn {
            subject_name,
            args,
            span,
            ..
        } => {
            push_if_match(subject_name, name, *span, uri, out);
            for a in args {
                collect_expr(a, name, uri, out);
            }
        }

        Stmt::Despawn {
            name: despawn_name,
            span,
            ..
        } => {
            push_if_match(despawn_name, name, *span, uri, out);
        }

        Stmt::Send {
            message, target, ..
        } => {
            collect_expr(message, name, uri, out);
            collect_expr(target, name, uri, out);
        }

        Stmt::Receive {
            variable, source, ..
        } => {
            push_if_match(variable, name, stmt.span(), uri, out);
            collect_expr(source, name, uri, out);
        }

        Stmt::Require { condition, .. } => {
            collect_expr(condition, name, uri, out);
        }

        Stmt::Perform { action, .. } => {
            collect_expr(action, name, uri, out);
        }

        Stmt::Remember {
            content,
            store_name,
            ..
        } => {
            collect_expr(content, name, uri, out);
            if let Some(sn) = store_name.as_deref() {
                push_if_match(sn, name, stmt.span(), uri, out);
            }
        }

        Stmt::Recall {
            query, store_name, ..
        } => {
            collect_expr(query, name, uri, out);
            if let Some(sn) = store_name.as_deref() {
                push_if_match(sn, name, stmt.span(), uri, out);
            }
        }

        Stmt::Forget {
            target, store_name, ..
        } => {
            collect_expr(target, name, uri, out);
            if let Some(sn) = store_name.as_deref() {
                push_if_match(sn, name, stmt.span(), uri, out);
            }
        }

        Stmt::Destructure { value, .. } => {
            collect_expr(value, name, uri, out);
        }

        Stmt::Trait {
            name: n,
            methods,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for method in methods {
                push_if_match(&method.name, name, method.span, uri, out);
            }
        }

        Stmt::Break { .. } | Stmt::Continue { .. } => {}
    }
}

// ── Declaration walker ──────────────────────────────────────────────────────
