use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum PackageModelExceptionCode {
    /// E0127 — Model already registered in catalog
    ModelManagerAlreadyExists = 127,
    /// E0128 — Not enough disk space for model
    ModelManagerInsufficientDiskSpace = 128,
    /// E0129 — I/O error in model manager
    ModelManagerIo = 129,
    /// E0130 — Model not found in catalog
    ModelManagerNotFound = 130,
}
