use serde::{Deserialize, Serialize};

/// One frame of an exception's stack trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name (or `<anonymous>` / `<global>`).
    pub function: String,
    /// File path, if known.
    pub file: Option<String>,
    /// Line number (1-indexed), if known.
    pub line: Option<usize>,
    /// Column number (1-indexed), if known.
    pub column: Option<usize>,
}

impl StackFrame {
    pub fn new(function: impl Into<String>) -> Self {
        Self {
            function: function.into(),
            file: None,
            line: None,
            column: None,
        }
    }

    pub fn at(mut self, file: impl Into<String>, line: usize, column: usize) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

impl std::fmt::Display for StackFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "    at {}", self.function)?;
        if let Some(ref file) = self.file {
            write!(f, " ({}", file)?;
            if let Some(line) = self.line {
                write!(f, ":{}", line)?;
                if let Some(col) = self.column {
                    write!(f, ":{}", col)?;
                }
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}
