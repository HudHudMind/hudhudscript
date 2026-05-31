use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum PackageResolverErrorCode {
    /// E0215 — Invalid module resolver path
    ResolverInvalidPath = 215,
    /// E0216 — Resolver could not locate module
    ResolverNotFound = 216,
}
