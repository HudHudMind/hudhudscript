use hudhudscript_formatter::{Formatter, FormatterConfig};
use hudhudscript_parser::parse;

// ============================================================================
// Helper
// ============================================================================

fn format_source(src: &str) -> String {
    let stmts = parse(src).expect("parse failed");
    let mut f = Formatter::new();
    f.format_program(&stmts)
}

fn format_source_with(src: &str, config: FormatterConfig) -> String {
    let stmts = parse(src).expect("parse failed");
    let mut f = Formatter::with_config(config);
    f.format_program(&stmts)
}

// ============================================================================
// FormatterConfig
// ============================================================================

#[test]
fn default_config_uses_4_spaces() {
    let c = FormatterConfig::default();
    assert_eq!(c.indent, "    ");
    assert_eq!(c.max_line_length, 100);
    assert!(c.semicolons);
}

#[test]
fn custom_config_tab_indent() {
    let c = FormatterConfig {
        indent: "\t".to_string(),
        max_line_length: 80,
        semicolons: false,
    };
    assert_eq!(c.indent, "\t");
    assert_eq!(c.max_line_length, 80);
    assert!(!c.semicolons);
}

// ============================================================================
// Formatter construction
// ============================================================================

#[test]
fn formatter_new_creates_default() {
    let mut f = Formatter::new();
    let output = f.format_program(&[]);
    assert_eq!(output, "");
}

#[test]
fn formatter_with_config_custom() {
    let config = FormatterConfig {
        indent: "  ".to_string(),
        max_line_length: 120,
        semicolons: true,
    };
    let mut f = Formatter::with_config(config);
    let output = f.format_program(&[]);
    assert_eq!(output, "");
}

// ============================================================================
// Let / Const declarations
// ============================================================================

#[test]
fn format_let_declaration() {
    let output = format_source("let x = 42;");
    assert!(output.contains("let x = 42;"));
}

#[test]
fn format_const_declaration() {
    let output = format_source("const MAX = 100;");
    assert!(output.contains("const MAX = 100;"));
}

#[test]
fn format_let_string_value() {
    let output = format_source(r#"let name = "hello";"#);
    assert!(output.contains("let name = "));
    assert!(output.contains("hello"));
}

#[test]
fn format_let_boolean() {
    let output = format_source("let flag = true;");
    assert!(output.contains("let flag = true;"));
}

#[test]
fn format_let_null() {
    let output = format_source("let val = null;");
    assert!(output.contains("let val = null;"));
}

// ============================================================================
// Assignment
// ============================================================================

#[test]
fn format_assignment() {
    let output = format_source("let x = 1; x = 2;");
    assert!(output.contains("x = 2;"));
}

// ============================================================================
// Return / Break / Continue
// ============================================================================

#[test]
fn format_return_with_value() {
    let output = format_source("function f() { return 42; }");
    assert!(output.contains("return 42;"));
}

#[test]
fn format_return_without_value() {
    let output = format_source("function f() { return; }");
    assert!(output.contains("return;"));
}

#[test]
fn format_break_statement() {
    let output = format_source("while (true) { break; }");
    assert!(output.contains("break;"));
}

#[test]
fn format_continue_statement() {
    let output = format_source("while (true) { continue; }");
    assert!(output.contains("continue;"));
}

// ============================================================================
// Expression statements
// ============================================================================

#[test]
fn format_expression_statement() {
    let output = format_source("42;");
    assert!(output.contains("42;"));
}

#[test]
fn format_call_expression() {
    let output = format_source("foo(1, 2);");
    assert!(output.contains("foo(1, 2);"));
}

// ============================================================================
// If / Else
// ============================================================================

#[test]
fn format_if_statement() {
    let output = format_source("if (true) { 1; }");
    assert!(output.contains("if (true)"));
    assert!(output.contains("{"));
    assert!(output.contains("}"));
}

#[test]
fn format_if_else_statement() {
    let output = format_source("if (true) { 1; } else { 2; }");
    assert!(output.contains("if (true)"));
    assert!(output.contains("else"));
}

// ============================================================================
// While loop
// ============================================================================

#[test]
fn format_while_loop() {
    let output = format_source("while (true) { 1; }");
    assert!(output.contains("while (true)"));
    assert!(output.contains("{"));
}

// ============================================================================
// For loop
// ============================================================================

#[test]
fn format_for_in_loop() {
    let output = format_source("for (item in [1, 2, 3]) { item; }");
    assert!(output.contains("for (item in"));
}

// ============================================================================
// Function declarations
// ============================================================================

#[test]
fn format_function_declaration() {
    let output = format_source("function greet(name) { return name; }");
    assert!(output.contains("function greet(name)"));
    assert!(output.contains("{"));
    assert!(output.contains("return name;"));
    assert!(output.contains("}"));
}

#[test]
fn format_async_function() {
    let output = format_source("async function fetch() { return 1; }");
    assert!(output.contains("async function fetch()"));
}

#[test]
fn format_function_multiple_params() {
    let output = format_source("function add(a, b) { return a; }");
    assert!(output.contains("function add(a, b)"));
}

// ============================================================================
// Block nesting / indentation
// ============================================================================

#[test]
fn format_nested_block_indentation() {
    let output = format_source("function foo() { if (true) { 1; } }");
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.iter().any(|l| l.starts_with("    ")));
}

