//! Standard Tool Library and Custom Tool API (Issue #117)
//!
//! Provides:
//! - `CustomTool` trait — implement this to register your own tool in Rust.
//! - `StandardTool` — built-in tools: `file_read`, `http_get`, `json_parse`.
//! - `register_standard_tools` helper — adds all built-ins to a `ToolRegistry`.

pub mod error;
pub mod registry;
pub mod sandbox;
pub mod tool;
pub mod tool_trait;

pub use error::*;
pub use registry::*;
pub use tool::*;
pub use tool_trait::*;
