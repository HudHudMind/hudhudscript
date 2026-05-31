use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ToolOpenapiErrorCode {
    /// E0157 — OpenAPI document failed to parse
    OpenApiParseError = 157,
    /// E0158 — OpenAPI tool registration failed
    OpenApiRegistryError = 158,
}
