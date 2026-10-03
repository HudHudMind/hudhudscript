//! Coverage: packed instruction decoding (`packed_instruction/unpack.rs`).
//!
//! Every packable `Instruction` kind is packed, fed to `unpack()` and its
//! decoded opcode and operand fields are asserted exactly (pack() is
//! injective on these kinds, so re-packing the decoded word pins every
//! operand). The suite also pins the codec's documented quirks: lossy
//! back-compat sentinels (register operands forced back to 255), packed-only
//! opcodes with no unpack arm, legacy raw opcodes, SymId u16 truncation,
//! i16/i8 range rejection, GT/GE operand swapping, unknown-opcode rejection.

use hudhudscript_bytecode::packed_instruction::{decode, encode, pack, unpack};
use hudhudscript_bytecode::{Instruction, SymId};

// ── helpers ──────────────────────────────────────────────────────────────────

/// Opcode byte a given instruction packs to (`None` = not packable).
fn opcode_of(i: &Instruction) -> Option<u8> {
    pack(i).map(|p| decode(p).0)
}

/// Full pack → unpack round-trip (`None` = not packable OR no unpack arm).
fn roundtrip(i: &Instruction) -> Option<Instruction> {
    pack(i).and_then(unpack)
}

/// RRR wire layout expectation: (opcode, arg1, hi_reg, lo_reg).
fn rrr_wire(i: &Instruction) -> Option<(u8, u8, u8, u8)> {
    match i {
        Instruction::IntAdd { dst, src1, src2 } => Some((118, *dst, *src1, *src2)),
        Instruction::IntSub { dst, src1, src2 } => Some((119, *dst, *src1, *src2)),
        Instruction::IntMul { dst, src1, src2 } => Some((120, *dst, *src1, *src2)),
        Instruction::IntMod { dst, src1, src2 } => Some((128, *dst, *src1, *src2)),
        Instruction::NumAdd { dst, src1, src2 } => Some((129, *dst, *src1, *src2)),
        Instruction::NumSub { dst, src1, src2 } => Some((130, *dst, *src1, *src2)),
        Instruction::NumMul { dst, src1, src2 } => Some((131, *dst, *src1, *src2)),
        Instruction::NumDiv { dst, src1, src2 } => Some((132, *dst, *src1, *src2)),
        Instruction::Index { dst, obj, idx } => Some((9, *dst, *obj, *idx)),
        Instruction::StrCat { dst, src1, src2 } => Some((10, *dst, *src1, *src2)),
        Instruction::StringIndexOf { dst, haystack, needle } => Some((16, *dst, *haystack, *needle)),
        Instruction::StringContains { dst, haystack, needle } => Some((17, *dst, *haystack, *needle)),
        Instruction::ArrayPush { dst, arr, val } => Some((15, *dst, *arr, *val)),
        Instruction::IndexArray { dst, obj, idx } => Some((149, *dst, *obj, *idx)),
        Instruction::IndexStringAscii { dst, obj, idx } => Some((150, *dst, *obj, *idx)),
        Instruction::IndexAssign { obj, idx, val } => Some((133, *obj, *idx, *val)),
        _ => None,
    }
}

/// RI wire layout expectation: (opcode, dst, src, imm).
fn ri_wire(i: &Instruction) -> Option<(u8, u8, u8, i16)> {
    match i {
        Instruction::IntAddI { dst, src, imm } => Some((11, *dst, *src, *imm)),
        Instruction::IntSubI { dst, src, imm } => Some((12, *dst, *src, *imm)),
        Instruction::IntMulI { dst, src, imm } => Some((122, *dst, *src, *imm)),
        Instruction::NumAddI { dst, src, imm } => Some((134, *dst, *src, *imm)),
        Instruction::NumSubI { dst, src, imm } => Some((135, *dst, *src, *imm)),
        Instruction::NumMulI { dst, src, imm } => Some((136, *dst, *src, *imm)),
        Instruction::NumDivI { dst, src, imm } => Some((137, *dst, *src, *imm)),
        _ => None,
    }
}

