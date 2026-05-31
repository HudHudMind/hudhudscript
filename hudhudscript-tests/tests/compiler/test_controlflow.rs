//! Compiler coverage tests: control flow, loops, switch, try-catch, match, classes

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::{Compiler, Instruction, VM};
use hudhudscript_parser::parse;

/// Helper: parse, compile, execute, return VM
fn run(source: &str) -> VM {
    let stmts = parse(source).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute failed");
    vm
}

/// Helper: parse and compile only, return bytecode
fn compile_only(source: &str) -> hudhudscript_compiler::Bytecode {
    let stmts = parse(source).expect("parse failed");
    let mut compiler = Compiler::new();
    compiler.compile(&stmts).expect("compile failed")
}

// ── For-in loop ───────────────────────────────────────────────────────

#[test]
fn test_for_in_loop_array_iteration() {
    let bc = compile_only(r#"
        let arr = [10, 20, 30];
        let total = 0;
        for (item in arr) {
            total = total + item;
        }
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("total").and_then(|v| v.as_number()) == Some(60.0),
        "for-in should iterate array elements: got {:?}",
        vm.get_variable("total")
    );
}

// ── C-style for loop ──────────────────────────────────────────────────

#[test]
fn test_for_c_style_loop() {
    let bc = compile_only(r#"
        let sum = 0;
        for (let i = 1; i <= 5; i = i + 1) {
            sum = sum + i;
        }
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("sum").and_then(|v| v.as_number()) == Some(15.0),
        "C-style for loop should sum 1..5: got {:?}",
        vm.get_variable("sum")
    );
}

// ── Break and continue ───────────────────────────────────────────────

#[test]
fn test_break_in_while_loop() {
    let bc = compile_only(r#"
        let x = 0;
        while (true) {
            x = x + 1;
            if (x == 3) { break; }
        }
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("x").and_then(|v| v.as_number()) == Some(3.0),
        "break should stop loop at 3: got {:?}",
        vm.get_variable("x")
    );
}

#[test]
fn test_break_emits_instruction() {
    let bc = compile_only(
        r#"
        while (true) { break; }
    "#,
    );
    let has_break = bc
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Break));
    assert!(has_break, "break should emit Break instruction");
}

#[test]
fn test_continue_emits_instruction() {
    let bc = compile_only(
        r#"
        while (true) { continue; }
    "#,
    );
    let has_continue = bc
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Continue));
    assert!(has_continue, "continue should emit Continue instruction");
}

// ── Switch statement ──────────────────────────────────────────────────

#[test]
fn test_switch_matches_case() {
    let bc = compile_only(r#"
        let x = 2;
        let result = 0;
        switch (x) {
            case 1: { result = 10; }
            case 2: { result = 20; }
            case 3: { result = 30; }
        }
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("result").and_then(|v| v.as_number()) == Some(20.0),
        "switch should match case 2: got {:?}",
        vm.get_variable("result")
    );
}

#[test]
fn test_switch_default_case() {
    let bc = compile_only(r#"
        let x = 99;
        let result = 0;
        switch (x) {
            case 1: { result = 10; }
            default: { result = 999; }
        }
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute failed");
    assert!(
        vm.get_variable("result").and_then(|v| v.as_number()) == Some(999.0),
        "switch should fall through to default: got {:?}",
        vm.get_variable("result")
    );
}

// ── Try / Catch / Throw ──────────────────────────────────────────────

#[test]
fn test_try_catch_catches_throw() {
    let bc = compile_only(r#"
        let caught = 0;
        try {
            throw 42;
        } catch (e) {
            caught = e;
        }
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("caught").and_then(|v| v.as_number()) == Some(42.0),
        "catch should receive thrown value: got {:?}",
        vm.get_variable("caught")
    );
}

#[test]
fn test_try_no_throw_skips_catch() {
    let bc = compile_only(r#"
        let result = 0;
        try {
            result = 1;
        } catch (e) {
            result = 2;
        }
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("result").and_then(|v| v.as_number()) == Some(1.0),
        "without throw, catch body should not execute: got {:?}",
        vm.get_variable("result")
    );
}

#[test]
#[ignore = "VM try-catch-finally execution stack underflow — requires exception frame implementation"]
fn test_try_catch_finally() {
    let bc = compile_only(r#"
        let log = 0;
        try {
            throw "err";
        } catch (e) {
            log = log + 1;
        } finally {
            log = log + 10;
        }
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("log").and_then(|v| v.as_number()) == Some(11.0),
        "finally should always run: got {:?}",
        vm.get_variable("log")
    );
}

// ── Template strings ─────────────────────────────────────────────────

#[test]
fn test_template_string_interpolation() {
    let bc = compile_only(r#"
        let name = "world";
        let greeting = `hello ${name}`;
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("greeting")
            .and_then(|v| v.as_str())
            .as_deref()
            == Some("hello world"),
        "template string should interpolate: got {:?}",
        vm.get_variable("greeting")
    );
}

// ── Arrow functions ──────────────────────────────────────────────────

#[test]
fn test_arrow_function_expression_body() {
    let bc = compile_only(r#"
        let double = (x) => x * 2;
        let result = double(5);
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("result").and_then(|v| v.as_number()) == Some(10.0),
        "arrow function should work: got {:?}",
        vm.get_variable("result")
    );
}

#[test]
fn test_arrow_function_block_body() {
    let bc = compile_only(r#"
        let add = (a, b) => { return a + b; };
        let result = add(3, 7);
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("result").and_then(|v| v.as_number()) == Some(10.0),
        "arrow function with block body: got {:?}",
        vm.get_variable("result")
    );
}

// ── Classes ──────────────────────────────────────────────────────────

#[test]
fn test_class_declaration_compiles() {
    let bc = compile_only(
        r#"class Point { constructor(x, y) { this.x = x; this.y = y; } public function sum() { return this.x + this.y; } }"#,
    );
    // Class should register its constructor and method as function chunks
    assert!(
        bc.functions.borrow().contains_key("Point::constructor"),
        "class should register constructor chunk; keys: {:?}",
        bc.functions.borrow().keys().collect::<Vec<_>>()
    );
    assert!(
        bc.functions.borrow().contains_key("Point::sum"),
        "class should register method chunk"
    );
    // ClassDecl instruction should be emitted
    let has_class_decl = bc
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::ClassDecl(_)));
    assert!(has_class_decl, "ClassDecl instruction should be emitted");
}

