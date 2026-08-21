use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

#[test]
fn test_zero_int_addition_and_multiplication_vm() {
    let src = "let x = 0;
let y = 0;
let r = x + y;
let m = x * y;";
    let ast = parse(src).expect("parse");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    let res = vm.execute(&bc);
    assert!(res.is_ok(), "VM execution failed: {:?}", res);
    assert_eq!(vm.get_variable("r").unwrap().as_int(), Some(0));
    assert_eq!(vm.get_variable("m").unwrap().as_int(), Some(0));
}

#[test]
fn test_float_addition_multiplication_and_division_vm() {
    let src = "let x = 1.0;
let y = 2.0;
let sum = x + y;
let multiply = x * y;
let division = x / y;";
    let ast = parse(src).expect("parse");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    let res = vm.execute(&bc);
    assert!(res.is_ok(), "VM execution failed: {:?}", res);
    assert_eq!(vm.get_variable("sum").unwrap().as_number(), Some(3.0));
    assert_eq!(vm.get_variable("multiply").unwrap().as_number(), Some(2.0));
    assert_eq!(vm.get_variable("division").unwrap().as_number(), Some(0.5));
}
