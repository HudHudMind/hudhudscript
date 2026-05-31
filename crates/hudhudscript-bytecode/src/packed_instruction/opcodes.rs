// Opcode constants for packed instruction encoding.
// Must stay in sync with hudhudscript_vm::vm::packed_ops.

pub(crate) const OP_RETURN: u8 = 19;
pub(crate) const OP_JUMP: u8 = 20;
pub(crate) const OP_JUMP_IF_FALSE: u8 = 21;
pub(crate) const OP_JUMP_IF_TRUE: u8 = 22;
pub(crate) const OP_INDEX: u8 = 29;
pub(crate) const OP_BREAK: u8 = 30;
pub(crate) const OP_CONTINUE: u8 = 31;
pub(crate) const OP_POP_TRY: u8 = 32;
pub(crate) const OP_THROW: u8 = 33;
pub(crate) const OP_SEND: u8 = 34;
pub(crate) const OP_REQUIRE: u8 = 35;
pub(crate) const OP_PERFORM: u8 = 36;
pub(crate) const OP_AWAIT: u8 = 37;
pub(crate) const OP_POP_LOOP: u8 = 38;
pub(crate) const OP_YIELD: u8 = 39;
pub(crate) const OP_ARRAY_PUSH: u8 = 40;
pub(crate) const OP_SPREAD_INTO_ARRAY: u8 = 41;
pub(crate) const OP_SPREAD_INTO_OBJECT: u8 = 42;
pub(crate) const OP_POP_FINALLY: u8 = 43;
pub(crate) const OP_LOAD_NUM_CONST: u8 = 69;

pub(crate) const OP_MAKE_ARRAY: u8 = 80;
pub(crate) const OP_MAKE_OBJECT: u8 = 81;
pub(crate) const OP_LOAD_INT_CONST: u8 = 88;
pub(crate) const OP_GET_PROPERTY: u8 = 89;
pub(crate) const OP_SET_PROPERTY: u8 = 90;
pub(crate) const OP_BIND_VAR: u8 = 91;
pub(crate) const OP_FOR_IN: u8 = 92;
pub(crate) const OP_RECEIVE: u8 = 94;
pub(crate) const OP_ITER_NEXT: u8 = 95;
pub(crate) const OP_PUSH_TRY: u8 = 96;
pub(crate) const OP_PUSH_FINALLY: u8 = 97;
pub(crate) const OP_END_FINALLY: u8 = 98;
pub(crate) const OP_CALL: u8 = 99;
pub(crate) const OP_METHOD_CALL: u8 = 100;
pub(crate) const OP_NEW_INSTANCE: u8 = 101;
pub(crate) const OP_SPAWN: u8 = 102;
pub(crate) const OP_SUPER_CALL: u8 = 103;
pub(crate) const OP_MAKE_GENERATOR: u8 = 104;
pub(crate) const OP_CALL_SPREAD: u8 = 105;
pub(crate) const OP_METHOD_CALL_SPREAD: u8 = 106;
pub(crate) const OP_PUSH_LOOP: u8 = 107;
pub(crate) const OP_MATCH_VARIANT: u8 = 108;
pub(crate) const OP_GET_STATIC: u8 = 109;
pub(crate) const OP_DESTRUCT_ARRAY: u8 = 110;
pub(crate) const OP_INT_SUB_CALL_1: u8 = 111;
pub(crate) const OP_INT_LE_JUMP_IF_FALSE: u8 = 112;
pub(crate) const OP_INT_LT_JUMP_IF_FALSE: u8 = 117;
pub(crate) const OP_INT_ADD_CALL_1: u8 = 113;
pub(crate) const OP_INDEX_FAST_ISLOT: u8 = 115;
pub(crate) const OP_INT_INCR_SLOT: u8 = 116;
pub(crate) const OP_INT_EQ_RR: u8 = 0;
pub(crate) const OP_INT_LT_RR: u8 = 1;
pub(crate) const OP_INT_LE_RR: u8 = 2;
pub(crate) const OP_INT_NE_RR: u8 = 3;
pub(crate) const OP_MOVE_RR: u8 = 4;
pub(crate) const OP_JUMP_IF_FALSE_R: u8 = 7;
pub(crate) const OP_JUMP_IF_TRUE_R: u8 = 8;
pub(crate) const OP_INDEX_RRR: u8 = 9;
pub(crate) const OP_STRCAT_RRR: u8 = 10;
pub(crate) const OP_INT_ADD_RI: u8 = 11;
pub(crate) const OP_INT_SUB_RI: u8 = 12;
pub(crate) const OP_NEG_R: u8 = 13;
pub(crate) const OP_NOT_R: u8 = 14;
pub(crate) const OP_ARRAY_PUSH_RRR: u8 = 15;
pub(crate) const OP_STRING_INDEX_OF_RRR: u8 = 16;
pub(crate) const OP_STRING_CONTAINS_RRR: u8 = 17;
pub(crate) const OP_STRCAT_MUT_RR: u8 = 18;

pub(crate) const OP_INT_ADD_RR: u8 = 118;
pub(crate) const OP_INT_SUB_RR: u8 = 119;
pub(crate) const OP_INT_MUL_RR: u8 = 120;
pub(crate) const OP_INT_MOD_RR: u8 = 128;
pub(crate) const OP_LOAD_INT_CONST_R: u8 = 121;

pub(crate) const OP_PUSH_REG: u8 = 124;
pub(crate) const OP_RETURN_R: u8 = 125;
pub(crate) const OP_INT_SUB_LOCAL_I: u8 = 114;
pub(crate) const OP_INT_ADD_LOCAL_I: u8 = 127;

pub(crate) const OP_STR_CAT: u8 = 126;
pub(crate) const OP_INT_MUL_RI: u8 = 122;

pub(crate) const OP_NUM_ADD_RR: u8 = 129;
pub(crate) const OP_NUM_SUB_RR: u8 = 130;
pub(crate) const OP_NUM_MUL_RR: u8 = 131;
pub(crate) const OP_NUM_DIV_RR: u8 = 132;

pub(crate) const OP_INDEX_ASSIGN_RRR: u8 = 133;

pub(crate) const OP_NUM_ADD_RI: u8 = 134;
pub(crate) const OP_NUM_SUB_RI: u8 = 135;
pub(crate) const OP_NUM_MUL_RI: u8 = 136;
pub(crate) const OP_NUM_DIV_RI: u8 = 137;
