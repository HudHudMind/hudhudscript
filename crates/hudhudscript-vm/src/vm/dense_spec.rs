//! Canonical dense opcode order — single source of truth.
//! DENSE_MAP, DENSE_COUNT, and D_* constants derive from this list.

use crate::vm::packed_ops::*;

pub(crate) const DENSE_OPCODE_ORDER: &[u8] = &[
    OP_INT_EQ_RR,
    OP_INT_LT_RR,
    OP_INT_LE_RR,
    OP_INT_NE_RR,
    OP_MOVE_RR,
    OP_JUMP_IF_FALSE_R,
    OP_JUMP_IF_TRUE_R,
    OP_INDEX_RRR,
    OP_STRCAT_RRR,
    OP_INT_ADD_RI,
    OP_INT_SUB_RI,
    OP_NEG_R,
    OP_NOT_R,
    OP_ARRAY_PUSH_RRR,
    OP_STRING_INDEX_OF_RRR,
    OP_STRING_CONTAINS_RRR,
    OP_STRCAT_MUT_RR,
    OP_RETURN,
    OP_JUMP,
    OP_JUMP_IF_FALSE,
    OP_JUMP_IF_TRUE,
    OP_INDEX,
    OP_BREAK,
    OP_CONTINUE,
    OP_POP_TRY,
    OP_THROW,
    OP_SEND,
    OP_REQUIRE,
    OP_PERFORM,
    OP_AWAIT,
    OP_POP_LOOP,
    OP_YIELD,
    OP_ARRAY_PUSH,
    OP_SPREAD_INTO_ARRAY,
    OP_SPREAD_INTO_OBJECT,
    OP_POP_FINALLY,
    OP_LOAD_NUM_CONST,
    OP_MAKE_ARRAY,
    OP_MAKE_OBJECT,
    OP_LOAD_INT_CONST,
    OP_GET_PROPERTY,
    OP_SET_PROPERTY,
    OP_BIND_VAR,
    OP_FOR_IN,
    OP_RECEIVE,
    OP_ITER_NEXT,
    OP_PUSH_TRY,
    OP_PUSH_FINALLY,
    OP_END_FINALLY,
    OP_CALL,
    OP_METHOD_CALL,
    OP_NEW_INSTANCE,
    OP_SPAWN,
    OP_SUPER_CALL,
    OP_MAKE_GENERATOR,
    OP_CALL_SPREAD,
    OP_METHOD_CALL_SPREAD,
    OP_PUSH_LOOP,
    OP_MATCH_VARIANT,
    OP_GET_STATIC,
    OP_DESTRUCT_ARRAY,
    OP_INT_LE_JUMP_IF_FALSE,
    OP_INT_SUB_LOCAL_I,
    OP_INT_LT_JUMP_IF_FALSE,
    OP_INT_ADD_RR,
    OP_INT_SUB_RR,
    OP_INT_MUL_RR,
    OP_LOAD_INT_CONST_R,
    OP_RETURN_R,
    OP_STR_CAT,
    OP_INT_ADD_LOCAL_I,
    OP_INT_MOD_RR,
    OP_NUM_ADD_RR,
    OP_NUM_SUB_RR,
    OP_NUM_MUL_RR,
    OP_NUM_DIV_RR,
    OP_INDEX_ASSIGN_RRR,
    OP_NUM_ADD_RI,
    OP_NUM_SUB_RI,
    OP_NUM_MUL_RI,
    OP_NUM_DIV_RI,
    OP_STR_REV_R,
    OP_NUM_MUL_ADD_ASSIGN,
    OP_INT_MUL_RI,
    OP_INT_MOD_I,
    OP_INT_CMP_LT_I,
    OP_INT_CMP_LE_I,
    OP_INT_CMP_EQ_I,
    OP_INT_CMP_NE_I,
    OP_NUM_MUL_ADD_INDEXED,
    OP_STR_CHAR_EQ_RR,
    OP_INT_LT_RR_JUMP_P,
    OP_INT_LE_RR_JUMP_P,
    // P1b: specialized index packed opcodes
    OP_INDEX_ARRAY_RRR,
    OP_INDEX_STRING_ASCII_RRR,
    OP_INT_CMP_RR_JUMP_P,
    OP_F_LOAD_NUM,
    OP_F_STORE_NUM,
    OP_F_ADD,
    OP_F_SUB,
    OP_F_MUL,
    OP_F_DIV,
    OP_F_SIN,
    OP_F_COS,
    OP_F_SQRT,
    OP_F_CONST,
    OP_F_MOVE,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::dispatch_table::dense_index;

    #[test]
    fn dense_map_has_index_array_rrr() {
        let idx = dense_index(OP_INDEX_ARRAY_RRR);
        assert_ne!(idx, 0xFF, "OP_INDEX_ARRAY_RRR must be in DENSE_MAP");
    }

    #[test]
    fn dense_map_has_index_string_ascii_rrr() {
        let idx = dense_index(OP_INDEX_STRING_ASCII_RRR);
        assert_ne!(idx, 0xFF, "OP_INDEX_STRING_ASCII_RRR must be in DENSE_MAP");
    }
}
