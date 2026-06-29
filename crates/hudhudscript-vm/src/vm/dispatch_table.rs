//! Dense dispatch mapping — derived from `dense_spec::DENSE_OPCODE_ORDER`.
//! This is the sole source of DENSE_MAP, DENSE_COUNT, and dense_index().

use crate::vm::dense_spec::DENSE_OPCODE_ORDER;

pub(crate) const DENSE_COUNT: u8 = DENSE_OPCODE_ORDER.len() as u8;

/// Sentinel for unmapped sparse opcodes.
const NO_MAP: u8 = 0xFF;

pub(crate) static DENSE_MAP: [u8; 256] = build_dense_map();

/// Returns the dense index (0..DENSE_COUNT-1) or NO_MAP.
pub(crate) const fn dense_index(opcode: u8) -> u8 {
    let mut i = 0usize;
    while i < DENSE_OPCODE_ORDER.len() {
        if DENSE_OPCODE_ORDER[i] == opcode {
            return i as u8;
        }
        i += 1;
    }
    NO_MAP
}

const fn build_dense_map() -> [u8; 256] {
    let mut map = [NO_MAP; 256];
    let mut i = 0usize;
    while i < DENSE_OPCODE_ORDER.len() {
        map[DENSE_OPCODE_ORDER[i] as usize] = i as u8;
        i += 1;
    }
    map
}

#[cfg(test)]
mod dense_tests {
    use super::*;
    use crate::vm::dense_ops::*;
    use crate::vm::packed_ops::*;

    #[test]
    fn test_dense_map_count_matches_spec() {
        let mut count = 0u8;
        for value in DENSE_MAP {
            if value != NO_MAP {
                count += 1;
            }
        }
        assert_eq!(count, DENSE_COUNT);
        assert_eq!(DENSE_COUNT as usize, DENSE_OPCODE_ORDER.len());
    }

    #[test]
    fn test_dense_spec_has_no_duplicate_sparse_opcodes() {
        for i in 0..DENSE_OPCODE_ORDER.len() {
            for j in i + 1..DENSE_OPCODE_ORDER.len() {
                assert_ne!(
                    DENSE_OPCODE_ORDER[i], DENSE_OPCODE_ORDER[j],
                    "duplicate sparse opcode in DENSE_OPCODE_ORDER: {}",
                    DENSE_OPCODE_ORDER[i]
                );
            }
        }
    }

    #[test]
    fn test_dense_map_matches_spec_order() {
        for i in 0..DENSE_OPCODE_ORDER.len() {
            let opcode = DENSE_OPCODE_ORDER[i] as usize;
            assert_eq!(DENSE_MAP[opcode], i as u8);
        }
    }

    #[test]
    fn test_critical_dense_indices() {
        assert_eq!(DENSE_MAP[OP_INT_MUL_RI as usize], D_INT_MUL_RI);
        assert_eq!(DENSE_MAP[OP_STR_REV_R as usize], D_STR_REV_R);
        assert_eq!(
            DENSE_MAP[OP_NUM_MUL_ADD_ASSIGN as usize],
            D_NUM_MUL_ADD_ASSIGN
        );
        assert_ne!(D_INT_MUL_RI, D_STR_REV_R);
        assert_ne!(D_INT_MUL_RI, D_NUM_MUL_ADD_ASSIGN);
        assert_ne!(D_STR_REV_R, D_NUM_MUL_ADD_ASSIGN);
    }