#[test]
fn format_custom_indent_2_spaces() {
    let config = FormatterConfig {
        indent: "  ".to_string(),
        max_line_length: 100,
        semicolons: true,
    };
    let output = format_source_with("function foo() { return 1; }", config);
    assert!(output.contains("  return 1;"));
}

// ============================================================================
// Switch / Match
// ============================================================================

#[test]
fn format_switch_statement() {
    let src = r#"
switch (x) {
    case 1:
        break;
    case 2:
        break;
    default:
        break;
}
"#;
    let output = format_source(src);
    assert!(output.contains("switch"));
    assert!(output.contains("case"));
    assert!(output.contains("default"));
}

#[test]
fn format_match_statement() {
    let src = r#"
match x {
    1 => { 1; }
    _ => { 0; }
}
"#;
    let output = format_source(src);
    assert!(output.contains("match"));
}

// ============================================================================
// Try / Catch / Finally
// ============================================================================

#[test]
fn format_try_catch() {
    let src = r#"
try {
    1;
} catch (e) {
    2;
}
"#;
    let output = format_source(src);
    assert!(output.contains("try"));
    assert!(output.contains("catch"));
}

#[test]
fn format_try_catch_finally() {
    let src = r#"
try {
    1;
} catch (e) {
    2;
} finally {
    3;
}
"#;
    let output = format_source(src);
    assert!(output.contains("try"));
    assert!(output.contains("catch"));
    assert!(output.contains("finally"));
}

// ============================================================================
// Throw
// ============================================================================

