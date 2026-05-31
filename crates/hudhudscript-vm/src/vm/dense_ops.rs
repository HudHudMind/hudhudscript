//! C1: Dense opcode constants for jump-table dispatch.
//!
//! These constants MUST stay in sync with `dispatch_table::DENSE_MAP`.
//! Each constant maps to the dense (contiguous 0..84) index of the
//! corresponding packed opcode.
//!
//! Generated from DENSE_MAP via:
//!   DENSE_MAP[OP_ORIGINAL as usize]

// register comparison / move
pub(crate) const D_INT_EQ_RR: u8 = 0;
pub(crate) const D_INT_LT_RR: u8 = 1;
pub(crate) const D_INT_LE_RR: u8 = 2;
pub(crate) const D_INT_NE_RR: u8 = 3;
pub(crate) const D_MOVE_RR: u8 = 4;

// control / arithmetic (packed register)
pub(crate) const D_JUMP_IF_FALSE_R: u8 = 5;
pub(crate) const D_JUMP_IF_TRUE_R: u8 = 6;
pub(crate) const D_INDEX_RRR: u8 = 7;
pub(crate) const D_STRCAT_RRR: u8 = 8;
pub(crate) const D_INT_ADD_RI: u8 = 9;
pub(crate) const D_INT_SUB_RI: u8 = 10;
pub(crate) const D_NEG_R: u8 = 11;
pub(crate) const D_NOT_R: u8 = 12;
pub(crate) const D_ARRAY_PUSH_RRR: u8 = 13;
pub(crate) const D_STRING_INDEX_OF_RRR: u8 = 14;
pub(crate) const D_STRING_CONTAINS_RRR: u8 = 15;
pub(crate) const D_STRCAT_MUT_RR: u8 = 16;

// return / jump / load / store / declare
pub(crate) const D_RETURN: u8 = 17;
pub(crate) const D_JUMP: u8 = 18;
pub(crate) const D_JUMP_IF_FALSE: u8 = 19;
pub(crate) const D_JUMP_IF_TRUE: u8 = 20;
pub(crate) const D_LOAD_VAR: u8 = 21;
pub(crate) const D_STORE_VAR: u8 = 22;
pub(crate) const D_DECL_VAR: u8 = 23;
pub(crate) const D_STORE_CONST: u8 = 24;

// structural / control
pub(crate) const D_INDEX: u8 = 25;
pub(crate) const D_BREAK: u8 = 26;
pub(crate) const D_CONTINUE: u8 = 27;
pub(crate) const D_POP_TRY: u8 = 28;
pub(crate) const D_THROW: u8 = 29;
pub(crate) const D_SEND: u8 = 30;
pub(crate) const D_REQUIRE: u8 = 31;
pub(crate) const D_PERFORM: u8 = 32;
pub(crate) const D_AWAIT: u8 = 33;
pub(crate) const D_POP_LOOP: u8 = 34;
pub(crate) const D_YIELD: u8 = 35;
pub(crate) const D_ARRAY_PUSH: u8 = 36;
pub(crate) const D_SPREAD_INTO_ARRAY: u8 = 37;
pub(crate) const D_SPREAD_INTO_OBJECT: u8 = 38;
pub(crate) const D_POP_FINALLY: u8 = 39;

// constants
pub(crate) const D_LOAD_NUM_CONST: u8 = 40;

// local slots
pub(crate) const D_LOAD_LOCAL: u8 = 41;
pub(crate) const D_STORE_LOCAL: u8 = 42;
pub(crate) const D_DECL_LOCAL: u8 = 43;

// array/object
pub(crate) const D_MAKE_ARRAY: u8 = 44;
pub(crate) const D_MAKE_OBJECT: u8 = 45;

// property / structural (88-110 range)
pub(crate) const D_LOAD_INT_CONST: u8 = 46;
pub(crate) const D_GET_PROPERTY: u8 = 47;
pub(crate) const D_SET_PROPERTY: u8 = 48;
pub(crate) const D_BIND_VAR: u8 = 49;
pub(crate) const D_FOR_IN: u8 = 50;
pub(crate) const D_RECEIVE: u8 = 51;
pub(crate) const D_ITER_NEXT: u8 = 52;
pub(crate) const D_PUSH_TRY: u8 = 53;
pub(crate) const D_PUSH_FINALLY: u8 = 54;
pub(crate) const D_END_FINALLY: u8 = 55;
pub(crate) const D_CALL: u8 = 56;
pub(crate) const D_METHOD_CALL: u8 = 57;
pub(crate) const D_NEW_INSTANCE: u8 = 58;
pub(crate) const D_SPAWN: u8 = 59;
pub(crate) const D_SUPER_CALL: u8 = 60;
pub(crate) const D_MAKE_GENERATOR: u8 = 61;
pub(crate) const D_CALL_SPREAD: u8 = 62;
pub(crate) const D_METHOD_CALL_SPREAD: u8 = 63;
pub(crate) const D_PUSH_LOOP: u8 = 64;
pub(crate) const D_MATCH_VARIANT: u8 = 65;
pub(crate) const D_GET_STATIC: u8 = 66;
pub(crate) const D_DESTRUCT_ARRAY: u8 = 67;

// super-instructions (111-117)
pub(crate) const D_INT_SUB_CALL_1: u8 = 68;
pub(crate) const D_INT_LE_JUMP_IF_FALSE: u8 = 69;
pub(crate) const D_INT_ADD_CALL_1: u8 = 70;
pub(crate) const D_INT_SUB_LOCAL_I: u8 = 71;
pub(crate) const D_INDEX_FAST_ISLOT: u8 = 72;
pub(crate) const D_INT_INCR_SLOT: u8 = 73;
pub(crate) const D_INT_LT_JUMP_IF_FALSE: u8 = 74;

// register math (118-127)
pub(crate) const D_INT_ADD_RR: u8 = 75;
pub(crate) const D_INT_SUB_RR: u8 = 76;
pub(crate) const D_INT_MUL_RR: u8 = 77;
pub(crate) const D_LOAD_INT_CONST_R: u8 = 78;
pub(crate) const D_LOAD_LOCAL_R: u8 = 79;
pub(crate) const D_STORE_LOCAL_R: u8 = 80;
pub(crate) const D_PUSH_REG: u8 = 81;
pub(crate) const D_RETURN_R: u8 = 82;
pub(crate) const D_STR_CAT: u8 = 83;
pub(crate) const D_INT_ADD_LOCAL_I: u8 = 84;
pub(crate) const D_INT_MOD_RR: u8 = 85;
pub(crate) const D_INT_MUL_RI: u8 = 95;

// float register math (129-132)
pub(crate) const D_NUM_ADD_RR: u8 = 86;
pub(crate) const D_NUM_SUB_RR: u8 = 87;
pub(crate) const D_NUM_MUL_RR: u8 = 88;
pub(crate) const D_NUM_DIV_RR: u8 = 89;
pub(crate) const D_INDEX_ASSIGN_RRR: u8 = 90;

// float immediate arithmetic (134-137)
pub(crate) const D_NUM_ADD_RI: u8 = 91;
pub(crate) const D_NUM_SUB_RI: u8 = 92;
pub(crate) const D_NUM_MUL_RI: u8 = 93;
pub(crate) const D_NUM_DIV_RI: u8 = 94;
