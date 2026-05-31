//! End-to-end pipeline tests: parse → compile → VM execute
//!
//! Each test verifies the FULL pipeline for a real HudHudScript program:
//! source code → parser → compiler → bytecode → VM → check globals.

use hudhudscript_bytecode::error::{CompileError, CompileResult};
use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn run(source: &str) -> CompileResult<VM> {
    let stmts = parse(source).map_err(|e| {
        hudhudscript_bytecode::error::compile_codes::runtime_error(format!("Parse error: {:?}", e))
    })?;
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts)?;
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

fn assert_number(vm: &VM, name: &str, expected: f64) {
    match vm.get_variable(name).and_then(|v| v.as_number()) {
        Some(n) => assert!(
            (n - expected).abs() < 1e-10,
            "expected {} = {}, got {}",
            name,
            expected,
            n
        ),
        other => panic!("expected number for '{}', got {:?}", name, other),
    }
}

fn assert_string(vm: &VM, name: &str, expected: &str) {
    match vm.get_variable(name).and_then(|v| v.as_str()) {
        Some(s) => assert_eq!(s, expected, "var '{}'", name),
        other => panic!("expected string for '{}', got {:?}", name, other),
    }
}

fn assert_number_array(vm: &VM, name: &str, expected: &[f64]) {
    match vm.get_variable(name).and_then(|v| v.as_array()) {
        Some(arr) => {
            assert_eq!(
                arr.len(),
                expected.len(),
                "array '{}' length mismatch",
                name
            );
            for (i, (got, exp)) in arr.iter().zip(expected.iter()).enumerate() {
                match got.as_number() {
                    Some(n) => assert!(
                        (n - exp).abs() < 1e-10,
                        "array '{}' element {}: expected {}, got {}",
                        name,
                        i,
                        exp,
                        n
                    ),
                    other => panic!(
                        "array '{}' element {}: expected number, got {:?}",
                        name, i, other
                    ),
                }
            }
        }
        other => panic!("expected array for '{}', got {:?}", name, other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Basic arithmetic
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_basic_arithmetic() {
    let vm = run(r#"
        let x = 1 + 2;
        let y = x * 3;
    "#)
    .unwrap();
    assert_number(&vm, "x", 3.0);
    assert_number(&vm, "y", 9.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. String concatenation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_string_concatenation() {
    let vm = run(r#"
        let s = "hello" + " world";
    "#)
    .unwrap();
    assert_string(&vm, "s", "hello world");
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Function definition and call
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_function_call() {
    let vm = run(r#"
        function add(a, b) { return a + b; }
        let r = add(3, 4);
    "#)
    .unwrap();
    assert_number(&vm, "r", 7.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. If/else
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_if_else() {
    let vm = run(r#"
        let x = 10;
        let r = "";
        if (x > 5) { r = "big"; } else { r = "small"; }
    "#)
    .unwrap();
    assert_string(&vm, "r", "big");
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. While loop
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_while_loop() {
    let vm = run(r#"
        let sum = 0;
        let i = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
    "#)
    .unwrap();
    assert_number(&vm, "sum", 10.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. For-in loop
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_for_in_loop() {
    let vm = run(r#"
        let arr = [1, 2, 3];
        let sum = 0;
        for (x in arr) {
            sum = sum + x;
        }
    "#)
    .unwrap();
    assert_number(&vm, "sum", 6.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Array indexing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_array_indexing() {
    let vm = run(r#"
        let arr = [10, 20, 30];
        let x = arr[1];
    "#)
    .unwrap();
    assert_number(&vm, "x", 20.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Object literal and property access
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_object_property_access() {
    let vm = run(r#"
        let obj = {name: "test", value: 42};
        let v = obj.value;
    "#)
    .unwrap();
    assert_number(&vm, "v", 42.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Enum declaration and access
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_enum_access() {
    let vm = run(r#"
        enum Color { Red, Green, Blue }
        let c = Color.Red;
    "#)
    .unwrap();
    // Enum variants may be stored as "Color::Red" string or similar
    let val = vm.get_variable("c");
    match val.and_then(|v| v.as_str()) {
        Some(s) => {
            assert!(
                s.contains("Red"),
                "enum variant should contain 'Red', got: {}",
                s
            );
        }
        other => {
            // Accept any value but report what we got
            panic!("expected enum variant string for 'c', got {:?}", other);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Match expression
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_match_expression() {
    let vm = run(r#"
        let x = 5;
        let r = "";
        match (x) {
            5 => { r = "five"; }
            _ => { r = "other"; }
        }
    "#)
    .unwrap();
    assert_string(&vm, "r", "five");
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Try/catch with throw
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_try_catch() {
    let vm = run(r#"
        let r = "";
        try {
            throw "oops";
        } catch (e) {
            r = "caught";
        }
    "#)
    .unwrap();
    assert_string(&vm, "r", "caught");
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Arrow function (expression body)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_arrow_function() {
    let vm = run(r#"
        let double = (x) => x * 2;
        let r = double(5);
    "#)
    .unwrap();
    assert_number(&vm, "r", 10.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 13. Closure (arrow function capturing outer variable)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_closure() {
    let vm = run(r#"
        function makeAdder(n) { return (x) => x + n; }
        let add5 = makeAdder(5);
        let r = add5(3);
    "#)
    .unwrap();
    assert_number(&vm, "r", 8.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 14. Template string interpolation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_template_string() {
    let vm = run(r#"
        let name = "world";
        let msg = `hello ${name}`;
    "#)
    .unwrap();
    assert_string(&vm, "msg", "hello world");
}

// ─────────────────────────────────────────────────────────────────────────────
// 15. Class with constructor
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_class_constructor() {
    let vm = run(r#"
        class Dog {
            constructor(name) {
                this.name = name;
            }
        }
        let d = new Dog("Rex");
        let n = d.name;
    "#)
    .unwrap();
    assert_string(&vm, "n", "Rex");
}

// ─────────────────────────────────────────────────────────────────────────────
// 16. filter builtin with arrow function
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_filter_builtin() {
    let vm = run(r#"
        let arr = [1, 2, 3, 4];
        let evens = filter(arr, (x) => x % 2 == 0);
    "#)
    .unwrap();
    assert_number_array(&vm, "evens", &[2.0, 4.0]);
}
