//! Yerleşik fonksiyon ad eş anlamlılarının TEK DOĞRULUK KAYNAĞI (v0.9.34,
//! JIT Full Expansion M1). Tablolar VM'den buraya taşındı: hem VM dispatch'i
//! hem JIT/AST lowering hattı aynı listeyi okur — iki liste ayrışınca
//! "VM tanıyor ama JIT bilmiyor" hatası doğar.
//!
//! Kural 8: Romanize/Latinize Japonca, Rusça, Farsça, Hintçe YASAK —
//! kendi yazı sistemi olan dil kendi yazısını kullanır.
//!
//! `canonical_builtin` yerelleştirilmiş adı kanonik İngilizce ada çözümler;
//! çözümlenemeyen ad None döner (çağıran, kullanıcı fonksiyonu olduğunu varsayar).

/// Kanonik İngilizce `print` + tüm yerel karşılıkları.
pub const PRINT_ALIASES: &[&str] = &[
    "print",    // English
    "yaz",      // Turkish
    "yazdır",   // Turkish (alt)
    "اطبع",     // Arabic
    "書く",     // Japanese (kanji)
    "表示",     // Japanese (kanji, display)
    "출력",     // Korean (Hangul)
    "drucken",  // German
    "drucke",   // German (alt)
    "imprimer", // French
    "affiche",  // French (alt)
    "imprimir", // Spanish + Portuguese
    "imprima",  // Portuguese (alt)
    "imprime",  // Spanish (conjugated form)
    "stampare", // Italian
    "stampa",   // Italian (alt)
    "drukuj",   // Polish
    "cetak",    // Indonesian
    "печать",   // Russian (Cyrillic)
    "εκτύπωση", // Greek
    "εκτύπωσε", // Greek (conjugated form)
    "چاپ",      // Persian
    "प्रिंट",     // Hindi (Devanagari)
    "छाप",      // Hindi (alt)
    "প্রিন্ট",    // Bengali
    "ছাপ",       // Bengali (alt)
    "พิมพ์",      // Thai
    "打印",     // Chinese (Hanzi)
    "çap",      // Kurdish (short form)
    "çap_bike", // Kurdish (tek kelime — iki kelimelik biçim parser ile uyumsuz)
    "ispiši",   // Serbian, Croatian, Bosnian (Latin)
    "ispis",    // Bosnian (alt)
    "štampaj",  // Bosnian, Serbian (Latin)
    "штампај",  // Serbian (Cyrillic)
    "испис",    // Serbian (Cyrillic)
    "in_ra",    // Vietnamese (`in` for-in anahtar kelimesiyle çakışır)
];

/// `println` eş anlamlıları — açık satır sonu ile yazım.
pub const PRINTLN_ALIASES: &[&str] = &[
    "println",  // English
    "satıryaz", // Turkish
];

/// `eprint` eş anlamlıları — stderr'e yazım (satır sonu yok).
pub const EPRINT_ALIASES: &[&str] = &[
    "eprint",  // English
    "hatayaz", // Turkish
];

/// `eprintln` eş anlamlıları — stderr'e satır sonu ile yazım.
pub const EPRINTLN_ALIASES: &[&str] = &[
    "eprintln",   // English
    "hatayazdır", // Turkish
];

/// `input` eş anlamlıları — stdin okuma.
pub const INPUT_ALIASES: &[&str] = &[
    "input",   // English
    "oku",     // Turkish
    "gir",     // Turkish (alt)
    "eingabe", // German
    "leer",    // Spanish
    "lire",    // French
];

/// `put` eş anlamlıları — stdout'a satır sonu OLMADAN yazım.
pub const PUT_ALIASES: &[&str] = &[
    "put",    // English
    "göster", // Turkish
];

/// `putf` eş anlamlıları — printf biçimli, satırsız yazım.
pub const PUTF_ALIASES: &[&str] = &[
    "putf",    // English
    "fgöster", // Turkish
];

/// Yerelleştirilmiş adı kanonik ada çözümler (print ailesi + diğerleri).
/// None = bilinen yerleşik eş anlamlısı değil.
pub fn canonical_builtin(name: &str) -> Option<&'static str> {
    if PRINT_ALIASES.contains(&name) {
        return Some("print");
    }
    if PRINTLN_ALIASES.contains(&name) {
        return Some("println");
    }
    if EPRINT_ALIASES.contains(&name) {
        return Some("eprint");
    }
    if EPRINTLN_ALIASES.contains(&name) {
        return Some("eprintln");
    }
    if INPUT_ALIASES.contains(&name) {
        return Some("input");
    }
    if PUT_ALIASES.contains(&name) {
        return Some("put");
    }
    if PUTF_ALIASES.contains(&name) {
        return Some("putf");
    }
    None
}

#[inline]
pub fn is_print_alias(name: &str) -> bool {
    PRINT_ALIASES.contains(&name)
}

#[inline]
pub fn is_println_alias(name: &str) -> bool {
    PRINTLN_ALIASES.contains(&name)
}

#[inline]
pub fn is_eprint_alias(name: &str) -> bool {
    EPRINT_ALIASES.contains(&name)
}

#[inline]
pub fn is_eprintln_alias(name: &str) -> bool {
    EPRINTLN_ALIASES.contains(&name)
}

#[inline]
pub fn is_input_alias(name: &str) -> bool {
    INPUT_ALIASES.contains(&name)
}

#[inline]
pub fn is_put_alias(name: &str) -> bool {
    PUT_ALIASES.contains(&name)
}

#[inline]
pub fn is_putf_alias(name: &str) -> bool {
    PUTF_ALIASES.contains(&name)
}
