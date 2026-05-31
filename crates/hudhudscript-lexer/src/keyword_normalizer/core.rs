//! Keyword Normalizer
//!
//! Pre-processes source code before Pest parsing by replacing multi-language
//! keywords with their canonical English equivalents. This eliminates the need
//! for 22-language choice chains in the Pest grammar.
//!
//! Strategy: longest-match-first replacement using word-boundary detection.
//! Only standalone tokens (not substrings of identifiers) are replaced.
//!
//! Performance (Audit v3 Finding 17.1 / PERF-13):
//!   Multi-word entries (9 out of ~2518) run through the legacy O(n·k) passes
//!   in phase 1. Single-word entries then run through a single O(n) pass
//!   (phase 2) that scans identifier boundaries and does an FxHashMap lookup
//!   per identifier.  Multilingual support is preserved byte-for-byte: the
//!   map key is the exact `&'static str` from `KEYWORD_MAP`.

/// A single keyword mapping: foreign keyword → canonical English keyword
pub(crate) struct KwMap {
    pub(crate) from: &'static str,
    pub(crate) to: &'static str,
}

/// Check if a character is a word character (identifier char).
///
/// Audit v3 Finding 18.1 + Audit v3 Finding 17.1: ASCII fast-path before the
/// Unicode fallback. The fallback uses `unicode_ident::is_xid_continue`,
/// which follows UAX #31 (identifier syntax) and includes combining marks
/// and script-specific vowel/diacritic codepoints needed by Bengali, Thai,
/// Devanagari, Arabic, Hebrew, etc. This fixes the regression where
/// `is_alphanumeric` alone dropped combining marks and split identifiers
/// like `কনস্ট্যান্ট` into `কন্` + `্ট্যান্ট`.
#[inline]
pub(crate) fn is_word_char(ch: char) -> bool {
    if (ch as u32) < 128 {
        matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
    } else {
        unicode_ident::is_xid_continue(ch) || ch.is_alphanumeric()
    }
}

/// Replace all occurrences of `from` that are not part of a larger identifier
/// with `to`. Handles Unicode word boundaries correctly.
/// Skips content inside string literals (", ', `) and comments (// and /* */).
pub(crate) fn replace_keyword(source: &str, from: &str, to: &str, out: &mut String) {
    let from_bytes = from.as_bytes();
    let from_len = from.len();
    let src_bytes = source.as_bytes();
    let src_len = src_bytes.len();
    let mut i = 0;
    // Track whether we're inside a string literal or comment
    let mut in_string: Option<u8> = None; // b'"', b'\'', b'`'
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while i < src_len {
        let b = src_bytes[i];

        // Handle newline — ends line comments
        if b == b'\n' {
            in_line_comment = false;
            out.push('\n');
            i += 1;
            continue;
        }

        // If inside a line comment, copy verbatim
        if in_line_comment {
            let ch = source[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        // If inside a block comment, look for */
        if in_block_comment {
            if b == b'*' && i + 1 < src_len && src_bytes[i + 1] == b'/' {
                out.push_str("*/");
                i += 2;
                in_block_comment = false;
            } else {
                let ch = source[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
            continue;
        }

        // If inside a string, look for the closing quote (handle backslash escapes)
        if let Some(quote) = in_string {
            if b == b'\\' && i + 1 < src_len {
                // Escaped char — copy backslash + full next character
                out.push('\\');
                let next_ch = source[i + 1..].chars().next().unwrap();
                out.push(next_ch);
                i += 1 + next_ch.len_utf8();
            } else if b == quote {
                in_string = None;
                out.push(b as char);
                i += 1;
            } else {
                let ch = source[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
            continue;
        }

        // Check for start of comments
        if b == b'/' && i + 1 < src_len {
            if src_bytes[i + 1] == b'/' {
                in_line_comment = true;
                out.push_str("//");
                i += 2;
                continue;
            }
            if src_bytes[i + 1] == b'*' {
                in_block_comment = true;
                out.push_str("/*");
                i += 2;
                continue;
            }
        }

        // Check for start of string literal
        if b == b'"' || b == b'\'' || b == b'`' {
            in_string = Some(b);
            out.push(b as char);
            i += 1;
            continue;
        }

        // Not inside string/comment — try keyword replacement
        if src_bytes[i..].starts_with(from_bytes) {
            // Check left boundary
            let left_ok = if i == 0 {
                true
            } else {
                let prefix = &source[..i];
                !prefix
                    .chars()
                    .next_back()
                    .map(is_word_char)
                    .unwrap_or(false)
            };

            // Check right boundary
            let right_ok = if i + from_len >= src_len {
                true
            } else {
                let suffix = &source[i + from_len..];
                !suffix.chars().next().map(is_word_char).unwrap_or(false)
            };

            if left_ok && right_ok {
                out.push_str(to);
                i += from_len;
                continue;
            }
        }
        // Copy one char
        let ch = source[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
}
