//! FAZ E regression: while loop with if/else counter mutation.

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

fn result_str(src: &str) -> String {
    run(src).unwrap().last_return_value().display_string()
}

#[test]
fn while_if_else_counter_both_branches() {
    // E1: counter mutated in both if/else branches
    let src = "let acc=0;let c=20;while(c>0){if(c%2==0){acc=(acc*3+c)%1000003;c=c-1;}else{acc=(acc*3+c)%1000003;c=c-1;}}return acc;";
    assert_eq!(result_str(src), "45922");
}

#[test]
fn while_if_else_counter_outside() {
    // Control: counter outside branches (should also work)
    let src = "let acc=0;let c=20;while(c>0){if(c%2==0){acc=(acc*3+c)%1000003;}else{acc=(acc*3+c)%1000003;}c=c-1;}return acc;";
    assert_eq!(result_str(src), "45922");
}

#[test]
fn while_nested_if_different_increments() {
    // E5: nested if with different increment amounts
    let src = "let i=0;let n=0;while(i<100){if(i>=20){if(i%10<4){n=n+1;i=i+20;}else{n=n+10;i=i+1;}}else{n=n+10;i=i+1;}}return n;";
    assert_eq!(result_str(src), "204");
}
