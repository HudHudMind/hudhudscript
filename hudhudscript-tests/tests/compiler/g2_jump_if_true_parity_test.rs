use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(source: &str) -> VM {
    let stmts = parse(source).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute failed");
    vm
}

#[test]
fn test_truthy_parity_int() {
    let code = r#"
        let res = 0;
        if (1) { res = res + 1; }
        if (0) { res = res + 10; }
        if (!0) { res = res + 100; }
        if (!1) { res = res + 1000; }
    "#;
    let vm = run(code);
    assert_eq!(vm.get_variable("res").and_then(|v| v.as_number()), Some(101.0)); // 1 + 100
}

#[test]
fn test_truthy_parity_string() {
    let code = r#"
        let res = 0;
        if ("hello") { res = res + 1; }
        if ("") { res = res + 10; }
        if (!"") { res = res + 100; }
        if (!"hello") { res = res + 1000; }
    "#;
    let vm = run(code);
    assert_eq!(vm.get_variable("res").and_then(|v| v.as_number()), Some(101.0));
}

#[test]
fn test_truthy_parity_null() {
    let code = r#"
        let res = 0;
        if (null) { res = res + 10; }
        if (!null) { res = res + 100; }
    "#;
    let vm = run(code);
    assert_eq!(vm.get_variable("res").and_then(|v| v.as_number()), Some(100.0));
}
