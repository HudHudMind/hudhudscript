//! Pluralization rules for i18n
//!
//! Implements CLDR-style plural categories and language-specific selection rules.

use std::collections::HashMap;

/// CLDR plural categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluralCategory {
    /// Exactly zero (used by Arabic, etc.)
    Zero,
    /// Exactly one (singular)
    One,
    /// Exactly two (dual form, used by Arabic)
    Two,
    /// A "few" items (used by Russian, Polish, Arabic, etc.)
    Few,
    /// A "many" items (used by Russian, Polish, Arabic, etc.)
    Many,
    /// Everything else / default
    Other,
}

/// Trait for language-specific plural selection.
pub trait PluralRule: Send + Sync {
    /// Given a numeric count, return the appropriate plural category.
    fn select(&self, count: f64) -> PluralCategory;
}

// ---------------------------------------------------------------------------
// Built-in rules
// ---------------------------------------------------------------------------

/// English pluralization: 1 -> One, everything else -> Other.
pub struct EnglishPlural;

impl PluralRule for EnglishPlural {
    fn select(&self, count: f64) -> PluralCategory {
        if (count - 1.0).abs() < f64::EPSILON {
            PluralCategory::One
        } else {
            PluralCategory::Other
        }
    }
}

/// Turkish pluralization: 1 -> One, everything else -> Other.
/// Turkish nouns don't take plural suffix when preceded by a number,
/// but for UI messages the one/other distinction is standard.
pub struct TurkishPlural;

impl PluralRule for TurkishPlural {
    fn select(&self, count: f64) -> PluralCategory {
        if (count - 1.0).abs() < f64::EPSILON {
            PluralCategory::One
        } else {
            PluralCategory::Other
        }
    }
}

/// Arabic pluralization (CLDR):
///   0 -> Zero, 1 -> One, 2 -> Two,
///   3-10 -> Few, 11-99 -> Many, rest -> Other.
pub struct ArabicPlural;

impl PluralRule for ArabicPlural {
    fn select(&self, count: f64) -> PluralCategory {
        let n = count.abs().floor() as u64;
        let mod100 = n % 100;
        match n {
            0 => PluralCategory::Zero,
            1 => PluralCategory::One,
            2 => PluralCategory::Two,
            _ if (3..=10).contains(&mod100) => PluralCategory::Few,
            _ if (11..=99).contains(&mod100) => PluralCategory::Many,
            _ => PluralCategory::Other,
        }
    }
}

/// Russian pluralization (CLDR):
///   mod10==1 && mod100!=11 -> One
///   mod10 in 2..4 && mod100 not in 12..14 -> Few
///   mod10==0 || mod10 in 5..9 || mod100 in 11..14 -> Many
///   else -> Other
pub struct RussianPlural;

impl PluralRule for RussianPlural {
    fn select(&self, count: f64) -> PluralCategory {
        let n = count.abs().floor() as u64;
        let mod10 = n % 10;
        let mod100 = n % 100;

        if mod10 == 1 && mod100 != 11 {
            PluralCategory::One
        } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
            PluralCategory::Few
        } else if mod10 == 0 || (5..=9).contains(&mod10) || (11..=14).contains(&mod100) {
            PluralCategory::Many
        } else {
            PluralCategory::Other
        }
    }
}

/// Look up the correct plural form of `key` in the catalog.
///
/// The catalog should contain keys suffixed with the plural category name:
///   `key.zero`, `key.one`, `key.two`, `key.few`, `key.many`, `key.other`
///
/// The `#` placeholder in the returned string is replaced with the count.
pub fn pluralize(
    key: &str,
    count: f64,
    rule: &dyn PluralRule,
    messages: &HashMap<String, String>,
) -> String {
    let category = rule.select(count);
    let suffix = match category {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    };

    let plural_key = format!("{}.{}", key, suffix);

    // Try exact category, then fall back to "other"
    let template = messages
        .get(&plural_key)
        .or_else(|| messages.get(&format!("{}.other", key)))
        .map(|s| s.as_str())
        .unwrap_or(key);

    // Replace `#` placeholder with count
    let count_str = if count.fract() == 0.0 {
        format!("{}", count as i64)
    } else {
        format!("{}", count)
    };

    template.replace('#', &count_str)
}
