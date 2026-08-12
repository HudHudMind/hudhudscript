// Must stay in sync with hudhudscript_bytecode::packed_instruction::opcodes

pub const OP_RETURN: u8 = 19;
pub const OP_JUMP: u8 = 20;
pub const OP_JUMP_IF_FALSE: u8 = 21;
pub const OP_JUMP_IF_TRUE: u8 = 22;
pub const OP_INDEX: u8 = 29;
pub const OP_BREAK: u8 = 30;
pub const OP_CONTINUE: u8 = 31;
pub const OP_POP_TRY: u8 = 32;
pub const OP_THROW: u8 = 33;
pub const OP_SEND: u8 = 34;
pub const OP_REQUIRE: u8 = 35;
pub const OP_PERFORM: u8 = 36;
pub const OP_AWAIT: u8 = 37;
pub const OP_POP_LOOP: u8 = 38;
pub const OP_YIELD: u8 = 39;
pub const OP_ARRAY_PUSH: u8 = 40;
pub const OP_SPREAD_INTO_ARRAY: u8 = 41;
pub const OP_SPREAD_INTO_OBJECT: u8 = 42;
pub const OP_POP_FINALLY: u8 = 43;
pub const OP_LOAD_NUM_CONST: u8 = 69;
pub const OP_MAKE_ARRAY: u8 = 80;
pub const OP_MAKE_OBJECT: u8 = 81;
pub const OP_LOAD_INT_CONST: u8 = 88;
pub const OP_GET_PROPERTY: u8 = 89;
pub const OP_SET_PROPERTY: u8 = 90;
pub const OP_BIND_VAR: u8 = 91;
pub const OP_FOR_IN: u8 = 92;
pub const OP_RECEIVE: u8 = 94;
pub const OP_ITER_NEXT: u8 = 95;
pub const OP_PUSH_TRY: u8 = 96;
pub const OP_PUSH_FINALLY: u8 = 97;
pub const OP_END_FINALLY: u8 = 98;
pub const OP_CALL: u8 = 99;
pub const OP_METHOD_CALL: u8 = 100;
pub const OP_NEW_INSTANCE: u8 = 101;
pub const OP_SPAWN: u8 = 102;
pub const OP_SUPER_CALL: u8 = 103;
pub const OP_MAKE_GENERATOR: u8 = 104;
pub const OP_CALL_SPREAD: u8 = 105;
pub const OP_METHOD_CALL_SPREAD: u8 = 106;
pub const OP_PUSH_LOOP: u8 = 107;
pub const OP_MATCH_VARIANT: u8 = 108;
pub const OP_GET_STATIC: u8 = 109;
pub const OP_DESTRUCT_ARRAY: u8 = 110;
pub const OP_INT_SUB_CALL_1: u8 = 111;
pub const OP_INT_LE_JUMP_IF_FALSE: u8 = 112;
pub const OP_INT_LT_JUMP_IF_FALSE: u8 = 117;
pub const OP_INT_ADD_CALL_1: u8 = 113;
pub const OP_INT_SUB_LOCAL_I: u8 = 114;
pub const OP_INT_ADD_LOCAL_I: u8 = 127;
pub const OP_INT_EQ_RR: u8 = 0;
pub const OP_INT_LT_RR: u8 = 1;
pub const OP_INT_LE_RR: u8 = 2;
pub const OP_INT_NE_RR: u8 = 3;
pub const OP_MOVE_RR: u8 = 4;
pub const OP_JUMP_IF_FALSE_R: u8 = 7;
pub const OP_JUMP_IF_TRUE_R: u8 = 8;
pub const OP_INDEX_RRR: u8 = 9;
pub const OP_STRCAT_RRR: u8 = 10;
pub const OP_INT_ADD_RI: u8 = 11;
pub const OP_INT_SUB_RI: u8 = 12;
pub const OP_NEG_R: u8 = 13;
pub const OP_NOT_R: u8 = 14;
pub const OP_ARRAY_PUSH_RRR: u8 = 15;
pub const OP_STRING_INDEX_OF_RRR: u8 = 16;
pub const OP_STRING_CONTAINS_RRR: u8 = 17;
pub const OP_STRCAT_MUT_RR: u8 = 18;

pub const OP_INT_ADD_RR: u8 = 118;
pub const OP_INT_SUB_RR: u8 = 119;
pub const OP_INT_MUL_RR: u8 = 120;
pub const OP_INT_MOD_RR: u8 = 128;
pub const OP_LOAD_INT_CONST_R: u8 = 121;
pub const OP_PUSH_REG: u8 = 124;
pub const OP_RETURN_R: u8 = 125;
pub const OP_STR_CAT: u8 = 126;

// DENSE_OPS
pub const OP_NUM_ADD_RR: u8 = 129;
pub const OP_NUM_SUB_RR: u8 = 130;
pub const OP_NUM_MUL_RR: u8 = 131;
pub const OP_NUM_DIV_RR: u8 = 132;
pub const OP_INDEX_ASSIGN_RRR: u8 = 133;
pub const OP_NUM_ADD_RI: u8 = 134;
pub const OP_NUM_SUB_RI: u8 = 135;
pub const OP_NUM_MUL_RI: u8 = 136;
pub const OP_NUM_DIV_RI: u8 = 137;
pub const OP_STR_REV_R: u8 = 138;
pub const OP_NUM_MUL_ADD_ASSIGN: u8 = 139;
pub const OP_INT_MUL_RI: u8 = 122;
pub const OP_INT_MOD_I: u8 = 140;
pub const OP_INT_CMP_LT_I: u8 = 141;
pub const OP_INT_CMP_LE_I: u8 = 142;
pub const OP_INT_CMP_EQ_I: u8 = 143;
pub const OP_INT_CMP_NE_I: u8 = 144;
pub const OP_NUM_MUL_ADD_INDEXED: u8 = 145;
pub const OP_STR_CHAR_EQ_RR: u8 = 146;
pub const OP_INT_LT_RR_JUMP_P: u8 = 147;
pub const OP_INT_LE_RR_JUMP_P: u8 = 148;

// P1b: specialized index packed opcodes — must match bytecode opcodes.rs
pub const OP_INDEX_ARRAY_RRR: u8 = 149;
pub const OP_INDEX_STRING_ASCII_RRR: u8 = 150;
// G4: genel cmp+branch (op arg1'de, CmpJumpPayload indeksi arg2'de).
pub const OP_INT_CMP_RR_JUMP_P: u8 = 151;
// G12: unboxed float ailesi.
pub const OP_F_LOAD_NUM: u8 = 154;
pub const OP_F_STORE_NUM: u8 = 155;
pub const OP_F_ADD: u8 = 156;
pub const OP_F_SUB: u8 = 157;
pub const OP_F_MUL: u8 = 158;
pub const OP_F_DIV: u8 = 159;
pub const OP_F_SIN: u8 = 160;
pub const OP_F_COS: u8 = 161;
pub const OP_F_SQRT: u8 = 162;
pub const OP_F_CONST: u8 = 163;
pub const OP_F_MOVE: u8 = 164;

/// G2: Map dense opcode to static name for telemetry.
/// G2.3 (Kural 7): isim kaynagi TEK — dense_ops macro'sundan turetilir
/// (const adindan stringify!). Elle tutulan match tablosu silindi;
/// eksik isim = "unknown" drift'i bir daha olusamaz.
#[cfg(feature = "telemetry")]
pub fn dense_name(dense: u8) -> &'static str {
    crate::vm::dense_ops::dense_const_name(dense)
}
