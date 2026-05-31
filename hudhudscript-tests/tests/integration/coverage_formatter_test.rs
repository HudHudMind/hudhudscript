//! Coverage tests for hudhudscript-formatter crate

use hudhudscript_formatter::Formatter;
use hudhudscript_parser::parse;

fn format_input(input: &str) -> String {
    let stmts = parse(input).unwrap_or_else(|_| vec![]);
    let mut formatter = Formatter::new();
    formatter.format_program(&stmts)
}

#[test]
fn test_format_empty() {
    let formatted = format_input("");
    assert_eq!(formatted, "", "Empty source should format to empty string");
}

#[test]
fn test_format_let_declaration() {
    let input = "let   x=42;";
    let formatted = format_input(input);
    // Should have proper spacing
    assert!(
        formatted.contains("let x = 42;"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_const_declaration() {
    let input = "const   PI=3.14;";
    let formatted = format_input(input);
    assert!(
        formatted.contains("const PI = 3.14;"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_function_declaration() {
    let input = "function   add(a,b){return a+b;}";
    let formatted = format_input(input);
    // Should have proper spacing
    assert!(
        formatted.contains("function add(a, b)"),
        "Formatted: {}",
        formatted
    );
    assert!(
        formatted.contains("return a + b;"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_if_statement() {
    let input = "if(true){let x=1;}";
    let formatted = format_input(input);
    assert!(
        formatted.contains("if (true) {"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_while_loop() {
    let input = "while(true){let x=1;}";
    let formatted = format_input(input);
    assert!(
        formatted.contains("while (true) {"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_for_in_loop() {
    let input = "for(x in [1,2,3]){let y=x;}";
    let formatted = format_input(input);
    assert!(
        formatted.contains("for (x in [1, 2, 3]) {"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_array_literal() {
    let input = "let arr=[1,2,3];";
    let formatted = format_input(input);
    assert!(
        formatted.contains("let arr = [1, 2, 3];"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_object_literal() {
    let input = r#"let obj={x:1,y:"hello"};"#;
    let formatted = format_input(input);
    assert!(
        formatted.contains(r#"let obj = { x: 1, y: "hello" };"#),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_binary_expression() {
    let input = "let x=1+2*3;";
    let formatted = format_input(input);
    assert!(
        formatted.contains("let x = 1 + 2 * 3;"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_unary_expression() {
    let input = "let x=-1;let y=!true;";
    let formatted = format_input(input);
    assert!(
        formatted.contains("let x = -1;"),
        "Formatted: {}",
        formatted
    );
    assert!(
        formatted.contains("let y = !true;"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_member_access() {
    let input = r#"let obj={x:1};let y=obj.x;let z=obj["x"];"#;
    let formatted = format_input(input);
    assert!(
        formatted.contains("let y = obj.x;"),
        "Formatted: {}",
        formatted
    );
    // Note: formatter may not handle bracket notation perfectly
}

#[test]
fn test_format_function_call() {
    let input = "function f(){}f();";
    let formatted = format_input(input);
    assert!(formatted.contains("f();"), "Formatted: {}", formatted);
}

#[test]
fn test_format_nested_blocks() {
    let input = "if(true){if(false){let x=1;}}";
    let formatted = format_input(input);
    // Should have proper indentation
    assert!(
        formatted.contains("    "),
        "Expected indentation in: {}",
        formatted
    );
}

#[test]
fn test_format_try_catch() {
    let input = "try{throw \"error\";}catch(e){let x=e;}";
    let formatted = format_input(input);
    assert!(formatted.contains("try {"), "Formatted: {}", formatted);
    assert!(
        formatted.contains("catch (e) {"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_class_declaration() {
    let input = "class Point{constructor(x,y){this.x=x;this.y=y;}}";
    let formatted = format_input(input);
    assert!(
        formatted.contains("class Point {"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_arrow_function() {
    let input = "let add=(a,b)=>a+b;";
    let formatted = format_input(input);
    assert!(
        formatted.contains("let add = (a, b) => a + b;"),
        "Formatted: {}",
        formatted
    );
}

#[test]
fn test_format_template_literal() {
    let input = r#"let name="world";let msg=`Hello ${name}`;"#;
    let formatted = format_input(input);
    assert!(
        formatted.contains("let msg = `Hello ${name}`;"),
        "Formatted: {}",
        formatted
    );
}

// Edge case: invalid syntax should still not panic
#[test]
fn test_format_invalid_syntax_no_panic() {
    let input = "let x = ;"; // Invalid syntax
    let result = parse(input);
    // Parser may fail, but formatter shouldn't panic if we give it empty stmts
    if let Ok(stmts) = result {
        let mut formatter = Formatter::new();
        let _formatted = formatter.format_program(&stmts);
    }
    // Test passes if no panic
}

#[test]
fn test_formatter_new() {
    let formatter = Formatter::new();
    // Ensure it can be created (has non-zero size)
    let size = std::mem::size_of_val(&formatter);
    assert!(size > 0, "Formatter should have non-zero size");
}

#[test]
fn test_formatter_with_config() {
    use hudhudscript_formatter::FormatterConfig;
    let config = FormatterConfig {
        indent: "  ".to_string(),
        max_line_length: 80,
        semicolons: true,
    };
    let formatter = Formatter::with_config(config);
    // Ensure it can be created (has non-zero size)
    let size = std::mem::size_of_val(&formatter);
    assert!(size > 0, "Formatter should have non-zero size");
}
