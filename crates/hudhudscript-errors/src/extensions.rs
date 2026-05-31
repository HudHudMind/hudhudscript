use crate::{ErrorCode, LocalizedErrorEntry};

impl ErrorCode {
    /// Look up embedded localized metadata for this error code.
    pub fn localized(self, locale: &str) -> LocalizedErrorEntry {
        crate::localized_error_entry(self, locale)
    }

    /// Localized title for the requested locale, with English fallback.
    pub fn title_in_locale(self, locale: &str) -> &'static str {
        self.localized(locale).title
    }

    /// Localized short description for the requested locale, with English fallback.
    pub fn short_description_in_locale(self, locale: &str) -> &'static str {
        self.localized(locale).short_description
    }

    /// Localized long description for the requested locale, with English fallback.
    pub fn long_description_in_locale(self, locale: &str) -> &'static str {
        self.localized(locale).long_description
    }

    /// Localized hints for the requested locale, with English fallback.
    pub fn hints_in_locale(self, locale: &str) -> Vec<&'static str> {
        self.localized(locale).hints
    }
}
