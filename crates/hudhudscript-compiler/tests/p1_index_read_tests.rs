// P1: Generic Index Read Kapatma tests.
// Verify compiler emits IndexArray/IndexStringAscii for typed locals.

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

fn compile_instructions(src: &str) -> Vec<Instruction> {
    let ast = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile failed");
    let mut all = bc.instructions.clone();
    for chunk in bc.functions.borrow().iter() {
        all.extend_from_slice(&chunk.instructions);
    }
    all
}

fn has_instruction<F>(insns: &[Instruction], pred: F) -> bool
where
    F: Fn(&Instruction) -> bool,
{
    insns.iter().any(pred)
}

#[test]
fn local_array_read_emits_indexarray() {
    let insns = compile_instructions("let a = [1,2,3]; let x = a[1];");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "local array read must emit IndexArray"
    );
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::Index { .. })),
        "local array read must NOT emit generic Index"
    );
}

#[test]
fn local_string_read_emits_indexstringascii() {
    let insns = compile_instructions("let s = \"hello\"; let c = s[1];");
    assert!(
        has_instruction(&insns, |i| matches!(
            i,
            Instruction::IndexStringAscii { .. }
        )),
        "local string read must emit IndexStringAscii"
    );
}

#[test]
fn uncalled_function_param_uses_generic_index() {
    // Function declared but never called — param type truly Unknown
    let insns = compile_instructions("fn f(arr) { return arr[0]; }");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::Index { .. })),
        "uncalled function param must use generic Index"
    );
}

#[test]
fn called_with_array_uses_indexarray() {
    // P4b: function called with array → param type propagated → IndexArray
    let insns = compile_instructions("fn f(arr) { return arr[0]; } print(f([1,2]));");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "function called with array must emit IndexArray"
    );
}

#[test]
fn called_with_mixed_types_uses_generic_index() {
    // Function called with array AND string → types conflict → Unknown → generic
    let insns = compile_instructions("fn f(x) { return x[0]; } print(f([1,2])); print(f(\"hi\"));");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::Index { .. })),
        "mixed-type calls must use generic Index"
    );
}

#[test]
fn local_array_in_loop_emits_indexarray() {
    let src = "
let total = 0;
let a = [10, 20, 30, 40, 50];
for (let i = 0; i < 5; i = i + 1) {
    total = total + a[i];
}
";
    let insns = compile_instructions(src);
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IndexArray { .. })),
        "array read in loop must emit IndexArray"
    );
}

#[test]
fn generic_index_still_works_for_objects() {
    // Objects still use generic Index (no specialization)
    let insns = compile_instructions("let o = {}; o[\"key\"] = 42; let v = o[\"key\"];");
    // Object index reads use generic Index
    let has_generic_index = has_instruction(&insns, |i| matches!(i, Instruction::Index { .. }));
    assert!(
        has_generic_index,
        "object index must still use generic Index"
    );
}