    #[test]
    fn test_all_named_dense_indices_are_in_spec() {
        let named: &[(&str, u8)] = &[
            ("D_INT_EQ_RR", D_INT_EQ_RR),
            ("D_INT_LT_RR", D_INT_LT_RR),
            ("D_INT_LE_RR", D_INT_LE_RR),
            ("D_INT_NE_RR", D_INT_NE_RR),
            ("D_MOVE_RR", D_MOVE_RR),
            ("D_JUMP_IF_FALSE_R", D_JUMP_IF_FALSE_R),
            ("D_JUMP_IF_TRUE_R", D_JUMP_IF_TRUE_R),
            ("D_INDEX_RRR", D_INDEX_RRR),
            ("D_STRCAT_RRR", D_STRCAT_RRR),
            ("D_INT_ADD_RI", D_INT_ADD_RI),
            ("D_INT_SUB_RI", D_INT_SUB_RI),
            ("D_NEG_R", D_NEG_R),
            ("D_NOT_R", D_NOT_R),
            ("D_ARRAY_PUSH_RRR", D_ARRAY_PUSH_RRR),
            ("D_STRING_INDEX_OF_RRR", D_STRING_INDEX_OF_RRR),
            ("D_STRING_CONTAINS_RRR", D_STRING_CONTAINS_RRR),
            ("D_STRCAT_MUT_RR", D_STRCAT_MUT_RR),
            ("D_RETURN", D_RETURN),
            ("D_JUMP", D_JUMP),
            ("D_JUMP_IF_FALSE", D_JUMP_IF_FALSE),
            ("D_JUMP_IF_TRUE", D_JUMP_IF_TRUE),
            ("D_LOAD_VAR", D_LOAD_VAR),
            ("D_STORE_VAR", D_STORE_VAR),
            ("D_DECL_VAR", D_DECL_VAR),
            ("D_STORE_CONST", D_STORE_CONST),
            ("D_INDEX", D_INDEX),
            ("D_BREAK", D_BREAK),
            ("D_CONTINUE", D_CONTINUE),
            ("D_POP_TRY", D_POP_TRY),
            ("D_THROW", D_THROW),
            ("D_SEND", D_SEND),
            ("D_REQUIRE", D_REQUIRE),
            ("D_PERFORM", D_PERFORM),
            ("D_AWAIT", D_AWAIT),
            ("D_POP_LOOP", D_POP_LOOP),
            ("D_YIELD", D_YIELD),
            ("D_ARRAY_PUSH", D_ARRAY_PUSH),
            ("D_SPREAD_INTO_ARRAY", D_SPREAD_INTO_ARRAY),
            ("D_SPREAD_INTO_OBJECT", D_SPREAD_INTO_OBJECT),
            ("D_POP_FINALLY", D_POP_FINALLY),
            ("D_LOAD_NUM_CONST", D_LOAD_NUM_CONST),
            ("D_LOAD_LOCAL", D_LOAD_LOCAL),
            ("D_STORE_LOCAL", D_STORE_LOCAL),
            ("D_DECL_LOCAL", D_DECL_LOCAL),
            ("D_MAKE_ARRAY", D_MAKE_ARRAY),
            ("D_MAKE_OBJECT", D_MAKE_OBJECT),
            ("D_LOAD_INT_CONST", D_LOAD_INT_CONST),
            ("D_GET_PROPERTY", D_GET_PROPERTY),
            ("D_SET_PROPERTY", D_SET_PROPERTY),
            ("D_BIND_VAR", D_BIND_VAR),
            ("D_FOR_IN", D_FOR_IN),
            ("D_RECEIVE", D_RECEIVE),
            ("D_ITER_NEXT", D_ITER_NEXT),
            ("D_PUSH_TRY", D_PUSH_TRY),
            ("D_PUSH_FINALLY", D_PUSH_FINALLY),
            ("D_END_FINALLY", D_END_FINALLY),
            ("D_CALL", D_CALL),
            ("D_METHOD_CALL", D_METHOD_CALL),
            ("D_NEW_INSTANCE", D_NEW_INSTANCE),
            ("D_SPAWN", D_SPAWN),
            ("D_SUPER_CALL", D_SUPER_CALL),
            ("D_MAKE_GENERATOR", D_MAKE_GENERATOR),
            ("D_CALL_SPREAD", D_CALL_SPREAD),
            ("D_METHOD_CALL_SPREAD", D_METHOD_CALL_SPREAD),
            ("D_PUSH_LOOP", D_PUSH_LOOP),
            ("D_MATCH_VARIANT", D_MATCH_VARIANT),
            ("D_GET_STATIC", D_GET_STATIC),
            ("D_DESTRUCT_ARRAY", D_DESTRUCT_ARRAY),
            ("D_INT_SUB_CALL_1", D_INT_SUB_CALL_1),
            ("D_INT_LE_JUMP_IF_FALSE", D_INT_LE_JUMP_IF_FALSE),
            ("D_INT_ADD_CALL_1", D_INT_ADD_CALL_1),
            ("D_INT_SUB_LOCAL_I", D_INT_SUB_LOCAL_I),
            ("D_INDEX_FAST_ISLOT", D_INDEX_FAST_ISLOT),
            ("D_INT_INCR_SLOT", D_INT_INCR_SLOT),
            ("D_INT_LT_JUMP_IF_FALSE", D_INT_LT_JUMP_IF_FALSE),
            ("D_INT_ADD_RR", D_INT_ADD_RR),
            ("D_INT_SUB_RR", D_INT_SUB_RR),
            ("D_INT_MUL_RR", D_INT_MUL_RR),
            ("D_LOAD_INT_CONST_R", D_LOAD_INT_CONST_R),
            ("D_RETURN_R", D_RETURN_R),
            ("D_STR_CAT", D_STR_CAT),
            ("D_INT_ADD_LOCAL_I", D_INT_ADD_LOCAL_I),
            ("D_INT_MOD_RR", D_INT_MOD_RR),
            ("D_NUM_ADD_RR", D_NUM_ADD_RR),
            ("D_NUM_SUB_RR", D_NUM_SUB_RR),
            ("D_NUM_MUL_RR", D_NUM_MUL_RR),
            ("D_NUM_DIV_RR", D_NUM_DIV_RR),
            ("D_INDEX_ASSIGN_RRR", D_INDEX_ASSIGN_RRR),
            ("D_NUM_ADD_RI", D_NUM_ADD_RI),
            ("D_NUM_SUB_RI", D_NUM_SUB_RI),
            ("D_NUM_MUL_RI", D_NUM_MUL_RI),
            ("D_NUM_DIV_RI", D_NUM_DIV_RI),
            ("D_STR_REV_R", D_STR_REV_R),
            ("D_NUM_MUL_ADD_ASSIGN", D_NUM_MUL_ADD_ASSIGN),
            ("D_INT_MUL_RI", D_INT_MUL_RI),
        ];
        for &(name, value) in named.iter() {
            assert_ne!(value, NO_MAP, "{name} missing from DENSE_OPCODE_ORDER");
            assert!(value < DENSE_COUNT, "{name} out of range");
        }
        // Uniqueness check
        let mut seen = std::collections::HashSet::new();
        for &(_name, value) in named.iter() {
            assert!(seen.insert(value), "duplicate dense index: value {value}");
        }
    }
}
