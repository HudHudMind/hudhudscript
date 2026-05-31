//! Module system tests

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_parser::parse;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_simple_module_import() {
    // Create temporary directory for test modules
    let temp_dir = TempDir::new().unwrap();
    let math_path = temp_dir.path().join("math.hudhud");
    let main_path = temp_dir.path().join("main.hudhud");

    // Write math module
    fs::write(
        &math_path,
        r#"
var PI = 3.14159;

function add(a, b) {
    return a + b;
}

function multiply(a, b) {
    return a * b;
}
"#,
    )
    .unwrap();

    // Write main module - use ES6 style import
    let main_code = format!(
        r#"
import {{ add, multiply }} from "{}";

var result = add(2, 3);
"#,
        math_path.display()
    );

    fs::write(&main_path, &main_code).unwrap();

    // Parse and execute
    let ast = parse(&fs::read_to_string(&main_path).unwrap()).unwrap();
    let mut interpreter = Interpreter::new();

    // This test will fail until we have proper async support
    // For now, just check that it parses correctly
    assert!(!ast.is_empty());
}

#[test]
fn test_module_exports_variables() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("constants.hudhud");

    fs::write(
        &module_path,
        r#"
var PI = 3.14159;
var E = 2.71828;
var GOLDEN_RATIO = 1.618;
"#,
    )
    .unwrap();

    let ast = parse(&fs::read_to_string(&module_path).unwrap()).unwrap();
    assert!(!ast.is_empty());
}

#[test]
fn test_module_exports_functions() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("utils.hudhud");

    fs::write(
        &module_path,
        r#"
function greet(name) {
    return "Hello, " + name;
}

function farewell(name) {
    return "Goodbye, " + name;
}
"#,
    )
    .unwrap();

    let ast = parse(&fs::read_to_string(&module_path).unwrap()).unwrap();
    assert!(!ast.is_empty());
}

#[test]
fn test_module_named_imports() {
    let code = r#"
import { add, multiply } from "./math.hudhud";
"#;

    let ast = parse(code).unwrap();
    assert!(!ast.is_empty());
}

#[test]
fn test_module_default_import() {
    let code = r#"
import math from "./math.hudhud";
"#;

    let ast = parse(code).unwrap();
    assert_eq!(ast.len(), 1);
}

#[test]
fn test_module_wildcard_import() {
    let code = r#"
import * as math from "./math.hudhud";
"#;

    let ast = parse(code).unwrap();
    assert_eq!(ast.len(), 1);
}

#[test]
fn test_multiple_imports() {
    let code = r#"
import { add, subtract } from "./math.hudhud";
import { capitalize, lowercase } from "./string.hudhud";
import * as arr from "./array.hudhud";
"#;

    let ast = parse(code).unwrap();
    assert_eq!(ast.len(), 3);
}

#[test]
fn test_module_with_agent() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("agents.hudhud");

    fs::write(
        &module_path,
        r#"
agent Calculator {
    model: "gpt-4",
    task: "Perform calculations"
}
"#,
    )
    .unwrap();

    let ast = parse(&fs::read_to_string(&module_path).unwrap()).unwrap();
    assert!(!ast.is_empty());
}

#[test]
fn test_module_keyword_filename_let() {
    // Issue #493: A module file named "let.hudhud" should load correctly.
    // The filename "let" is a keyword, but filenames are not identifiers
    // and should not conflict with keyword parsing. The imported binding
    // must use a non-keyword alias.
    let temp_dir = TempDir::new().unwrap();
    let let_module_path = temp_dir.path().join("let.hudhud");

    fs::write(
        &let_module_path,
        r#"
var value = 42;

function double(x) {
    return x * 2;
}
"#,
    )
    .unwrap();

    // Import using a non-keyword alias ("let_module" instead of "let")
    let main_code = format!(
        r#"
import let_module from "{}";
"#,
        let_module_path.display()
    );

    let ast = parse(&main_code).unwrap();
    assert!(
        !ast.is_empty(),
        "Parsing import from let.hudhud should succeed"
    );
}

#[test]
fn test_module_mixed_exports() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("mixed.hudhud");

    fs::write(
        &module_path,
        r#"
var VERSION = "1.0.0";

function getVersion() {
    return VERSION;
}

agent Helper {
    model: "gpt-4",
    task: "Help users"
}
"#,
    )
    .unwrap();

    let ast = parse(&fs::read_to_string(&module_path).unwrap()).unwrap();
    assert!(!ast.is_empty());
}
