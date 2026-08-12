//! A3.4 — subject intents reach runtime

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn compile_and_execute(source: &str) -> VM {
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode).expect("execute");
    vm
}

#[test]
fn subject_intents_are_available_at_runtime() {
    let source = r#"
subject Agent {
    state status: "idle"
    intent Patrol
    intent Investigate
    can move
}
let a = spawn Agent
"#;
    let vm = compile_and_execute(source);
    let intents = vm.subject_intents("Agent");
    assert_eq!(intents, vec!["Patrol", "Investigate"]);
}

#[test]
fn subject_without_intents_has_empty_intents() {
    let source = r#"
subject Drone {
    state battery: 100
    can fly
}
let d = spawn Drone
"#;
    let vm = compile_and_execute(source);
    assert!(vm.subject_intents("Drone").is_empty());
}
