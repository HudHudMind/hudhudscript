//! Source code span and position tracking for error reporting

use serde::{Deserialize, Serialize};

/// Represents a position in source code (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset in source (0-indexed)
    pub offset: usize,
}

impl Position {
    /// Create a new position
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Create a position at the start of the file
    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::start()
    }
}

/// Represents a span of source code between two positions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

impl Span {
    /// Create a new span
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a span at the start of the file
    pub fn start() -> Self {
        Self {
            start: Position::start(),
            end: Position::start(),
        }
    }

    /// Combine two spans into one that covers both
    pub fn merge(self, other: Span) -> Span {
        let start = if self.start.offset < other.start.offset {
            self.start
        } else {
            other.start
        };

        let end = if self.end.offset > other.end.offset {
            self.end
        } else {
            other.end
        };

        Span { start, end }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::start()
    }
}
