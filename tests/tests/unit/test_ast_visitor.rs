use hudhudscript_ast::visitor::*;
use hudhudscript_ast::{Decl, Expr, Literal, Span, Stmt};

/// A simple visitor that counts all statement and expression nodes.
struct CountingVisitor {
    stmt_count: usize,
    expr_count: usize,
    decl_count: usize,
}

impl AstVisitor for CountingVisitor {
    fn visit_stmt(&mut self, _stmt: &Stmt) -> VisitControl {
        self.stmt_count += 1;
        VisitControl::Continue
    }
    fn visit_expr(&mut self, _expr: &Expr) -> VisitControl {
        self.expr_count += 1;
        VisitControl::Continue
    }
    fn visit_decl(&mut self, _decl: &Decl) -> VisitControl {
        self.decl_count += 1;
        VisitControl::Continue
    }
}

fn dummy_span() -> Span {
    Span {
        start: hudhudscript_ast::Position {
            offset: 0,
            line: 1,
            column: 1,
        },
        end: hudhudscript_ast::Position {
            offset: 0,
            line: 1,
            column: 1,
        },
    }
}

#[test]
fn test_counting_visitor_simple() {
    let stmts = vec![
        Stmt::Let {
            name: "x".into(),
            value: Expr::Literal(Literal::Number(42.0, false), dummy_span()),
            span: dummy_span(),
        },
        Stmt::Expr(Expr::Identifier("x".into(), dummy_span())),
    ];

    let mut visitor = CountingVisitor {
        stmt_count: 0,
        expr_count: 0,
        decl_count: 0,
    };
    walk_stmts(&mut visitor, &stmts);

    assert_eq!(visitor.stmt_count, 2); // Let + Expr stmt
    assert_eq!(visitor.expr_count, 2); // Literal(42) + Identifier(x)
}

#[test]
fn test_stop_control() {
    /// A visitor that stops after seeing 1 statement.
    struct StopAfterOne {
        count: usize,
    }
    impl AstVisitor for StopAfterOne {
        fn visit_stmt(&mut self, _stmt: &Stmt) -> VisitControl {
            self.count += 1;
            if self.count >= 1 {
                VisitControl::Stop
            } else {
                VisitControl::Continue
            }
        }
    }

    let stmts = vec![
        Stmt::Break { span: dummy_span() },
        Stmt::Break { span: dummy_span() },
        Stmt::Break { span: dummy_span() },
    ];

    let mut visitor = StopAfterOne { count: 0 };
    walk_stmts(&mut visitor, &stmts);
    assert_eq!(visitor.count, 1); // Stopped after the first
}

#[test]
fn test_skip_children() {
    /// A visitor that skips children of If statements.
    struct SkipIfChildren {
        expr_count: usize,
    }
    impl AstVisitor for SkipIfChildren {
        fn visit_stmt(&mut self, stmt: &Stmt) -> VisitControl {
            if matches!(stmt, Stmt::If { .. }) {
                VisitControl::SkipChildren
            } else {
                VisitControl::Continue
            }
        }
        fn visit_expr(&mut self, _expr: &Expr) -> VisitControl {
            self.expr_count += 1;
            VisitControl::Continue
        }
    }

    let stmts = vec![Stmt::If {
        condition: Expr::Literal(Literal::Boolean(true), dummy_span()),
        then_branch: Box::new(Stmt::Expr(Expr::Literal(
            Literal::Number(1.0, false),
            dummy_span(),
        ))),
        else_branch: None,
        span: dummy_span(),
    }];

    let mut visitor = SkipIfChildren { expr_count: 0 };
    walk_stmts(&mut visitor, &stmts);
    // Children of If were skipped, so no expressions visited
    assert_eq!(visitor.expr_count, 0);
}
