use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum StoragePersistenceErrorCode {
    /// E0188 — Snapshot Or Restore I/O Failure
    PersistenceIo = 188,
    /// E0189 — No Snapshot Found For Agent
    PersistenceNotFound = 189,
    /// E0190 — Snapshot Serialization Or Parse Failure
    PersistenceSerialization = 190,
}
