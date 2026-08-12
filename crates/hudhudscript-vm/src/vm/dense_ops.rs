//! Dense opcode constants — derived from `dispatch_table::dense_index()`.
//! No manual numbers. Add new constants with `dense_index(OP_XXX)` only.
//!
//! G2.3 (Kural 7 — tek kaynak): sabitler VE isimleri tek macro'dan üretilir.
//! `dense_const_name` isimleri const tanımından `stringify!` ile türetir;
//! yeni bir dense sabiti eklendiğinde adı otomatik gelir. Elle tutulan
//! ikinci bir isim tablosu YASAK ("unknown" drift'inin kök nedeni oydu).

use crate::vm::dispatch_table::dense_index;
use crate::vm::packed_ops::*;

macro_rules! dense_ops {
    ($( $d:ident = $op:ident ),* $(,)?) => {
        $( pub(crate) const $d: u8 = dense_index($op); )*

        /// HAM packed opcode (paketlenmiş u32'nin düşük baytı, OP_* değeri)
        /// → mnemonic ("D_" öneki atılmış const adı). Telemetri histogramları
        /// ham OP koduyla indekslenir (`step_one`: `p & 0xFF`); dense indeksle
        /// DEĞİL — G2.3'ün "unknown" kök nedeni bu iki uzayın karışmasıydı.
        /// Yalnız telemetri yazıcısı çağırır; sıcak yolda kullanılmaz.
        #[cfg(feature = "telemetry")]
        pub fn dense_const_name(raw_op: u8) -> &'static str {
            $( if raw_op == $op { return &stringify!($d)[2..]; } )*
            "unknown"
        }
    };
}