// ── codec primitives: 32-bit layout [opcode:u8][arg1:u8][arg2:u16] ───────────

#[test]
fn encode_bit_layout_is_opcode_arg1_arg2() {
    assert_eq!(encode(0xFF, 0xAB, 0xCDEF), 0xCDEFABFF);
    assert_eq!(decode(0xCDEFABFF), (0xFF, 0xAB, 0xCDEF));
    assert_eq!(decode(encode(139, 2, 0x0304)), (139, 2, 0x0304));
    assert_eq!(encode(0, 0, 0), 0);
}

// ── identity round-trips: opcode + operands survive exactly ─────────────────

#[test]
fn identity_roundtrips_preserve_opcode_and_operands() {
    let t: Vec<(Instruction, u8)> = vec![
        // zero-arg
        (Instruction::Break, 30), (Instruction::Continue, 31), (Instruction::TryEnd, 32), (Instruction::FinallyEnd, 43),
        // jumps, including i16 boundaries and both conditional forms
        (Instruction::Jump(0), 20), (Instruction::Jump(32767), 20), (Instruction::Jump(-32768), 20),
        (Instruction::JumpIfFalse { src: 255, offset: -2 }, 21), (Instruction::JumpIfTrue { src: 255, offset: 777 }, 22),
        (Instruction::JumpIfFalse { src: 200, offset: -300 }, 7), (Instruction::JumpIfTrue { src: 0, offset: -1 }, 8),
        // try/finally/iter, canonical loop forms
        (Instruction::TryBegin(-1), 96), (Instruction::FinallyBegin(2048), 97), (Instruction::FinallyExit(-2048), 98),
        (Instruction::IterNext { iter_reg: 255, var_sym_idx: 0, end_offset: -77 }, 95),
        (Instruction::ForIn { iter_reg: 255, var_sym_idx: 4444, end_offset: 0 }, 92),
        // symbol / payload / table indices (u16 boundary values)
        (Instruction::BindVar(SymId(7)), 91), (Instruction::BindVar(SymId(0xFFFF)), 91), (Instruction::CallSpread(SymId(0xABCD)), 105),
        (Instruction::LoopBegin(0xFFFF), 107), (Instruction::MatchVariant(0x0BEB), 108), (Instruction::GetStatic(0xCAFE), 109),
        (Instruction::DestructArray(300, true), 110), (Instruction::DestructArray(0, false), 110),
        (Instruction::IntLeJumpIfFalse(42), 112), (Instruction::IntLtJumpIfFalse(43), 117),
        // int + float register arithmetic (register boundaries 0/254/255/200)
        (Instruction::IntAdd { dst: 200, src1: 255, src2: 0 }, 118), (Instruction::IntSub { dst: 1, src1: 254, src2: 2 }, 119),
        (Instruction::IntMul { dst: 3, src1: 4, src2: 5 }, 120), (Instruction::IntMod { dst: 6, src1: 7, src2: 8 }, 128),
        (Instruction::NumAdd { dst: 9, src1: 10, src2: 11 }, 129), (Instruction::NumSub { dst: 12, src1: 13, src2: 14 }, 130),
        (Instruction::NumMul { dst: 15, src1: 16, src2: 17 }, 131), (Instruction::NumDiv { dst: 18, src1: 19, src2: 20 }, 132),
        // index / string / array RRR family
        (Instruction::Index { dst: 21, obj: 22, idx: 23 }, 9), (Instruction::StrCat { dst: 24, src1: 25, src2: 26 }, 10),
        (Instruction::StringIndexOf { dst: 27, haystack: 28, needle: 29 }, 16), (Instruction::StringContains { dst: 30, haystack: 31, needle: 32 }, 17),
        (Instruction::ArrayPush { dst: 33, arr: 34, val: 35 }, 15), (Instruction::IndexArray { dst: 36, obj: 37, idx: 38 }, 149),
        (Instruction::IndexStringAscii { dst: 39, obj: 40, idx: 41 }, 150), (Instruction::IndexAssign { obj: 42, idx: 43, val: 44 }, 133),
        // move / unary / return / const / fused
        (Instruction::StringConcat { regs_start: 16, count: 5, dst: 230 }, 153), (Instruction::StrCatMut { dst: 4, src2: 5 }, 18),
        (Instruction::NumMulAddAssign { dst: 200, mul: 255, add: 0 }, 139), (Instruction::Move { dst: 0, src: 255 }, 4),
        (Instruction::Neg { dst: 9, src: 250 }, 13), (Instruction::Not { dst: 8, src: 251 }, 14),
        (Instruction::Return { src: 199 }, 125), (Instruction::LoadIntConst { dst: 255, const_idx: 0xFFFF }, 121),
        // immediate arithmetic (i8 boundary immediates)
        (Instruction::IntAddI { dst: 7, src: 9, imm: -3 }, 11), (Instruction::IntSubI { dst: 1, src: 2, imm: -128 }, 12),
        (Instruction::IntMulI { dst: 3, src: 4, imm: 127 }, 122), (Instruction::NumAddI { dst: 5, src: 6, imm: -1 }, 134),
        (Instruction::NumSubI { dst: 8, src: 9, imm: -128 }, 135), (Instruction::NumMulI { dst: 10, src: 11, imm: 127 }, 136),
        (Instruction::NumDivI { dst: 12, src: 13, imm: -2 }, 137),
        // comparisons that round-trip identically (GT/GE do not — see below)
        (Instruction::IntCmp { dst: 200, src1: 3, src2: 8, op: 0 }, 1), (Instruction::IntCmp { dst: 200, src1: 3, src2: 8, op: 1 }, 2),
        (Instruction::IntCmp { dst: 200, src1: 3, src2: 8, op: 4 }, 0), (Instruction::IntCmp { dst: 200, src1: 3, src2: 8, op: 5 }, 3),
        // canonical call sentinels
        (Instruction::Require { src: 255 }, 35), (Instruction::Perform { src: 255 }, 36),
        (Instruction::Await { src: 255, dst: 255 }, 37), (Instruction::Yield { src: 255 }, 39),
        (Instruction::MethodCall { dst: 255, obj: 255, payload_idx: 0xFEED, first_arg: 0, arg_count: 0 }, 100),
        (Instruction::SuperCall { dst: 255, payload_idx: 0xBEEF, first_arg: 0, arg_count: 0 }, 103),
    ];
    for (instr, wire) in t {
        let p = pack(&instr).unwrap_or_else(|| panic!("pack failed: {instr:?}"));
        let (op, a1, a2) = decode(p);
        assert_eq!(op, wire, "wire opcode for {instr:?}");
        // RRR kinds: arg1 = dst/obj, arg2 = (hi_reg << 8) | lo_reg.
        if let Some((_, want_a1, hi, lo)) = rrr_wire(&instr) {
            assert_eq!(a1, want_a1, "arg1 (dst/obj) for {instr:?}");
            assert_eq!(((a2 >> 8) & 0xFF) as u8, hi, "hi register for {instr:?}");
            assert_eq!((a2 & 0xFF) as u8, lo, "lo register for {instr:?}");
        }
        // RI kinds: arg2 = (imm_i8 << 8) | src, imm sign-extends to i16.
        if let Some((_, dst, src, imm)) = ri_wire(&instr) {
            assert_eq!(a1, dst, "arg1 (dst) for {instr:?}");
            assert_eq!((a2 & 0xFF) as u8, src, "src register for {instr:?}");
            assert_eq!(((a2 >> 8) as u8) as i8 as i16, imm, "sign-extended imm for {instr:?}");
        }
        let back = unpack(p).unwrap_or_else(|| panic!("unpack failed: {instr:?}"));
        // pack() is injective on these kinds: same word <=> same operands.
        assert_eq!(pack(&back), Some(p), "decoded operands for {instr:?}");
    }
}

