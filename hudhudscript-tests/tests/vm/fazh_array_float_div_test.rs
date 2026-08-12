//! FAZ H regression: array-read float div/mod silently returning null.

use hudhudscript_vm::VM;

fn run(src: &str) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

fn fazh_display_str(src: &str) -> String {
    let vm = run(src).unwrap();
    let val = vm.last_return_value();
    val.as_bigint().map(|b| b.to_string()).unwrap_or_else(|| val.display_string())
}

#[test]
fn fazh_array_read_float_div() {
    // a2[0] / a2[1] where both are float from array read
    let src = "let a2=[7.5,2.5];return a2[0]/a2[1];";
    assert_eq!(fazh_display_str(src), "3");
}

#[test]
fn fazh_array_read_float_mod() {
    // r % s where r=6.0, s=4.0 from locals (assigned from array)
    let src = "let arr=[6.0,4.0];let r=arr[0];let s=arr[1];return r%s;";
    assert_eq!(fazh_display_str(src), "2");
}

#[test]
fn fazh_locals_assigned_from_array_div() {
    let src = "let arr=[6.0,4.0];let r=arr[0];let s=arr[1];return r/s;";
    assert_eq!(fazh_display_str(src), "1.5");
}

#[test]
fn fazh_computed_float_div_unaffected() {
    // Computed float division was already working — should stay fixed
    let src = "let u=3.0+3.0;let w=4.0*1.0;return u/w;";
    assert_eq!(fazh_display_str(src), "1.5");
}

#[test]
fn fazh_int_div_via_bigint_still_works() {
    // FAZ D regression guard: BigInt division must still work
    let src = "let a=1000000000;let big=a*a*1234;let seven=7;return big/seven;";
    assert_eq!(fazh_display_str(src), "176285714285714285714");
}

#[test]
fn fazh_int_mod_via_bigint_still_works() {
    let src = "let a=1000000000;let big=a*a*1234;let seven=7;return big%seven;";
    assert_eq!(fazh_display_str(src), "2");
}
