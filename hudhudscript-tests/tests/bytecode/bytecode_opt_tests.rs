//! Unit tests for bytecode optimizations (BYTECODE_OPTIMIZATION.md).
//! Tests peephole, self-move elimination, and super-instruction fusion.

use hudhudscript_bytecode::{Bytecode, Instruction};

fn make_bytecode(instrs: Vec<Instruction>) -> Bytecode {
    let mut bc = Bytecode::default();
    for i in instrs {
        bc.push_instr(i);
    }
    bc
}

// ── Self-Move elimination ──────────────────────────────────────────

#[test]
fn test_self_move_eliminated() {
    let mut bc = Bytecode::default();
    bc.push_instr(Instruction::Move { dst: 5, src: 5 });
    bc.push_instr(Instruction::Move { dst: 5, src: 3 });
    assert_eq!(bc.instructions.len(), 1, "Self-move should be eliminated");
    assert!(matches!(bc.instructions[0], Instruction::Move { dst: 5, src: 3 }));
}

#[test]
fn test_non_self_move_preserved() {
    let mut bc = Bytecode::default();
    bc.push_instr(Instruction::Move { dst: 5, src: 3 });
    assert_eq!(bc.instructions.len(), 1);
}

// ── Result Forwarding peephole ─────────────────────────────────────

#[test]
fn test_peephole_loadconst_move_folded() {
    let mut bc = make_bytecode(vec![
        Instruction::LoadConst { dst: 3, const_idx: 0 },
        Instruction::Move { dst: 1, src: 3 },
    ]);
    bc.peephole_optimize();
    assert_eq!(bc.instructions.len(), 1, "LoadConst+Move → 1 instruction");
    assert!(matches!(bc.instructions[0], Instruction::LoadConst { dst: 1, const_idx: 0 }));
}

#[test]
fn test_peephole_loadintconst_move_folded() {
    let mut bc = make_bytecode(vec![
        Instruction::LoadIntConst { dst: 6, const_idx: 2 },
        Instruction::Move { dst: 2, src: 6 },
    ]);
    bc.peephole_optimize();
    assert_eq!(bc.instructions.len(), 1);
    assert!(matches!(bc.instructions[0], Instruction::LoadIntConst { dst: 2, const_idx: 2 }));
}

#[test]
fn test_peephole_intadd_move_folded() {
    let mut bc = make_bytecode(vec![
        Instruction::IntAdd { dst: 4, src1: 1, src2: 2 },
        Instruction::Move { dst: 7, src: 4 },
    ]);
    bc.peephole_optimize();
    assert_eq!(bc.instructions.len(), 1);
    assert!(matches!(bc.instructions[0], Instruction::IntAdd { dst: 7, src1: 1, src2: 2 }));
}

#[test]
fn test_peephole_loadconst_storeglobal_fused() {
    let mut bc = make_bytecode(vec![
        Instruction::LoadConst { dst: 130, const_idx: 0 },
        Instruction::StoreGlobal { src: 130, sym: 5 },
    ]);
    bc.peephole_optimize();
    assert_eq!(bc.instructions.len(), 1, "LoadConst+StoreGlobal → StoreGlobalConst");
    assert!(matches!(bc.instructions[0], Instruction::StoreGlobalConst { sym: 5, const_idx: 0 }));
}

#[test]
fn test_peephole_no_fold_different_registers() {
    let mut bc = make_bytecode(vec![
        Instruction::LoadConst { dst: 3, const_idx: 0 },
        Instruction::Move { dst: 1, src: 5 },
    ]);
    bc.peephole_optimize();
    assert_eq!(bc.instructions.len(), 2, "Should not fold when registers differ");
}

#[test]
fn test_peephole_chained_fold() {
    let mut bc = make_bytecode(vec![
        Instruction::LoadConst { dst: 3, const_idx: 0 },
        Instruction::Move { dst: 1, src: 3 },
        Instruction::Move { dst: 2, src: 1 },
    ]);
    bc.peephole_optimize();
    assert_eq!(bc.instructions.len(), 1, "Chained folds → 1 instruction");
    assert!(matches!(bc.instructions[0], Instruction::LoadConst { dst: 2, const_idx: 0 }));
}

// ── Super-instruction existence ────────────────────────────────────

#[test]
fn test_storeglobalconst_instruction_exists() {
    let bc = make_bytecode(vec![
        Instruction::StoreGlobalConst { sym: 5, const_idx: 0 },
    ]);
    assert_eq!(bc.instructions.len(), 1);
    assert!(matches!(bc.instructions[0], Instruction::StoreGlobalConst { .. }));
}

#[test]
fn test_arraypushlocal_instruction_exists() {
    let bc = make_bytecode(vec![
        Instruction::ArrayPushLocal { arr: 1, val: 2 },
    ]);
    assert_eq!(bc.instructions.len(), 1);
    assert!(matches!(bc.instructions[0], Instruction::ArrayPushLocal { .. }));
}

#[test]
fn test_arraypushlocal_max_register() {
    assert_eq!(
        Instruction::ArrayPushLocal { arr: 10, val: 5 }.max_register(),
        10
    );
    assert_eq!(
        Instruction::ArrayPushLocal { arr: 3, val: 7 }.max_register(),
        7
    );
}

#[test]
fn test_storeglobalconst_max_register() {
    assert_eq!(
        Instruction::StoreGlobalConst { sym: 0, const_idx: 0 }.max_register(),
        0
    );
}

#[test]
fn test_string_concat_assign_instruction_exists() {
    // STRREV: StringConcatAssign instruction tanımlı ve pack edilebiliyor.
    // Peephole bu instruction'ı ÜRETMEZ (jump offset sorunu); derleyici doğrudan emit eder.
    let instr = Instruction::StringConcatAssign { dst: 1, src: 2 };
    assert_eq!(instr.max_register(), 2);
    
    // Round-trip via bytecode
    let mut bc = make_bytecode(vec![instr]);
    assert_eq!(bc.instructions.len(), 1);
    assert!(matches!(bc.instructions[0], Instruction::StringConcatAssign { dst: 1, src: 2 }));
}
