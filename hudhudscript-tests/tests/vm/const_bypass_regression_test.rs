//! P4C: const-bypass regression test — verifies that set_var_by_sym
//! enforces the const-reassignment guard (guard now lives in the single
//! canonical body in set_var_by_sym, shared by both code paths).
//!
//! Bug: set_var had the guard, set_var_by_sym did NOT — const variables
//! could be silently mutated through the SymId path. Fixed in commit
//! 083a540af (P4C.1).
//!
//! This test verifies:
//! (a) Writing to a const variable through the SymId path produces
//!     "Cannot reassign to constant variable" error.
//! (b) A closure capturing a const still sees the correct value.

use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(source: &str) -> CompileResult<VM> {
    let stmts = parse(source).map_err(|e| {
        hudhudscript_bytecode::error::compile_codes::runtime_error(format!("Parse: {:?}", e))
    })?;
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts)?;
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

fn run_should_fail(source: &str) -> String {
    match run(source) {
        Ok(_) => panic!("Expected runtime error, got success"),
        Err(e) => format!("{}", e),
    }
}

#[test]
fn const_reassignment_guarded_in_sym_path() {
    // Setting a const variable through set_var (which delegates to
    // set_var_by_sym) must produce an error — the guard lives in
    // set_var_by_sym now.
    let source = r#"
const C = 42;
C = 99;
"#;
    let msg = run_should_fail(source);
    assert!(
        msg.contains("Cannot assign to constant"),
        "Expected 'Cannot assign to constant' error, got: {}",
        msg
    );
}

#[test]
fn const_in_function_body_guarded() {
    // Const inside a function body — reassignment should also fail.
    let source = r#"
fn f() {
    const X = 10;
    X = 20;
}
f();
"#;
    let msg = run_should_fail(source);
    assert!(
        msg.contains("Cannot assign to constant"),
        "Expected const guard error, got: {}",
        msg
    );
}

#[test]
fn closure_capturing_const_works() {
    // A closure capturing a const variable should read the correct value.
    let source = r#"
const C = 42;
let r = 0;
fn make_closure() {
    let c = C;
    r = c;
}
make_closure();
"#;
    let vm = run(source).expect("Closure capturing const should succeed");
    let r = vm.get_variable("r").expect("r not found");
    assert!(
        (r.as_number().unwrap() - 42.0).abs() < 1e-10,
        "Closure should capture const value 42"
    );
}
