use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum PackageGraphExceptionCode {
    /// E0102 — Circular dependency in module graph
    GraphCircularDependency = 102,
    /// E0103 — Module missing from dependency graph
    GraphModuleNotFound = 103,
}
