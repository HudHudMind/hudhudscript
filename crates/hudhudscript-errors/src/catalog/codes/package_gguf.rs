use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum PackageGgufErrorCode {
    /// E0082 — GGUF file has invalid magic bytes
    GgufInvalidMagic = 82,
    /// E0083 — GGUF metadata contains invalid UTF-8
    GgufInvalidUtf8 = 83,
    /// E0084 — GGUF file shorter than header size
    GgufTooShort = 84,
    /// E0085 — GGUF parser hit unexpected end of file
    GgufUnexpectedEof = 85,
    /// E0086 — GGUF file uses an unsupported version
    GgufUnsupportedVersion = 86,
}
