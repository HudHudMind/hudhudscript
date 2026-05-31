//! Compact 32-bit instruction codec (Issue #1060, P7.3).
//!
//! Packs simple [`Instruction`] variants into a single `u32` for
//! cache-friendly dispatch.
//!
//! ## Layout
//!
//! ```text
//! [opcode: u8][arg1: u8][arg2: u16] = 32 bits
//! ```
//!
//! - `opcode` — unique number for each packable variant (0–255).
//! - `arg1`   — first small argument (0–255), e.g. argument count.
//! - `arg2`   — second argument (0–65535), e.g. constant index, jump
//!              target, or `SymId.0` (truncated to 16 bits).
//!
//! Complex instructions that carry `String`, `Vec`, `Option`, or
//! operands wider than 16 bits return `None` from [`pack`].

use super::{Instruction, SymId};

pub(crate) mod opcodes;
pub(crate) mod pack;
pub(crate) mod unpack;

pub use pack::pack;
pub use unpack::unpack;

/// Encode the three fields into a `u32`.
#[inline(always)]
pub const fn encode(opcode: u8, arg1: u8, arg2: u16) -> u32 {
    (opcode as u32) | ((arg1 as u32) << 8) | ((arg2 as u32) << 16)
}

/// Decode a `u32` into `(opcode, arg1, arg2)`.
#[inline(always)]
pub const fn decode(packed: u32) -> (u8, u8, u16) {
    let opcode = (packed & 0xFF) as u8;
    let arg1 = ((packed >> 8) & 0xFF) as u8;
    let arg2 = ((packed >> 16) & 0xFFFF) as u16;
    (opcode, arg1, arg2)
}