// ── range rejection: operands that cannot fit the packed layout ──────────────

#[test]
fn out_of_range_operands_are_rejected_by_pack() {
    assert_eq!(pack(&Instruction::Jump(32768)), None);
    assert_eq!(pack(&Instruction::Jump(-32769)), None);
    assert_eq!(pack(&Instruction::Jump(70000)), None);
    assert_eq!(pack(&Instruction::TryBegin(32768)), None);
    assert_eq!(pack(&Instruction::FinallyBegin(70000)), None);
    assert_eq!(pack(&Instruction::FinallyExit(-40000)), None);
    assert_eq!(pack(&Instruction::LoopBegin(0x1_0000)), None);
    assert_eq!(pack(&Instruction::MatchVariant(0x1_0000)), None);
    assert_eq!(pack(&Instruction::GetStatic(0x1_0000)), None);
    assert_eq!(pack(&Instruction::IntCmp { dst: 1, src1: 2, src2: 3, op: 9 }), None);
    // Immediates outside i8 do not fit the packed layout.
    assert_eq!(pack(&Instruction::IntAddI { dst: 1, src: 2, imm: 200 }), None);
    assert_eq!(pack(&Instruction::IntAddI { dst: 1, src: 2, imm: -129 }), None);
    assert_eq!(pack(&Instruction::NumAddI { dst: 1, src: 2, imm: 200 }), None);
    // Non-canonical register forms are unpacked-only.
    assert_eq!(pack(&Instruction::ForIn { iter_reg: 3, var_sym_idx: 1, end_offset: 2 }), None);
    assert_eq!(pack(&Instruction::Receive { var_sym_idx: 1, src: 2 }), None);
    assert_eq!(pack(&Instruction::IterNext { iter_reg: 1, var_sym_idx: 2, end_offset: 3 }), None);
    assert_eq!(pack(&Instruction::MethodCall { dst: 4, obj: 5, payload_idx: 6, first_arg: 7, arg_count: 8 }), None);
    assert_eq!(pack(&Instruction::SuperCall { dst: 9, payload_idx: 6, first_arg: 7, arg_count: 8 }), None);
}

