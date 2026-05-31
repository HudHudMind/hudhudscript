use hudhudscript_errors::{Error, ErrorCode, SourcePosition};
use serde::{Deserialize, Serialize};

use crate::frame::StackFrame;
use crate::{ExceptionCode, ExceptionEntry};

/// A runtime exception. Carries an [`ExceptionCode`] from the catalog along
/// with a formatted message, source position, cause chain, stack, and hints.
///
/// `Exception` mirrors [`hudhudscript_errors::Error`] structurally — both
/// reference the same code identifier — but adds runtime concerns
/// (cause chain, call stack) that don't make sense for a static error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exception {
    /// Stable code identifying which catalog entry this exception is.
    pub code: ExceptionCode,
    /// Formatted runtime message.
    pub message: String,
    /// Source position where the exception was raised.
    pub position: Option<SourcePosition>,
    /// Underlying cause, if this exception was raised in response to another.
    pub cause: Option<Box<Exception>>,
    /// Call stack at the point the exception was raised.
    pub stack: Vec<StackFrame>,
    /// Free-form hints that may help the user resolve the issue.
    pub hints: Vec<String>,
}

impl Exception {
    /// Construct a new exception from a catalog code and a formatted message.
    pub fn new(code: ExceptionCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            position: None,
            cause: None,
            stack: Vec::new(),
            hints: Vec::new(),
        }
    }

    /// Construct a new exception using the catalog's `short_description`.
    pub fn from_code(code: ExceptionCode) -> Self {
        Self {
            code,
            message: code.entry().short_description.to_string(),
            position: None,
            cause: None,
            stack: Vec::new(),
            hints: Vec::new(),
        }
    }

    /// Attach a source position.
    pub fn at(mut self, position: SourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    pub fn maybe_at(mut self, position: Option<SourcePosition>) -> Self {
        if let Some(p) = position {
            self.position = Some(p);
        }
        self
    }

    /// Chain a causing exception (typically the lower-level error that
    /// triggered this one).
    pub fn caused_by(mut self, cause: Exception) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Push a frame onto the call stack.
    pub fn push_frame(mut self, frame: StackFrame) -> Self {
        self.stack.push(frame);
        self
    }

    /// Attach a hint.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    /// Reach the static catalog entry for this exception.
    pub fn entry(&self) -> &'static ExceptionEntry {
        self.code.entry()
    }

    pub fn long_code(&self) -> &'static str {
        self.code.long_code()
    }

    pub fn short_code(&self) -> &'static str {
        self.code.short_code()
    }

    pub fn title(&self) -> &'static str {
        self.code.title()
    }

    pub fn category(&self) -> crate::ExceptionCategory {
        self.code.category()
    }

    /// Catalog short description.
    pub fn short_description(&self) -> &'static str {
        self.code.short_description()
    }

    /// Catalog long description (multi-paragraph).
    pub fn long_description(&self) -> &'static str {
        self.code.long_description()
    }

    /// Catalog static hints (in addition to the per-instance `self.hints`).
    pub fn catalog_hints(&self) -> &'static [&'static str] {
        self.code.hints()
    }

    /// Optional bad-code example from the catalog.
    pub fn example_bad(&self) -> Option<&'static str> {
        self.entry().example_bad
    }

    /// Optional fixed-code example from the catalog.
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

    /// Project this exception into its sibling [`Error`] from the errors
    /// crate. Discards runtime context (cause, stack, hints) and keeps only
    /// the catalog code, message, and position.
    pub fn to_error(&self) -> Error {
        Error {
            code: self.code.as_error_code(),
            message: self.message.clone(),
            position: self.position.clone(),
            context: Vec::new(),
            payload: hudhudscript_errors::ErrorPayload::empty(),
        }
    }
}

impl std::fmt::Display for Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.entry();
        write!(
            f,
            "[{}] {}: {}",
            entry.short_code, entry.title, self.message
        )?;
        if let Some(ref pos) = self.position {
            write!(f, " at {}", pos)?;
        }
        for frame in &self.stack {
            write!(f, "\n{}", frame)?;
        }
        for hint in &self.hints {
            write!(f, "\n    hint: {}", hint)?;
        }
        if let Some(ref cause) = self.cause {
            write!(f, "\ncaused by: {}", cause)?;
        }
        Ok(())
    }
}

impl Exception {
    /// Render this exception in full: catalog code, title, message, position,
    /// stack trace, runtime + catalog hints, before/after examples, see_also,
    /// cause chain. This is the format user-facing reporters should print at
    /// the top level.
    pub fn render_full(&self) -> String {
        use std::fmt::Write;
        let entry = self.entry();
        let mut s = String::new();
        let _ = writeln!(s, "[{}] {}", entry.short_code, entry.title);
        // Strip the catalog prefix if the message already includes it.
        let prefix = format!("[{}] {} — ", entry.short_code, entry.title);
        let body = self.message.strip_prefix(&prefix).unwrap_or(&self.message);
        if !body.is_empty() && body != entry.short_description {
            let _ = writeln!(s, "  {}", body);
        }
        if let Some(ref pos) = self.position {
            let _ = writeln!(s, "  at {}", pos);
        }
        let _ = writeln!(s);
        let _ = writeln!(s, "  {}", entry.short_description);
        if !entry.long_description.is_empty() && entry.long_description != entry.short_description {
            let _ = writeln!(s);
            for line in entry.long_description.lines() {
                let _ = writeln!(s, "  {}", line);
            }
        }
        if !self.stack.is_empty() {
            let _ = writeln!(s);
            let _ = writeln!(s, "  stack:");
            for frame in &self.stack {
                let _ = writeln!(s, "  {}", frame);
            }
        }
        let all_hints: Vec<&str> = entry
            .hints
            .iter()
            .copied()
            .chain(self.hints.iter().map(String::as_str))
            .collect();
        if !all_hints.is_empty() {
            let _ = writeln!(s);
            let _ = writeln!(s, "  hints:");
            for h in &all_hints {
                let _ = writeln!(s, "    • {}", h);
            }
        }
        if let Some(bad) = entry.example_bad {
            let _ = writeln!(s);
            let _ = writeln!(s, "  example (incorrect):");
            for line in bad.lines() {
                let _ = writeln!(s, "    {}", line);
            }
        }
        if let Some(good) = entry.example_good {
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
        if let Some(ref cause) = self.cause {
            let _ = writeln!(s);
            let _ = writeln!(s, "caused by:");
            for line in cause.render_full().lines() {
                let _ = writeln!(s, "  {}", line);
            }
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

impl std::error::Error for Exception {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

// ---------------------------------------------------------------------------
// Conversions between Error and Exception
// ---------------------------------------------------------------------------

impl From<Error> for Exception {
    fn from(err: Error) -> Exception {
        Exception {
            code: err.code.into(),
            message: err.message,
            position: err.position,
            cause: None,
            stack: Vec::new(),
            hints: Vec::new(),
        }
    }
}

impl From<Exception> for Error {
    fn from(exc: Exception) -> Error {
        Error {
            code: exc.code.as_error_code(),
            message: exc.message,
            position: exc.position,
            context: Vec::new(),
            payload: hudhudscript_errors::ErrorPayload::empty(),
        }
    }
}

impl From<ErrorCode> for Exception {
    fn from(code: ErrorCode) -> Exception {
        Exception::from_code(code.into())
    }
}

/// Result type alias for exception-returning functions.
pub type Result<T> = std::result::Result<T, Exception>;
