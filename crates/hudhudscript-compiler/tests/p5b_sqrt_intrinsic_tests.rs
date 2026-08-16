// P5c: Math.sqrt intrinsic correctness — local shadow guard tests.

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
fn builtin_math_sqrt_number_emits_numsqrt() {
    let insns = compile_instructions("print(Math.sqrt(9.0));");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::NumSqrt { .. })),
        "builtin Math.sqrt(9.0) must emit NumSqrt"
    );
}

#[test]
fn shadowed_math_sqrt_does_not_emit_numsqrt() {
    let insns = compile_instructions("let Math = {}; print(Math.sqrt(9.0));");
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::NumSqrt { .. })),
        "shadowed Math.sqrt must NOT emit NumSqrt"
    );
}

#[test]
fn function_param_math_sqrt_does_not_emit_numsqrt() {
    let insns = compile_instructions("fn f(Math) { return Math.sqrt(9.0); } print(f({}));");
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::NumSqrt { .. })),
        "function param Math.sqrt must NOT emit NumSqrt"
    );
}

#[test]
fn unknown_arg_sqrt_emits_numsqrt() {
    // G8: Math is not shadowed → NumSqrt emitted even for unconstrained param
    let insns = compile_instructions("fn f(x) { return Math.sqrt(x); }");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::NumSqrt { .. })),
        "G8: unshadowed Math.sqrt must emit NumSqrt"
    );
}

#[test]
fn global_reassign_math_sqrt_not_numsqrt() {
    let insns = compile_instructions("Math = {}; print(Math.sqrt(9.0));");
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::NumSqrt { .. })),
        "Math = {{}} must disable Math.sqrt intrinsic"
    );
}

#[test]
fn global_reassign_math_floor_not_intrinsic() {
    let insns = compile_instructions("Math = {}; let x = 100; print(Math.floor(x / 2));");
    // Math.floor intrinsic blocked → MethodCall fallback
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::MethodCall { .. })),
        "Math = {{}} must fall back to MethodCall for Math.floor"
    );
}
