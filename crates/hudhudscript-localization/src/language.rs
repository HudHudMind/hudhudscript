//! Language detection and configuration

use serde::{Deserialize, Serialize};

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// English (SVO - Subject-Verb-Object)
    English,
    /// Turkish (SOV - Subject-Object-Verb, agglutinative)
    Turkish,
    /// Japanese (SOV + Particles)
    Japanese,
    /// Arabic (VSO - Verb-Subject-Object, RTL)
    Arabic,
    /// Russian (SVO, flexible word order)
    Russian,
    /// Chinese (SVO, topic-prominent)
    Chinese,
    /// Spanish (SVO)
    Spanish,
    /// German (V2 - Verb-second, SOV in subordinate clauses)
    German,
    /// French (SVO)
    French,
    /// Italian (SVO)
    Italian,
    /// Portuguese (SVO)
    Portuguese,
    /// Polish (SVO)
    Polish,
    /// Thai (SVO, tonal language)
    Thai,
    /// Indonesian (SVO)
    Indonesian,
    /// Vietnamese (SVO, tonal language)
    Vietnamese,
    /// Greek (SVO)
    Greek,
    /// Serbian (SVO, Cyrillic script)
    Serbian,
    /// Bosnian (SVO, Latin script)
    Bosnian,
    /// Croatian (SVO, Latin script)
    Croatian,
    /// Kurdish (SOV, Kurmanji dialect)
    Kurdish,
    /// Persian/Farsi (SOV, RTL)
    Persian,
    /// Hindi (SOV, Devanagari script)
    Hindi,
    /// Bengali (SOV, Bengali script)
    Bengali,
    /// Korean (SOV, Hangul script)
    Korean,
}

impl Language {
    /// Get language code (ISO 639-1)
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Turkish => "tr",
            Language::Japanese => "ja",
            Language::Arabic => "ar",
            Language::Russian => "ru",
            Language::Chinese => "zh",
            Language::Spanish => "es",
            Language::German => "de",
            Language::French => "fr",
            Language::Italian => "it",
            Language::Portuguese => "pt",
            Language::Polish => "pl",
            Language::Thai => "th",
            Language::Indonesian => "id",
            Language::Vietnamese => "vi",
            Language::Greek => "el",
            Language::Serbian => "sr",
            Language::Bosnian => "bs",
            Language::Croatian => "hr",
            Language::Kurdish => "ku",
            Language::Persian => "fa",
            Language::Hindi => "hi",
            Language::Bengali => "bn",
            Language::Korean => "ko",
        }
    }

    /// Get language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Turkish => "Türkçe",
            Language::Japanese => "日本語",
            Language::Arabic => "العربية",
            Language::Russian => "Русский",
            Language::Chinese => "中文",
            Language::Spanish => "Español",
            Language::German => "Deutsch",
            Language::French => "Français",
            Language::Italian => "Italiano",
            Language::Portuguese => "Português",
            Language::Polish => "Polski",
            Language::Thai => "ไทย",
            Language::Indonesian => "Bahasa Indonesia",
            Language::Vietnamese => "Tiếng Việt",
            Language::Greek => "Ελληνικά",
            Language::Serbian => "Српски",
            Language::Bosnian => "Bosanski",
            Language::Croatian => "Hrvatski",
            Language::Kurdish => "Kurdî",
            Language::Persian => "فارسی",
            Language::Hindi => "हिन्दी",
            Language::Bengali => "বাংলা",
            Language::Korean => "한국어",
        }
    }

    /// Get word order type
    pub fn word_order(&self) -> WordOrder {
        match self {
            Language::English => WordOrder::SVO,
            Language::Turkish => WordOrder::SOV,
            Language::Japanese => WordOrder::SOV,
            Language::Arabic => WordOrder::VSO,
            Language::Russian => WordOrder::SVO,
            Language::Chinese => WordOrder::SVO,
            Language::Spanish => WordOrder::SVO,
            Language::German => WordOrder::V2,
            Language::French => WordOrder::SVO,
            Language::Italian => WordOrder::SVO,
            Language::Portuguese => WordOrder::SVO,
            Language::Polish => WordOrder::SVO,
            Language::Thai => WordOrder::SVO,
            Language::Indonesian => WordOrder::SVO,
            Language::Vietnamese => WordOrder::SVO,
            Language::Greek => WordOrder::SVO,
            Language::Serbian => WordOrder::SVO,
            Language::Bosnian => WordOrder::SVO,
            Language::Croatian => WordOrder::SVO,
            Language::Kurdish => WordOrder::SOV,
            Language::Persian => WordOrder::SOV,
            Language::Hindi => WordOrder::SOV,
            Language::Bengali => WordOrder::SOV,
            Language::Korean => WordOrder::SOV,
        }
    }

    /// Check if language uses particles
    pub fn uses_particles(&self) -> bool {
        matches!(self, Language::Japanese | Language::Chinese)
    }

    /// Check if language is agglutinative
    pub fn is_agglutinative(&self) -> bool {
        matches!(self, Language::Turkish | Language::Japanese)
    }

    /// Check if language is right-to-left
    pub fn is_rtl(&self) -> bool {
        matches!(
            self,
            Language::Arabic | Language::Persian | Language::Kurdish
        )
    }
}

