use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

/// Test IntModCmpI dispatch via compiled script execution.
/// The compiler's fuse_slot pass converts IntModI+IntCmpI to IntModCmpI.

#[test]
fn test_intmodcmpi_basic() {
    // 5 % 2 == 1 -> true
    let (vm, _) = run_script("let x = 5; let out = x % 2 == 1;");
    assert_eq!(get_bool(&vm, "out"), Some(true));

    // 4 % 2 == 1 -> false
    let (vm, _) = run_script("let x = 4; let out = x % 2 == 1;");
    assert_eq!(get_bool(&vm, "out"), Some(false));

    // 6 % 2 == 0 -> true (collatz pattern)
    let (vm, _) = run_script("let x = 6; let out = x % 2 == 0;");
    assert_eq!(get_bool(&vm, "out"), Some(true));
}

#[test]
fn test_intmodcmpi_negative() {
    let (vm, _) = run_script("let x = -5; let out = x % 2 == -1;");
    assert_eq!(get_bool(&vm, "out"), Some(true));

    let (vm, _) = run_script("let x = -4; let out = x % 2 == 0;");
    assert_eq!(get_bool(&vm, "out"), Some(true));
}

#[test]
fn test_intmodcmpi_neq() {
    let (vm, _) = run_script("let x = 5; let out = x % 3 != 0;");
    assert_eq!(get_bool(&vm, "out"), Some(true));
}

#[test]
fn test_intmodcmpi_result_is_bool() {
    let (vm, _) = run_script("let x = 10; let out = x % 3 == 1;");
    let val = vm.get_global("out");
    let v = val.unwrap();
    assert!(
        v.is_bool(),
        "IntModCmpI result should be bool, got value type"
    );
}

fn get_bool(vm: &VM, var: &str) -> Option<bool> {
    vm.get_global(var).and_then(|v| v.as_bool())
}

fn run_script(src: &str) -> (VM, hudhudscript_bytecode::Bytecode) {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let bc_copy = bc.clone();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute must not fail");
    (vm, bc_copy)
}
