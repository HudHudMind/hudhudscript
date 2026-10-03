//! Coverage tests for UI declaration parsing
//! (`crates/hudhudscript-parser/src/parser/declarations/ui.rs`).
//!
//! Every test parses a `ui` declaration through the public
//! `hudhudscript_parser::parse` entry point and pins the exact resulting
//! `Decl::UiApp` AST shape: app name, entry screen, screens (name, params,
//! body), components (name, props, body) and widget nodes (type, label,
//! props, style split, children, spans).

use hudhudscript_ast::{Decl, Expr, Literal, Stmt, UiComponentDecl, UiNode, UiScreenDecl};
use hudhudscript_errors::ErrorCode;
use hudhudscript_parser::parse;

// ── helpers ──────────────────────────────────────────────────────────

/// Parse a source that must yield exactly one statement.
#[rustfmt::skip]
fn parse_first(src: &str) -> Stmt {
    let stmts = parse(src).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert_eq!(stmts.len(), 1);
    stmts.into_iter().next().unwrap()
}

/// Assert the source fails to parse and return the unified error.
#[rustfmt::skip]
fn parse_err(src: &str) -> hudhudscript_parser::ParseError {
    match parse(src) {
        Ok(stmts) => panic!("expected a parse error, got {} statements", stmts.len()),
        Err(e) => e,
    }
}

/// Borrowed `Decl::UiApp` fields (span kept aside for the span tests).
struct UiAppRef<'a> {
    name: &'a String,
    entry_screen: &'a Option<String>,
    screens: &'a Vec<UiScreenDecl>,
    components: &'a Vec<UiComponentDecl>,
    span: &'a hudhudscript_ast::Span,
}

/// Extract the `UiApp` declaration out of a parsed statement.
fn ui_app(stmt: &Stmt) -> UiAppRef<'_> {
    match stmt {
        Stmt::Decl(Decl::UiApp {
            name,
            entry_screen,
            screens,
            components,
            span,
        }) => UiAppRef {
            name,
            entry_screen,
            screens,
            components,
            span,
        },
        other => panic!("expected Stmt::Decl(Decl::UiApp), got {other:?}"),
    }
}

/// Widget fields as (type, label, props, children, style).
type WidgetRef<'a> = (
    &'a str,
    &'a Option<Expr>,
    &'a Vec<(String, Expr)>,
    &'a Vec<UiNode>,
    &'a Vec<(String, Expr)>,
);

/// Extract the widget fields out of a UI body node.
fn as_widget(node: &UiNode) -> WidgetRef<'_> {
    match node {
        UiNode::Widget {
            widget_type,
            label,
            props,
            events: _,
            children,
            style,
            span: _,
        } => (widget_type, label, props, children, style),
        other => panic!("expected UiNode::Widget, got {other:?}"),
    }
}

/// Unwrap a string literal expression (quotes already stripped).
fn as_string(expr: &Expr) -> &str {
    match expr {
        Expr::Literal(Literal::String(s), _) => s,
        other => panic!("expected string literal, got {other:?}"),
    }
}

/// Unwrap an int literal expression.
fn as_int(expr: &Expr) -> i64 {
    match expr {
        Expr::Literal(Literal::Int(i), _) => *i,
        other => panic!("expected int literal, got {other:?}"),
    }
}

// ── app-level members ────────────────────────────────────────────────

#[test]
fn ui_app_empty_members_shape_and_span() {
    // 16-character single-line source: the span must cover it entirely.
    let stmt = parse_first("ui Dashboard { }");
    let app = ui_app(&stmt);
    assert_eq!(app.name, "Dashboard");
    assert_eq!(*app.entry_screen, None);
    assert!(app.screens.is_empty());
    assert!(app.components.is_empty());
    assert_eq!((app.span.start.line, app.span.start.column), (1, 1));
    assert_eq!(app.span.end.column, 17, "span ends after the closing brace");
}

#[test]
fn ui_app_entry_member_sets_entry_screen() {
    let stmt = parse_first("ui App { entry: Main; }");
    let app = ui_app(&stmt);
    assert_eq!(app.name, "App");
    assert_eq!(*app.entry_screen, Some("Main".to_string()));
    assert!(app.screens.is_empty());
    assert!(app.components.is_empty());
}

