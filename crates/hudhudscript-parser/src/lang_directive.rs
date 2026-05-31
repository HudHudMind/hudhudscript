//! Shebang-style language directive parser.
//!
//! Recognizes `#!lang=tr`, `#!dil=tr`, `#!langue=fr`, etc. on the first line
//! of a source file and extracts the locale code.

/// All recognized words for "language" across supported locales.
const LANG_KEYS: &[&str] = &[
    // English
    "lang",
    // Turkish
    "dil",
    // Japanese
    "言語",
    "gengo",
    // Arabic
    "لغة",
    "lugha",
    // Spanish / Portuguese (shared word)
    "idioma",
    // German
    "sprache",
    // French
    "langue",
    // Russian
    "язык",
    "yazyk",
    // Chinese
    "语言",
    "yuyan",
    // Italian
    "lingua",
    // Korean
    "언어",
    "eoneo",
    // Indonesian / Malay
    "bahasa",
    // Vietnamese
    "ngôn_ngữ",
    // Thai
    "ภาษา",
    // Greek
    "γλώσσα",
    // Bosnian / Croatian / Serbian
    "jezik",
    // Polish
    "język",
    // Persian / Farsi
    "زبان",
    // Kurdish
    "ziman",
];

/// Parse a language directive from the first line of source.
///
/// The directive must appear on the very first line and start with `#!`.
/// Returns the locale code (e.g. `"tr"`, `"ja"`, `"ar"`) if a valid directive is found.
///
/// # Examples
///
/// ```
/// use hudhudscript_parser::parse_lang_directive;
///
/// assert_eq!(parse_lang_directive("#!lang=tr\nlet x = 1;"), Some("tr".to_string()));
/// assert_eq!(parse_lang_directive("#!dil=tr\nlet x = 1;"), Some("tr".to_string()));
/// assert_eq!(parse_lang_directive("#!langue=fr\nlet x = 1;"), Some("fr".to_string()));
/// assert_eq!(parse_lang_directive("let x = 1;"), None);
/// ```
pub fn parse_lang_directive(source: &str) -> Option<String> {
    let first_line = source.lines().next()?;
    let trimmed = first_line.trim();

    if !trimmed.starts_with("#!") {
        return None;
    }

    let directive = &trimmed[2..]; // Remove "#!"

    // Sort keys by length descending so longer keys match first
    // (e.g. "ngôn_ngữ" before "ng..." if there were a collision).
    let mut keys: Vec<&str> = LANG_KEYS.to_vec();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));

    for key in &keys {
        if let Some(rest) = directive.strip_prefix(key) {
            // Allow optional whitespace around '='
            let rest = rest.trim_start();
            let rest = rest.strip_prefix('=')?;
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }

    None
}

/// Strip the language directive line from source, replacing it with an empty line
/// to preserve line numbers for error reporting.
pub fn strip_lang_directive(source: &str) -> String {
    if parse_lang_directive(source).is_some() {
        // Replace first line with empty line to keep line numbers stable
        match source.find('\n') {
            Some(pos) => {
                let mut result = String::with_capacity(source.len());
                // Preserve the newline so line numbering stays correct
                result.push_str(&source[pos..]);
                result
            }
            None => {
                // The entire source is just the directive — return empty
                String::new()
            }
        }
    } else {
        source.to_string()
    }
}
