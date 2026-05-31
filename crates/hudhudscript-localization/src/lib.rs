//! HudHudScript Localization Engine
//!
//! Multi-language support for natural language programming.
//! Supports 23 languages including English, Turkish, Japanese, Arabic, Russian, Chinese,
//! Spanish, German, French, Italian, Portuguese, Polish, Thai, Indonesian, Vietnamese,
//! Greek, Serbian, Bosnian, Croatian, Kurdish, Persian, Hindi, and Bengali.
//!
//! ## i18n / multi-language message support
//!
//! - [`catalog::MessageCatalog`] — load translated messages from JSON/YAML
//! - [`plural`] — CLDR-style pluralization rules for English, Turkish, Arabic, Russian
//! - [`interpolation::interpolate`] — variable substitution and inline plural expressions
//! - [`locale::Locale`] — BCP-47 locale parsing, system detection, fallback chains

mod catalog;
mod interpolation;
mod keyword_map;
mod language;
mod languages;
mod locale;
mod plural;
mod translator;

pub use catalog::{CatalogError, MessageCatalog};
pub use interpolation::interpolate;
pub use keyword_map::{Keyword, KeywordMap};
pub use language::{Language, LanguageDetector, WordOrder};
pub use locale::{FallbackChain, Locale};
pub use plural::{
    pluralize, ArabicPlural, EnglishPlural, PluralCategory, PluralRule, RussianPlural,
    TurkishPlural,
};
pub use translator::Translator;
