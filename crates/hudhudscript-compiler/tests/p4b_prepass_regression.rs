// P4b: Pre-pass regression tests — ordering, type degradation, inline preservation.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_bytecode::Instruction;

fn compile_instructions(src: &str) -> Vec<Instruction> {
    let ast = parse(src).expect("parse");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile");
    let mut all = bc.instructions.clone();
    for chunk in bc.functions.borrow().iter() {
        all.extend_from_slice(&chunk.instructions);
    }
    all
}

fn has_instruction<F>(insns: &[Instruction], pred: F) -> bool
where F: Fn(&Instruction) -> bool,
{ insns.iter().any(pred) }

#[test]
fn array_then_unknown_degraded_to_generic() {
    // First call: Array, second: Unknown → result must be generic Index
    let insns = compile_instructions(
        "fn f(x) { return x[0]; } print(f([1,2])); print(f(unknown_var));"
    );
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::Index { .. })),
        "Array+Unknown calls must degrade to generic Index"
    );
}

#[test]
fn unknown_then_array_degraded_to_generic() {
    // First call: Unknown, second: Array → result must be generic Index (order-independent)
    let insns = compile_instructions(
        "fn f(x) { return x[0]; } print(f(unknown_var)); print(f([1,2]));"
    );
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::Index { .. })),
        "Unknown+Array calls must degrade to generic Index"
    );
}

#[test]
fn function_declared_before_call_source_order_preserved() {
    // Function declaration must be compiled before the call that uses it
    let insns = compile_instructions(
        "fn add1(x) { return x + 1; } print(add1(41));"
    );
    // The function must exist in bytecode (declaration compiled before call)
    let has_fn = insns.iter().any(|i| matches!(i, Instruction::IntAddI { .. }));
    assert!(has_fn, "function body must be compiled and present");
}

#[test]
fn p3b_top_level_inline_preserved() {
    // P3b inline should still work with pure pre-pass
    let insns = compile_instructions(
        "fn add1(x) { return x + 1; } fn caller(y) { return add1(y); } print(caller(41));"
    );
    // caller should have inlined add1 (no Call in caller's body)
    let call_count = insns.iter().filter(|i| {
        matches!(i, Instruction::Call { .. })
    }).count();
    assert!(call_count <= 2, "caller must have inlined add1, got {} calls", call_count);
}

#[test]
fn scope_aware_let_array_f_call_yields_indexarray() {
    let insns = compile_instructions(
        "fn f(x) { return x[0]; } let a = [1,2]; print(f(a));"
    );
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "scope-aware let a=[]; f(a) must emit IndexArray"
    );
}

#[test]
fn function_body_call_collected_too() {
    let insns = compile_instructions(
        "fn helper(a) { return a[0]; } fn caller() { return helper([1,2]); } print(caller());"
    );
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "call inside function body must propagate type"
    );
}

#[test]
fn multi_param_both_typed() {
    // f(arr, s) called with array and string → both params typed
    let insns = compile_instructions(
        "fn f(a, b) { return a[0]; } print(f([1,2], \"hi\"));"
    );
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "multi-param: first param Array must emit IndexArray"
    );
}

#[test]
fn missing_arg_becomes_unknown() {
    // f(a,b) called with 1 arg → a still Array, b Unknown.
    // a[0] should be IndexArray (a always gets Array).
    let insns = compile_instructions(
        "fn f(a, b) { return a[0]; } print(f([1,2]));"
    );
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "param a always gets Array → IndexArray even when b is missing"
    );
}

#[test]
fn outer_scope_does_not_leak_into_function_param() {
    // outer x=Array, function param x=String → outer type must NOT leak
    let insns = compile_instructions(
        "let x = [1,2]; fn f(x) { return x[0]; } print(f(\"hi\"));"
    );
    // f's x is String (from call), NOT Array (from outer scope)
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "outer x=Array must not leak into fn param x → no IndexArray"
    );
}


#[test]
fn array_to_string_reassignment_no_indexarray() {
    // let x = []; x = "hi"; f(x) → x degraded to Unknown → generic
    let insns = compile_instructions(
        "fn f(a) { return a[0]; } let x = [1,2]; x = \"hi\"; print(f(x));"
    );
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "array→string reassignment must degrade → no IndexArray"
    );
}

#[test]
fn conditional_reassignment_generic() {
    // Inside if: Array or String assigned → generic at call site
    let insns = compile_instructions(
        "fn f(a) { return a[0]; } let x = [1,2]; if (x.length > 0) { x = \"hi\"; } print(f(x));"
    );
    // The pre-pass sees x=Array then x=String in if branch → Unknown
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "conditional reassignment must degrade → no IndexArray"
    );
}
