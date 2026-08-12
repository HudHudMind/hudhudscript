//! A2.1 Bug1: throw from function unwinds to caller's try/catch

use hudhudscript_vm::VM;

fn run(src: &str) -> Result<String, String> {
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    vm.execute(&bc).map(|_| vm.last_return_value().display_string()).map_err(|e| format!("{}", e))
}

#[test]
fn throw_from_function_caught() {
    let src = "fn __t() { fn boom() { throw \"e\"; } let caught = 0; try { boom(); } catch(err) { caught = 1; } return caught; } __t()";
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "1", "caught should be 1, got: {}", r);
}

#[test]
fn throw_directly_in_try_works() {
    let src = "fn __t() { let caught = 0; try { throw \"e\"; } catch(err) { caught = 1; } return caught; } __t()";
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "1", "direct throw should be caught, got: {}", r);
}
