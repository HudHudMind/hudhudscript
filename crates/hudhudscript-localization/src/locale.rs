//! Locale management — parsing, system detection, and fallback chains
//!
//! A [`Locale`] represents a BCP-47-style language tag (e.g. `tr-TR`, `en-US`).
//! [`FallbackChain`] determines the resolution order when a message is missing
//! in the primary locale.

/// A parsed locale identifier.
///
/// Supports forms like `en`, `en-US`, `zh-Hans-CN`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale {
    /// ISO 639-1 language code (lowercase), e.g. "en", "tr"
    pub language: String,
    /// Optional ISO 3166-1 region code (uppercase), e.g. "US", "TR"
    pub region: Option<String>,
    /// Optional script tag (title-case), e.g. "Hans", "Latn"
    pub script: Option<String>,
}

impl Locale {
    /// Parse a locale string such as `"tr-TR"`, `"en"`, or `"zh-Hans-CN"`.
    ///
    /// Supports `-` and `_` as separators. Components are normalized to
    /// conventional casing (language lowercase, script titlecase, region uppercase).
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        // Strip encoding suffix like ".UTF-8"
        let s = s.split('.').next().unwrap_or(s);
        let parts: Vec<&str> = s.split(['-', '_']).collect();

        let language = parts.first().map(|p| p.to_lowercase()).unwrap_or_default();

        let (script, region) = match parts.len() {
            // "en"
            1 => (None, None),
            // "en-US" or "zh-Hans"
            2 => {
                let second = parts[1];
                if second.len() == 4 && second.chars().next().is_some_and(|c| c.is_uppercase()) {
                    // Script tag (4 letters, starts uppercase): "Hans"
                    (Some(titlecase(second)), None)
                } else {
                    // Region code
                    (None, Some(second.to_uppercase()))
                }
            }
            // "zh-Hans-CN"
            _ => {
                let script = Some(titlecase(parts[1]));
                let region = Some(parts[2].to_uppercase());
                (script, region)
            }
        };

        Self {
            language,
            region,
            script,
        }
    }

    /// Format as a BCP-47-style tag: `"en"`, `"en-US"`, `"zh-Hans-CN"`.
    pub fn to_tag(&self) -> String {
        let mut tag = self.language.clone();
        if let Some(ref sc) = self.script {
            tag.push('-');
            tag.push_str(sc);
        }
        if let Some(ref rg) = self.region {
            tag.push('-');
            tag.push_str(rg);
        }
        tag
    }

    /// Detect the system locale by reading `LC_ALL`, `LC_MESSAGES`, or `LANG`
    /// environment variables (in that order).
    ///
    /// Returns English (`en`) if none are set.
    pub fn detect_system_locale() -> Self {
        let raw = std::env::var("LC_ALL")
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .or_else(|_| std::env::var("LANG"))
            .unwrap_or_else(|_| "en".to_string());

        // Strip encoding suffix (e.g. ".UTF-8") and handle "C" / "POSIX"
        let tag = raw.split('.').next().unwrap_or("en");
        let tag = if tag == "C" || tag == "POSIX" {
            "en"
        } else {
            tag
        };

        Self::parse(tag)
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_tag())
    }
}

fn titlecase(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => {
            let mut out = first.to_uppercase().to_string();
            for ch in c {
                out.extend(ch.to_lowercase());
            }
            out
        }
    }
}

// ---------------------------------------------------------------------------
// Fallback chain
// ---------------------------------------------------------------------------

/// A chain of locales to try when looking up translations.
///
/// For example, `tr-TR` -> `tr` -> `en`.
#[derive(Debug, Clone)]
pub struct FallbackChain {
    /// Ordered list of locale tags to try.
    chain: Vec<String>,
}

impl FallbackChain {
    /// Build a fallback chain from a primary locale.
    ///
    /// Automatically strips region, then script, then defaults to `"en"`.
    /// Duplicate entries are removed.
    pub fn new(primary: &Locale) -> Self {
        let mut chain = Vec::new();
        let full = primary.to_tag();

        chain.push(full.clone());

        // Without region
        if primary.region.is_some() {
            let mut stripped = primary.clone();
            stripped.region = None;
            let tag = stripped.to_tag();
            if !chain.contains(&tag) {
                chain.push(tag);
            }
        }

        // Without script and region (just language)
        if primary.script.is_some() {
            let tag = primary.language.clone();
            if !chain.contains(&tag) {
                chain.push(tag);
            }
        }

        // Default fallback
        let default = "en".to_string();
        if !chain.contains(&default) {
            chain.push(default);
        }

        Self { chain }
    }

    /// Build a custom fallback chain from explicit tags.
    pub fn from_tags(tags: Vec<String>) -> Self {
        Self { chain: tags }
    }

    /// Iterate over the chain in priority order.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.chain.iter().map(|s| s.as_str())
    }

    /// Get the chain as a slice of tags.
    pub fn tags(&self) -> &[String] {
        &self.chain
    }
}
