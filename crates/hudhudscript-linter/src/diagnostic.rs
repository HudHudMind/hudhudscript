//! Lint diagnostic output.

use crate::Severity;
use hudhudscript_ast::Span;
use serde::{Deserialize, Serialize};

/// A single lint diagnostic emitted by the linter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintDiagnostic {
    /// Machine-readable rule code, e.g. `"naming-convention"`.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Source location.
    pub span: Span,
}

impl LintDiagnostic {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        severity: Severity,
        span: Span,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity,
            span,
        }
    }
}

impl std::fmt::Display for LintDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} ({}:{}): {}",
            self.severity, self.code, self.span.start.line, self.span.start.column, self.message,
        )
    }
}