#[test]
fn ui_app_generic_fields_are_consumed_as_metadata() {
    // ui_field members validate but must not surface as entry/screens/components.
    let stmt = parse_first(r#"ui App { title: "Main" version: 2 }"#);
    let app = ui_app(&stmt);
    assert_eq!(*app.entry_screen, None);
    assert_eq!(app.screens.len(), 0);
    assert_eq!(app.components.len(), 0);
}

#[test]
fn ui_app_members_preserve_declaration_order() {
    let src = r#"
        ui App {
            entry: Main;
            screen Main { }
            screen Login { }
            component Card { }
            component Row { }
        }
    "#;
    let stmt = parse_first(src);
    let app = ui_app(&stmt);
    assert_eq!(*app.entry_screen, Some("Main".to_string()));
    let screen_names: Vec<&str> = app.screens.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(screen_names, ["Main", "Login"]);
    let component_names: Vec<&str> = app.components.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(component_names, ["Card", "Row"]);
}

// ── screens and components ───────────────────────────────────────────

#[test]
fn ui_screen_with_labelled_widget() {
    let src = r#"
        ui App {
            screen Home {
                text "Hello"
            }
        }
    "#;
    let stmt = parse_first(src);
    let app = ui_app(&stmt);
    assert_eq!(app.components.len(), 0);
    assert_eq!(app.screens.len(), 1);
    let screen = &app.screens[0];
    assert_eq!(screen.name, "Home");
    assert!(screen.params.is_empty());
    assert_eq!(screen.body.len(), 1);
    let (widget_type, label, props, children, style) = as_widget(&screen.body[0]);
    assert_eq!(widget_type, "text");
    assert_eq!(as_string(label.as_ref().unwrap()), "Hello");
    assert_eq!(props.len(), 0);
    assert_eq!(style.len(), 0);
    assert_eq!(children.len(), 0);
}

#[test]
fn ui_screen_params_captured_in_order() {
    let stmt = parse_first("ui App { screen Home(user, role) { } }");
    let app = ui_app(&stmt);
    assert_eq!(app.screens.len(), 1);
    assert_eq!(app.screens[0].name, "Home");
    assert_eq!(app.screens[0].params, ["user", "role"]);
    assert!(app.screens[0].body.is_empty());
}

#[test]
fn ui_component_shape_has_empty_props() {
    let src = r#"
        ui App {
            component Card {
                text "Title"
            }
        }
    "#;
    let stmt = parse_first(src);
    let app = ui_app(&stmt);
    assert_eq!(app.screens.len(), 0);
    assert_eq!(app.components.len(), 1);
    let component = &app.components[0];
    assert_eq!(component.name, "Card");
    assert!(
        component.props.is_empty(),
        "parser leaves component props empty"
    );
    assert_eq!(component.body.len(), 1);
    let (widget_type, label, _, _, _) = as_widget(&component.body[0]);
    assert_eq!(widget_type, "text");
    assert_eq!(as_string(label.as_ref().unwrap()), "Title");
}

// ── widget props, style split, labels ────────────────────────────────

#[test]
fn ui_widget_style_and_plain_props_are_split() {
    let src = r#"
        ui App {
            screen Home {
                button "Go" {
                    id: "save"
                    color: "blue"
                    size: 32
                    opacity: 0.5
                }
            }
        }
    "#;
    let stmt = parse_first(src);
    let app = ui_app(&stmt);
    let (widget_type, label, props, children, style) = as_widget(&app.screens[0].body[0]);
    assert_eq!(widget_type, "button");
    assert_eq!(as_string(label.as_ref().unwrap()), "Go");
    // Non-style props stay in `props` in source order.
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].0, "id");
    assert_eq!(as_string(&props[0].1), "save");
    // Style keywords are routed to `style` in source order.
    assert_eq!(style.len(), 3);
    assert_eq!(style[0].0, "color");
    assert_eq!(as_string(&style[0].1), "blue");
    assert_eq!(style[1].0, "size");
    assert_eq!(as_int(&style[1].1), 32);
    assert_eq!(style[2].0, "opacity");
    match &style[2].1 {
        Expr::Literal(Literal::Number(f, is_float), _) => {
            assert!((f - 0.5).abs() < 1e-12, "opacity value, got {f}");
            assert!(*is_float, "0.5 must parse as a float literal");
        }
        other => panic!("expected Number, got {other:?}"),
    }
    assert_eq!(children.len(), 0);
}

