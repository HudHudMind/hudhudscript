//! A3.3 — field correspondence + deterministic view read/write

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

fn get_int_var(vm: &VM, name: &str) -> i64 {
    let val = vm.get_global(name);
    val.map(|v: Value16| {
        if let Some(i) = v.as_int() {
            i
        } else if let Some(n) = v.as_number() {
            n as i64
        } else {
            -999
        }
    })
    .unwrap_or(-999)
}

#[test]
fn correspond_writes_to_all_locations() {
    let source = r#"
subject Base {
    state shared: 0
}
subject ViewA of Base {
    state shared: 0
}
subject ViewB of Base {
    state shared: 0
}
compose Base {
    state shared: correspond
}
let s = spawn Base
s.shared = 42
s.__view_name = "ViewA"
let view_a_val = s.shared
s.__view_name = "ViewB"
let view_b_val = s.shared
s.__view_name = ""
let base_val = s.shared
"#;
    let vm = compile_and_execute(source);
    assert_eq!(get_int_var(&vm, "base_val"), 42);
    assert_eq!(get_int_var(&vm, "view_a_val"), 42);
    assert_eq!(get_int_var(&vm, "view_b_val"), 42);
}

#[test]
fn separate_writes_only_to_active_view() {
    let source = r#"
subject Base {
    state shared: 0
}
subject ViewA of Base {
    state shared: 0
}
subject ViewB of Base {
    state shared: 0
}
compose Base {
    state shared: separate
}
let s = spawn Base
let base_before = s.shared
s.__view_name = "ViewA"
s.shared = 7
let active_val = s.shared
s.__view_name = "ViewB"
let other_view_val = s.shared
"#;
    let vm = compile_and_execute(source);
    assert_eq!(get_int_var(&vm, "base_before"), 0);
    assert_eq!(get_int_var(&vm, "active_val"), 7);
    assert_eq!(get_int_var(&vm, "other_view_val"), 0);
}

#[test]
fn view_read_is_deterministic_when_field_exists_in_multiple_views() {
    // Without __view_name, base value is read first, so result is deterministic.
    let source = r#"
subject Base {
    state val: 1
}
subject Zebra of Base {
    state val: 2
}
subject Alpha of Base {
    state val: 3
}
let s = spawn Base
let v = s.val
"#;
    for _ in 0..3 {
        let vm = compile_and_execute(source);
        assert_eq!(get_int_var(&vm, "v"), 1);
    }
}

#[test]
fn explicit_view_name_takes_precedence() {
    let source = r#"
subject Base {
    state val: 1
}
subject ViewA of Base {
    state val: 2
}
let s = spawn Base
s.__view_name = "ViewA"
let v = s.val
"#;
    let vm = compile_and_execute(source);
    assert_eq!(get_int_var(&vm, "v"), 2);
}
