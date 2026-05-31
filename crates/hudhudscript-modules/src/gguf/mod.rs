//! GGUF file format parser
//!
//! Parses the header of `.gguf` files to extract model metadata such as
//! architecture, context length, quantization type, embedding dimensions, and
//! vocabulary size — without requiring the full file to be loaded into memory.

pub mod error;
pub mod parser;
pub mod reader;
pub mod types;

pub use error::*;
pub use parser::*;
pub use reader::*;
pub use types::*;

/// GGUF magic number (`GGUF` in little-endian).
pub const GGUF_MAGIC: u32 = 0x46475547; // 'G','G','U','F'
