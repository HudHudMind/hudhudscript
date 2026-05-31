use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum StorageIndexExceptionCode {
    /// E0110 — Document Id Already Exists In Index
    IndexDuplicateId = 110,
    /// E0111 — Embedding Step Failed During Indexing
    IndexEmbedding = 111,
    /// E0112 — Document Not Found In Index
    IndexNotFound = 112,
}
