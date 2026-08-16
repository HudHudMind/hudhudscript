// P4b: Correctness tests — String + Number must NOT emit NumAdd.

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

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
fn unknown_param_plus_float_not_numadd() {
    let insns = compile_instructions("fn f(x) { return x + 1.0; } print(f(\"a\"));");
    // x is Unknown → must NOT emit NumAdd/NumAddI.
    // IntAdd/IntAddReturn/IntAddI are the safe fallback (pre-P4 behavior).
    assert!(
        !has_instruction(&insns, |i| matches!(
            i,
            Instruction::NumAdd { .. } | Instruction::NumAddI { .. }
        )),
        "Unknown + Number must NOT emit NumAdd/NumAddI"
    );
}

#[test]
fn known_number_plus_float_emits_numadd() {
    let insns = compile_instructions("fn f(x) { let y = x + 1.0; return y; } print(f(3.14));");
    // x is used with float literal in caller → still Unknown in callee.
    // But we check that for known Number operands, NumAdd IS emitted.
    let insns2 = compile_instructions("let x = 3.14; let y = x + 1.0;");
    assert!(
        has_instruction(&insns2, |i| matches!(
            i,
            Instruction::NumAdd { .. } | Instruction::NumAddI { .. }
        )),
        "Number + Number must emit NumAdd"
    );
}

#[test]
fn int_plus_int_still_emits_intadd() {
    let insns = compile_instructions("let x = 1; let y = 2; let z = x + y;");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IntAdd { .. })),
        "Int + Int must still emit IntAdd"
    );
}
