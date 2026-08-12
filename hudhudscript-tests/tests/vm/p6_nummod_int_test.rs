use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run_global(src: &str, name: &str) -> hudhudscript_bytecode::Value16 {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute failed");
    vm.get_global(name)
        .unwrap_or(hudhudscript_bytecode::Value16::null())
}

// Helper: a % b via Number operand (forces NumMod, e.g. 17.0 % 5).
fn mod_via_fn(a: i64, b: i64) -> hudhudscript_bytecode::Value16 {
    let src = format!("fn mod_op(a,b) {{ return (a * 1.0) % b; }} let r = mod_op({a},{b});");
    run_global(&src, "r")
}

#[test]
fn test_intmod_returns_int() {
    // 17 % 5 → Int (not Number), since both operands are statically Int.
    let src = "let r = 17 % 5;";
    let v = run_global(src, "r");
    assert!(v.is_int(), "int % int must produce Int");
    assert_eq!(v.as_int(), Some(2));
}

#[test]
fn test_nummod_number_operand_returns_number() {
    // (17 * 1.0) % 5 → Number, via NumMod fast path.
    let v = mod_via_fn(17, 5);
    assert_eq!(v.as_number(), Some(2.0));
    assert!(v.is_number(), "result must be Number (not Int)");

    // (-17) % 5 == -2
    let v = mod_via_fn(-17, 5);
    assert_eq!(v.as_number(), Some(-2.0));

    // 17 % (-5) == 2
    let v = mod_via_fn(17, -5);
    assert_eq!(v.as_number(), Some(2.0));

    // modulo by zero -> error
    let src = "fn mod_op(a,b) { return a % b; } let r = mod_op(17,0);";
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    assert!(vm.execute(&bc).is_err(), "modulo by zero should error");
}
