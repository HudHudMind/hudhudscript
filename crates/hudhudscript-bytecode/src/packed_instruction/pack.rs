use super::opcodes::*;
use super::{encode, Instruction, SymId};

/// Pack an instruction into a `u32`. Returns `None` for complex instructions
/// whose operands cannot fit into the 32-bit layout.
pub fn pack(instr: &Instruction) -> Option<u32> {
    match instr {
        // --- zero-arg ---
        // StrCat removed — use StrCat (register-based)
        Instruction::Break => Some(encode(OP_BREAK, 0, 0)),
        Instruction::Continue => Some(encode(OP_CONTINUE, 0, 0)),
        Instruction::TryEnd => Some(encode(OP_POP_TRY, 0, 0)),
        Instruction::Throw { .. } => None, // Register operand
        Instruction::Send { .. } => None, // Register operands, unpacked
        Instruction::Despawn { .. } => None, // Register operand, unpacked
        Instruction::ViewAs { .. } => None, // Register operand, unpacked
        Instruction::Require { .. } => Some(encode(OP_REQUIRE, 0, 0)),
        Instruction::Perform { .. } => Some(encode(OP_PERFORM, 0, 0)),
        Instruction::Await { .. } => Some(encode(OP_AWAIT, 0, 0)),
        Instruction::LoopEnd => Some(encode(OP_POP_LOOP, 0, 0)),
        Instruction::Yield { .. } => Some(encode(OP_YIELD, 0, 0)),
        Instruction::SpreadIntoArray { .. } => None, // register ops, unpacked
        Instruction::SpreadIntoObject { .. } => None, // register ops, unpacked
        Instruction::FinallyEnd => Some(encode(OP_POP_FINALLY, 0, 0)),

        // --- arg2-only: u32 index → u16 ---
        // LoadNumConst/LoadIntConst removed — use LoadNumConst/LoadIntConst
        // (packed via LoadIntConst/LoadNumConst handlers)
        Instruction::Jump(offset) => Some(encode(OP_JUMP, 0, i16::try_from(*offset).ok()? as u16)),
        Instruction::JumpIfFalse { src: 255, offset } => Some(encode(OP_JUMP_IF_FALSE, 0, i16::try_from(*offset).ok()? as u16)),
        Instruction::JumpIfTrue { src: 255, offset } => Some(encode(OP_JUMP_IF_TRUE, 0, i16::try_from(*offset).ok()? as u16)),
        Instruction::JumpIfFalse { src, offset } => Some(encode(OP_JUMP_IF_FALSE_R, *src, i16::try_from(*offset).ok()? as u16)),
        Instruction::JumpIfTrue { src, offset } => Some(encode(OP_JUMP_IF_TRUE_R, *src, i16::try_from(*offset).ok()? as u16)),

        // --- arg2 = u32 symbol index (u16, back compat only) ---

        // --- S2.2a slot-direct locals (register variants preferred) ---

        // --- arg2-only: u16 element count ---

        // --- SymId in arg2 ---
        // GetProperty / SetProperty are register-based, handled in unpacked path
        Instruction::BindVar(sym) => Some(encode(OP_BIND_VAR, 0, sym.0 as u16)),
        Instruction::ForIn { iter_reg: 255, var_sym_idx: sym, end_offset: 0 } => Some(encode(OP_FOR_IN, 0, *sym)),
        Instruction::Receive { .. } => None, // Register operands

        // --- i32 relative offset → i16 ---
        Instruction::IterNext { iter_reg: 255, var_sym_idx: 0, end_offset } => Some(encode(OP_ITER_NEXT, 0, *end_offset as u16)),
        Instruction::TryBegin(offset) => Some(encode(OP_PUSH_TRY, 0, i16::try_from(*offset).ok()? as u16)),
        Instruction::FinallyBegin(offset) => Some(encode(OP_PUSH_FINALLY, 0, i16::try_from(*offset).ok()? as u16)),
        Instruction::FinallyExit(offset) => Some(encode(OP_END_FINALLY, 0, i16::try_from(*offset).ok()? as u16)),

        // --- u32 payload index → u16 ---
        Instruction::MethodCall { dst: 255, obj: 255, payload_idx, first_arg: 0, arg_count: 0 } => Some(encode(OP_METHOD_CALL, 0, u16::try_from(*payload_idx).ok()?)),
        Instruction::NewInstance { payload_idx: idx, .. } => Some(encode(OP_NEW_INSTANCE, 0, u16::try_from(*idx).ok()?)),
        Instruction::Spawn { .. } => None, // Unpacked — register operands
        Instruction::SuperCall { dst: 255, payload_idx, first_arg: 0, arg_count: 0 } => Some(encode(OP_SUPER_CALL, 0, u16::try_from(*payload_idx).ok()?)),
        Instruction::MakeGenerator { payload_idx: idx, .. } => Some(encode(OP_MAKE_GENERATOR, 0, u16::try_from(*idx).ok()?)),
        Instruction::CallSpread(sym) => Some(encode(OP_CALL_SPREAD, 0, sym.0 as u16)),
        Instruction::MethodCallSpread(sym) => Some(encode(OP_METHOD_CALL_SPREAD, 0, sym.0 as u16)),
        Instruction::LoopBegin(idx) => Some(encode(OP_PUSH_LOOP, 0, u16::try_from(*idx).ok()?)),
        Instruction::MatchVariant(idx) => Some(encode(OP_MATCH_VARIANT, 0, u16::try_from(*idx).ok()?)),
        Instruction::GetStatic(idx) => Some(encode(OP_GET_STATIC, 0, u16::try_from(*idx).ok()?)),
        Instruction::DestructArray(count, has_rest) => Some(encode(OP_DESTRUCT_ARRAY, *has_rest as u8, *count)),

        // --- Slot-based super-instructions ---
        Instruction::IntLeJumpIfFalse(idx) => Some(encode(OP_INT_LE_JUMP_IF_FALSE, 0, u16::try_from(*idx).ok()?)),
        Instruction::IntLtJumpIfFalse(idx) => Some(encode(OP_INT_LT_JUMP_IF_FALSE, 0, u16::try_from(*idx).ok()?)),
        Instruction::IntSubCall1(idx) => Some(encode(OP_INT_SUB_CALL_1, 0, u16::try_from(*idx).ok()?)),
        Instruction::IntAddCall1(idx) => Some(encode(OP_INT_ADD_CALL_1, 0, u16::try_from(*idx).ok()?)),

        // --- IndexFast/IndexAssignFast removed → use Index/IndexAssign ---

        // --- Register-based VM instructions (packed) ---
        Instruction::IntAdd { dst, src1, src2 } => {
            Some(encode(OP_INT_ADD_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16)))
        }
        Instruction::IntSub { dst, src1, src2 } => {
            Some(encode(OP_INT_SUB_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16)))
        }
        Instruction::IntMul { dst, src1, src2 } => {
            Some(encode(OP_INT_MUL_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16)))
        }
        Instruction::IntMod { dst, src1, src2 } => {
            Some(encode(OP_INT_MOD_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16)))
        }
        Instruction::LoadIntConst { dst, const_idx } => Some(encode(OP_LOAD_INT_CONST_R, *dst, *const_idx)),

        Instruction::Return { src } => Some(encode(OP_RETURN_R, *src, 0)),

        // --- Register comparison (packed by op) ---
        Instruction::IntCmpI { dst, src, imm, op } => {
            let imm_i8 = i8::try_from(*imm).ok()?;
            let opcode = match *op {
                0 => OP_INT_CMP_LT_I,
                1 => OP_INT_CMP_LE_I,
                4 => OP_INT_CMP_EQ_I,
                5 => OP_INT_CMP_NE_I,
                _ => return None,
            };
            Some(encode(opcode, *dst, ((*src as u16) << 8) | (imm_i8 as u8 as u16)))
        }
        Instruction::IntDivI { .. } => None, // unpacked only
        Instruction::IntModI { dst, src, imm } => {
            let imm_i8 = i8::try_from(*imm).ok()?;
            Some(encode(OP_INT_MOD_I, *dst, ((*src as u16) << 8) | (imm_i8 as u8 as u16)))
        }
        Instruction::IntModCmpI { .. } => None, // unpacked only
        Instruction::NumMulAddIndexed { .. } => None, // unpacked (4 regs, no room in 32-bit packed)
        Instruction::StrCharEqRR { .. } => None, // unpacked (4 regs)
        Instruction::IntLtRRJumpPacked(idx) => Some(encode(OP_INT_LT_RR_JUMP_P, 0, u16::try_from(*idx).ok()?)),
        Instruction::IntLeRRJumpPacked(idx) => Some(encode(OP_INT_LE_RR_JUMP_P, 0, u16::try_from(*idx).ok()?)),
        Instruction::IntCmp { dst, src1, src2, op } => match *op {
            0 => Some(encode(OP_INT_LT_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16))),
            1 => Some(encode(OP_INT_LE_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16))),
            4 => Some(encode(OP_INT_EQ_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16))),
            5 => Some(encode(OP_INT_NE_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16))),
            _ => None,
        },
        Instruction::Move { dst, src } => {
            Some(encode(OP_MOVE_RR, *dst, *src as u16))
        }
        Instruction::Index { dst, obj, idx } => {
            Some(encode(OP_INDEX_RRR, *dst, ((*obj as u16) << 8) | (*idx as u16)))
        }
        Instruction::StrCat { dst, src1, src2 } => {
            Some(encode(OP_STRCAT_RRR, *dst, ((*src1 as u16) << 8) | (*src2 as u16)))
        }
        Instruction::StrCatMut { dst, src2 } => {
            Some(encode(OP_STRCAT_MUT_RR, *dst, *src2 as u16))
        }
        Instruction::StringIndexOf { dst, haystack, needle } => {
            Some(encode(OP_STRING_INDEX_OF_RRR, *dst, ((*haystack as u16) << 8) | (*needle as u16)))
        }
        Instruction::StringContains { dst, haystack, needle } => {
            Some(encode(OP_STRING_CONTAINS_RRR, *dst, ((*haystack as u16) << 8) | (*needle as u16)))
        }
        Instruction::IntAddI { dst, src, imm } => {
            let imm_i8 = i8::try_from(*imm).ok()?;
            Some(encode(OP_INT_ADD_RI, *dst, ((imm_i8 as u8 as u16) << 8) | (*src as u16)))
        }
        Instruction::IntSubI { dst, src, imm } => {
            let imm_i8 = i8::try_from(*imm).ok()?;
            Some(encode(OP_INT_SUB_RI, *dst, ((imm_i8 as u8 as u16) << 8) | (*src as u16)))
        }
        Instruction::IntMulI { dst, src, imm } => {
            let imm_i8 = i8::try_from(*imm).ok()?;
            Some(encode(OP_INT_MUL_RI, *dst, ((imm_i8 as u8 as u16) << 8) | (*src as u16)))
        }
        Instruction::Neg { dst, src } => {
            Some(encode(OP_NEG_R, *dst, *src as u16))
        }
        Instruction::Not { dst, src } => {
            Some(encode(OP_NOT_R, *dst, *src as u16))
        }
        Instruction::ArrayPush { dst, arr, val } => {
            Some(encode(OP_ARRAY_PUSH_RRR, *dst, ((*arr as u16) << 8) | (*val as u16)))
        }
        Instruction::StringConcat { regs_start, count, dst } => {
            Some(encode(OP_STRING_CONCAT_RR, *dst, ((*regs_start as u16) << 8) | (*count as u16)))
        }
        Instruction::NumAdd { dst, src1, src2 } => {
            Some(encode(OP_NUM_ADD_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16)))
        }
        Instruction::NumSub { dst, src1, src2 } => {
            Some(encode(OP_NUM_SUB_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16)))
        }
        Instruction::NumMul { dst, src1, src2 } => {
            Some(encode(OP_NUM_MUL_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16)))
        }
        Instruction::NumDiv { dst, src1, src2 } => {
            Some(encode(OP_NUM_DIV_RR, *dst, ((*src1 as u16) << 8) | (*src2 as u16)))
        }
        Instruction::NumMulAddAssign { dst, mul, add } => {
            Some(encode(OP_NUM_MUL_ADD_ASSIGN, *dst, ((*mul as u16) << 8) | (*add as u16)))
        }
        Instruction::IndexAssign { obj, idx, val } => {
            Some(encode(OP_INDEX_ASSIGN_RRR, *obj, ((*idx as u16) << 8) | (*val as u16)))
        }
        Instruction::NumAddI { dst, src, imm } => {
            let imm_i8 = i8::try_from(*imm).ok()?;
            Some(encode(OP_NUM_ADD_RI, *dst, ((imm_i8 as u8 as u16) << 8) | (*src as u16)))
        }
        Instruction::NumSubI { dst, src, imm } => {
            let imm_i8 = i8::try_from(*imm).ok()?;
            Some(encode(OP_NUM_SUB_RI, *dst, ((imm_i8 as u8 as u16) << 8) | (*src as u16)))
        }
        Instruction::NumMulI { dst, src, imm } => {
            let imm_i8 = i8::try_from(*imm).ok()?;
            Some(encode(OP_NUM_MUL_RI, *dst, ((imm_i8 as u8 as u16) << 8) | (*src as u16)))
        }
        Instruction::NumDivI { dst, src, imm } => {
            let imm_i8 = i8::try_from(*imm).ok()?;
            Some(encode(OP_NUM_DIV_RI, *dst, ((imm_i8 as u8 as u16) << 8) | (*src as u16)))
        }

        Instruction::IntLtRRJumpPacked(..) | Instruction::IntLeRRJumpPacked(..) => {
            None // already packed by the variants above
        }
        // --- Complex instructions → unpacked ---
        Instruction::EnumDecl(..)
        | Instruction::DeclStore { .. }
        | Instruction::Remember { .. } | Instruction::Recall { .. } | Instruction::Forget { .. }
        | Instruction::ClassDecl(..) | Instruction::TraitCheck(..)
        | Instruction::LoadModule(..) | Instruction::DefineFunction(..)
        | Instruction::ClassStaticDecl(..)
        | Instruction::DestructObject(..)
        | Instruction::WriteBackReceiver(..)
        | Instruction::TailCall { .. }
        | Instruction::ForIn { .. }
        | Instruction::IterNext { .. }
        | Instruction::MethodCall { .. }
        | Instruction::SuperCall { .. }
        // Register ops with complex payloads → unpacked
        | Instruction::IntDiv { .. }
        | Instruction::NumMod { .. }
        | Instruction::LoadConst { .. }
        | Instruction::LoadNumConst { .. }
        | Instruction::IntLeRRJumpIfFalse { .. }
        | Instruction::IntLtRRJumpIfFalse { .. }
        | Instruction::IntAddReturn { .. }
        | Instruction::IntSubReturn { .. }
        | Instruction::MakeArray { .. }
        | Instruction::MakeObject { .. }
        | Instruction::Call { .. }
        | Instruction::LoadGlobal { .. }
        | Instruction::StoreGlobal { .. }
        | Instruction::StoreGlobalConst { .. }
        | Instruction::DeclGlobal { .. }
        | Instruction::StoreConst { .. }
        | Instruction::SetProperty { .. }
        | Instruction::GetProperty { .. }
        | Instruction::StringIndexOf { .. }
        | Instruction::StringContains { .. }
        | Instruction::IntDivI { .. }
        | Instruction::IntModI { .. }
        | Instruction::IntModCmpI { .. }
        | Instruction::IntCmpIJumpIfFalse { .. }
        | Instruction::IntCmpIJumpIfTrue { .. }
        | Instruction::IntCmpRRJumpIfFalse { .. }
        | Instruction::IntAddIJump { .. }
        | Instruction::LoopEndIntAddIJump { .. }
        | Instruction::IntSubIJump { .. }
        | Instruction::ReturnConst { .. }
        | Instruction::IntMulReturn { .. }
        | Instruction::IntDivReturn { .. }
        | Instruction::IntCmpIReturn { .. }
        | Instruction::ArrayPushIntConst { .. } => None,
        // P1b: specialized index packed opcodes — same RRR layout as Index
        Instruction::IndexArray { dst, obj, idx }
        | Instruction::IndexStringAscii { dst, obj, idx } => {
            #[allow(unused_assignments)]
            let mut op = OP_INDEX_ARRAY_RRR;
            if let Instruction::IndexStringAscii { .. } = instr { op = OP_INDEX_STRING_ASCII_RRR; }
            Some(encode(op, *dst, ((*obj as u16) << 8) | (*idx as u16)))
        }
        | Instruction::ArrayPushConst { .. }
        // P2: not packable yet — single-cycle ops, packed overhead not justified
        // P8: MakeArray2 not packable (3 registers)
        | Instruction::MakeArray2 { .. }
        | Instruction::ArrayLen { .. }
        | Instruction::StringLen { .. }
        | Instruction::ArrayPop { .. }
        // P5: NumSqrt not packable (uses f64 math, complex)
        | Instruction::NumSqrt { .. }
        | Instruction::Index2D { .. }
        | Instruction::IndexAssign2D { .. }
        | Instruction::IndexAssignArray { .. }
        | Instruction::IntMulAddAssign { .. }
        | Instruction::PropertySubAssign { .. }
        | Instruction::StrCat3 { .. }
        | Instruction::FloatMulAdd { .. }
        | Instruction::FloatAdd { .. }
        | Instruction::FloatMul { .. }
        | Instruction::IntMulMod { .. }
        | Instruction::IntMulModI { .. } => None,
    }
}
