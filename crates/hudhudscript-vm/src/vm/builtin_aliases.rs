//! Single source of truth for builtin function name aliases.
//!
//! Why this exists: print aliases were duplicated in both dispatch_core.rs
//! match arm and util.rs builtin_name_set (Kural 7 violation). Two lists
//! out of sync = VM recognizes but parser doesn't count as builtin = bug.
//! This file is the single source — both places read from here.
//!
//! Kural 8: Romanized/Latinized Japanese, Russian, Persian, Hindi etc. FORBIDDEN.
//! Languages with their own scripts use their own scripts.

/// Canonical English `print` + all native equivalents.
/// Each language: native print word, in its own writing system.
pub(crate) const PRINT_ALIASES: &[&str] = &[
    "print",       // English
    "yaz",         // Turkish
    "yazdır",      // Turkish (alt)
    "اطبع",         // Arabic
    "書く",         // Japanese (kanji)
    "表示",         // Japanese (kanji, display)
    "출력",         // Korean (Hangul)
    "drucken",     // German
    "drucke",      // German (alt)
    "imprimer",    // French
    "affiche",     // French (alt)
    "imprimir",    // Spanish + Portuguese
    "imprima",     // Portuguese (alt)
    "imprime",     // Spanish (conjugated form)
    "stampare",    // Italian
    "stampa",      // Italian (alt)
    "drukuj",      // Polish
    "cetak",       // Indonesian
    "печать",       // Russian (Cyrillic)
    "εκτύπωση",     // Greek
    "εκτύπωσε",     // Greek (conjugated form)
    "چاپ",          // Persian
    "प्रिंट",       // Hindi (Devanagari)
    "छाप",          // Hindi (alt)
    "প্রিন্ট",       // Bengali
    "ছাপ",          // Bengali (alt)
    "พิมพ์",         // Thai
    "打印",         // Chinese (Hanzi)
    "çap",         // Kurdish (short form)
    "çap_bike",    // Kurdish (single word — two-word version parser-incompatible)
    "ispiši",      // Serbian, Croatian, Bosnian (Latin)
    "ispis",       // Bosnian (alt)
    "štampaj",     // Bosnian, Serbian (Latin)
    "штампај",      // Serbian (Cyrillic)
    "испис",        // Serbian (Cyrillic)
    "in_ra",       // Vietnamese (`in` conflicts with for-in keyword)
];

/// Helper: check if a name is a print alias.
#[inline]
pub(crate) fn is_print_alias(name: &str) -> bool {
    PRINT_ALIASES.contains(&name)
}

/// `println` aliases — explicit newline print.
/// Currently same behavior as `print` since all prints add newline.
pub(crate) const PRINTLN_ALIASES: &[&str] = &[
    "println",     // English
    "satıryaz",     // Turkish
];

/// `eprint` aliases — print to stderr (no newline).
pub(crate) const EPRINT_ALIASES: &[&str] = &[
    "eprint",      // English
    "hatayaz",      // Turkish
];

/// `eprintln` aliases — print to stderr with newline.
pub(crate) const EPRINTLN_ALIASES: &[&str] = &[
    "eprintln",    // English
    "hatayazdır",   // Turkish
];

/// `input` aliases — read from stdin.
pub(crate) const INPUT_ALIASES: &[&str] = &[
    "input",       // English
    "oku",         // Turkish
    "gir",         // Turkish (alt)
    "eingabe",     // German
    "leer",        // Spanish
    "lire",        // French
];

/// `put` aliases — stdout write WITHOUT newline.
pub(crate) const PUT_ALIASES: &[&str] = &[
    "put",         // English
    "göster",      // Turkish
];

/// `putf` aliases — printf-style formatted write WITHOUT newline.
pub(crate) const PUTF_ALIASES: &[&str] = &[
    "putf",        // English
    "fgöster",     // Turkish
];

#[inline]
pub(crate) fn is_put_alias(name: &str) -> bool {
    PUT_ALIASES.contains(&name)
}

#[inline]
pub(crate) fn is_putf_alias(name: &str) -> bool {
    PUTF_ALIASES.contains(&name)
}

#[inline]
pub(crate) fn is_input_alias(name: &str) -> bool {
    INPUT_ALIASES.contains(&name)
}

#[inline]
pub(crate) fn is_println_alias(name: &str) -> bool {
    PRINTLN_ALIASES.contains(&name)
}

#[inline]
pub(crate) fn is_eprint_alias(name: &str) -> bool {
    EPRINT_ALIASES.contains(&name)
}

#[inline]
pub(crate) fn is_eprintln_alias(name: &str) -> bool {
    EPRINTLN_ALIASES.contains(&name)
}