// ── IntCmp GT/GE: packed as operand-swapped LT/LE (decode is NOT identity) ──

#[test]
fn int_cmp_gt_ge_decode_as_swapped_lt_le() {
    let gt = Instruction::IntCmp { dst: 5, src1: 3, src2: 8, op: 2 };
    assert_eq!(opcode_of(&gt), Some(1)); // GT rides the OP_INT_LT_RR opcode
    match roundtrip(&gt) {
        Some(Instruction::IntCmp { dst, src1, src2, op }) => {
            assert_eq!((dst, src1, src2, op), (5, 8, 3, 0));
        }
        other => panic!("IntCmp GT swap round-trip failed: {other:?}"),
    }
    let ge = Instruction::IntCmp { dst: 5, src1: 3, src2: 8, op: 3 };
    assert_eq!(opcode_of(&ge), Some(2)); // GE rides the OP_INT_LE_RR opcode
    match roundtrip(&ge) {
        Some(Instruction::IntCmp { dst, src1, src2, op }) => {
            assert_eq!((dst, src1, src2, op), (5, 8, 3, 1));
        }
        other => panic!("IntCmp GE swap round-trip failed: {other:?}"),
    }
}

// ── lossy sentinel round-trips (register operands dropped by the packer) ────

#[test]
fn sentinel_ops_roundtrip_lossily() {
    // Non-sentinel register operands are dropped by the packer (back compat):
    // every decode comes back with the 255 sentinel register.
    match roundtrip(&Instruction::Require { src: 9 }) {
        Some(Instruction::Require { src }) => assert_eq!(src, 255),
        other => panic!("Require lossy round-trip failed: {other:?}"),
    }
    match roundtrip(&Instruction::Perform { src: 3 }) {
        Some(Instruction::Perform { src }) => assert_eq!(src, 255),
        other => panic!("Perform lossy round-trip failed: {other:?}"),
    }
    match roundtrip(&Instruction::Await { src: 1, dst: 2 }) {
        Some(Instruction::Await { src, dst }) => assert_eq!((src, dst), (255, 255)),
        other => panic!("Await lossy round-trip failed: {other:?}"),
    }
    match roundtrip(&Instruction::Yield { src: 77 }) {
        Some(Instruction::Yield { src }) => assert_eq!(src, 255),
        other => panic!("Yield lossy round-trip failed: {other:?}"),
    }
}

