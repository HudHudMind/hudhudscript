use serde::{Deserialize, Serialize};

/// A source location that optionally carries a file path.
///
/// This is intentionally a superset of `hudhudscript_ast::Position` — the AST
/// crate must stay dependency-free, so we cannot add a file path there.
/// Conversions from the AST `Position` are provided via `From`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourcePosition {
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    pub column: usize,
    /// 0-indexed byte offset into the source text.
    pub offset: usize,
    /// Optional file path (empty for REPL / in-memory sources).
    pub file_path: Option<String>,
}

impl SourcePosition {
    /// Create a new source position without a file path.
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
            file_path: None,
        }
    }

    /// Attach a file path to this position.
    pub fn with_file(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref path) = self.file_path {
            write!(f, "{}:{}:{}", path, self.line, self.column)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}
