//! P0 regression tests for integer arithmetic return types.
//!
//! Verifies that integer `/` and `%` operands keep producing integer
//! results so the fast Int tag path is not polluted by floats.
//! See PERFORMANCE_REGRESSION_FIX_PLAN.md P0.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(source: &str) -> Result<(), String> {
    let mut vm = VM::with_locale(hudhudscript_vm::OutputLocale::Default);
    let ast = parse(source).map_err(|e| format!("parse: {}", e))?;
    let bytecode = Compiler::new()
        .compile(&ast)
        .map_err(|e| format!("compile: {}", e))?;
    vm.execute(&bytecode).map_err(|e| format!("{}", e))
}

#[test]
fn int_div_result_is_integer() {
    run(r#"
        let a = 17;
        let b = 5;
        let r = a / b;
        if (r != 3) { throw "int div value wrong"; }
    "#)
    .unwrap();
}

#[test]
fn int_mod_result_is_integer() {
    run(r#"
        let a = 17;
        let b = 5;
        let r = a % b;
        if (r != 2) { throw "int mod value wrong"; }
    "#)
    .unwrap();
}

#[test]
fn chained_int_div_mod_stays_integer() {
    run(r#"
        let n = 12345;
        n = n / 10;
        n = n % 100;
        if (n != 34) { throw "chained value wrong"; }
    "#)
    .unwrap();
}

#[test]
fn count_set_bits_loop_correctness() {
    run(r#"
        function countSetBits(n) {
            let count = 0;
            while (n != 0) {
                count = count + (n % 2);
                n = n / 2;
            }
            return count;
        }
        let total = 0;
        for (let i = 0; i < 1000; i = i + 1) {
            total = total + countSetBits(i);
        }
        if (total != 4932) { throw "count_set_bits total wrong"; }
    "#)
    .unwrap();
}

#[test]
fn float_div_keeps_fractional_part() {
    run(r#"
        let r = 17.0 / 5.0;
        if (r < 3.39 || r > 3.41) { throw "float div value wrong"; }
    "#)
    .unwrap();
}
