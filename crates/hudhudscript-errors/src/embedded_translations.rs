use crate::{ErrorCode, ErrorEntry, ERROR_TABLE};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
struct RawTranslatedErrorEntry {
    title: String,
    short_description: String,
    long_description: String,
    #[serde(default)]
    hints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EmbeddedErrorTranslation {
    pub title: String,
    pub short_description: String,
    pub long_description: String,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EmbeddedLocaleCatalog {
    locale: String,
    language: String,
    errors: HashMap<String, EmbeddedErrorTranslation>,
}

impl EmbeddedLocaleCatalog {
    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn get(&self, code: ErrorCode) -> Option<&EmbeddedErrorTranslation> {
        self.errors.get(code.short_code())
    }
}

#[derive(Debug, Clone)]
pub struct LocalizedErrorEntry {
    pub title: &'static str,
    pub short_description: &'static str,
    pub long_description: &'static str,
    pub hints: Vec<&'static str>,
}

const EMBEDDED_TRANSLATION_SOURCES: &[(&str, &str)] = &[
    (
        "ar",
        include_str!("../translations/errors_ar.json"),
    ),
    (
        "bn",
        include_str!("../translations/errors_bn.json"),
    ),
    (
        "bs",
        include_str!("../translations/errors_bs.json"),
    ),
    (
        "de",
        include_str!("../translations/errors_de.json"),
    ),
    (
        "el",
        include_str!("../translations/errors_el.json"),
    ),
    (
        "es",
        include_str!("../translations/errors_es.json"),
    ),
    (
        "fa",
        include_str!("../translations/errors_fa.json"),
    ),
    (
        "fr",
        include_str!("../translations/errors_fr.json"),
    ),
    (
        "hi",
        include_str!("../translations/errors_hi.json"),
    ),
    (
        "hr",
        include_str!("../translations/errors_hr.json"),
    ),
    (
        "id",
        include_str!("../translations/errors_id.json"),
    ),
    (
        "it",
        include_str!("../translations/errors_it.json"),
    ),
    (
        "ja",
        include_str!("../translations/errors_ja.json"),
    ),
    (
        "ko",
        include_str!("../translations/errors_ko.json"),
    ),
    (
        "ku",
        include_str!("../translations/errors_ku.json"),
    ),
    (
        "pl",
        include_str!("../translations/errors_pl.json"),
    ),
    (
        "pt-br",
        include_str!("../translations/errors_pt-BR.json"),
    ),
    (
        "ru",
        include_str!("../translations/errors_ru.json"),
    ),
    (
        "sr",
        include_str!("../translations/errors_sr.json"),
    ),
    (
        "th",
        include_str!("../translations/errors_th.json"),
    ),
    (
        "tr",
        include_str!("../translations/errors_tr.json"),
    ),
    (
        "vi",
        include_str!("../translations/errors_vi.json"),
    ),
    (
        "zh-cn",
        include_str!("../translations/errors_zh-CN.json"),
    ),
];

static EMBEDDED_CATALOGS: OnceLock<HashMap<String, EmbeddedLocaleCatalog>> = OnceLock::new();
static AVAILABLE_LOCALES: OnceLock<Vec<&'static str>> = OnceLock::new();

fn normalize_locale_tag(locale: &str) -> String {
    locale.trim().replace('_', "-").to_ascii_lowercase()
}

fn current_error_locale() -> Option<String> {
    std::env::var("HUDHUD_LOCALE")
        .ok()
        .map(|locale| normalize_locale_tag(&locale))
        .filter(|locale| !locale.is_empty() && locale != "default")
}

fn english_entry_for(short_code: &str) -> Option<&'static ErrorEntry> {
    let index = short_code
        .strip_prefix('E')?
        .parse::<usize>()
        .ok()?
        .checked_sub(1)?;
    ERROR_TABLE.get(index)
}

fn placeholder_like(code: &str, text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    let code_lower = code.to_ascii_lowercase();
    normalized.contains(&format!("error {code_lower}"))
        || normalized.contains("short desc for")
        || normalized.contains("short description for")
        || normalized.contains("long description for")
        || normalized.contains("description for e")
        || normalized.contains("hint 1")
        || normalized.contains("check documentation")
        || normalized.contains("to be completed")
}

fn usable_translation(
    code: &str,
    raw: RawTranslatedErrorEntry,
    english: &'static ErrorEntry,
) -> Option<EmbeddedErrorTranslation> {
    if raw.title.trim().is_empty()
        || raw.short_description.trim().is_empty()
        || raw.long_description.trim().is_empty()
    {
        return None;
    }

    // Hint SAYISI İngilizce ile eşleşmek zorunda DEĞİL: başlık/açıklama çevrilmişse
    // çeviri kullanılır. (Eskiden count mismatch tüm çeviriyi düşürüyordu → tüm
    // hatalar İngilizce görünüyordu. B-full bug bu.) Sadece boş-string hint reddedilir.
    if raw.hints.iter().any(|hint| hint.trim().is_empty()) {
        return None;
    }

    if placeholder_like(code, &raw.title)
        || placeholder_like(code, &raw.short_description)
        || placeholder_like(code, &raw.long_description)
        || raw.hints.iter().any(|hint| placeholder_like(code, hint))
    {
        return None;
    }

    let unchanged_main = raw.title == english.title
        && raw.short_description == english.short_description
        && raw.long_description == english.long_description;
    let unchanged_hints = raw
        .hints
        .iter()
        .map(String::as_str)
        .eq(english.hints.iter().copied());
    if unchanged_main && unchanged_hints {
        return None;
    }

    Some(EmbeddedErrorTranslation {
        title: raw.title,
        short_description: raw.short_description,
        long_description: raw.long_description,
        hints: raw.hints,
    })
}

fn load_embedded_catalogs() -> HashMap<String, EmbeddedLocaleCatalog> {
    let mut catalogs = HashMap::new();

    for (embedded_key, json_text) in EMBEDDED_TRANSLATION_SOURCES {
        let raw_value = match serde_json::from_str::<serde_json::Value>(json_text) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let Some(raw_object) = raw_value.as_object() else {
            continue;
        };

        let locale = raw_object
            .get("locale")
            .and_then(|value| value.as_str())
            .map(normalize_locale_tag)
            .filter(|locale| !locale.is_empty())
            .unwrap_or_else(|| (*embedded_key).to_string());

        let language = raw_object
            .get("language")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .filter(|language| !language.trim().is_empty())
            .unwrap_or_else(|| locale.clone());

        let mut errors = HashMap::new();
        let Some(raw_errors) = raw_object.get("errors").and_then(|value| value.as_object()) else {
            continue;
        };
        for (short_code, raw_entry_value) in raw_errors {
            let Some(english) = english_entry_for(short_code) else {
                continue;
            };
            let Ok(raw_entry) =
                serde_json::from_value::<RawTranslatedErrorEntry>(raw_entry_value.clone())
            else {
                continue;
            };

            if let Some(translation) = usable_translation(short_code, raw_entry, english) {
                errors.insert(short_code.clone(), translation);
            }
        }

        catalogs.insert(
            locale,
            EmbeddedLocaleCatalog {
                locale: raw_object
                    .get("locale")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| embedded_key.to_string()),
                language,
                errors,
            },
        );
    }

