//! Dense dispatch mapping — derived from `dense_spec::DENSE_OPCODE_ORDER`.
//! This is the sole source of DENSE_MAP, DENSE_COUNT, and dense_index().

use crate::vm::dense_spec::DENSE_OPCODE_ORDER;

#[doc(hidden)]
pub const DENSE_COUNT: u8 = DENSE_OPCODE_ORDER.len() as u8;

/// Sentinel for unmapped sparse opcodes.
#[doc(hidden)]
pub const NO_MAP: u8 = 0xFF;

#[doc(hidden)]
pub static DENSE_MAP: [u8; 256] = build_dense_map();

/// Returns the dense index (0..DENSE_COUNT-1) or NO_MAP.
#[doc(hidden)]
pub const fn dense_index(opcode: u8) -> u8 {
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
