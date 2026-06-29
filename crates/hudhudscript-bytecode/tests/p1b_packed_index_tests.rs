// P1b: pack/unpack roundtrip tests for IndexArray/IndexStringAscii.

use hudhudscript_bytecode::Instruction;
use hudhudscript_bytecode::packed_instruction::{pack, unpack};

#[test]
fn pack_index_array_returns_some() {
    let instr = Instruction::IndexArray { dst: 5, obj: 10, idx: 3 };
    let p = pack(&instr);
    assert!(p.is_some(), "IndexArray must be packable, got None");
}

#[test]
fn pack_index_string_ascii_returns_some() {
    let instr = Instruction::IndexStringAscii { dst: 5, obj: 10, idx: 3 };
    let p = pack(&instr);
    assert!(p.is_some(), "IndexStringAscii must be packable, got None");
}

#[test]
fn unpack_pack_index_array_roundtrip() {
    let original = Instruction::IndexArray { dst: 5, obj: 10, idx: 3 };
    let p = pack(&original).expect("pack");
    let unpacked = unpack(p).expect("unpack");
    assert_eq!(
        format!("{:?}", unpacked),
        format!("{:?}", original),
        "IndexArray pack/unpack roundtrip mismatch"
    );
}

#[test]
fn unpack_pack_index_string_ascii_roundtrip() {
    let original = Instruction::IndexStringAscii { dst: 7, obj: 12, idx: 4 };
    let p = pack(&original).expect("pack");
    let unpacked = unpack(p).expect("unpack");
    assert_eq!(
        format!("{:?}", unpacked),
        format!("{:?}", original),
        "IndexStringAscii pack/unpack roundtrip mismatch"
    );
}

#[test]
fn packed_opcode_differs_between_index_array_and_string_ascii() {
    // Same operands, different opcodes — must produce different packed values.
    let arr = pack(&Instruction::IndexArray { dst: 1, obj: 2, idx: 3 }).unwrap();
    let str = pack(&Instruction::IndexStringAscii { dst: 1, obj: 2, idx: 3 }).unwrap();
    assert_ne!(arr, str, "IndexArray and IndexStringAscii packed values must differ");
}

#[test]
fn generic_index_still_packs() {
    let p = pack(&Instruction::Index { dst: 1, obj: 2, idx: 3 });
    assert!(p.is_some(), "generic Index must still be packable");
}
