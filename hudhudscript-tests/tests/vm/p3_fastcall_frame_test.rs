//! P3-A1: FastCall frame trim regression tests.
//! Lock: recursive fib correctness after Arc::clone removal in fast_call_push_frame.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run_fib(n: i64) -> i64 {
    let src = format!(
        "function fib(n) {{ if (n <= 1) {{ return n; }} return fib(n - 1) + fib(n - 2); }} let out = fib({n});"
    );
    let ast = parse(&src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute failed");
    vm.get_global("out").and_then(|v| v.as_int()).unwrap_or(-1)
}

#[test]
fn p3_recursive_fib_10_correct() {
    assert_eq!(run_fib(10), 55);
}

#[test]
fn p3_recursive_fib_20_correct() {
    assert_eq!(run_fib(20), 6765);
}

#[test]
fn p3_teardown_trim_recursive_fib_25_correct() {
    assert_eq!(run_fib(25), 75025);
}
