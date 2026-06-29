//! Packed instruction codec round-trip tests.
//!
//! Moved from `hudhudscript-bytecode/src/packed_instruction/tests.rs` to the
//! external test repo as part of I2-A1 (private inline test consolidation).
//!
//! Note: this file only tests pack/unpack pairs that are symmetric in the
//! current bytecode layout. Many variants intentionally round-trip through
//! canonical register forms (e.g. zero-arg opcodes decode to src=255).

use hudhudscript_bytecode::packed_instruction::{decode, encode, pack, unpack};
use hudhudscript_bytecode::{Instruction, SymId};

/// Helper: assert round-trip for a given instruction.
fn assert_round_trip(instr: &Instruction) {
    let packed = pack(instr).unwrap_or_else(|| {
        panic!("pack returned None for {:?}", instr);
    });
    let unpacked = unpack(packed).unwrap_or_else(|| {
        panic!("unpack returned None for packed {:#010X}", packed);
    });
    // Compare debug representations (Instruction doesn't impl PartialEq)
    assert_eq!(
        format!("{:?}", instr),
        format!("{:?}", unpacked),
        "round-trip mismatch for {:#010X}",
        packed,
    );
}

#[test]
fn packed_round_trip_zero_arg() {
    let instrs = vec![
        Instruction::Break,
        Instruction::Continue,
        Instruction::TryEnd,
    ];
    for instr in &instrs {
        assert_round_trip(instr);
    }
}

#[test]
fn packed_round_trip_load_int_const() {
    assert_round_trip(&Instruction::LoadIntConst {
        dst: 255,
        const_idx: 0,
    });
    assert_round_trip(&Instruction::LoadIntConst {
        dst: 255,
        const_idx: 255,
    });
}

#[test]
fn packed_round_trip_return() {
    assert_round_trip(&Instruction::Return { src: 127 });
}

#[test]
fn packed_round_trip_jumps() {
    assert_round_trip(&Instruction::Jump(0));
    assert_round_trip(&Instruction::Jump(1000));
    assert_round_trip(&Instruction::Jump(32767)); // i16 max
    assert_round_trip(&Instruction::Jump(-1)); // backward
    assert_round_trip(&Instruction::Jump(-32768)); // i16 min
}

#[test]
fn packed_jump_overflow() {
    assert!(pack(&Instruction::Jump(i16::MAX as i32 + 1)).is_none());
    assert!(pack(&Instruction::Jump(i16::MIN as i32 - 1)).is_none());
}

#[test]
fn packed_round_trip_bind_var() {
    assert_round_trip(&Instruction::BindVar(SymId(7)));
    assert_round_trip(&Instruction::BindVar(SymId(65535)));
}

#[test]
fn packed_round_trip_call_variants() {
    assert_round_trip(&Instruction::MethodCall {
        dst: 255,
        obj: 255,
        payload_idx: 20,
        first_arg: 0,
        arg_count: 0,
    });
    assert_round_trip(&Instruction::SuperCall {
        dst: 255,
        payload_idx: 8,
        first_arg: 0,
        arg_count: 0,
    });
}

#[test]
fn packed_round_trip_push_loop() {
    assert_round_trip(&Instruction::LoopBegin(0));
    assert_round_trip(&Instruction::LoopBegin(255));
    assert_round_trip(&Instruction::LoopBegin(65535));
}

#[test]
fn packed_push_loop_overflow() {
    assert!(pack(&Instruction::LoopBegin(u16::MAX as u32 + 1)).is_none());
}

#[test]
fn packed_round_trip_match_variant() {
    assert_round_trip(&Instruction::MatchVariant(1));
}

#[test]
fn packed_round_trip_get_static() {
    assert_round_trip(&Instruction::GetStatic(3));
}

#[test]
fn packed_round_trip_destruct_array() {
    assert_round_trip(&Instruction::DestructArray(3, false));
    assert_round_trip(&Instruction::DestructArray(5, true));
}

#[test]
fn packed_round_trip_iter_push_try() {
    assert_round_trip(&Instruction::IterNext {
        iter_reg: 255,
        var_sym_idx: 0,
        end_offset: 500,
    });
    assert_round_trip(&Instruction::TryBegin(200));
}

#[test]
fn packed_complex_returns_none() {
    // Instructions with register/String/Vec/Option payloads cannot be packed.
    assert!(pack(&Instruction::EnumDecl(0)).is_none());
    assert!(pack(&Instruction::ClassDecl(0)).is_none());
    assert!(pack(&Instruction::LoadModule(0)).is_none());
    assert!(pack(&Instruction::DefineFunction(0)).is_none());
    assert!(pack(&Instruction::DestructObject(0)).is_none());
    assert!(pack(&Instruction::Spawn {
        payload_idx: 5,
        first_arg: 0,
        arg_count: 0
    })
    .is_none());
}

#[test]
fn packed_unpack_unknown_opcode() {
    // Forge a packed value with an unused opcode
    let bad = encode(254, 0, 0);
    assert!(unpack(bad).is_none());
}

#[test]
fn packed_bit_layout() {
    // Verify the bit layout is correct
    let packed = encode(0xAB, 0xCD, 0x1234);
    let (op, a1, a2) = decode(packed);
    assert_eq!(op, 0xAB);
    assert_eq!(a1, 0xCD);
    assert_eq!(a2, 0x1234);
}
