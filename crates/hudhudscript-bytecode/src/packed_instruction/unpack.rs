use super::opcodes::*;
use super::{decode, Instruction, SymId};

pub fn unpack(packed: u32) -> Option<Instruction> {
    let (opcode, arg1, arg2) = decode(packed);
    match opcode {
        // zero-arg
        OP_STR_CAT => None, // Back compat: use StrCat
        OP_BREAK => Some(Instruction::Break),
        OP_CONTINUE => Some(Instruction::Continue),
        OP_POP_TRY => Some(Instruction::TryEnd),
        OP_THROW => Some(Instruction::Throw { src: 255 }),
        OP_SEND => None, // Back compat: use Send with register operands
        OP_REQUIRE => Some(Instruction::Require { src: 255 }),
        OP_PERFORM => Some(Instruction::Perform { src: 255 }),
        OP_AWAIT => Some(Instruction::Await { src: 255, dst: 255 }),
        OP_YIELD => Some(Instruction::Yield { src: 255 }),
        OP_ARRAY_PUSH => None,        // register-based ArrayPush, not packable
        OP_SPREAD_INTO_ARRAY => None, // Back compat: use SpreadIntoArray registers
        OP_SPREAD_INTO_OBJECT => None, // Back compat: use SpreadIntoObject registers
        OP_POP_FINALLY => Some(Instruction::FinallyEnd),

        // arg2 = u32 index
        OP_LOAD_NUM_CONST => None, // Back compat: use LoadNumConst
        OP_LOAD_INT_CONST => None, // Back compat: use LoadIntConst
        OP_JUMP => Some(Instruction::Jump((arg2 as i16) as i32)),
        OP_JUMP_IF_FALSE => Some(Instruction::JumpIfFalse {
            src: 255,
            offset: arg2 as i16,
        }),
        OP_JUMP_IF_TRUE => Some(Instruction::JumpIfTrue {
            src: 255,
            offset: arg2 as i16,
        }),
        OP_JUMP_IF_FALSE_R => Some(Instruction::JumpIfFalse {
            src: arg1,
            offset: arg2 as i16,
        }),
        OP_JUMP_IF_TRUE_R => Some(Instruction::JumpIfTrue {
            src: arg1,
            offset: arg2 as i16,
        }),

        // slot+immediate ops removed — only register equivalents remain
        // (kept for backward compat; decode to nothing)

        // super-instructions
        OP_INT_SUB_CALL_1 => Some(Instruction::IntSubCall1(arg2 as u32)),
        OP_INT_ADD_CALL_1 => Some(Instruction::IntAddCall1(arg2 as u32)),
        OP_INT_LE_JUMP_IF_FALSE => Some(Instruction::IntLeJumpIfFalse(arg2 as u32)),
        OP_INT_LT_JUMP_IF_FALSE => Some(Instruction::IntLtJumpIfFalse(arg2 as u32)),

        // arg2 = u16 element count

        // arg2 = SymId
        OP_GET_PROPERTY => None, // register-based GetProperty
        OP_SET_PROPERTY => None, // register-based SetProperty
        OP_BIND_VAR => Some(Instruction::BindVar(SymId(arg2 as u32))),
        OP_FOR_IN => Some(Instruction::ForIn {
            iter_reg: 255,
            var_sym_idx: arg2 as u16,
            end_offset: 0,
        }),
        OP_RECEIVE => Some(Instruction::Receive {
            var_sym_idx: arg2 as u16,
            src: 255,
        }),

        // arg2 = i16 relative offset
        OP_ITER_NEXT => Some(Instruction::IterNext {
            iter_reg: 255,
            var_sym_idx: 0,
            end_offset: arg2 as i16,
        }),
        OP_PUSH_TRY => Some(Instruction::TryBegin((arg2 as i16) as i32)),
        OP_PUSH_FINALLY => Some(Instruction::FinallyBegin((arg2 as i16) as i32)),
        OP_END_FINALLY => Some(Instruction::FinallyExit((arg2 as i16) as i32)),

        // call-family
        OP_METHOD_CALL => Some(Instruction::MethodCall {
            dst: 255,
            obj: 255,
            payload_idx: arg2 as u16,
            first_arg: 0,
            arg_count: 0,
        }),
        OP_NEW_INSTANCE => None, // Back compat: register-based NewInstance
        OP_SPAWN => None,        // Back compat: use Spawn with register operands
        OP_SUPER_CALL => Some(Instruction::SuperCall {
            dst: 255,
            payload_idx: arg2 as u16,
            first_arg: 0,
            arg_count: 0,
        }),
        OP_MAKE_GENERATOR => None, // Back compat: register-based MakeGenerator
        OP_CALL_SPREAD => Some(Instruction::CallSpread(SymId(arg2 as u32))),
        OP_METHOD_CALL_SPREAD => None, // Back compat: register-based MethodCallSpread
        OP_PUSH_LOOP => Some(Instruction::LoopBegin(arg2 as u32)),
        OP_MATCH_VARIANT => Some(Instruction::MatchVariant(arg2 as u32)),
        OP_GET_STATIC => Some(Instruction::GetStatic(arg2 as u32)),
        OP_DESTRUCT_ARRAY => Some(Instruction::DestructArray(arg2, arg1 != 0)),

        // IndexFast, IntIncrSlot removed — dead opcodes
        // Register-based VM opcodes
        OP_INT_EQ_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::IntCmp {
                dst: arg1,
                src1,
                src2,
                op: 4,
            })
        }
        OP_INT_LT_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::IntCmp {
                dst: arg1,
                src1,
                src2,
                op: 0,
            })
        }
        OP_INT_LE_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::IntCmp {
                dst: arg1,
                src1,
                src2,
                op: 1,
            })
        }
        OP_INT_NE_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::IntCmp {
                dst: arg1,
                src1,
                src2,
                op: 5,
            })
        }
        OP_MOVE_RR => Some(Instruction::Move {
            dst: arg1,
            src: arg2 as u8,
        }),
        OP_INT_ADD_RI => {
            let src = (arg2 & 0xFF) as u8;
            let imm = ((arg2 >> 8) as i8) as i16;
            Some(Instruction::IntAddI {
                dst: arg1,
                src,
                imm,
            })
        }
        OP_INT_SUB_RI => {
            let src = (arg2 & 0xFF) as u8;
            let imm = ((arg2 >> 8) as i8) as i16;
            Some(Instruction::IntSubI {
                dst: arg1,
                src,
                imm,
            })
        }
        OP_INT_MUL_RI => {
            let dst = arg1;
            let src = (arg2 & 0xFF) as u8;
            let imm = ((arg2 >> 8) & 0xFF) as i8 as i16;
            Some(Instruction::IntMulI { dst, src, imm })
        }
        OP_NEG_R => Some(Instruction::Neg {
            dst: arg1,
            src: arg2 as u8,
        }),
        OP_NOT_R => Some(Instruction::Not {
            dst: arg1,
            src: arg2 as u8,
        }),
        OP_ARRAY_PUSH_RRR => {
            let arr = ((arg2 >> 8) & 0xFF) as u8;
            let val = (arg2 & 0xFF) as u8;
            Some(Instruction::ArrayPush {
                dst: arg1,
                arr,
                val,
            })
        }
        OP_STRING_CONCAT_RR => {
            let regs_start = ((arg2 >> 8) & 0xFF) as u8;
            let count = (arg2 & 0xFF) as u8;
            Some(Instruction::StringConcat {
                regs_start,
                count,
                dst: arg1,
            })
        }
        OP_INDEX_RRR => {
            let obj = ((arg2 >> 8) & 0xFF) as u8;
            let idx = (arg2 & 0xFF) as u8;
            Some(Instruction::Index {
                dst: arg1,
                obj,
                idx,
            })
        }
        OP_STRCAT_RRR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::StrCat {
                dst: arg1,
                src1,
                src2,
            })
        }
        OP_STRCAT_MUT_RR => Some(Instruction::StrCatMut {
            dst: arg1,
            src2: arg2 as u8,
        }),
        OP_STRING_INDEX_OF_RRR => {
            let haystack = ((arg2 >> 8) & 0xFF) as u8;
            let needle = (arg2 & 0xFF) as u8;
            Some(Instruction::StringIndexOf {
                dst: arg1,
                haystack,
                needle,
            })
        }
        OP_STRING_CONTAINS_RRR => {
            let haystack = ((arg2 >> 8) & 0xFF) as u8;
            let needle = (arg2 & 0xFF) as u8;
            Some(Instruction::StringContains {
                dst: arg1,
                haystack,
                needle,
            })
        }
        OP_INT_ADD_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::IntAdd {
                dst: arg1,
                src1,
                src2,
            })
        }
        OP_INT_SUB_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::IntSub {
                dst: arg1,
                src1,
                src2,
            })
        }
        OP_INT_MUL_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::IntMul {
                dst: arg1,
                src1,
                src2,
            })
        }
        OP_INT_MOD_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::IntMod {
                dst: arg1,
                src1,
                src2,
            })
        }
        OP_LOAD_INT_CONST_R => Some(Instruction::LoadIntConst {
            dst: arg1,
            const_idx: arg2,
        }),

        OP_RETURN_R => Some(Instruction::Return { src: arg1 }),

        // Float register arithmetic (129-132)
        OP_NUM_ADD_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::NumAdd {
                dst: arg1,
                src1,
                src2,
            })
        }
        OP_NUM_SUB_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::NumSub {
                dst: arg1,
                src1,
                src2,
            })
        }
        OP_NUM_MUL_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::NumMul {
                dst: arg1,
                src1,
                src2,
            })
        }
        OP_NUM_DIV_RR => {
            let src1 = ((arg2 >> 8) & 0xFF) as u8;
            let src2 = (arg2 & 0xFF) as u8;
            Some(Instruction::NumDiv {
                dst: arg1,
                src1,
                src2,
            })
        }
        OP_NUM_MUL_ADD_ASSIGN => {
            let mul = ((arg2 >> 8) & 0xFF) as u8;
            let add = (arg2 & 0xFF) as u8;
            Some(Instruction::NumMulAddAssign {
                dst: arg1,
                mul,
                add,
            })
        }
        // IndexAssign (133)
        OP_INDEX_ASSIGN_RRR => {
            let idx = ((arg2 >> 8) & 0xFF) as u8;
            let val = (arg2 & 0xFF) as u8;
            Some(Instruction::IndexAssign {
                obj: arg1,
                idx,
                val,
            })
        }
        // Float immediate arithmetic (134-137)
        OP_NUM_ADD_RI => {
            let src = (arg2 & 0xFF) as u8;
            let imm = ((arg2 >> 8) as i8) as i16;
            Some(Instruction::NumAddI {
                dst: arg1,
                src,
                imm,
            })
        }
        OP_NUM_SUB_RI => {
            let src = (arg2 & 0xFF) as u8;
            let imm = ((arg2 >> 8) as i8) as i16;
            Some(Instruction::NumSubI {
                dst: arg1,
                src,
                imm,
            })
        }
        OP_NUM_MUL_RI => {
            let src = (arg2 & 0xFF) as u8;
            let imm = ((arg2 >> 8) as i8) as i16;
            Some(Instruction::NumMulI {
                dst: arg1,
                src,
                imm,
            })
        }
        OP_NUM_DIV_RI => {
            let src = (arg2 & 0xFF) as u8;
            let imm = ((arg2 >> 8) as i8) as i16;
            Some(Instruction::NumDivI {
                dst: arg1,
                src,
                imm,
            })
        }

        // P1b: specialized index packed opcodes
        OP_INDEX_ARRAY_RRR => {
            let obj = ((arg2 >> 8) & 0xFF) as u8;
            let idx = (arg2 & 0xFF) as u8;
            Some(Instruction::IndexArray {
                dst: arg1,
                obj,
                idx,
            })
        }
        OP_INDEX_STRING_ASCII_RRR => {
            let obj = ((arg2 >> 8) & 0xFF) as u8;
            let idx = (arg2 & 0xFF) as u8;
            Some(Instruction::IndexStringAscii {
                dst: arg1,
                obj,
                idx,
            })
        }

        _ => None,
    }
}
