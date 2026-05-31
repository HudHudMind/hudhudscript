//! GGUF types — quantization, metadata, and value representations.

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Quantization enum
// ---------------------------------------------------------------------------

/// GGML / GGUF quantization type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GgufQuantization {
    Q4_0,
    Q4_1,
    Q5_0,
    Q5_1,
    Q8_0,
    F16,
    F32,
    Unknown(u32),
}

impl fmt::Display for GgufQuantization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Q4_0 => write!(f, "Q4_0"),
            Self::Q4_1 => write!(f, "Q4_1"),
            Self::Q5_0 => write!(f, "Q5_0"),
            Self::Q5_1 => write!(f, "Q5_1"),
            Self::Q8_0 => write!(f, "Q8_0"),
            Self::F16 => write!(f, "F16"),
            Self::F32 => write!(f, "F32"),
            Self::Unknown(v) => write!(f, "Unknown({})", v),
        }
    }
}

impl From<u32> for GgufQuantization {
    fn from(value: u32) -> Self {
        match value {
            2 => Self::Q4_0,
            3 => Self::Q4_1,
            6 => Self::Q5_0,
            7 => Self::Q5_1,
            8 => Self::Q8_0,
            1 => Self::F16,
            0 => Self::F32,
            other => Self::Unknown(other),
        }
    }
}

// ---------------------------------------------------------------------------
// Metadata struct
// ---------------------------------------------------------------------------

/// High-level metadata extracted from a GGUF header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgufMetadata {
    /// Model architecture (e.g. `"llama"`, `"gpt2"`).
    pub architecture: String,
    /// Maximum context length (tokens).
    pub context_length: u64,
    /// Quantization type of the tensor data.
    pub quantization: GgufQuantization,
    /// Embedding / hidden-state dimension.
    pub embedding_length: u64,
    /// Vocabulary size.
    pub vocab_size: u64,
    /// Total file size in bytes (caller-supplied or zero).
    pub file_size: u64,
}

// ---------------------------------------------------------------------------
// GGUF value type constants
// ---------------------------------------------------------------------------

/// GGUF metadata value type codes.
pub const GGUF_TYPE_UINT8: u32 = 0;
pub const GGUF_TYPE_INT8: u32 = 1;
pub const GGUF_TYPE_UINT16: u32 = 2;
pub const GGUF_TYPE_INT16: u32 = 3;
pub const GGUF_TYPE_UINT32: u32 = 4;
pub const GGUF_TYPE_INT32: u32 = 5;
pub const GGUF_TYPE_FLOAT32: u32 = 6;
pub const GGUF_TYPE_BOOL: u32 = 7;
pub const GGUF_TYPE_STRING: u32 = 8;
pub const GGUF_TYPE_ARRAY: u32 = 9;
pub const GGUF_TYPE_UINT64: u32 = 10;
pub const GGUF_TYPE_INT64: u32 = 11;
pub const GGUF_TYPE_FLOAT64: u32 = 12;

/// Represents a single metadata value read from the header.
#[derive(Debug, Clone)]
pub enum GgufValue {
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(()),
    Str(String),
    Array(()),
    Other,
}

impl GgufValue {
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::U32(v) => Some(*v as u64),
            Self::I32(v) => Some(*v as u64),
            Self::U64(v) => Some(*v),
            Self::I64(v) => Some(*v as u64),
            Self::F32(v) => Some(*v as u64),
            Self::F64(v) => Some(*v as u64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Self::Str(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }
}