#[test]
fn symid_above_u16_truncates_on_pack() {
    // Documented u16 back-compat truncation: only the low 16 bits survive.
    match roundtrip(&Instruction::BindVar(SymId(0x1_0001))) {
        Some(Instruction::BindVar(sym)) => assert_eq!(sym.0, 1),
        other => panic!("BindVar truncation round-trip failed: {other:?}"),
    }
}

// ── packed-only opcodes: packable on the wire, no unpack arm ────────────────

#[test]
fn packed_only_opcodes_have_no_unpack_arm() {
    // These pack to real opcodes but unpack() returns None: the VM decodes
    // them directly from the packed form (documented in unpack.rs comments).
    let t: Vec<(Instruction, u8)> = vec![
        (Instruction::NewInstance { payload_idx: 3, first_arg: 0, arg_count: 0 }, 101),
        (Instruction::MakeGenerator { payload_idx: 4, first_arg: 0, arg_count: 0 }, 104),
        (Instruction::LoopEnd, 38), (Instruction::IntModI { dst: 1, src: 2, imm: 3 }, 140),
        (Instruction::IntCmpI { dst: 1, src: 2, imm: 3, op: 0 }, 141), (Instruction::IntCmpI { dst: 1, src: 2, imm: 3, op: 1 }, 142),
        (Instruction::IntCmpI { dst: 1, src: 2, imm: 3, op: 4 }, 143), (Instruction::IntCmpI { dst: 1, src: 2, imm: 3, op: 5 }, 144),
        (Instruction::IntLtRRJumpPacked(5), 147), (Instruction::IntLeRRJumpPacked(6), 148),
        (Instruction::IntCmpRRJumpPacked { op: 2, payload_idx: 9 }, 151),
        (Instruction::FLoadNum { fslot: 1, src: 2 }, 154), (Instruction::FStoreNum { dst: 1, fslot: 2 }, 155),
        (Instruction::FAdd { d: 1, a: 2, b: 3 }, 156), (Instruction::FSub { d: 1, a: 2, b: 3 }, 157),
        (Instruction::FMul { d: 1, a: 2, b: 3 }, 158), (Instruction::FDiv { d: 1, a: 2, b: 3 }, 159),
        (Instruction::FSin { d: 1, s: 2 }, 160), (Instruction::FCos { d: 1, s: 2 }, 161),
        (Instruction::FSqrt { d: 1, s: 2 }, 162), (Instruction::FConst { d: 1, const_idx: 2 }, 163),
        (Instruction::FMove { d: 1, s: 2 }, 164),
    ];
    for (instr, wire) in t {
        assert_eq!(opcode_of(&instr), Some(wire), "wire opcode for {instr:?}");
        assert!(roundtrip(&instr).is_none(), "unpack must reject packed-only {instr:?}");
    }
}

// ── legacy raw opcodes decoded straight from encode() ───────────────────────

