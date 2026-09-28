//! Loop Engineering sentetik AST üreticileri (M3, v0.9.36) — hir_loop.rs
//! yardımcıları. Saf inşaat: Span::default, değişmez adlar.

use hudhudscript_ast::stmt::Stmt;
use hudhudscript_ast::{BinaryOp, Expr, GoalSpecAst, Literal, Span, UnaryOp};

pub(crate) fn fn_decl(name: &str, body: Vec<Stmt>) -> Stmt {
    Stmt::Function {
        name: name.to_string(),
        params: vec![],
        body,
        is_async: false,
        is_generator: false,
        type_params: vec![],
        span: Span::default(),
    }
}

pub(crate) fn call_stmt(f: &str) -> Stmt {
    Stmt::Expr(call(f))
}

pub(crate) fn call(f: &str) -> Expr {
    Expr::Call {
        callee: Box::new(ident(f)),
        args: vec![],
        span: Span::default(),
    }
}

pub(crate) fn let_obj(name: &str, goal: &Option<GoalSpecAst>) -> Stmt {
    let mut props: Vec<(String, Expr)> = vec![("success".into(), bool_lit(false))];
    if let Some(g) = goal {
        props.push(("goal_metric".into(), str_lit(&g.metric)));
        props.push(("goal_target".into(), g.target.clone()));
    }
    Stmt::Let {
        name: name.into(),
        value: Expr::Object { properties: props, span: Span::default() },
        span: Span::default(),
    }
}

pub(crate) fn result_set(prop: &str, v: Expr) -> Stmt {
    Stmt::Assignment {
        target: Expr::Member {
            object: Box::new(ident("result")),
            property: prop.into(),
            span: Span::default(),
        },
        value: v,
        span: Span::default(),
    }
}

pub(crate) fn success_of(v: &str) -> Expr {
    Expr::Member {
        object: Box::new(ident(v)),
        property: "success".into(),
        span: Span::default(),
    }
}

pub(crate) fn ident(n: &str) -> Expr {
    Expr::Identifier(n.into(), Span::default())
}
pub(crate) fn int(v: i64) -> Expr {
    Expr::Literal(Literal::Int(v), Span::default())
}
pub(crate) fn bool_lit(v: bool) -> Expr {
    Expr::Literal(Literal::Boolean(v), Span::default())
}
pub(crate) fn str_lit(s: &str) -> Expr {
    Expr::Literal(Literal::String(s.into()), Span::default())
}
pub(crate) fn bin(l: Expr, op: BinaryOp, r: Expr) -> Expr {
    Expr::Binary {
        left: Box::new(l),
        op,
        right: Box::new(r),
        span: Span::default(),
    }
}
pub(crate) fn eq(l: Expr, r: Expr) -> Expr {
    bin(l, BinaryOp::Eq, r)
}
pub(crate) fn gt(l: Expr, r: Expr) -> Expr {
    bin(l, BinaryOp::Gt, r)
}
pub(crate) fn ge(l: Expr, r: Expr) -> Expr {
    bin(l, BinaryOp::Ge, r)
}
pub(crate) fn add(l: Expr, r: Expr) -> Expr {
    bin(l, BinaryOp::Add, r)
}
pub(crate) fn sub(l: Expr, r: Expr) -> Expr {
    bin(l, BinaryOp::Sub, r)
}
pub(crate) fn and(l: Expr, r: Expr) -> Expr {
    bin(l, BinaryOp::And, r)
}
pub(crate) fn and3(a: Expr, b: Expr, c: Expr) -> Expr {
    and(and(a, b), c)
}
pub(crate) fn not_cont() -> Expr {
    Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(ident("__ll_cont")),
        span: Span::default(),
    }
}
pub(crate) fn wrap_block(stmts: Vec<Stmt>) -> Stmt {
    Stmt::Block { statements: stmts, span: Span::default() }
}
pub(crate) fn let_int(n: &str, v: i64) -> Stmt {
    Stmt::Let { name: n.into(), value: int(v), span: Span::default() }
}
pub(crate) fn let_bool(n: &str, v: bool) -> Stmt {
    Stmt::Let { name: n.into(), value: bool_lit(v), span: Span::default() }
}
pub(crate) fn assign_int(n: &str, v: i64) -> Stmt {
    Stmt::Assignment { target: ident(n), value: int(v), span: Span::default() }
}
pub(crate) fn assign_bool(n: &str, v: bool) -> Stmt {
    Stmt::Assignment { target: ident(n), value: bool_lit(v), span: Span::default() }
}
pub(crate) fn assign_expr(n: &str, v: Expr) -> Stmt {
    Stmt::Assignment { target: ident(n), value: v, span: Span::default() }
}
pub(crate) fn assign_int_expr(n: &str, v: Expr) -> Stmt {
    Stmt::Assignment { target: ident(n), value: v, span: Span::default() }
}
