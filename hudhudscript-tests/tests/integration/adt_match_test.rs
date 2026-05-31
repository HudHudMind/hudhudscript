//! Tests for Issue #92: ADTs (enum declarations) and pattern matching (match statement)

use hudhudscript_ast::{MatchPattern, Stmt};
use hudhudscript_parser::parse;

// ── Enum Declaration Tests ────────────────────────────────────────────────────

#[test]
fn test_enum_simple() {
    let src = r#"enum Direction { North, South, East, West }"#;
    let stmts = parse(src).expect("parse failed");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::EnumDecl { name, variants, .. } => {
            assert_eq!(name, "Direction");
            assert_eq!(variants.len(), 4);
            assert_eq!(variants[0].name, "North");
            assert_eq!(variants[3].name, "West");
        }
        other => panic!("Expected EnumDecl, got {:?}", other),
    }
}

#[test]
fn test_enum_with_fields() {
    let src = r#"enum Shape { Circle(radius), Rectangle(width, height), Point }"#;
    let stmts = parse(src).expect("parse failed");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::EnumDecl { name, variants, .. } => {
            assert_eq!(name, "Shape");
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name, "Circle");
            assert_eq!(variants[0].fields, vec!["radius"]);
            assert_eq!(variants[1].name, "Rectangle");
            assert_eq!(variants[1].fields, vec!["width", "height"]);
            assert_eq!(variants[2].name, "Point");
            assert!(variants[2].fields.is_empty());
        }
        other => panic!("Expected EnumDecl, got {:?}", other),
    }
}

#[test]
fn test_enum_trailing_comma() {
    let src = r#"enum Color { Red, Green, Blue, }"#;
    let stmts = parse(src).expect("parse failed");
    match &stmts[0] {
        Stmt::EnumDecl { variants, .. } => assert_eq!(variants.len(), 3),
        other => panic!("Expected EnumDecl, got {:?}", other),
    }
}

// ── Match Statement Tests ─────────────────────────────────────────────────────

#[test]
fn test_match_wildcard() {
    let src = r#"
        match x {
            _ => { let result = 0; }
}
    "#;
    let stmts = parse(src).expect("parse failed");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
            assert_eq!(arms[0].pattern, MatchPattern::Wildcard);
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

#[test]
fn test_match_identifier_pattern() {
    let src = r#"
        match status {
            active => { let msg = "running"; }
            _ => { let msg = "stopped"; }
}
    "#;
    let stmts = parse(src).expect("parse failed");
    match &stmts[0] {
        Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            assert!(matches!(&arms[0].pattern, MatchPattern::Identifier(n) if n == "active"));
            assert_eq!(arms[1].pattern, MatchPattern::Wildcard);
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

#[test]
fn test_match_enum_variant_no_binding() {
    let src = r#"
        match shape {
            Shape::Circle => { let area = 0; }
            Shape::Point => { let area = 0; }
            _ => { let area = 0; }
}
    "#;
    let stmts = parse(src).expect("parse failed");
    match &stmts[0] {
        Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);
            assert!(matches!(
                &arms[0].pattern,
                MatchPattern::EnumVariant { enum_name, variant, binding: None }
                if enum_name == "Shape" && variant == "Circle"
            ));
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

#[test]
fn test_match_enum_variant_with_binding() {
    let src = r#"
        match shape {
            Shape::Circle(r) => { let area = r; }
            _ => { let area = 0; }
}
    "#;
    let stmts = parse(src).expect("parse failed");
    match &stmts[0] {
        Stmt::Match { arms, .. } => {
            assert!(matches!(
                &arms[0].pattern,
                MatchPattern::EnumVariant { enum_name, variant, binding: Some(b) }
                if enum_name == "Shape" && variant == "Circle" && b == "r"
            ));
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

#[test]
fn test_match_literal_number() {
    let src = r#"
        match code {
            42 => { let found = true; }
            _ => { let found = false; }
}
    "#;
    let stmts = parse(src).expect("parse failed");
    match &stmts[0] {
        Stmt::Match { arms, .. } => {
            assert!(matches!(
                &arms[0].pattern,
                MatchPattern::Literal(hudhudscript_ast::Literal::Number(n, _)) if *n == 42.0
            ));
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ── Combined: enum + match ────────────────────────────────────────────────────

#[test]
fn test_enum_then_match() {
    let src = r#"
        enum Status { Active, Inactive, Pending }
        match s {
            Status::Active => { let ok = true; }
            Status::Inactive => { let ok = false; }
            _ => { let ok = false; }
}
    "#;
    let stmts = parse(src).expect("parse failed");
    assert_eq!(stmts.len(), 2);
    assert!(matches!(&stmts[0], Stmt::EnumDecl { name, .. } if name == "Status"));
    assert!(matches!(&stmts[1], Stmt::Match { .. }));
}
