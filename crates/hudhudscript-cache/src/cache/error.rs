use hudhudscript_governance::{ConstitutionId, LawId, RuleId};
use std::fmt;

/// Cache errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    ConstitutionNotFound(ConstitutionId),
    LawNotFound(LawId),
    RuleNotFound(RuleId),
    IdCollision(String),
    SerializationError(String),
    DeserializationError(String),
    QuotaExceeded(String),
    DuplicateContent { key: String, existing: String },
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            CacheError::ConstitutionNotFound(id) => write!(f, "Constitution not found: {}", id),
            CacheError::LawNotFound(id) => write!(f, "Law not found: {}", id),
            CacheError::RuleNotFound(id) => write!(f, "Rule not found: {}", id),
            CacheError::IdCollision(s) => write!(f, "Cache ID collision: {}", s),
            CacheError::SerializationError(s) => write!(f, "Serialization error: {}", s),
            CacheError::DeserializationError(s) => write!(f, "Deserialization error: {}", s),
            CacheError::QuotaExceeded(s) => write!(f, "Quota exceeded: {}", s),
            CacheError::DuplicateContent { key, existing } => write!(
                f,
                "Duplicate content detected for key {}: already exists as {}",
                key, existing
            ),
        }
    }
}

impl std::error::Error for CacheError {}

impl CacheError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            CacheError::ConstitutionNotFound(..) => {
                hudhudscript_errors::ErrorCode::CacheConstitutionNotFound
            }
            CacheError::DeserializationError(..) => {
                hudhudscript_errors::ErrorCode::CacheDeserializationError
            }
            CacheError::DuplicateContent { .. } => {
                hudhudscript_errors::ErrorCode::CacheDuplicateContent
            }
            CacheError::IdCollision(..) => hudhudscript_errors::ErrorCode::CacheIdCollision,
            CacheError::LawNotFound(..) => hudhudscript_errors::ErrorCode::CacheLawNotFound,
            CacheError::QuotaExceeded(..) => hudhudscript_errors::ErrorCode::CacheQuotaExceeded,
            CacheError::RuleNotFound(..) => hudhudscript_errors::ErrorCode::CacheRuleNotFound,
            CacheError::SerializationError(..) => {
                hudhudscript_errors::ErrorCode::CacheSerializationError
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<CacheError> for hudhudscript_errors::Error {
    fn from(e: CacheError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
