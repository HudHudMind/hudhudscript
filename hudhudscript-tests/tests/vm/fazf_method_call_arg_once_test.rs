//! FAZ F regression: method-call arguments evaluated exactly once.

use hudhudscript_vm::VM;

fn run(src: &str) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler
        .compile(&stmts)
        .map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

#[test]
fn fazf_method_arg_called_once() {
    // ri() called exactly once per push — seed bumps from 0 to 1
    let src =
        "let seed=0;fn ri(m){seed=seed+1;return seed;}let arr=[];arr.push(ri(10));return arr[0];";
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn fazf_method_arg_counter_three_pushes() {
    // Three pushes → seed=3 (each ri called exactly once)
    let src = "let seed=0;fn ri(m){seed=seed+1;return seed;}let arr=[];arr.push(ri(10));arr.push(ri(10));arr.push(ri(10));return seed;";
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "3");
}

#[test]
fn fazf_nested_method_call_pop_push() {
    // b.push(a.pop()) — pop evaluates once, push evaluates once
    let src = "let a=[1,2,3];let b=[];b.push(a.pop());return a.length;";
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "2");
}