    catalogs
}

fn embedded_catalogs() -> &'static HashMap<String, EmbeddedLocaleCatalog> {
    EMBEDDED_CATALOGS.get_or_init(load_embedded_catalogs)
}

fn resolve_catalog(locale: &str) -> Option<&'static EmbeddedLocaleCatalog> {
    let normalized = normalize_locale_tag(locale);
    let catalogs = embedded_catalogs();
    if let Some(catalog) = catalogs.get(&normalized) {
        return Some(catalog);
    }

    let primary = normalized.split('-').next().unwrap_or("");
    if primary.is_empty() {
        return None;
    }

    if let Some(catalog) = catalogs.get(primary) {
        return Some(catalog);
    }

    let mut matches = catalogs
        .iter()
        .filter(|(key, _)| key.split('-').next().unwrap_or("") == primary);
    let first = matches.next()?;
    if matches.next().is_none() {
        Some(first.1)
    } else {
        None
    }
}

pub fn available_embedded_error_locales() -> &'static [&'static str] {
    AVAILABLE_LOCALES
        .get_or_init(|| {
            let mut locales: Vec<&'static str> = EMBEDDED_TRANSLATION_SOURCES
                .iter()
                .map(|(locale, _)| *locale)
                .collect();
            locales.sort_unstable();
            locales
        })
        .as_slice()
}

pub fn embedded_error_catalog(locale: &str) -> Option<&'static EmbeddedLocaleCatalog> {
    resolve_catalog(locale)
}

pub fn active_embedded_error_catalog() -> Option<&'static EmbeddedLocaleCatalog> {
    resolve_catalog(&current_error_locale()?)
}

pub fn localized_error_entry(code: ErrorCode, locale: &str) -> LocalizedErrorEntry {
    let english = code.entry();
    if let Some(catalog) = resolve_catalog(locale) {
        if let Some(translation) = catalog.get(code) {
            return LocalizedErrorEntry {
                title: translation.title.as_str(),
                short_description: translation.short_description.as_str(),
                long_description: translation.long_description.as_str(),
                hints: translation.hints.iter().map(String::as_str).collect(),
            };
        }
    }

    LocalizedErrorEntry {
        title: english.title,
        short_description: english.short_description,
        long_description: english.long_description,
        hints: english.hints.to_vec(),
    }
}

pub(crate) fn active_locale_tag() -> Option<String> {
    current_error_locale()
}

/// G1: "E0182" short_code + locale → çevrili başlık/açıklama. Bulunamazsa None.
pub fn localized_by_short_code(short_code: &str, locale: &str) -> Option<LocalizedErrorEntry> {
    let num = short_code.strip_prefix('E')?.parse::<u32>().ok()?;
    if num == 0 {
        return None;
    }
    let code = ErrorCode(num);
    if locale == "en" {
        return None;
    }
    Some(localized_error_entry(code, locale))
}