dense_ops! {
    D_INT_EQ_RR = OP_INT_EQ_RR,
    D_INT_LT_RR = OP_INT_LT_RR,
    D_INT_LE_RR = OP_INT_LE_RR,
    D_INT_NE_RR = OP_INT_NE_RR,
    D_MOVE_RR = OP_MOVE_RR,
    D_JUMP_IF_FALSE_R = OP_JUMP_IF_FALSE_R,
    D_JUMP_IF_TRUE_R = OP_JUMP_IF_TRUE_R,
    D_INDEX_RRR = OP_INDEX_RRR,
    D_STRCAT_RRR = OP_STRCAT_RRR,
    D_INT_ADD_RI = OP_INT_ADD_RI,
    D_INT_SUB_RI = OP_INT_SUB_RI,
    D_NEG_R = OP_NEG_R,
    D_NOT_R = OP_NOT_R,
    D_ARRAY_PUSH_RRR = OP_ARRAY_PUSH_RRR,
    D_STRING_INDEX_OF_RRR = OP_STRING_INDEX_OF_RRR,
    D_STRING_CONTAINS_RRR = OP_STRING_CONTAINS_RRR,
    D_STRCAT_MUT_RR = OP_STRCAT_MUT_RR,
    D_RETURN = OP_RETURN,
    D_JUMP = OP_JUMP,
    D_JUMP_IF_FALSE = OP_JUMP_IF_FALSE,
    D_JUMP_IF_TRUE = OP_JUMP_IF_TRUE,
    D_INDEX = OP_INDEX,
    D_BREAK = OP_BREAK,
    D_CONTINUE = OP_CONTINUE,
    D_POP_TRY = OP_POP_TRY,
    D_THROW = OP_THROW,
    D_SEND = OP_SEND,
    D_REQUIRE = OP_REQUIRE,
    D_PERFORM = OP_PERFORM,
    D_AWAIT = OP_AWAIT,
    D_POP_LOOP = OP_POP_LOOP,
    D_YIELD = OP_YIELD,
    D_ARRAY_PUSH = OP_ARRAY_PUSH,
    D_SPREAD_INTO_ARRAY = OP_SPREAD_INTO_ARRAY,
    D_SPREAD_INTO_OBJECT = OP_SPREAD_INTO_OBJECT,
    D_POP_FINALLY = OP_POP_FINALLY,
    D_LOAD_NUM_CONST = OP_LOAD_NUM_CONST,
    D_MAKE_ARRAY = OP_MAKE_ARRAY,
    D_MAKE_OBJECT = OP_MAKE_OBJECT,
    D_LOAD_INT_CONST = OP_LOAD_INT_CONST,
    D_GET_PROPERTY = OP_GET_PROPERTY,
    D_SET_PROPERTY = OP_SET_PROPERTY,
    D_BIND_VAR = OP_BIND_VAR,
    D_FOR_IN = OP_FOR_IN,
    D_RECEIVE = OP_RECEIVE,
    D_ITER_NEXT = OP_ITER_NEXT,
    D_PUSH_TRY = OP_PUSH_TRY,
    D_PUSH_FINALLY = OP_PUSH_FINALLY,
    D_END_FINALLY = OP_END_FINALLY,
    D_CALL = OP_CALL,
    D_METHOD_CALL = OP_METHOD_CALL,
    D_NEW_INSTANCE = OP_NEW_INSTANCE,
    D_SPAWN = OP_SPAWN,
    D_SUPER_CALL = OP_SUPER_CALL,
    D_MAKE_GENERATOR = OP_MAKE_GENERATOR,
    D_CALL_SPREAD = OP_CALL_SPREAD,
    D_METHOD_CALL_SPREAD = OP_METHOD_CALL_SPREAD,
    D_PUSH_LOOP = OP_PUSH_LOOP,
    D_MATCH_VARIANT = OP_MATCH_VARIANT,
    D_GET_STATIC = OP_GET_STATIC,
    D_DESTRUCT_ARRAY = OP_DESTRUCT_ARRAY,
    D_INT_LE_JUMP_IF_FALSE = OP_INT_LE_JUMP_IF_FALSE,
    D_INT_SUB_LOCAL_I = OP_INT_SUB_LOCAL_I,
    D_INT_LT_JUMP_IF_FALSE = OP_INT_LT_JUMP_IF_FALSE,
    D_INT_ADD_RR = OP_INT_ADD_RR,
    D_INT_SUB_RR = OP_INT_SUB_RR,
    D_INT_MUL_RR = OP_INT_MUL_RR,
    D_LOAD_INT_CONST_R = OP_LOAD_INT_CONST_R,
    D_RETURN_R = OP_RETURN_R,
    D_STR_CAT = OP_STR_CAT,
    D_INT_ADD_LOCAL_I = OP_INT_ADD_LOCAL_I,
    D_INT_MOD_RR = OP_INT_MOD_RR,
    D_NUM_ADD_RR = OP_NUM_ADD_RR,
    D_NUM_SUB_RR = OP_NUM_SUB_RR,
    D_NUM_MUL_RR = OP_NUM_MUL_RR,
    D_NUM_DIV_RR = OP_NUM_DIV_RR,
    D_INDEX_ASSIGN_RRR = OP_INDEX_ASSIGN_RRR,
    D_NUM_ADD_RI = OP_NUM_ADD_RI,
    D_NUM_SUB_RI = OP_NUM_SUB_RI,
    D_NUM_MUL_RI = OP_NUM_MUL_RI,
    D_NUM_DIV_RI = OP_NUM_DIV_RI,
    D_STR_REV_R = OP_STR_REV_R,
    D_NUM_MUL_ADD_ASSIGN = OP_NUM_MUL_ADD_ASSIGN,
    D_INT_MUL_RI = OP_INT_MUL_RI,
    D_INT_MOD_I = OP_INT_MOD_I,
    D_INT_CMP_LT_I = OP_INT_CMP_LT_I,
    D_INT_CMP_LE_I = OP_INT_CMP_LE_I,
    D_INT_CMP_EQ_I = OP_INT_CMP_EQ_I,
    D_INT_CMP_NE_I = OP_INT_CMP_NE_I,
    D_NUM_MUL_ADD_INDEXED = OP_NUM_MUL_ADD_INDEXED,
    D_STR_CHAR_EQ_RR = OP_STR_CHAR_EQ_RR,
    D_INT_LT_RR_JUMP_P = OP_INT_LT_RR_JUMP_P,
    D_INT_LE_RR_JUMP_P = OP_INT_LE_RR_JUMP_P,
    // P1b
    D_INDEX_ARRAY_RRR = OP_INDEX_ARRAY_RRR,
    D_INDEX_STRING_ASCII_RRR = OP_INDEX_STRING_ASCII_RRR,
    // G4
    D_INT_CMP_RR_JUMP_P = OP_INT_CMP_RR_JUMP_P,
    // G12
    D_F_LOAD_NUM = OP_F_LOAD_NUM,
    D_F_STORE_NUM = OP_F_STORE_NUM,
    D_F_ADD = OP_F_ADD,
    D_F_SUB = OP_F_SUB,
    D_F_MUL = OP_F_MUL,
    D_F_DIV = OP_F_DIV,
    D_F_SIN = OP_F_SIN,
    D_F_COS = OP_F_COS,
    D_F_SQRT = OP_F_SQRT,
    D_F_CONST = OP_F_CONST,
    D_F_MOVE = OP_F_MOVE,
}
