//! Terminal Markdown rendering with syntax highlighting and streaming support.
//!
//! Extracted from `hudhudscript-formatter` so that other crates can render
//! Markdown to the terminal without pulling in the full code-formatter.

pub mod markdown;
pub mod streaming;
pub mod syntax;
pub mod theme;