#[test]
fn ui_widget_expression_label() {
    // A non-string label is parsed as a full expression (Rule::expression path).
    let stmt = parse_first("ui App { screen Home { text count + 1 } }");
    let app = ui_app(&stmt);
    let (_, label, _, _, _) = as_widget(&app.screens[0].body[0]);
    match label.as_ref().unwrap() {
        Expr::Binary {
            left, op, right, ..
        } => {
            assert!(matches!(**left, Expr::Identifier(ref n, _) if n == "count"));
            assert_eq!(*op, hudhudscript_ast::BinaryOp::Add);
            assert!(matches!(**right, Expr::Literal(Literal::Int(1), _)));
        }
        other => panic!("expected Binary label, got {other:?}"),
    }
}

#[test]
fn ui_widget_nested_children() {
    let src = r#"
        ui App {
            screen Home {
                column {
                    text "a"
                    row {
                        text "b"
                    }
                }
            }
        }
    "#;
    let stmt = parse_first(src);
    let app = ui_app(&stmt);
    let (widget_type, label, _, children, _) = as_widget(&app.screens[0].body[0]);
    assert_eq!(widget_type, "column");
    assert!(label.is_none(), "column has no label");
    assert_eq!(children.len(), 2, "column has two direct children");
    let (t0, l0, _, c0, _) = as_widget(&children[0]);
    assert_eq!(t0, "text");
    assert_eq!(as_string(l0.as_ref().unwrap()), "a");
    assert_eq!(c0.len(), 0);
    let (t1, l1, _, c1, _) = as_widget(&children[1]);
    assert_eq!(t1, "row");
    assert!(l1.is_none());
    assert_eq!(c1.len(), 1, "row nests one widget");
    let (t2, l2, _, _, _) = as_widget(&c1[0]);
    assert_eq!(t2, "text");
    assert_eq!(as_string(l2.as_ref().unwrap()), "b");
}

#[test]
fn ui_widget_span_offsets() {
    // Single line: the widget pair starts at byte 23 (column 24). The
    // `ui_widget` grammar rule skips implicit whitespace after the label
    // while probing the optional `{ ... }` groups, so the pair consumes the
    // single space after the closing quote too: the span ends at byte 33
    // (column 34), one past that space.
    let src = r#"ui App { screen Home { text "Hi" } }"#;
    let stmt = parse_first(src);
    let app = ui_app(&stmt);
    match &app.screens[0].body[0] {
        UiNode::Widget { span, .. } => {
            assert_eq!((span.start.line, span.start.column), (1, 24));
            assert_eq!(span.end.column, 34);
        }
        other => panic!("expected Widget, got {other:?}"),
    }
}

// ── expression statements inside UI bodies ───────────────────────────

#[test]
fn ui_body_expression_statement_becomes_expr_node() {
    // `1 + 2` cannot start a widget/var/platform/event, so it falls through
    // to the expression alternative and becomes UiNode::Expr.
    let stmt = parse_first("ui App { screen Home { 1 + 2 } }");
    let app = ui_app(&stmt);
    assert_eq!(app.screens[0].body.len(), 1);
    match &app.screens[0].body[0] {
        UiNode::Expr(expr) => match expr {
            Expr::Binary {
                left, op, right, ..
            } => {
                assert!(matches!(**left, Expr::Literal(Literal::Int(1), _)));
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Add);
                assert!(matches!(**right, Expr::Literal(Literal::Int(2), _)));
            }
            other => panic!("expected Binary, got {other:?}"),
        },
        other => panic!("expected UiNode::Expr, got {other:?}"),
    }
}

// ── malformed UI declarations ────────────────────────────────────────

#[test]
fn ui_decl_unclosed_brace_is_invalid_syntax_error() {
    let err = parse_err("ui App {");
    assert_eq!(err.code, ErrorCode::ParseInvalidSyntax);
    assert!(
        err.message.starts_with("Invalid syntax"),
        "got: {}",
        err.message
    );
}

#[test]
fn ui_entry_missing_name_is_invalid_syntax_error() {
    let err = parse_err("ui App { entry: }");
    assert_eq!(err.code, ErrorCode::ParseInvalidSyntax);
    assert!(
        err.message.starts_with("Invalid syntax"),
        "got: {}",
        err.message
    );
    // pest echoes the offending source line in its error rendering.
    assert!(err.message.contains("entry:"), "got: {}", err.message);
}

#[test]
fn ui_decl_missing_opening_brace_is_invalid_syntax_error() {
    let err = parse_err("ui App }");
    assert_eq!(err.code, ErrorCode::ParseInvalidSyntax);
    assert!(
        err.message.starts_with("Invalid syntax"),
        "got: {}",
        err.message
    );
}
