// P3b: Integration test — verify function-body inline works.

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

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

#[test]
fn caller_inlines_add1() {
    let insns = compile_instructions(
        "fn add1(x) { return x + 1; } fn caller(y) { return add1(y); } print(caller(41));",
    );
    let call_count = insns
        .iter()
        .filter(|i| {
            matches!(
                i,
                Instruction::Call { .. }
                    | Instruction::IntAddCall1(..)
                    | Instruction::IntSubCall1(..)
            )
        })
        .count();
    // caller's body should NOT have a Call to add1 (it was inlined)
    // The only Call should be to print/caller
    assert!(
        call_count <= 2,
        "expected ≤2 calls, got {}: {:?}",
        call_count,
        insns
            .iter()
            .filter(|i| matches!(i, Instruction::Call { .. }))
            .collect::<Vec<_>>()
    );
}
