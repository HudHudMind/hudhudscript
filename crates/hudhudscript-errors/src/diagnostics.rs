use serde::{Deserialize, Serialize};

use crate::SourcePosition;

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// A hard error that prevents further processing.
    Error,
    /// A warning — the program may still compile / run.
    Warning,
    /// An informational note (e.g. a suggestion).
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// A structured diagnostic message.
///
/// Diagnostics are the primary way the compiler communicates problems (and
/// suggestions) to the user. They carry enough context for an IDE / LSP to
/// render squiggly underlines, quick-fixes, etc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Human-readable error message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Where in the source the problem was detected.
    pub position: Option<SourcePosition>,
    /// Optional machine-readable error code (e.g. "E0001").
    pub code: Option<String>,
    /// Optional hints or notes to help the user fix the issue.
    pub hints: Vec<String>,
    /// Optional source code snippet showing the offending lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_snippet: Option<String>,
}

impl Diagnostic {
    /// Create a new error-level diagnostic.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: Severity::Error,
            position: None,
            code: None,
            hints: Vec::new(),
            source_snippet: None,
        }
    }

    /// Create a new warning-level diagnostic.
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: Severity::Warning,
            position: None,
            code: None,
            hints: Vec::new(),
            source_snippet: None,
        }
    }

    /// Create a new info-level diagnostic.
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: Severity::Info,
            position: None,
            code: None,
            hints: Vec::new(),
            source_snippet: None,
        }
    }

    /// Attach a source snippet to this diagnostic.
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.source_snippet = Some(snippet.into());
        self
    }

    /// Extract a snippet of context lines from the source code.
    pub fn extract_snippet_from_source(mut self, source: &str, context_lines: usize) -> Self {
        if let Some(ref pos) = self.position {
            let lines: Vec<&str> = source.lines().collect();
            let line_idx = pos.line.saturating_sub(1);
            let start = line_idx.saturating_sub(context_lines);
            let end = (line_idx + context_lines + 1).min(lines.len());
            let mut snippet = String::new();
            for (i, line) in lines[start..end].iter().enumerate() {
                let actual_line = start + i + 1;
                let marker = if actual_line == pos.line { ">" } else { " " };
                snippet.push_str(&format!("{} {:4} | {}\n", marker, actual_line, line));
                if actual_line == pos.line {
                    let caret_offset = pos.column.saturating_sub(1);
                    snippet.push_str(&format!("       | {}^\n", " ".repeat(caret_offset)));
                }
            }
            self.source_snippet = Some(snippet);
        }
        self
    }

    /// Attach a source position.
    pub fn at(mut self, position: SourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Attach an error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add a hint.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.severity)?;
        if let Some(ref code) = self.code {
            write!(f, "[{}]", code)?;
        }
        if let Some(ref pos) = self.position {
            write!(f, " at {}", pos)?;
        }
        write!(f, ": {}", self.message)?;
        if let Some(ref snippet) = self.source_snippet {
            write!(f, "\n{}", snippet)?;
        }
        for hint in &self.hints {
            write!(f, "\n  hint: {}", hint)?;
        }
        Ok(())
    }
}