/// Word order types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum WordOrder {
    /// Subject-Verb-Object (English, Russian, Chinese, Spanish, French, Italian, Portuguese)
    SVO,
    /// Subject-Object-Verb (Turkish, Japanese)
    SOV,
    /// Verb-Subject-Object (Arabic)
    VSO,
    /// Verb-second (German)
    V2,
}

/// Language detector
pub struct LanguageDetector;

impl LanguageDetector {
    /// Detect language from source code
    pub fn detect(source: &str) -> Language {
        // Simple heuristic-based detection
        // Check for language-specific scripts first (unambiguous), then keywords.

        // Hindi detection (Devanagari script: U+0900–U+097F)
        if source
            .chars()
            .any(|c| ('\u{0900}'..='\u{097F}').contains(&c))
        {
            return Language::Hindi;
        }

        // Bengali detection (Bengali script: U+0980–U+09FF)
        if source
            .chars()
            .any(|c| ('\u{0980}'..='\u{09FF}').contains(&c))
        {
            return Language::Bengali;
        }

        // Persian/Farsi detection: Persian-specific characters (U+06F0–U+06FF are Persian digits,
        // and Persian uses Arabic Extended block too). We distinguish by presence of
        // Persian-specific letters (پ, چ, ژ, گ) which do not appear in Arabic.
        if source
            .chars()
            .any(|c| matches!(c, '\u{067E}' | '\u{0686}' | '\u{0698}' | '\u{06AF}'))
        {
            return Language::Persian;
        }

        // Arabic detection (Arabic script: U+0600–U+06FF, but after Persian check above)
        if source
            .chars()
            .any(|c| ('\u{0600}'..='\u{06FF}').contains(&c))
        {
            return Language::Arabic;
        }

        // Korean detection (Hangul Syllables U+AC00–U+D7AF, Jamo U+1100–U+11FF)
        if source.chars().any(|c| {
            ('\u{AC00}'..='\u{D7AF}').contains(&c) || // Hangul Syllables
            ('\u{1100}'..='\u{11FF}').contains(&c) // Hangul Jamo
        }) {
            return Language::Korean;
        }

        // Japanese detection (hiragana/katakana characters)
        if source.chars().any(|c| {
            ('\u{3040}'..='\u{309F}').contains(&c) || // Hiragana
            ('\u{30A0}'..='\u{30FF}').contains(&c) // Katakana
        }) {
            return Language::Japanese;
        }

        // Chinese detection (CJK Unified Ideographs, after Japanese check)
        if source.chars().any(|c| {
            ('\u{4E00}'..='\u{9FFF}').contains(&c) || // CJK Unified Ideographs
            ('\u{3400}'..='\u{4DBF}').contains(&c) // CJK Extension A
        }) {
            return Language::Chinese;
        }

        // Serbian detection (Cyrillic with Serbian-specific letters: Љ, Њ, Џ, ђ, ћ, љ, њ, џ)
        // These characters are unique to Serbian/Bosnian Cyrillic and don't appear in Russian.
        if source.chars().any(|c| {
            matches!(
                c,
                '\u{0452}' | '\u{045B}' | '\u{045F}' | // ђ ћ џ
            '\u{0402}' | '\u{040B}' | '\u{040F}' // Ђ Ћ Џ
            )
        }) {
            return Language::Serbian;
        }

        // Russian detection (Cyrillic script, after Serbian check)
        if source
            .chars()
            .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
        {
            return Language::Russian;
        }

        // Kurdish detection (Latin-script Kurmanji keywords)
        if source.contains("bikar_bîne")
            || source.contains("ajans")
            || source.contains("eger") && source.contains("wekî_din")
        {
            return Language::Kurdish;
        }

        // Turkish detection (Turkish-specific characters or keywords)
        if source.contains("kullan")
            || source.contains("ajani")
            || source.contains("olarak")
            || source
                .chars()
                .any(|c| matches!(c, 'ı' | 'ğ' | 'ş' | 'İ' | 'Ğ' | 'Ş'))
        {
            return Language::Turkish;
        }

        // German detection (check before Spanish to avoid "agent" collision)
        if source.contains("verwenden") || (source.contains("wenn") && source.contains("sonst")) {
            return Language::German;
        }

        // French detection
        if source.contains("utiliser") || (source.contains("si") && source.contains("alors")) {
            return Language::French;
        }

        // Italian detection
        if source.contains("usare")
            || (source.contains("agente") && source.contains("se") && source.contains("allora"))
        {
            return Language::Italian;
        }

        // Portuguese detection (check before Spanish; Portuguese has unique markers)
        if source.contains("usar") && (source.contains("então") || source.contains("senão")) {
            return Language::Portuguese;
        }

        // Spanish detection
        if source.contains("usar")
            || source.contains("agente")
            || (source.contains("si") && source.contains("entonces"))
        {
            return Language::Spanish;
        }

        // Default to English
        Language::English
    }

    /// Get all supported languages
    pub fn supported_languages() -> Vec<Language> {
        vec![
            Language::English,
            Language::Turkish,
            Language::Japanese,
            Language::Arabic,
            Language::Russian,
            Language::Chinese,
            Language::Spanish,
            Language::German,
            Language::French,
            Language::Italian,
            Language::Portuguese,
            Language::Polish,
            Language::Thai,
            Language::Indonesian,
            Language::Vietnamese,
            Language::Greek,
            Language::Serbian,
            Language::Bosnian,
            Language::Croatian,
            Language::Kurdish,
            Language::Persian,
            Language::Hindi,
            Language::Bengali,
            Language::Korean,
        ]
    }
}
