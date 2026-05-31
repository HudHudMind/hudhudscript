//! AUTO-GENERATED exception catalog.
//!
//! Single source of truth for every error/exception code in HudHudScript.
//! Edit `crates/hudhudscript-errors/tools/gen_rust.py` and the JSON content
//! files, then regenerate. Do not hand-edit entries.
//!
//! 323 entries across 23 categories.

pub mod category;
pub mod codes;
pub mod entry;
pub mod table;

pub use category::ExceptionCategory;
pub use codes::ExceptionCode;
pub use entry::ExceptionEntry;
