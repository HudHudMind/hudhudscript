use serde::{Deserialize, Serialize};

use crate::{ErrorCode, ErrorEntry, LocalizedErrorEntry, SourcePosition};

/// The runtime representation of an error: a stable [`ErrorCode`] from the
/// catalog, plus the per-occurrence context (formatted message, source
/// position, and arbitrary key/value context fields).
///
/// This is the type that flows through `Result<T, Error>` everywhere in the
/// pipeline. The catalog metadata (title, descriptions, category) is reachable
/// via `error.code.entry()` without storing it inline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Error {
    /// Stable code identifying which catalog entry this error is.
    pub code: ErrorCode,
    /// Formatted runtime message — usually substitutes values into the catalog's
    /// short_description template (e.g. `"unexpected character '@'"`).
    pub message: String,
    /// Where in the source the error was detected.
    pub position: Option<SourcePosition>,
    /// Free-form key/value context (`("variable", "x")`, `("limit", "10")`, ...).
    pub context: Vec<(String, String)>,
    /// Optional typed payload for variants that carry runtime values
    /// (e.g. `Return(Value)`, `Throw(Value)`, `Yield(Value)`). The payload
    /// is opaque to the catalog — consumers downcast via [`ErrorPayload::downcast_ref`].
    /// (v0.4.48 — TAM CONSOLIDATION: needed because the unified `Error`
    /// type cannot embed phase-specific types like `Value` directly.)
    #[serde(default, skip)]
    pub payload: ErrorPayload,
}

/// Opaque payload slot for an [`Error`]. Used by phase crates that need to
/// round-trip a typed value (e.g. an interpreter `Value` for a `Return`
/// signal) through the unified error channel without dragging the phase
/// type into the errors crate.
#[derive(Clone, Default)]
pub struct ErrorPayload(pub Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>);

impl ErrorPayload {
    /// Construct an empty payload.
    pub fn empty() -> Self {
        Self(None)
    }

    /// Wrap a value in a new payload.
    pub fn new<T: std::any::Any + Send + Sync>(value: T) -> Self {
        Self(Some(std::sync::Arc::new(value)))
    }

    /// Returns true if no payload is attached.
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Attempt to downcast the payload to a reference of type `T`.
    pub fn downcast_ref<T: std::any::Any + Send + Sync>(&self) -> Option<&T> {
        self.0.as_ref().and_then(|arc| arc.downcast_ref::<T>())
    }

    /// Get the inner Arc, if any.
    pub fn as_arc(&self) -> Option<std::sync::Arc<dyn std::any::Any + Send + Sync>> {
        self.0.clone()
    }
}

impl std::fmt::Debug for ErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_some() {
            write!(f, "ErrorPayload(<opaque>)")
        } else {
            write!(f, "ErrorPayload(None)")
        }
    }
}

// Payload is excluded from equality / serialization. Two errors with the
// same code/message/position/context compare equal regardless of payload.
impl PartialEq for ErrorPayload {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for ErrorPayload {}

impl Serialize for ErrorPayload {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_none()
    }
}

impl<'de> Deserialize<'de> for ErrorPayload {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> std::result::Result<Self, D::Error> {
        Ok(Self::default())
    }
}

impl Error {
    /// Construct a new error from a catalog code and a formatted message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            position: None,
            context: Vec::new(),
            payload: ErrorPayload::empty(),
        }
    }

    /// Construct a new error using the catalog's `short_description` as the message.
    pub fn from_code(code: ErrorCode) -> Self {
        Self {
            code,
            message: code.entry().short_description.to_string(),
            position: None,
            context: Vec::new(),
            payload: ErrorPayload::empty(),
        }
    }

    /// Attach a typed payload to this error. Used by phase crates to
    /// round-trip values (e.g. interpreter `Value` in a `Return` signal).
    pub fn with_payload<T: std::any::Any + Send + Sync>(mut self, value: T) -> Self {
        self.payload = ErrorPayload::new(value);
        self
    }

    /// Downcast the payload to a reference of type `T`.
    pub fn payload_ref<T: std::any::Any + Send + Sync>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }

    /// Attach a source position.
    pub fn at(mut self, position: SourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Attach a position only if `Some`.
    pub fn maybe_at(mut self, position: Option<SourcePosition>) -> Self {
        if let Some(p) = position {
            self.position = Some(p);
        }
        self
    }

    /// Add a context key/value pair.
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }

    /// Look up a context value by key. Returns the first match.
    pub fn context_get(&self, key: &str) -> Option<&str> {
        self.context
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Reach the static catalog entry for this error.
    pub fn entry(&self) -> &'static ErrorEntry {
        self.code.entry()
    }

    /// Catalog long code (e.g. `"HHS_E_LEX_UNEXPECTED_CHAR"`).
    pub fn long_code(&self) -> &'static str {
        self.code.long_code()
    }

    /// Catalog short code (e.g. `"E0001"`).
    pub fn short_code(&self) -> &'static str {
        self.code.short_code()
    }

    /// Catalog title (e.g. `"Unexpected Char"`).
    pub fn title(&self) -> &'static str {
        self.code.title()
    }

    /// Catalog category (e.g. `ErrorCategory::Lex`).
    pub fn category(&self) -> crate::ErrorCategory {
        self.code.category()
    }

    /// Catalog short description.
    pub fn short_description(&self) -> &'static str {
        self.code.short_description()
    }

    /// Catalog long description.
    pub fn long_description(&self) -> &'static str {
        self.code.long_description()
    }

    /// Catalog hints.
    pub fn hints(&self) -> &'static [&'static str] {
        self.code.hints()
    }

    /// Optional snippet of code that triggers this issue.
    pub fn example_bad(&self) -> Option<&'static str> {
        self.entry().example_bad
    }

    /// Optional snippet of corrected code.
    pub fn example_good(&self) -> Option<&'static str> {
        self.entry().example_good
    }

    /// Related codes the user may want to consult.
    pub fn see_also(&self) -> &'static [&'static str] {
        self.entry().see_also
    }

    /// Version this code was introduced in.
    pub fn since_version(&self) -> &'static str {
        self.entry().since_version
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.entry();
        let localized = crate::embedded_translations::active_locale_tag()
            .map(|locale| self.code.localized(locale.as_str()));
        let title = localized
            .as_ref()
            .map_or(entry.title, |localized| localized.title);
        let message = if self.message == entry.short_description {
            localized
                .as_ref()
                .map_or(entry.short_description, |localized| {
                    localized.short_description
                })
        } else {
            self.message.as_str()
        };
        write!(f, "[{}] {}: {}", entry.short_code, title, message)?;
        if let Some(ref pos) = self.position {
            write!(f, " at {}", pos)?;
        }
        Ok(())
    }
}

