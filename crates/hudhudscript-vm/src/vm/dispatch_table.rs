//! C1: Dense opcode remap table for jump-table dispatch.
//!
//! The packed instruction format uses sparse opcodes (0..127 with gaps).
//! A `match` on sparse values compiles to a binary-search decision tree
//! instead of a proper jump table, causing ~70% indirect branch miss rate.
//!
//! This module provides a dense remap: old sparse opcode → 0..DENSE_MAX.
//! The VM uses the dense index in a contiguous `match`, guaranteeing
//! LLVM emits a jump table (one memory access + one indirect jump).
//!
//! MUST stay in sync with:
//!   - `bytecode::packed_instruction::opcodes`   (encoder)
//!   - `vm::packed_ops`                          (decoder constants)

/// Number of distinct packed opcodes. Must match the number of entries
/// in the remap table that are not 0xFF.
pub(crate) const DENSE_COUNT: u8 = 95;

/// Sentinel: opcode has no handler (fallthrough to unpacked dispatch).
const NO_MAP: u8 = 0xFF;

/// Dense remap table indexed by sparse opcode (0..=127).
/// Returns the dense index (0..DENSE_COUNT-1) or NO_MAP.
pub(crate) static DENSE_MAP: [u8; 256] = build_dense_map();

const fn build_dense_map() -> [u8; 256] {
    let mut map = [NO_MAP; 256];
    let mut next = 0u8;

    // Order: must match the match arms in dispatch_general.rs EXACTLY.
    // Grouped by original opcode ranges for readability.

    // 0..4: register comparison / move
    map[0] = next; next += 1;  // OP_INT_EQ_RR          = 0
    map[1] = next; next += 1;  // OP_INT_LT_RR          = 1
    map[2] = next; next += 1;  // OP_INT_LE_RR          = 2
    map[3] = next; next += 1;  // OP_INT_NE_RR          = 3
    map[4] = next; next += 1;  // OP_MOVE_RR            = 4

    // 7..18: control / arithmetic
    map[7] = next; next += 1;  // OP_JUMP_IF_FALSE_R    = 7
    map[8] = next; next += 1;  // OP_JUMP_IF_TRUE_R     = 8
    map[9] = next; next += 1;  // OP_INDEX_RRR          = 9
    map[10] = next; next += 1; // OP_STRCAT_RRR         = 10
    map[11] = next; next += 1; // OP_INT_ADD_RI         = 11
    map[12] = next; next += 1; // OP_INT_SUB_RI         = 12
    map[13] = next; next += 1; // OP_NEG_R              = 13
    map[14] = next; next += 1; // OP_NOT_R              = 14
    map[15] = next; next += 1; // OP_ARRAY_PUSH_RRR     = 15
    map[16] = next; next += 1; // OP_STRING_INDEX_OF_RRR = 16
    map[17] = next; next += 1; // OP_STRING_CONTAINS_RRR = 17
    map[18] = next; next += 1; // OP_STRCAT_MUT_RR      = 18

    // 19..26: return / jump / load / store
    map[19] = next; next += 1; // OP_RETURN             = 19
    map[20] = next; next += 1; // OP_JUMP               = 20
    map[21] = next; next += 1; // OP_JUMP_IF_FALSE      = 21
    map[22] = next; next += 1; // OP_JUMP_IF_TRUE       = 22
    map[23] = next; next += 1; // OP_LOAD_VAR           = 23
    map[24] = next; next += 1; // OP_STORE_VAR          = 24
    map[25] = next; next += 1; // OP_DECL_VAR           = 25
    map[26] = next; next += 1; // OP_STORE_CONST        = 26

    // 29..43: index / break / continue / pop / throw / send / etc.
    map[29] = next; next += 1; // OP_INDEX              = 29
    map[30] = next; next += 1; // OP_BREAK              = 30
    map[31] = next; next += 1; // OP_CONTINUE           = 31
    map[32] = next; next += 1; // OP_POP_TRY            = 32
    map[33] = next; next += 1; // OP_THROW              = 33
    map[34] = next; next += 1; // OP_SEND               = 34
    map[35] = next; next += 1; // OP_REQUIRE            = 35
    map[36] = next; next += 1; // OP_PERFORM            = 36
    map[37] = next; next += 1; // OP_AWAIT              = 37
    map[38] = next; next += 1; // OP_POP_LOOP           = 38
    map[39] = next; next += 1; // OP_YIELD              = 39
    map[40] = next; next += 1; // OP_ARRAY_PUSH         = 40
    map[41] = next; next += 1; // OP_SPREAD_INTO_ARRAY  = 41
    map[42] = next; next += 1; // OP_SPREAD_INTO_OBJECT = 42
    map[43] = next; next += 1; // OP_POP_FINALLY        = 43

    // 69: load num const
    map[69] = next; next += 1; // OP_LOAD_NUM_CONST     = 69

    // 76..78: local slot ops
    map[76] = next; next += 1; // OP_LOAD_LOCAL         = 76
    map[77] = next; next += 1; // OP_STORE_LOCAL        = 77
    map[78] = next; next += 1; // OP_DECL_LOCAL         = 78

    // 80..81: array/object creation
    map[80] = next; next += 1; // OP_MAKE_ARRAY         = 80
    map[81] = next; next += 1; // OP_MAKE_OBJECT        = 81

    // 88..92+94..110: property / bind / for-in / iter / try / call / etc.
    map[88] = next; next += 1;   // OP_LOAD_INT_CONST     = 88
    map[89] = next; next += 1;   // OP_GET_PROPERTY       = 89
    map[90] = next; next += 1;   // OP_SET_PROPERTY       = 90
    map[91] = next; next += 1;   // OP_BIND_VAR           = 91
    map[92] = next; next += 1;   // OP_FOR_IN             = 92
    map[94] = next; next += 1;   // OP_RECEIVE            = 94
    map[95] = next; next += 1;   // OP_ITER_NEXT          = 95
    map[96] = next; next += 1;   // OP_PUSH_TRY           = 96
    map[97] = next; next += 1;   // OP_PUSH_FINALLY       = 97
    map[98] = next; next += 1;   // OP_END_FINALLY        = 98
    map[99] = next; next += 1;   // OP_CALL               = 99
    map[100] = next; next += 1;  // OP_METHOD_CALL        = 100
    map[101] = next; next += 1;  // OP_NEW_INSTANCE       = 101
    map[102] = next; next += 1;  // OP_SPAWN              = 102
    map[103] = next; next += 1;  // OP_SUPER_CALL         = 103
    map[104] = next; next += 1;  // OP_MAKE_GENERATOR     = 104
    map[105] = next; next += 1;  // OP_CALL_SPREAD        = 105
    map[106] = next; next += 1;  // OP_METHOD_CALL_SPREAD = 106
    map[107] = next; next += 1;  // OP_PUSH_LOOP          = 107
    map[108] = next; next += 1;  // OP_MATCH_VARIANT      = 108
    map[109] = next; next += 1;  // OP_GET_STATIC         = 109
    map[110] = next; next += 1;  // OP_DESTRUCT_ARRAY     = 110

    // 111..117: super-instructions
    map[111] = next; next += 1;  // OP_INT_SUB_CALL_1         = 111
    map[112] = next; next += 1;  // OP_INT_LE_JUMP_IF_FALSE  = 112
    map[113] = next; next += 1;  // OP_INT_ADD_CALL_1         = 113
    map[114] = next; next += 1;  // OP_INT_SUB_LOCAL_I        = 114
    map[115] = next; next += 1;  // OP_INDEX_FAST_ISLOT       = 115
    map[116] = next; next += 1;  // OP_INT_INCR_SLOT          = 116
    map[117] = next; next += 1;  // OP_INT_LT_JUMP_IF_FALSE  = 117

    // 118..127: register ops
    map[118] = next; next += 1;  // OP_INT_ADD_RR          = 118
    map[119] = next; next += 1;  // OP_INT_SUB_RR          = 119
    map[120] = next; next += 1;  // OP_INT_MUL_RR          = 120
    map[121] = next; next += 1;  // OP_LOAD_INT_CONST_R    = 121
    map[122] = next; next += 1;  // OP_LOAD_LOCAL_R        = 122
    map[123] = next; next += 1;  // OP_STORE_LOCAL_R       = 123
    map[124] = next; next += 1;  // OP_PUSH_REG            = 124
    map[125] = next; next += 1;  // OP_RETURN_R            = 125
    map[126] = next; next += 1;  // OP_STR_CAT             = 126
    map[127] = next; next += 1;  // OP_INT_ADD_LOCAL_I     = 127
    map[128] = next; next += 1;  // OP_INT_MOD_RR          = 128

    // 129-132: float register arithmetic
    map[129] = next; next += 1;  // OP_NUM_ADD_RR = 129
    map[130] = next; next += 1;  // OP_NUM_SUB_RR = 130
    map[131] = next; next += 1;  // OP_NUM_MUL_RR = 131
    map[132] = next; next += 1;  // OP_NUM_DIV_RR = 132

    map[133] = next; next += 1;  // OP_INDEX_ASSIGN_RRR = 133

    // 134-137: float immediate arithmetic
    map[134] = next; next += 1;  // OP_NUM_ADD_RI = 134
    map[135] = next; next += 1;  // OP_NUM_SUB_RI = 135
    map[136] = next; next += 1;  // OP_NUM_MUL_RI = 136
    map[137] = next; next += 1;  // OP_NUM_DIV_RI = 137

    // Verify count: next should equal DENSE_COUNT
    // (compile-time assertion not possible in const fn, checked at init)

    map[122] = next; // OP_INT_MUL_RI = 122
    map
}

#[test]
fn test_dense_map_count() {
    let mut count = 0u8;
    for i in 0..256 {
        if DENSE_MAP[i] != NO_MAP {
            count += 1;
        }
    }
    assert_eq!(count, DENSE_COUNT, "DENSE_COUNT mismatch: table has {count} entries");
}
