//! Keyword mapping for multi-language support
//!
//! Maps natural language keywords to canonical forms.
//! All language mappings are now modularized in the languages/ directory.

pub mod english;
pub mod keyword;
pub mod map;

pub use keyword::Keyword;
pub use map::KeywordMap;
