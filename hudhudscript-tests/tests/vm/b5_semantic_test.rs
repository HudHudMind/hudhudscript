//! B5-TEST2: Semantic regression tests for local-ident direct-reg.
//! Pattern: vm.execute() + vm.get_global() (from recursion_regression_test.rs).

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn compile_and_run(src: &str) -> VM {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute failed");
    vm
}

/// Horner correctness: horner_test([1,2,3], 10) should return 321.
#[test]
fn b5_horner_semantic() {
    let src = r#"
        fn horner_test(coeffs, x) {
            let result = coeffs[2];
            let i = 1;
            while (i >= 0) {
                result = result * x + coeffs[i];
                i = i - 1;
            }
            return result;
        }
        let y = horner_test([1,2,3], 10);
    "#;
    let vm = compile_and_run(src);
    let val = vm.get_global("y").and_then(|v| v.as_int().or_else(|| v.as_number().map(|n| n as i64)));
    assert_eq!(
        val,
        Some(321),
        "horner([1,2,3], 10) should be 321, got {:?}",
        val
    );
}

/// Binary correctness: a + b = 30.
#[test]
fn b5_simple_add_semantic() {
    let src = r#"
        let a = 10;
        let b = 20;
        let c = a + b;
    "#;
    let vm = compile_and_run(src);
    let val = vm.get_global("c").and_then(|v| v.as_int().or_else(|| v.as_number().map(|n| n as i64)));
    assert_eq!(val, Some(30), "a + b should be 30, got {:?}", val);
}

/// Index correctness: arr[1] = 5.
#[test]
fn b5_index_semantic() {
    let src = r#"
        let arr = [4,5,6];
        let i = 1;
        let x = arr[i];
    "#;
    let vm = compile_and_run(src);
    let val = vm.get_global("x").and_then(|v| v.as_int().or_else(|| v.as_number().map(|n| n as i64)));
    assert_eq!(val, Some(5), "arr[1] should be 5, got {:?}", val);
}
