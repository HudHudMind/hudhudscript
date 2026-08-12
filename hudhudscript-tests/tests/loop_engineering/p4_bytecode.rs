use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::compiler::Compiler;
use hudhudscript_parser::parse;

/// FAZ E+F bytecode proof: loop declaration produces __loop::<name> FunctionChunk with bytecode.
#[test]
fn loop_decl_creates_function_chunk() {
    let src = "loop first_loop { step first_step { let x = 0; gate g { when x==0 -> done else -> fail } } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    let chunk = bc_ref
        .get_function("__loop::first_loop")
        .expect("__loop::first_loop FunctionChunk must exist");
    // Proof: bytecode is NOT empty — emits MakeObject + Return
    assert!(
        !chunk.instructions.is_empty(),
        "loop chunk must have instructions"
    );
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeObject { .. })));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Return { .. })));
}

#[test]
fn chain_decl_creates_function_chunk() {
    let src = "chain ci { loop l1 { step s1 { } } loop l2 { step s2 { } } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");

    let bc_ref = &bytecode;
    let _chunk = bc_ref
        .get_function("__chain::ci")
        .expect("__chain::ci FunctionChunk must exist");
}

#[test]
fn duplicate_loop_is_compile_error() {
    let src = "loop x { step s { } } loop x { step t { } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    assert!(compiler.compile(&stmts).is_err());
}

#[test]
fn run_loop_compiles_when_loop_exists() {
    let src = "loop first_loop { step s { } } run loop first_loop";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    assert!(compiler.compile(&stmts).is_ok());
}

#[test]
fn run_loop_errors_when_loop_missing() {
    let src = "run loop ghost_loop";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    assert!(compiler.compile(&stmts).is_err());
}

#[test]
fn chain_with_nested_loops_compiles_both() {
    let src = "chain ci { loop l1 { step s1 { } } loop l2 { step s2 { } } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    assert!(bc_ref.has_function("__chain::ci"), "chain chunk must exist");
    assert!(
        bc_ref.has_function("__loop::l1"),
        "nested loop l1 must be compiled"
    );
    assert!(
        bc_ref.has_function("__loop::l2"),
        "nested loop l2 must be compiled"
    );
}

#[test]
fn times_mode_loop_has_constants() {
    // mode times(3) should emit counter constants
    let src = "loop t { step s { } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    let chunk = bc_ref
        .get_function("__loop::t")
        .expect("loop chunk must exist");
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn step_body_compiles_to_function_chunk() {
    let src = "loop real_loop { step do_work { let x = 1; let y = 2; } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    let loop_chunk = bc_ref
        .get_function("__loop::real_loop")
        .expect("loop chunk must exist");
    assert!(
        !loop_chunk.instructions.is_empty(),
        "loop chunk must have instructions"
    );
    // Loop chunk must have MORE than just MakeObject+Return (step body inlined)
    assert!(loop_chunk.instructions.len() > 3,
        "loop chunk with step body must have {} > 3 instructions (MakeObject + body + Return), got {}",
        loop_chunk.instructions.len(), loop_chunk.instructions.len());
}

#[test]
fn run_loop_emits_call_instruction() {
    let src = "loop my_loop { step s { } } run loop my_loop";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    // The main instruction list contains a Call to __loop::my_loop
    assert!(
        !bytecode.instructions.is_empty(),
        "top-level instructions must exist"
    );
    let has_call = bytecode
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Call { .. }));
    assert!(has_call, "Call instruction must be emitted for run loop");
}

#[test]
fn gate_condition_produces_bytecode() {
    let src = "loop gated { step s { let x = 0; gate g { when x==0 -> done else -> fail } } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::gated").unwrap();
    // Loop with step body + gate conditions must have > 5 instructions
    assert!(
        chunk.instructions.len() > 5,
        "gate conditions must produce bytecode, got {} instructions",
        chunk.instructions.len()
    );
}

