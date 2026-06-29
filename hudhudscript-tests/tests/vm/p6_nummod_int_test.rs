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

// Helper: a % b via function params (forces NumMod)
fn mod_via_fn(a: i64, b: i64) -> hudhudscript_bytecode::Value16 {
    let src = format!("fn mod_op(a,b) {{ return a % b; }} let r = mod_op({a},{b});");
    run_global(&src, "r")
}

#[test]
fn test_nummod_int_fastpath_matches_fmod() {
    // 17 % 5 == 2 (int%int via NumMod fast path, result Number)
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