#[test]
fn legacy_raw_decode_paths() {
    // THROW decodes with the 255 sentinel register.
    match unpack(encode(33, 0, 0)) {
        Some(Instruction::Throw { src }) => assert_eq!(src, 255),
        other => panic!("raw THROW decode failed: {other:?}"),
    }
    // RECEIVE: arg2 is the symbol index, src is the sentinel.
    match unpack(encode(94, 0, 55)) {
        Some(Instruction::Receive { var_sym_idx, src }) => {
            assert_eq!(var_sym_idx, 55);
            assert_eq!(src, 255);
        }
        other => panic!("raw RECEIVE decode failed: {other:?}"),
    }
    // FOR_IN: arg2 is the loop variable symbol, iter register is the sentinel.
    match unpack(encode(92, 0, 77)) {
        Some(Instruction::ForIn { iter_reg, var_sym_idx, end_offset }) => {
            assert_eq!((iter_reg, var_sym_idx, end_offset), (255, 77, 0));
        }
        other => panic!("raw FOR_IN decode failed: {other:?}"),
    }
    // ITER_NEXT: arg2 is a signed i16 end offset (0xFFB3 == -77).
    match unpack(encode(95, 0, 0xFFB3)) {
        Some(Instruction::IterNext { iter_reg, var_sym_idx, end_offset }) => {
            assert_eq!((iter_reg, var_sym_idx, end_offset), (255, 0, -77));
        }
        other => panic!("raw ITER_NEXT decode failed: {other:?}"),
    }
    // METHOD_CALL / SUPER_CALL: arg2 is the payload index, registers sentinel.
    match unpack(encode(100, 0, 321)) {
        Some(Instruction::MethodCall { dst, obj, payload_idx, first_arg, arg_count }) => {
            assert_eq!((dst, obj, payload_idx, first_arg, arg_count), (255, 255, 321, 0, 0));
        }
        other => panic!("raw METHOD_CALL decode failed: {other:?}"),
    }
    match unpack(encode(103, 0, 322)) {
        Some(Instruction::SuperCall { dst, payload_idx, first_arg, arg_count }) => {
            assert_eq!((dst, payload_idx, first_arg, arg_count), (255, 322, 0, 0));
        }
        other => panic!("raw SUPER_CALL decode failed: {other:?}"),
    }
    // Super-instructions: arg2 is a u32 index truncated to u16.
    match unpack(encode(111, 0, 77)) {
        Some(Instruction::IntSubCall1(idx)) => assert_eq!(idx, 77),
        other => panic!("raw INT_SUB_CALL_1 decode failed: {other:?}"),
    }
    match unpack(encode(113, 0, 88)) {
        Some(Instruction::IntAddCall1(idx)) => assert_eq!(idx, 88),
        other => panic!("raw INT_ADD_CALL_1 decode failed: {other:?}"),
    }
    match unpack(encode(112, 0, 42)) {
        Some(Instruction::IntLeJumpIfFalse(idx)) => assert_eq!(idx, 42),
        other => panic!("raw INT_LE_JUMP_IF_FALSE decode failed: {other:?}"),
    }
    match unpack(encode(117, 0, 43)) {
        Some(Instruction::IntLtJumpIfFalse(idx)) => assert_eq!(idx, 43),
        other => panic!("raw INT_LT_JUMP_IF_FALSE decode failed: {other:?}"),
    }
    // Legacy JUMP_IF_FALSE ignores arg1 — src is always the 255 sentinel.
    match unpack(encode(21, 99, 500)) {
        Some(Instruction::JumpIfFalse { src, offset }) => {
            assert_eq!(src, 255);
            assert_eq!(offset, 500);
        }
        other => panic!("raw JUMP_IF_FALSE decode failed: {other:?}"),
    }
}

