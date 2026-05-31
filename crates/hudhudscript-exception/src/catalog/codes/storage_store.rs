use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum StorageStoreExceptionCode {
    /// E0263 — Vector Store Dimension Mismatch
    StoreDimensionMismatch = 263,
    /// E0264 — Vector Store Configuration Is Invalid
    StoreInvalidConfig = 264,
    /// E0265 — Store Entry Missing By Key
    StoreNotFound = 265,
    /// E0266 — Vector Store Persist Operation Failed
    StorePersistError = 266,
}