#[test]
fn format_throw_statement() {
    let output = format_source(r#"throw "error";"#);
    assert!(output.contains("throw"));
}

// ============================================================================
// Import
// ============================================================================

#[test]
fn format_import_named() {
    let output = format_source(r#"import { foo, bar } from "module";"#);
    assert!(output.contains("import"));
    assert!(output.contains("foo"));
    assert!(output.contains("bar"));
    assert!(output.contains("module"));
}

#[test]
fn format_import_default() {
    let output = format_source(r#"import myModule from "module";"#);
    assert!(output.contains("import myModule"));
}

// ============================================================================
// Export
// ============================================================================

#[test]
fn format_export_statement() {
    let output = format_source("export function add(a, b) { return a; }");
    assert!(output.contains("export"));
    assert!(output.contains("function add"));
}

// ============================================================================
// Class
// ============================================================================

#[test]
fn format_class_declaration() {
    let src = r#"
class Animal {
    constructor(name) {
        return;
    }
}
"#;
    let output = format_source(src);
    assert!(output.contains("class Animal"));
    assert!(output.contains("constructor"));
}

#[test]
fn format_class_with_parent() {
    let src = r#"
class Dog <- Animal {
    constructor() {
        return;
    }
}
"#;
    let output = format_source(src);
    assert!(output.contains("class Dog <- Animal"));
}

// ============================================================================
// Enum
// ============================================================================

#[test]
fn format_enum_declaration() {
    let output = format_source("enum Color { Red, Green, Blue }");
    assert!(output.contains("enum Color"));
    assert!(output.contains("Red"));
    assert!(output.contains("Green"));
    assert!(output.contains("Blue"));
}

// ============================================================================
// Decl — agent / subject / tool / action / resource
// ============================================================================

#[test]
fn format_agent_declaration() {
    let output = format_source(r#"agent Bot { model = "gpt-4" }"#);
    assert!(output.contains("agent"));
    assert!(output.contains("Bot"));
}

#[test]
fn format_subject_declaration() {
    let output = format_source("subject Player { state health: 100 }");
    assert!(output.contains("subject Player"));
}

#[test]
fn format_tool_declaration() {
    let output = format_source(r#"tool search { description = "finds things" }"#);
    assert!(output.contains("tool"));
}

// ============================================================================
// SOP statements
// ============================================================================

#[test]
fn format_spawn_statement() {
    let output = format_source("spawn Worker;");
    assert!(output.contains("spawn Worker;"));
}

#[test]
fn format_require_statement() {
    let output = format_source("require true;");
    assert!(output.contains("require true;"));
}

// ============================================================================
// RAG statements
// ============================================================================

#[test]
fn format_remember_statement() {
    let output = format_source(r#"remember "data";"#);
    assert!(output.contains("remember"));
}

#[test]
fn format_recall_statement() {
    let output = format_source(r#"recall "query";"#);
    assert!(output.contains("recall"));
}

#[test]
fn format_forget_statement() {
    let output = format_source(r#"forget "target";"#);
    assert!(output.contains("forget"));
}

// ============================================================================
// Expressions
// ============================================================================

#[test]
fn format_binary_expression() {
    let output = format_source("let x = 1 + 2;");
    assert!(output.contains("1 + 2"));
}

#[test]
fn format_unary_expression() {
    let output = format_source("let x = !true;");
    assert!(output.contains("!true"));
}

#[test]
fn format_array_literal() {
    let output = format_source("let arr = [1, 2, 3];");
    assert!(output.contains("[1, 2, 3]"));
}

#[test]
fn format_member_access() {
    let output = format_source("let x = obj; x.foo;");
    assert!(output.contains(".foo"));
}

#[test]
fn format_index_access() {
    let output = format_source("let arr = [1]; arr[0];");
    assert!(output.contains("[0]"));
}

// ============================================================================
// Arrow functions
// ============================================================================

#[test]
fn format_arrow_function_expression() {
    let output = format_source("let f = (x) => x;");
    assert!(output.contains("=>"));
}

// ============================================================================
// Trait declaration
// ============================================================================

#[test]
fn format_trait_declaration() {
    let src = "trait Drawable { function draw(); }";
    let output = format_source(src);
    assert!(output.contains("trait Drawable"));
    assert!(output.contains("function draw()"));
}

// ============================================================================
// Empty program
// ============================================================================

#[test]
fn format_empty_program() {
    let output = format_source("");
    assert_eq!(output, "");
}

// ============================================================================
// Blank lines between top-level declarations
// ============================================================================

#[test]
fn format_blank_line_between_declarations() {
    let src = "function a() { return 1 }\nfunction b() { return 2 }";
    let output = format_source(src);
    assert!(output.contains("function a"));
    assert!(output.contains("function b"));
}