#[test]
fn back_compat_and_unknown_opcodes_decode_to_none() {
    // Documented back-compat rejects inside unpack().
    assert!(unpack(encode(126, 0, 0)).is_none()); // OP_STR_CAT
    assert!(unpack(encode(34, 9, 9)).is_none()); // OP_SEND
    assert!(unpack(encode(40, 1, 2)).is_none()); // OP_ARRAY_PUSH (slot form)
    assert!(unpack(encode(41, 1, 2)).is_none()); // OP_SPREAD_INTO_ARRAY
    assert!(unpack(encode(42, 1, 2)).is_none()); // OP_SPREAD_INTO_OBJECT
    assert!(unpack(encode(69, 1, 2)).is_none()); // OP_LOAD_NUM_CONST
    assert!(unpack(encode(88, 1, 2)).is_none()); // OP_LOAD_INT_CONST
    assert!(unpack(encode(89, 1, 2)).is_none()); // OP_GET_PROPERTY
    assert!(unpack(encode(90, 1, 2)).is_none()); // OP_SET_PROPERTY
    assert!(unpack(encode(102, 1, 2)).is_none()); // OP_SPAWN
    assert!(unpack(encode(106, 1, 2)).is_none()); // OP_METHOD_CALL_SPREAD
    // 5, 6, 200, 250 and 0xFF are unassigned opcode bytes.
    assert!(unpack(encode(5, 255, 0xFFFF)).is_none());
    assert!(unpack(encode(6, 0, 0)).is_none());
    assert!(unpack(encode(200, 1, 2)).is_none());
    assert!(unpack(encode(250, 255, 65535)).is_none());
    assert!(unpack(0xFFFF_FFFF).is_none());
}

// ── kinds the packer explicitly rejects (register / complex operands) ───────

#[test]
fn unpackable_kinds_return_none_from_pack() {
    assert_eq!(pack(&Instruction::Throw { src: 5 }), None);
    assert_eq!(pack(&Instruction::Send { message: 1, target: 2 }), None);
    assert_eq!(pack(&Instruction::Despawn { reg: 3 }), None);
    assert_eq!(pack(&Instruction::ViewAs { obj: 1, view_sym: 2 }), None);
    assert_eq!(pack(&Instruction::Spawn { name_sym: 1, first_arg: 2, arg_count: 3 }), None);
    assert_eq!(pack(&Instruction::SpreadIntoArray { dst: 1, src: 2 }), None);
    assert_eq!(pack(&Instruction::SpreadIntoObject { dst: 1, src: 2 }), None);
    assert_eq!(pack(&Instruction::MethodCallSpread { dst: 1, obj: 2, args: 3, method_sym: SymId(4) }), None);
    assert_eq!(pack(&Instruction::MakeArray { dst: 1, count: 2 }), None);
    assert_eq!(pack(&Instruction::Call { dst: 1, payload_idx: 2, first_arg: 3, arg_count: 4 }), None);
    assert_eq!(pack(&Instruction::GetProperty { dst: 1, obj: 2, prop_sym: 3 }), None);
    assert_eq!(pack(&Instruction::SetProperty { dst: 1, obj: 2, val: 3, prop_sym: 4 }), None);
    assert_eq!(pack(&Instruction::TailCall { func_reg: 1, first_arg_reg: 2, arg_count: 3 }), None);
    assert_eq!(pack(&Instruction::LoadConst { dst: 1, const_idx: 2 }), None);
    assert_eq!(pack(&Instruction::LoadNumConst { dst: 1, const_idx: 2 }), None);
    assert_eq!(pack(&Instruction::IntDiv { dst: 1, src1: 2, src2: 3 }), None);
    assert_eq!(pack(&Instruction::IntSubCall1(5)), None);
    assert_eq!(pack(&Instruction::IntAddCall1(5)), None);
    assert_eq!(pack(&Instruction::MakeArray2 { dst: 1, a: 2, b: 3 }), None);
    assert_eq!(pack(&Instruction::NumSqrt { dst: 1, src: 2 }), None);
}
