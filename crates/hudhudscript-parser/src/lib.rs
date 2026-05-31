//! HudHudScript Parser
//!
//! This crate provides parsing functionality to construct AST from tokens.
//! Supports multi-language parsing (English, Turkish, Japanese, Arabic) via Pest PEG parser.

#![allow(clippy::result_large_err)]

pub mod bidi;
pub mod cache;
mod error;
/// Issue #99: String interner for zero-copy identifier deduplication.
pub mod interner;
pub mod lang_directive;
pub mod parser;
mod pest_parser;

pub use bidi::{contains_rtl, strip_bidi_controls};
pub use cache::ParseCache;
pub use error::{parse_codes, ParseError, ParseResult};
pub use interner::{intern_identifier, SharedInterner, StringInterner};
pub use lang_directive::{parse_lang_directive, strip_lang_directive};
pub use parser::converters::{arabic_to_ascii, japanese_numeral_to_number};
pub use pest_parser::{parse, parse_with_lang_directive, parse_with_recovery, HudHudParser};

// Inline tests forbidden — all tests live in hudhud-script-tests/tests/
