//! A3.2 — composition order + error propagation

use hudhudscript_bytecode::Value16;
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

fn get_state_log(vm: &VM, var_name: &str) -> Vec<String> {
    let subj = vm.get_global(var_name).expect("subject variable");
    let obj = subj.as_object().expect("subject instance object");
    let log_val = obj.get("log").expect("log field");
    log_val
        .as_array()
        .expect("log array")
        .iter()
        .filter_map(|v: &Value16| v.as_string().map(|s| s.to_string()))
        .collect()
}

fn get_string_var(vm: &VM, var_name: &str) -> String {
    vm.get_global(var_name)
        .and_then(|v: Value16| v.as_string().map(|s| s.to_string()))
        .unwrap_or_default()
}

#[test]
fn before_runs_before_base_after_runs_after() {
    let source = r#"
subject Base {
    state log: []
}
subject BeforeView of Base {
    on test_before_after(self) { self.log.push("before") }
}
subject AfterView of Base {
    on test_before_after(self) { self.log.push("after") }
}
on test_before_after(self) { self.log.push("base") }
compose Base {
    on test_before_after: before BeforeView
    on test_before_after: after AfterView
}
let s = spawn Base
s.test_before_after()
"#;
    let vm = compile_and_execute(source);
    assert_eq!(get_state_log(&vm, "s"), vec!["before", "base", "after"]);
}

#[test]
fn override_skips_base() {
    let source = r#"
subject Base {
    state log: []
}
subject OverrideView of Base {
    on test_override(self) { self.log.push("override"); return "override_result" }
}
on test_override(self) { self.log.push("base"); return "base_result" }
compose Base {
    on test_override: override OverrideView
}
let s = spawn Base
let captured = s.test_override()
"#;
    let vm = compile_and_execute(source);
    assert_eq!(get_state_log(&vm, "s"), vec!["override"]);
    assert_eq!(get_string_var(&vm, "captured"), "override_result");
}

#[test]
fn combine_reducer_last_result_wins() {
    let source = r#"
subject Base {
    state log: []
}
subject CombineView of Base {
    on test_combine(self) { self.log.push("combine"); return "combine_result" }
}
on test_combine(self) { self.log.push("base"); return "base_result" }
compose Base {
    on test_combine: combine [CombineView]
}
let s = spawn Base
let captured = s.test_combine()
"#;
    let vm = compile_and_execute(source);
    assert_eq!(get_state_log(&vm, "s"), vec!["base", "combine"]);
    assert_eq!(get_string_var(&vm, "captured"), "combine_result");
}

#[test]
fn child_error_propagates() {
    let source = r#"
subject Base {}
subject BoomView of Base {
    on test_error(self) { throw "boom" }
}
on test_error(self) { return "base" }
compose Base {
    on test_error: before BoomView
}
let s = spawn Base
s.test_error()
"#;
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    let result = vm.execute(&bytecode);
    assert!(result.is_err(), "child ability error must propagate");
}