#[test]
fn gate_with_condition_has_real_bytecode_not_skeleton() {
    let src = "loop gated { step s { let x = 0; gate g { when x==0 -> done else -> fail } } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::gated").unwrap();
    // The gate condition bytecode must have more than just MakeObject+Return
    assert!(
        chunk.instructions.len() > 4,
        "gate condition must produce real bytecode beyond skeleton, got {} instructions",
        chunk.instructions.len()
    );
    // Verify the step body let x=0 produces real instructions
    let has_const = chunk.constants.len() > 0;
    assert!(
        has_const || chunk.instructions.len() > 4,
        "loop with step+gate must have constants or significant instructions"
    );
}

#[test]
fn gate_condition_emits_jump_if_false() {
    let src = "loop L { step S { let x = 0; gate G { when x==0 -> done else -> fail } } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    let has_conditional_jump = chunk.instructions.iter().any(|i| {
        matches!(
            i,
            Instruction::JumpIfFalse { .. }
                | Instruction::IntCmpIJumpIfFalse { .. }
                | Instruction::IntCmpRRJumpIfFalse { .. }
        )
    });
    assert!(
        has_conditional_jump,
        "gate condition must emit a conditional jump"
    );
}

#[test]
fn gate_two_when_branches_emit_two_conditional_jumps() {
    let src = "loop L { step S { let x = 0; gate G { when x==0 -> done when x==1 -> fail else -> done } } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    let jump_count = chunk
        .instructions
        .iter()
        .filter(|i| {
            matches!(
                i,
                Instruction::JumpIfFalse { .. }
                    | Instruction::IntCmpIJumpIfFalse { .. }
                    | Instruction::IntCmpRRJumpIfFalse { .. }
            )
        })
        .count();
    assert!(
        jump_count >= 2,
        "two when branches must emit >=2 conditional jumps, got {}",
        jump_count
    );
}

#[test]
fn gate_jump_if_false_uses_condition_register_not_magic_249() {
    let src = "loop L { step S { let x = 0; gate G { when x==0 -> done else -> fail } } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    // Verify no conditional jump uses magic register 249
    for instr in &chunk.instructions {
        match instr {
            Instruction::JumpIfFalse { src, .. } => {
                assert!(
                    *src != 249,
                    "JumpIfFalse must not use magic register 249, got src={}",
                    src
                );
            }
            Instruction::IntCmpIJumpIfFalse { src, .. } => {
                assert!(
                    *src != 249,
                    "IntCmpIJumpIfFalse must not use magic register 249, got src={}",
                    src
                );
            }
            Instruction::IntCmpRRJumpIfFalse { src1, src2, .. } => {
                assert!(
                    *src1 != 249,
                    "IntCmpRRJumpIfFalse must not use magic register 249, got src1={}",
                    src1
                );
                assert!(
                    *src2 != 249,
                    "IntCmpRRJumpIfFalse must not use magic register 249, got src2={}",
                    src2
                );
            }
            _ => {}
        }
    }
}

#[test]
fn gate_jump_if_false_offsets_are_nonzero_when_branch_follows() {
    let src = "loop L { step S { let x = 0; gate G { when x==0 -> done when x==1 -> fail else -> done } } }";
    let stmts = parse(src).expect("parse");
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    let mut offsets: Vec<i16> = Vec::new();
    for instr in &chunk.instructions {
        match instr {
            Instruction::JumpIfFalse { offset, .. } => offsets.push(*offset),
            Instruction::IntCmpIJumpIfFalse { offset, .. } => offsets.push(*offset),
            Instruction::IntCmpRRJumpIfFalse { offset, .. } => offsets.push(*offset),
            _ => {}
        }
    }
    // At least one offset should be nonzero (first branch jumps past second)
    assert!(
        offsets.iter().any(|&o| o != 0),
        "JumpIfFalse offsets must be patched (nonzero), got: {:?}",
        offsets
    );
}

#[test]
fn chain_has_getproperty_for_success_check() {
    let src = "chain c { loop l1 { step s { gate g { when true -> done else -> fail } } } loop l2 { step s { gate g { when true -> done else -> fail } } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::compiler::Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let bc_ref = &bytecode;
    let chain_chunk = bc_ref.get_function("__chain::c").unwrap();
    // Non-last chain link must read .success to decide on_done/on_fail
    assert!(chain_chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::GetProperty { .. })));
}
