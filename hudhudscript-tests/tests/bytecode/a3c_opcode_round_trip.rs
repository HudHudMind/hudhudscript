#![cfg(feature = "stack-vm-legacy")]
#![cfg(feature = "stack-vm-legacy")]
//! A3c — round-trip coverage for the 19 new `IntXxx` opcodes.  Ensures
//! both the enum variant and the packed `pack()` / `unpack()` codec
//! agree on every shape.
//!
//! Kural 1 compliant: brand-new test file, no existing tests touched.

use hudhudscript_bytecode::packed_instruction::{pack, unpack};
use hudhudscript_bytecode::{Instruction, BYTECODE_VERSION};

fn assert_round_trip(instr: &Instruction) {
    let packed = pack(instr).unwrap_or_else(|| panic!("pack None for {:?}", instr));
    let unpacked = unpack(packed).unwrap_or_else(|| panic!("unpack None for {:#010X}", packed));
    assert_eq!(
        format!("{:?}", instr),
        format!("{:?}", unpacked),
        "round-trip mismatch {:#010X}",
        packed
    );
}

#[test]
fn a3c_bytecode_version_is_17() {
    // A3c landed at v17; A2 super-instruction fusion bumped to v18.
    // The assertion widened (>=17) keeps the A3c contract intact while
    // allowing subsequent perf-oriented bumps (A2, future fuses) to
    // proceed without re-invalidating this test.  Kural 7c: no
    // fallback — the lower bound is the real invariant A3c owns.
    assert!(
        BYTECODE_VERSION >= 17,
        "A3c requires BYTECODE_VERSION ≥ 17 (adds 12 IntXxx + 7 IntXxxISlot); got {}",
        BYTECODE_VERSION
    );
}

#[test]
fn a3c_base_int_ops_pack_round_trip() {
    for instr in [
        Instruction::IntAdd,
        Instruction::IntSub,
        Instruction::IntMul,
        Instruction::IntDiv,
        Instruction::IntMod,
        Instruction::IntNeg,
        Instruction::IntLt,
        Instruction::IntLe,
        Instruction::IntGt,
        Instruction::IntGe,
        Instruction::IntEq,
        Instruction::IntNe,
    ] {
        assert_round_trip(&instr);
    }
}

#[test]
fn a3c_int_slot_ops_pack_round_trip() {
    // Basic: slot ≤ 255, imm fits i16 — takes the packed fast path.
    for instr in [
        Instruction::IntSubISlot { slot: 0, imm: 1 },
        Instruction::IntSubISlot { slot: 255, imm: -1 },
        Instruction::IntAddISlot { slot: 3, imm: 100 },
        Instruction::IntLtISlot { slot: 2, imm: 2 },
        Instruction::IntLeISlot {
            slot: 7,
            imm: -1000,
        },
        Instruction::IntGtISlot { slot: 0, imm: 42 },
        Instruction::IntGeISlot { slot: 1, imm: 0 },
        Instruction::IntEqISlot { slot: 4, imm: -1 },
    ] {
        assert_round_trip(&instr);
    }
}

#[test]
fn a3c_int_slot_overflow_falls_through_to_unpacked() {
    // slot > 255 cannot fit in the packed u8 — pack() must return None.
    assert!(pack(&Instruction::IntSubISlot { slot: 256, imm: 1 }).is_none());
    assert!(pack(&Instruction::IntAddISlot { slot: 1000, imm: 1 }).is_none());
}
