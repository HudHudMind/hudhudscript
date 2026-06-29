//! V0.7.138-P0-T: Index hot-path regression tests.
//!
//! Covers the Int-first + Number-fallback index semantics restored for
//! `matrix_multiply` and `palindrome` regressions.  All assertions run
//! through the VM end-to-end.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_parser::parse;

fn run_get_last(src: &str) -> hudhudscript_bytecode::Value16 {
    let ast = parse(src).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.execute(&ast).expect("execute failed");
    interp.get_variable("__last_expr").unwrap_or(hudhudscript_bytecode::Value16::null())
}

#[test]
fn array_index_int() {
    let val = run_get_last(r#"let arr = [10, 20, 30]; arr[1];"#);
    assert_eq!(val.as_int(), Some(20));
}

#[test]
fn array_index_number() {
    let val = run_get_last(r#"let arr = [10, 20, 30]; arr[1.0];"#);
    assert_eq!(val.as_int(), Some(20));
}

#[test]
fn string_ascii_index_int() {
    let val = run_get_last(r#"let s = "abc"; s[1];"#);
    assert_eq!(val.as_string(), Some("b".to_string()));
}

#[test]
fn string_ascii_index_number() {
    let val = run_get_last(r#"let s = "abc"; s[1.0];"#);
    assert_eq!(val.as_string(), Some("b".to_string()));
}

#[test]
fn index_assign_int() {
    let val = run_get_last(r#"let arr = [0, 0, 0]; arr[1] = 42; arr[1];"#);
    assert_eq!(val.as_int(), Some(42));
}

#[test]
fn index_assign_number() {
    let val = run_get_last(r#"let arr = [0, 0, 0]; arr[1.0] = 42; arr[1.0];"#);
    assert_eq!(val.as_int(), Some(42));
}

#[test]
fn matrix_multiply_sample_runs() {
    let src = r#"
let size = 30;
let a = [];
let b = [];
let c = [];
for (let i = 0; i < size; i = i + 1) {
    let row_a = [];
    let row_b = [];
    let row_c = [];
    for (let j = 0; j < size; j = j + 1) {
        row_a.push(i + j);
        row_b.push(i - j);
        row_c.push(0);
    }
    a.push(row_a);
    b.push(row_b);
    c.push(row_c);
}
for (let i = 0; i < size; i = i + 1) {
    for (let j = 0; j < size; j = j + 1) {
        let sum = 0;
        for (let k = 0; k < size; k = k + 1) {
            sum = sum + a[i][k] * b[k][j];
        }
        c[i][j] = sum;
    }
}
print(c[0][0]);
"#;
    let ast = parse(src).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.execute(&ast).expect("matrix_multiply sample failed");
}

#[test]
fn palindrome_sample_runs() {
    let src = r#"
function isPalindrome(s) {
    let n = s.length;
    for (let i = 0; i < n / 2; i = i + 1) {
        if (s[i] != s[n - 1 - i]) {
            return false;
        }
    }
    return true;
}
let result = isPalindrome("aquickbrownfoxjmpsoverthelazydogaquickbrownfoxjmpsoverthelazydog");
print(result);
"#;
    let ast = parse(src).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.execute(&ast).expect("palindrome sample failed");
}
