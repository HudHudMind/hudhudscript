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
fn unparenthesized_if_emits_conditional_jump() {
    let insns = compile_instructions("let cond = false; if cond { print(1); }");
    assert!(
        has_instruction(&insns, |i| matches!(
            i,
            Instruction::JumpIfFalse { .. }
                | Instruction::IntCmpIJumpIfFalse { .. }
                | Instruction::IntCmpRRJumpIfFalse { .. }
                | Instruction::IntCmpRRJumpPacked { .. }
        )),
        "unparenthesized if must emit a conditional jump instruction"
    );
}

#[test]
fn unparenthesized_while_emits_conditional_jump_and_loop() {
    let insns = compile_instructions("let i = 0; while i < 5 { i = i + 1; }");
    assert!(
        has_instruction(&insns, |i| matches!(
            i,
            Instruction::JumpIfFalse { .. }
                | Instruction::IntCmpIJumpIfFalse { .. }
                | Instruction::IntCmpRRJumpIfFalse { .. }
                | Instruction::IntCmpRRJumpPacked { .. }
                | Instruction::IntLtRRJumpIfFalse { .. }
        )),
        "unparenthesized while must emit condition check jump"
    );
    assert!(
        has_instruction(&insns, |i| matches!(
            i,
            Instruction::Jump(_)
                | Instruction::IntAddIJump { .. }
                | Instruction::LoopEndIntAddIJump { .. }
        )),
        "unparenthesized while must emit backward jump instruction"
    );
}

#[test]
fn unparenthesized_if_else_chain_emits_jumps() {
    let insns = compile_instructions(
        "let x = 10; if x == 5 { print(1); } else if x == 10 { print(2); } else { print(3); }",
    );
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::Jump(_))),
        "if-else chain must emit forward jump to exit"
    );
}
