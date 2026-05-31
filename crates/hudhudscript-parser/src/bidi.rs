//! BiDi (Bidirectional) text handling for RTL languages
//!
//! Removes Unicode BiDi control characters that can cause parsing issues

/// Unicode BiDi control characters that should be stripped
const BIDI_CONTROLS: &[char] = &[
    '\u{200E}', // LEFT-TO-RIGHT MARK (LRM)
    '\u{200F}', // RIGHT-TO-LEFT MARK (RLM)
    '\u{202A}', // LEFT-TO-RIGHT EMBEDDING (LRE)
    '\u{202B}', // RIGHT-TO-LEFT EMBEDDING (RLE)
    '\u{202C}', // POP DIRECTIONAL FORMATTING (PDF)
    '\u{202D}', // LEFT-TO-RIGHT OVERRIDE (LRO)
    '\u{202E}', // RIGHT-TO-LEFT OVERRIDE (RLO)
    '\u{2066}', // LEFT-TO-RIGHT ISOLATE (LRI)
    '\u{2067}', // RIGHT-TO-LEFT ISOLATE (RLI)
    '\u{2068}', // FIRST STRONG ISOLATE (FSI)
    '\u{2069}', // POP DIRECTIONAL ISOLATE (PDI)
];

/// Strip BiDi control characters from source code
///
/// These characters are invisible but can cause parsing issues,
/// especially in RTL languages like Arabic and Hebrew.
pub fn strip_bidi_controls(source: &str) -> String {
    source
        .chars()
        .filter(|c| !BIDI_CONTROLS.contains(c))
        .collect()
}

/// Detect if source contains RTL characters
pub fn contains_rtl(source: &str) -> bool {
    source.chars().any(|c| {
        matches!(c,
            // Arabic
            '\u{0600}'..='\u{06FF}' |
            '\u{0750}'..='\u{077F}' |
            '\u{08A0}'..='\u{08FF}' |
            '\u{FB50}'..='\u{FDFF}' |
            '\u{FE70}'..='\u{FEFF}' |
            // Hebrew
            '\u{0590}'..='\u{05FF}' |
            '\u{FB1D}'..='\u{FB4F}'
        )
    })
}
