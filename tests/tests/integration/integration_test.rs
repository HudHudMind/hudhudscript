//! Integration tests for the interpreter

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_parser::parse;

fn run_program(source: &str) -> String {
    let ast = parse(source).expect("Parser error");

    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&ast).expect("Runtime error");

    result.to_string()
}

#[test]
fn test_simple_arithmetic() {
    let source = "let x = 10 + 5;";
    run_program(source);
    // Variable is defined, no error
}

#[test]
fn test_variable_assignment() {
    let source = r#"
        let x = 42;
        x = 100;
    "#;
    run_program(source);
}

#[test]
fn test_if_statement() {
    let source = r#"
        let x = 10;
        if (x > 5) {
            x = 20;
        }
    "#;
    run_program(source);
}

#[test]
fn test_while_loop() {
    let source = r#"
        let i = 0;
        while (i < 5) {
            i = i + 1;
        }
    "#;
    run_program(source);
}

#[test]
fn test_for_loop() {
    let source = r#"
        let sum = 0;
        let i = 0;
        for (; i < 10; ) {
            sum = sum + i;
            i = i + 1;
        }
    "#;
    run_program(source);
}

#[test]
fn test_array_operations() {
    let source = r#"
        let arr = [1, 2, 3, 4, 5];
        let first = arr[0];
        let last = arr[4];
    "#;
    run_program(source);
}

#[test]
fn test_object_operations() {
    let source = r#"
        let obj = {x: 10, y: 20};
        let x_val = obj.x;
    "#;
    run_program(source);
}

#[test]
fn test_string_concatenation() {
    let source = r#"
        let greeting = "Hello, " + "World!";
    "#;
    run_program(source);
}

#[test]
fn test_boolean_logic() {
    let source = r#"
        let a = true;
        let b = false;
        let c = a && b;
        let d = a || b;
        let e = !a;
    "#;
    run_program(source);
}

#[test]
fn test_comparison_operators() {
    let source = r#"
        let a = 10;
        let b = 20;
        let eq = a == b;
        let ne = a != b;
        let lt = a < b;
        let gt = a > b;
        let le = a <= b;
        let ge = a >= b;
    "#;
    run_program(source);
}

#[test]
fn test_nested_scopes() {
    let source = r#"
        let x = 10;
        if (true) {
            let y = 20;
            if (true) {
                let z = 30;
            }
        }
    "#;
    run_program(source);
}

#[test]
fn test_fibonacci() {
    let source = r#"
        let a = 0;
        let b = 1;
        let n = 10;
        let i = 0;
        
        while (i < n) {
            let temp = a + b;
            a = b;
            b = temp;
            i = i + 1;
        }
    "#;
    run_program(source);
}