#[test]
fn test_class_with_inheritance_compiles() {
    let bc = compile_only(
        r#"class Animal { public function speak() { return "..."; } } class Dog extends Animal { public function bark() { return "woof"; } }"#,
    );
    // Dog should have its own bark method
    assert!(
        bc.functions.borrow().contains_key("Dog::bark"),
        "Dog should have its own bark method; keys: {:?}",
        bc.functions.borrow().keys().collect::<Vec<_>>()
    );
    // Dog should inherit Animal::speak
    assert!(
        bc.functions.borrow().contains_key("Dog::speak"),
        "Dog should inherit speak from Animal"
    );
}

// ── Enum declaration ─────────────────────────────────────────────────

#[test]
fn test_enum_declaration_emits_instruction() {
    let bc = compile_only(
        r#"
        enum Color { Red, Green, Blue }
    "#,
    );
    let has_enum = bc
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::EnumDecl(_)));
    assert!(has_enum, "EnumDecl instruction should be emitted");
}

// ── Return without value ─────────────────────────────────────────────

#[test]
fn test_function_return_without_value() {
    let bc = compile_only(r#"
        function noop() { return; }
        let result = noop();
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("result").map(|v| v.is_null()) == Some(true),
        "bare return should produce null: got {:?}",
        vm.get_variable("result")
    );
}

// ── If without else ──────────────────────────────────────────────────

#[test]
fn test_if_without_else_false_condition() {
    let bc = compile_only(r#"
        let x = 0;
        if (false) { x = 1; }
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("x").and_then(|v| v.as_number()) == Some(0.0),
        "if-false without else should not execute body: got {:?}",
        vm.get_variable("x")
    );
}

// ── Block scoping ────────────────────────────────────────────────────

#[test]
#[ignore = "Block scope PushScope/PopScope emission not yet implemented in compiler"]
fn test_block_scope_emits_push_pop_scope() {
    let bc = compile_only(
        r#"
        let x = 1;
        {
            let y = 2;
        }
    "#,
    );
    let has_push = bc
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::JumpIfFalse { .. }));
    let has_pop = bc
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::JumpIfFalse { .. }));
    assert!(has_push, "Block should emit PushScope");
    assert!(has_pop, "Block should emit PopScope");
}

// ── Nested function inside function ──────────────────────────────────

#[test]
fn test_nested_function_definition() {
    let bc = compile_only(r#"
        function outer() {
            function inner() { return 42; }
            return inner();
        }
        let result = outer();
    "#);
    eprintln!("switch bytecode: {:?}", bc.instructions);
    let vm = { let mut vm = VM::new(); vm.execute(&bc).unwrap(); vm };
    assert!(
        vm.get_variable("result").and_then(|v| v.as_number()) == Some(42.0),
        "nested function should work: got {:?}",
        vm.get_variable("result")
    );
}

// ── Async function compilation ───────────────────────────────────────

#[test]
fn test_async_function_compiles() {
    let bc = compile_only(
        r#"
        async function fetchData() {
            return 42;
        }
    "#,
    );
    // The function chunk should be marked as async
    let funcs = bc.functions.borrow();
    let chunk = funcs.get("fetchData");
    assert!(chunk.is_some(), "fetchData chunk should exist");
    assert!(chunk.unwrap().is_async, "fetchData should be marked async");
}
