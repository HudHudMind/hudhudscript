use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum PackageModuleErrorCode {
    /// E0131 — Module already loaded
    ModuleLoaderAlreadyLoaded = 131,
    /// E0132 — Module file not found by loader
    ModuleLoaderModuleNotFound = 132,
    /// E0133 — Failed to parse module source
    ModuleLoaderParseError = 133,
    /// E0134 — Failed to read module file
    ModuleLoaderReadError = 134,
}
