//! VM dense opcode spec (DENSE_OPCODE_ORDER) — specialized packed index
//! opcodes must be reachable through the dispatch table.

use hudhudscript_vm::vm::dispatch_table::dense_index;
use hudhudscript_vm::vm::packed_ops::{OP_INDEX_ARRAY_RRR, OP_INDEX_STRING_ASCII_RRR};

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
