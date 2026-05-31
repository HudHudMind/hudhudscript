//! Kimi - parser coverage tests
//! Test edilen crate: hudhudscript-parser

use hudhudscript_ast::Stmt;
use hudhudscript_parser::parse;

#[test]
fn parse_let_declaration() {
    let stmts = parse("let x = 42;").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Let { .. }));
}

#[test]
fn parse_const_declaration() {
    let stmts = parse("const PI = 3.14;").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Const { .. }));
}

#[test]
fn parse_function_declaration() {
    let stmts = parse("function add(a, b) { return a + b; }").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Function { .. }));
}

#[test]
fn parse_if_statement() {
    let stmts = parse("if (x > 0) { return true; }").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::If { .. }));
}

#[test]
fn parse_while_loop() {
    let stmts = parse("while (x < 10) { x = x + 1; }").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::While { .. }));
}

#[test]
fn parse_for_in_loop() {
    let stmts = parse("for (item in list) { print(item); }").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::For { .. }));
}

#[test]
fn parse_try_catch() {
    let stmts = parse("try { risky(); } catch (e) { handle(e); }").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Try { .. }));
}

#[test]
fn parse_class_declaration() {
    let stmts = parse("class Point {}").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Class(..)));
}

#[test]
fn parse_binary_expression() {
    let stmts = parse("let x = 1 + 2;").unwrap();
    // Should parse the binary expression
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_invalid_syntax_error() {
    // Parser should handle or error on invalid syntax
    let result = parse("}{}{}{");
    // Unbalanced braces should produce a parse error
    assert!(result.is_err(), "Unbalanced braces should fail to parse");
}

#[test]
fn parse_empty_source() {
    let stmts = parse("").unwrap();
    assert!(stmts.is_empty());
}

#[test]
fn parse_array_literal() {
    let stmts = parse("let arr = [1, 2, 3];").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_object_literal() {
    let stmts = parse("let obj = { name: \"test\", value: 42 };").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_arrow_function() {
    let stmts = parse("let add = (x, y) => x + y;").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_switch_statement() {
    let stmts = parse("switch (x) { case 1: break; default: break; }").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Switch { .. }));
}

#[test]
fn parse_import_statement() {
    let stmts = parse("import { foo } from \"module\";").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Import { .. }));
}

#[test]
fn parse_agent_declaration() {
    let stmts = parse("agent TestAgent { }").unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Decl(..)));
}

#[test]
fn parse_template_string() {
    let stmts = parse("let msg = `Hello ${name}!`;").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_comments_ignored() {
    let stmts = parse("// single line comment\nlet x = 1;").unwrap();
    assert_eq!(stmts.len(), 1);
}
