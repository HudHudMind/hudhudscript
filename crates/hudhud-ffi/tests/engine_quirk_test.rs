//! Engine regression gate for the await+module-call register-collision bug
//! the Flutter bridge exposed.
//!
//! History: the compiler's temp registers (base 128..=254) could alias
//! nested RegAlloc zones (bases 0..=223). On threads whose counters start
//! cold (a Dart worker isolate, a fresh engine thread), the first
//! `await <module>.<method>(...)` compile landed the receiver and the
//! argument on the same register 128 and miscompiled ("Cannot call
//! method ... on number"). Fixed by moving the temp region above the
//! allocator zones (224..=254) in the compiler's regalloc.
//!
//! NOTE for test authors: do NOT name host modules/methods after
//! HudHudScript keywords in ANY supported language — the lexer
//! normalizes them (e.g. Turkish "bekle" → "await"). The method names
//! used here ("triple", "delayMs") are keyword-free.

use hudhudscript_bytecode::ObjMap;
use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn setup() -> VM {
    let mut vm = VM::new();
    vm.register_method(
        "dart",
        "triple",
        Box::new(|args: &[Value16]| {
            Ok(Value16::int(args.first().and_then(|v| v.as_int()).unwrap_or(0) * 3))
        }),
    );
    let mut obj = ObjMap::default();
    obj.insert("__module", Value16::string("dart"));
    obj.insert("__loaded", Value16::bool_(true));
    vm.define_global("dart".to_string(), Value16::object(obj));
    vm
}

fn run(vm: &mut VM, src: &str) -> Result<(), String> {
    let ast = parse(src).map_err(|e| format!("parse: {e}"))?;
    let bc = Compiler::new().compile(&ast).map_err(|e| format!("compile: {e}"))?;
    vm.execute(&bc).map_err(|e| format!("exec: {e}"))
}

#[test]
fn await_module_call_all_forms() {
    // Fresh VM, await+module-call as the very first program.
    let mut vm1 = setup();
    assert!(
        run(&mut vm1, "let x = await dart.triple(4)").is_ok(),
        "first-program await+call must compile correctly"
    );
    assert_eq!(vm1.get_variable_owned("x").and_then(|v| v.as_int()), Some(12));

    // Split form.
    let mut vm2 = setup();
    assert!(run(&mut vm2, "let t = dart.triple(4)\nlet x = await t").is_ok());

    // Plain module call as the first program.
    let mut vm4 = setup();
    assert!(run(&mut vm4, "let t = dart.triple(4)").is_ok());

    // With a minted promise pool (mirrors the bridge's ensure_pool).
    let mut vm7 = setup();
    for _ in 0..64 {
        let (_tx, rx) = std::sync::mpsc::channel::<Result<Value16, String>>();
        vm7.register_promise_owned(rx);
    }
    assert!(run(&mut vm7, "let z = 1").is_ok());
    assert!(run(&mut vm7, "let x = await dart.triple(4)").is_ok());
}

/// Dart-worker replication: registration + await+module-call entirely on
/// a freshly spawned thread (cold thread-local allocator counters).
#[test]
fn second_thread_await_module_call() {
    let handle = std::thread::spawn(|| {
        let mut vm = VM::new();
        vm.register_method(
            "dart",
            "delayMs",
            Box::new(|args: &[Value16]| {
                Ok(Value16::int(args.first().and_then(|v| v.as_int()).unwrap_or(0) * 3))
            }),
        );
        let mut obj = ObjMap::default();
        obj.insert("__module", Value16::string("dart"));
        obj.insert("__loaded", Value16::bool_(true));
        vm.define_global("dart".to_string(), Value16::object(obj));

        let ast = parse("let x = await dart.delayMs(5)").unwrap();
        let bc = Compiler::new().compile(&ast).unwrap();
        vm.execute(&bc).expect("second-thread await+call must execute");
        assert_eq!(vm.get_variable_owned("x").and_then(|v| v.as_int()), Some(15));
    });
    handle.join().unwrap();
}
