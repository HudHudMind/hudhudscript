//! FAZ D regression: BigInt register-register division and modulo.

use hudhudscript_vm::VM;

fn run(src: &str) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

fn result_str(src: &str) -> String {
    let vm = run(src).unwrap();
    let val = vm.get_global("r").unwrap_or(hudhudscript_bytecode::Value16::null());
    val.as_bigint().map(|b| b.to_string()).unwrap_or_else(|| val.display_string())
}

#[test]
fn bigint_reg_div_seven() {
    assert_eq!(result_str("let a=1000000000;let big=a*a*1234;let seven=7;let r=big/seven;"),
               "176285714285714285714");
}

#[test]
fn bigint_reg_mod_seven() {
    assert_eq!(result_str("let a=1000000000;let big=a*a*1234;let seven=7;let r=big%seven;"),
               "2");
}

#[test]
fn bigint_reg_div_var() {
    assert_eq!(result_str("let a=1000000000;let big=a*a*1234;let b2=a*a*12;let r=big/b2;"),
               "102");
}

#[test]
fn bigint_reg_mod_var() {
    assert_eq!(result_str("let a=1000000000;let big=a*a*1234;let b2=a*a*12;let r=big%b2;"),
               "10000000000000000000");
}