impl Error {
    pub fn localized(&self, locale: &str) -> LocalizedErrorEntry {
        self.code.localized(locale)
    }

    /// Render this error in full.
    pub fn render_full(&self) -> String {
        if let Some(locale) = crate::embedded_translations::active_locale_tag() {
            return self.render_full_in_locale(locale.as_str());
        }
        self.render_full_with_localized(None)
    }

    /// Render the error using an embedded locale catalog.
    pub fn render_full_in_locale(&self, locale: &str) -> String {
        self.render_full_with_localized(Some(self.code.localized(locale)))
    }

    fn render_full_with_localized(&self, localized: Option<LocalizedErrorEntry>) -> String {
        use std::fmt::Write;
        let entry = self.entry();
        let title = localized
            .as_ref()
            .map_or(entry.title, |localized| localized.title);
        let short_description = localized
            .as_ref()
            .map_or(entry.short_description, |localized| {
                localized.short_description
            });
        let long_description = localized
            .as_ref()
            .map_or(entry.long_description, |localized| {
                localized.long_description
            });
        let hints: Vec<&str> = localized
            .as_ref()
            .map_or_else(|| entry.hints.to_vec(), |localized| localized.hints.clone());
        let mut s = String::new();
        let _ = writeln!(s, "[{}] {}", entry.short_code, title);
        let prefix = format!("[{}] {} — ", entry.short_code, entry.title);
        let localized_prefix = format!("[{}] {} — ", entry.short_code, title);
        let body = self
            .message
            .strip_prefix(&prefix)
            .or_else(|| self.message.strip_prefix(&localized_prefix))
            .unwrap_or(&self.message);
        let body = if body == entry.short_description {
            short_description
        } else {
            body
        };
        if !body.is_empty() && body != short_description {
            let _ = writeln!(s, "  {}", body);
        }
        if let Some(ref pos) = self.position {
            let _ = writeln!(s, "  at {}", pos);
        }
        let _ = writeln!(s);
        let _ = writeln!(s, "  {}", short_description);
        if !long_description.is_empty() && long_description != short_description {
            let _ = writeln!(s);
            for line in long_description.lines() {
                let _ = writeln!(s, "  {}", line);
            }
        }
        if !hints.is_empty() {
            let _ = writeln!(s);
            let _ = writeln!(s, "  hints:");
            for h in hints {
                let _ = writeln!(s, "    • {}", h);
            }
        }
        if let Some(bad) = entry.example_bad {
            let bad: &str = bad;
            let _ = writeln!(s);
            let _ = writeln!(s, "  example (incorrect):");
            for line in bad.lines() {
                let _ = writeln!(s, "    {}", line);
            }
        }
        if let Some(good) = entry.example_good {
            let good: &str = good;
            let _ = writeln!(s);
            let _ = writeln!(s, "  example (corrected):");
            for line in good.lines() {
                let _ = writeln!(s, "    {}", line);
            }
        }
        if !entry.see_also.is_empty() {
            let _ = writeln!(s);
            let _ = write!(s, "  see also:");
            for sa in entry.see_also {
                let _ = write!(s, " {}", sa);
            }
            let _ = writeln!(s);
        }
        let _ = writeln!(s);
        let _ = writeln!(
            s,
            "  long code: {}  |  category: {}  |  since: {}",
            entry.long_code, entry.category, entry.since_version
        );
        s
    }
}

impl std::error::Error for Error {}

/// Render an error with source code snippet showing the problematic line.
pub fn render_with_source(error: &Error, source: &str) -> String {
    let mut output = error.render_full();

    if let Some(ref pos) = error.position {
        let line_num = pos.line;
        if line_num > 0 {
            if let Some(source_line) = source.lines().nth(line_num - 1) {
                output.push_str(&format!("\n  |\n{:>3}| {}\n  |", line_num, source_line));
                if pos.column > 0 {
                    output.push_str(&format!(" {}^", " ".repeat(pos.column - 1)));
                }
                output.push('\n');
            }
        }
    }

    output
}

/// Render an error with source context using the requested locale.
pub fn render_with_source_in_locale(error: &Error, source: &str, locale: &str) -> String {
    let mut output = error.render_full_in_locale(locale);

    if let Some(ref pos) = error.position {
        let line_num = pos.line;
        if line_num > 0 {
            if let Some(source_line) = source.lines().nth(line_num - 1) {
                output.push_str(&format!("\n  |\n{:>3}| {}\n  |", line_num, source_line));
                if pos.column > 0 {
                    output.push_str(&format!(" {}^", " ".repeat(pos.column - 1)));
                }
                output.push('\n');
            }
        }
    }

    output
}
