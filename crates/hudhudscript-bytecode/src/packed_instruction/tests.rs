use super::*;

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
        Instruction::Print,
        Instruction::Return,
        Instruction::PushScope,
        Instruction::PopScope,
        Instruction::Index,
        Instruction::Break,
        Instruction::Continue,
        Instruction::TryEnd,
        Instruction::Throw,
        Instruction::Send { message: 0, target: 1 },
        Instruction::Require { .. },
        Instruction::Perform { .. },
        Instruction::Await { .. },
        Instruction::LoopEnd,
        Instruction::Yield { .. },
        Instruction::ArrayPush,
        Instruction::SpreadIntoArray,
    ];
    for instr in &instrs {
        assert_round_trip(instr);
    }
}

#[test]
fn packed_round_trip_load_const() {
    assert_round_trip(&Instruction::LoadConst(0));
    assert_round_trip(&Instruction::LoadConst(255));
    assert_round_trip(&Instruction::LoadConst(65535));
}

#[test]
fn packed_load_const_overflow() {
    // u32 > u16::MAX cannot fit into the packed fast path.
    assert!(pack(&Instruction::LoadConst(65536)).is_none());
}

#[test]
fn packed_round_trip_jumps() {
    // Jumps are now i32 relative offsets packed as i16 (-32768..=32767).
    assert_round_trip(&Instruction::Jump(0));
    assert_round_trip(&Instruction::Jump(1000));
    assert_round_trip(&Instruction::Jump(32767));   // i16 max
    assert_round_trip(&Instruction::Jump(-1));      // backward
    assert_round_trip(&Instruction::Jump(-32768));  // i16 min
    assert_round_trip(&Instruction::JumpIfFalse { src: 255, offset: 42 });
    assert_round_trip(&Instruction::JumpIfTrue { src: 255, offset: 100 });
    assert_round_trip(&Instruction::JumpIfFalse { src: 255, offset: -500 });
}

#[test]
fn packed_jump_overflow() {
    // Out-of-i16 range falls through to unpacked (None from pack).
    assert!(pack(&Instruction::Jump(32768)).is_none());    // > i16::MAX
    assert!(pack(&Instruction::Jump(-32769)).is_none());   // < i16::MIN
    assert!(pack(&Instruction::JumpIfFalse { src: 255, offset: 70000 }).is_none());
}

#[test]
fn packed_round_trip_var_ops() {
    assert_round_trip(&Instruction::LoadGlobal { dst: 0, sym: 0 });
    assert_round_trip(&Instruction::LoadGlobal { dst: 1, sym: 65535 });
    assert_round_trip(&Instruction::StoreGlobal { src: 0, sym: 100 });
    assert_round_trip(&Instruction::DeclGlobal { src: 0, sym: 200 });
    assert_round_trip(&Instruction::StoreConst { src: 0, sym: 300 });
}

#[test]
fn packed_var_overflow() {
    // LoadGlobal uses u32 which can overflow u16 packing
    assert!(pack(&Instruction::LoadGlobal { dst: 0, sym: 65536 }).is_none());
}

#[test]
fn packed_round_trip_make_array_object() {
    assert_round_trip(&Instruction::MakeArray(0));
    assert_round_trip(&Instruction::MakeArray(10));
    assert_round_trip(&Instruction::MakeObject(5));
}

#[test]
fn packed_round_trip_symid_single() {
    assert_round_trip(&Instruction::GetProperty(SymId(0)));
    assert_round_trip(&Instruction::GetProperty(SymId(65535)));
    assert_round_trip(&Instruction::SetProperty(SymId(42)));
    assert_round_trip(&Instruction::BindVar(SymId(7)));
    assert_round_trip(&Instruction::ForIn(SymId(99)));
    assert_round_trip(&Instruction::InstanceOf(SymId(10)));
    assert_round_trip(&Instruction::Receive { var_sym: SymId(5), src: 255 });
}

#[test]
fn packed_symid_overflow() {
    assert!(pack(&Instruction::GetProperty(SymId(65536))).is_none());
}

#[test]
fn packed_round_trip_call_variants() {
    // CROSS-2c: each call-family variant now carries a single u32
    // call_payloads index; operand/argc live in the payload.
    assert_round_trip(&Instruction::MethodCall(20));
    assert_round_trip(&Instruction::NewInstance { payload_idx: 30, first_arg: 0, arg_count: 0 });
    // Spawn is now register-based (not packable in u32)
    assert!(pack(&Instruction::Spawn { payload_idx: 5, first_arg: 0, arg_count: 0 }).is_none());
    assert_round_trip(&Instruction::SuperCall(8));
    assert_round_trip(&Instruction::MakeGenerator { payload_idx: 15, first_arg: 0, arg_count: 0 });
}

#[test]
fn packed_round_trip_push_loop() {
    // CROSS-2b: LoopBegin carries a u32 payload index, packed into
    // arg2 (u16 wide).  Verify boundaries 0, 255, 65535.
    assert_round_trip(&Instruction::LoopBegin(0));
    assert_round_trip(&Instruction::LoopBegin(255));
    assert_round_trip(&Instruction::LoopBegin(65535));
}

#[test]
fn packed_push_loop_overflow() {
    // payload idx > u16::MAX cannot fit into the packed fast path.
    assert!(pack(&Instruction::LoopBegin(65536)).is_none());
}

#[test]
fn packed_round_trip_match_variant() {
    // CROSS-2d: carries a single u32 two_sym_payloads index.
    assert_round_trip(&Instruction::MatchVariant(1));
}

#[test]
fn packed_round_trip_get_static() {
    // CROSS-2d: carries a single u32 two_sym_payloads index.
    assert_round_trip(&Instruction::GetStatic(3));
}

#[test]
fn packed_round_trip_destruct_array() {
    assert_round_trip(&Instruction::DestructArray(3, false));
    assert_round_trip(&Instruction::DestructArray(5, true));
}

#[test]
fn packed_round_trip_iter_push_try() {
    assert_round_trip(&Instruction::IterNext { iter_reg: 255, var_sym: SymId(0), end_offset: 500 });
    assert_round_trip(&Instruction::TryBegin(200));
}

#[test]
fn packed_complex_returns_none() {
    // Instructions with String/Vec/Option payloads cannot be packed.
    // CROSS-2a: the 7 previously-boxed variants now carry `u32`
    // side-table indices but are still explicitly excluded from
    // packing (their runtime payload is opaque to the packer).
    // CROSS-2d: `StoreTyped` now carries a u32 two_sym_payloads
    // index — still kept in the unpacked set for uniformity with
    // the rest of the declaration-style variants.
    assert!(pack(&Instruction::StoreTyped(0)).is_none());
    assert!(pack(&Instruction::EnumDecl(0)).is_none());
    assert!(pack(&Instruction::ClassDecl(0)).is_none());
    assert!(pack(&Instruction::LoadModule(0)).is_none());
    assert!(pack(&Instruction::DefineFunction(0)).is_none());
    assert!(pack(&Instruction::DestructObject(0)).is_none());
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
