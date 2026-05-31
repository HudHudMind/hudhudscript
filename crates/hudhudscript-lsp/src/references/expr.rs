use super::push_if_match;
use super::stmt::collect_stmt;
use hudhudscript_ast::*;
use tower_lsp::lsp_types::{Location, Url};

pub(crate) fn collect_expr(expr: &Expr, name: &str, uri: &Url, out: &mut Vec<Location>) {
    match expr {
        Expr::Identifier(id, span) => {
            push_if_match(id, name, *span, uri, out);
        }

        Expr::Binary { left, right, .. } => {
            collect_expr(left, name, uri, out);
            collect_expr(right, name, uri, out);
        }

        Expr::Unary { expr: inner, .. } => {
            collect_expr(inner, name, uri, out);
        }

        Expr::Call { callee, args, .. } => {
            collect_expr(callee, name, uri, out);
            for a in args {
                collect_expr(a, name, uri, out);
            }
        }

        Expr::Perform { action, .. } => {
            collect_expr(action, name, uri, out);
        }

        Expr::Member {
            object,
            property,
            span,
            ..
        }
        | Expr::OptionalMember {
            object,
            property,
            span,
            ..
        } => {
            collect_expr(object, name, uri, out);
            push_if_match(property, name, *span, uri, out);
        }

        Expr::Index { object, index, .. } => {
            collect_expr(object, name, uri, out);
            collect_expr(index, name, uri, out);
        }

        Expr::Array { elements, .. } => {
            for e in elements {
                collect_expr(e, name, uri, out);
            }
        }

        Expr::Object { properties, .. } => {
            for (key, val) in properties {
                push_if_match(key, name, expr.span(), uri, out);
                collect_expr(val, name, uri, out);
            }
        }

        Expr::TemplateString { parts, .. } => {
            for part in parts {
                if let TemplateStringPart::Interpolation(inner) = part {
                    collect_expr(inner, name, uri, out);
                }
            }
        }

        Expr::ArrowFunction { params, body, .. } => {
            for p in params {
                push_if_match(p, name, expr.span(), uri, out);
            }
            match body {
                ArrowFunctionBody::Expression(e) => collect_expr(e, name, uri, out),
                ArrowFunctionBody::Block(stmts) => {
                    for s in stmts {
                        collect_stmt(s, name, uri, out);
                    }
                }
            }
        }

        Expr::Await { expr: inner, .. } => {
            collect_expr(inner, name, uri, out);
        }

        Expr::New {
            class_name,
            args,
            span,
            ..
        } => {
            push_if_match(class_name, name, *span, uri, out);
            for a in args {
                collect_expr(a, name, uri, out);
            }
        }

        Expr::Spread { expr: inner, .. } => {
            collect_expr(inner, name, uri, out);
        }

        Expr::Yield { value, .. } => {
            if let Some(v) = value {
                collect_expr(v, name, uri, out);
            }
        }

        Expr::Spawn { args, .. } => {
            for arg in args {
                collect_expr(arg, name, uri, out);
            }
        }

        Expr::ViewAs { instance, .. } => {
            collect_expr(instance, name, uri, out);
        }

        Expr::Ternary { condition, true_expr, false_expr, .. } => {
            collect_expr(condition, name, uri, out);
            collect_expr(true_expr, name, uri, out);
            collect_expr(false_expr, name, uri, out);
        }

        Expr::Literal(..) | Expr::This(..) => {}
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

pub(crate) fn collect_class_member(
    member: &ClassMember,
    name: &str,
    uri: &Url,
    out: &mut Vec<Location>,
) {
    match member {
        ClassMember::Field {
            name: n,
            initializer,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            if let Some(init) = initializer {
                collect_expr(init, name, uri, out);
            }
        }
        ClassMember::Method {
            name: n,
            params,
            body,
            span,
            ..
        } => {
            push_if_match(n, name, *span, uri, out);
            for p in params {
                push_if_match(&p.name, name, p.span, uri, out);
            }
            for s in body {
                collect_stmt(s, name, uri, out);
            }
        }
        ClassMember::Constructor {
            params, body, span, ..
        } => {
            for p in params {
                push_if_match(&p.name, name, p.span, uri, out);
            }
            for s in body {
                collect_stmt(s, name, uri, out);
            }
            // suppress unused warning
            let _ = span;
        }
    }
}

pub(crate) fn collect_match_pattern(
    pattern: &MatchPattern,
    name: &str,
    span: Span,
    uri: &Url,
    out: &mut Vec<Location>,
) {
    match pattern {
        MatchPattern::Identifier(id) => {
            push_if_match(id, name, span, uri, out);
        }
        MatchPattern::EnumVariant {
            enum_name,
            variant,
            binding,
            ..
        } => {
            push_if_match(enum_name, name, span, uri, out);
            push_if_match(variant, name, span, uri, out);
            if let Some(b) = binding.as_deref() {
                push_if_match(b, name, span, uri, out);
            }
        }
        MatchPattern::Or(sub_patterns) => {
            for sub in sub_patterns {
                collect_match_pattern(sub, name, span, uri, out);
            }
        }
        MatchPattern::Wildcard | MatchPattern::Literal(_) => {}
    }
}
