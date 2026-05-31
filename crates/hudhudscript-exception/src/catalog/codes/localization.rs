use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum LocalizationExceptionCode {
    /// E0024 — Localization Catalog I/O Failure
    CatalogIo = 24,
    /// E0025 — Localization Catalog JSON Parse Error
    CatalogJson = 25,
    /// E0026 — Localization Catalog YAML Parse Error
    CatalogYaml = 26,
}
