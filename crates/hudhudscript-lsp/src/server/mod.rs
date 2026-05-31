//! Language server — tower-lsp LanguageServer trait implementation
//!
//! Implements hover (#296), go-to-definition (#297), document symbols (#299),
//! and context-aware completion (#300).

pub mod backend;
pub mod capabilities;
pub mod helpers;
pub mod methods;

pub use backend::HudHudLanguageServer;
pub use capabilities::HudHudServerCapabilities;
pub use helpers::{completion_kind_to_lsp, parse_diagnostics, position_to_offset};
