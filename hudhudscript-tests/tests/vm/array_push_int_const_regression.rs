//! Regression test for ArrayPushIntConst const_idx remap (commit 6bb33bc1a).
//!
//! Bug: when merge_function_constant_pools remaps int constant indices after
//! function compilation, LoadIntConst indices were remapped but
//! ArrayPushIntConst indices were NOT. This caused array.push(0) inside
//! functions to read garbage from a stale constant pool slot.
//!
//! Minimal repro (from samples/repro/literal_zero_reads_reg0.hud):
//! inside a function, arr.push(0) reads the wrong value instead of 0.

use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

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

#[test]
fn array_push_int_const_in_function_maps_correctly() {
    // Inside fn, arr.push(0), arr.push(1), arr.push(2) must push correct
    // values. Before fix, the constant pool merge missed ArrayPushIntConst.
    let source = r#"
let seed = 12345;
let r0 = 0; let r1 = 0; let r2 = 0; let r3 = 0;
fn test_push() {
    let arr = [];
    arr.push(0);
    arr.push(1);
    arr.push(2);
    arr.push(seed);
    r0 = arr[0];
    r1 = arr[1];
    r2 = arr[2];
    r3 = arr[3];
}
test_push();
"#;

    let vm = run(source).expect("Execution should not error");

    let v0 = vm.get_variable("r0").expect("r0 not found");
    assert!(
        (v0.as_number().expect("number") - 0.0).abs() < 1e-10,
        "push(0) should be 0, got {:?}",
        v0
    );

    let v1 = vm.get_variable("r1").expect("r1 not found");
    assert!((v1.as_number().expect("number") - 1.0).abs() < 1e-10);

    let v2 = vm.get_variable("r2").expect("r2 not found");
    assert!((v2.as_number().expect("number") - 2.0).abs() < 1e-10);

    let v3 = vm.get_variable("r3").expect("r3 not found");
    assert!((v3.as_number().expect("number") - 12345.0).abs() < 1e-10);
}

#[test]
fn array_push_int_const_across_functions_no_cross_contamination() {
    // Multiple functions with array.push(int literal) should not cross-
    // contaminate each other's constant pool mappings.
    let source = r#"
let s1 = 0; let s2 = 0; let s3 = 0; let s4 = 0;
fn f1() {
    let a = [];
    a.push(10);
    a.push(20);
    s1 = a[0]; s2 = a[1];
}
fn f2() {
    let b = [];
    b.push(30);
    b.push(40);
    s3 = b[0]; s4 = b[1];
}
f1(); f2();
"#;

    let vm = run(source).expect("Execution should not error");

    let s1 = vm.get_variable("s1").expect("s1 not found");
    assert!((s1.as_number().expect("number") - 10.0).abs() < 1e-10);

    let s2 = vm.get_variable("s2").expect("s2 not found");
    assert!((s2.as_number().expect("number") - 20.0).abs() < 1e-10);

    let s3 = vm.get_variable("s3").expect("s3 not found");
    assert!((s3.as_number().expect("number") - 30.0).abs() < 1e-10);

    let s4 = vm.get_variable("s4").expect("s4 not found");
    assert!((s4.as_number().expect("number") - 40.0).abs() < 1e-10);
}
