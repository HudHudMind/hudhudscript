//! B5 regression tests: script-to-script `await` chains must keep the
//! caller's global scope.
//!
//! History: spawned async/generator VMs were seeded with a bare
//! `globals.clone()` snapshot.  During execution top-level bindings live
//! in `shared_globals_vec` (shared symbols) or main-frame register slots
//! (main-only symbols) — the `globals` HashMap only receives them at the
//! END of `execute()`.  Host-registered module methods (the bridge's
//! `console.log`) lived in a VM-side `ModuleRegistry` and SOP subject
//! state in `subject_instances`, neither of which was copied.  Result:
//! `console.log` → "Unknown method 'log'", top-level `let` reads →
//! "Undefined variable", subject state reads → "Property not found".
//! Fixed by the `SpawnContext` bundle captured into every spawned VM.

use hudhudscript_bytecode::ObjMap;
use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn execute(source: &str) -> Result<VM, String> {
    let ast = parse(source).map_err(|e| format!("parse: {e}"))?;
    let bytecode = Compiler::new().compile(&ast).map_err(|e| format!("compile: {e}"))?;
    let mut vm = VM::new();
    vm.execute(&bytecode).map_err(|e| format!("execute: {e}"))?;
    Ok(vm)
}

fn int_variable(vm: &VM, name: &str) -> i64 {
    vm.get_variable_owned(name)
        .and_then(|v| v.as_int())
        .unwrap_or_else(|| panic!("{name} must be a published int"))
}

/// The minimal fleet repro: outer async awaits inner async, inner reads a
/// top-level `let`.  Used to fail with "Undefined variable: greeting_tag".
#[test]
fn async_awaits_async_keeps_top_level_let() {
    let vm = execute(
        r#"
let greeting_tag = 7

async function inner(x) {
    return x + greeting_tag
}

async function outer(x) {
    let r = await inner(x)
    return r * 2
}

let result = await outer(10)
"#,
    )
    .expect("async -> async await must keep top-level bindings");

    assert_eq!(int_variable(&vm, "result"), 34);
}

/// Single-level async reading a top-level binding (the loss happened on
/// the FIRST spawn, not only in async -> async chains).
#[test]
fn single_async_keeps_top_level_let() {
    let vm = execute(
        r#"
let base = 100

async function f(x) {
    return x + base
}

let result = await f(1)
"#,
    )
    .expect("single async must keep top-level bindings");

    assert_eq!(int_variable(&vm, "result"), 101);
}

/// Bridge `console.log` pattern: a host method registered through
/// `register_method` plus a `__module`-marked global object.  Called from
/// inside an async body it used to hit an empty ModuleRegistry in the
/// spawned VM ("Unknown method 'log' on object").
#[test]
fn async_body_calls_host_registered_module_method() {
    let mut vm = VM::new();
    vm.register_method(
        "console",
        "log",
        Box::new(|args: &[Value16]| {
            Ok(Value16::int(args.first().and_then(|v| v.as_int()).unwrap_or(0) * 3))
        }),
    );
    let mut obj = ObjMap::default();
    obj.insert("__module", Value16::string("console"));
    obj.insert("__loaded", Value16::bool_(true));
    vm.define_global("console".to_string(), Value16::object(obj));

    let src = r#"
async function inner(x) {
    return console.log(x)
}

async function outer(x) {
    return await inner(x)
}

let result = await outer(5)
"#;
    let ast = parse(src).map_err(|e| format!("parse: {e}")).unwrap();
    let bytecode = Compiler::new()
        .compile(&ast)
        .map_err(|e| format!("compile: {e}"))
        .unwrap();
    vm.execute(&bytecode)
        .expect("async body must reach the host-registered module method");

    assert_eq!(int_variable(&vm, "result"), 15);
}

/// Fleet "spawn edilen özne durumu → Property not found" symptom: a SOP
/// subject spawned at top level, its state read inside an async body.
/// `subject_instances` used to be empty in the spawned VM.
#[test]
fn async_body_reads_subject_state() {
    let vm = execute(
        r#"
subject Counter has Mayor {
    state count: 41
}

spawn Counter
let handle = spawn Counter

async function read_count() {
    return handle.count
}

let result = await read_count()
"#,
    )
    .expect("async body must read spawned subject state");

    assert_eq!(int_variable(&vm, "result"), 41);
}

/// The async body must also see top-level FUNCTION bindings reached by
/// name (not by direct call index), e.g. through `let g = fn_name`.
#[test]
fn async_body_sees_top_level_function_value() {
    let vm = execute(
        r#"
fn helper(x) {
    return x + 1
}

let helper_ref = helper

async function inner() {
    return helper_ref(9)
}

let result = await inner()
"#,
    )
    .expect("async body must see top-level function values");

    assert_eq!(int_variable(&vm, "result"), 10);
}
