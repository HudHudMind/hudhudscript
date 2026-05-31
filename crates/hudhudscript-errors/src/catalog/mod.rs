//! AUTO-GENERATED error catalog.
//!
//! Single source of truth for every error/exception code in HudHudScript.
//! Edit `crates/hudhudscript-errors/tools/gen_rust.py` and the JSON content
//! files, then regenerate. Do not hand-edit entries.
//!
//! 323 entries across 23 categories.

pub mod category;
pub use category::ErrorCategory;

pub mod codes;
pub use codes::ErrorCode;

pub mod query;
pub use query::ErrorEntry;

pub mod table;
pub use table::{ERROR_GROUPS, ERROR_TABLE};
