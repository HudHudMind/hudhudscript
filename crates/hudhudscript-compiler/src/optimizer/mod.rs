//! Bytecode optimization passes (Issue #605)
//!
//! Provides dead-code elimination, constant folding, and peephole
//! optimizations on compiled instruction streams.

use crate::bytecode::{FunctionChunk, Instruction, Value16};
use hudhudscript_bytecode::CallPayload;
use std::collections::HashMap;
use std::sync::Arc;

/// Optimization level controlling which passes are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// No optimizations.
    None,
    /// Basic optimizations: constant folding, simple dead-code elimination.
    Basic,
    /// Aggressive: all basic optimizations plus peephole patterns.
    Aggressive,
}

mod constant_fold;
mod dead_code;
mod entry;
mod fuse_slot;
mod fuse_super;
mod fuse_super_extra;
mod inline;
pub(crate) mod inline_compile;
pub mod licm;
mod peephole;
mod utils;

pub use constant_fold::*;
pub use dead_code::*;
pub use entry::*;
pub use fuse_slot::*;
pub use fuse_super::*;
pub use inline::*;
pub use licm::*;
pub use peephole::*;
pub use utils::*;
