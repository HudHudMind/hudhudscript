//! REPL configuration.

use std::path::PathBuf;

/// Configuration for a REPL session.
#[derive(Debug, Clone)]
pub struct ReplConfig {
    /// The prompt string displayed before each input line.
    pub prompt: String,
    /// Path to the history file (if any).
    pub history_file: Option<PathBuf>,
    /// Maximum number of history entries to keep.
    pub max_history: usize,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            prompt: "hud> ".to_string(),
            history_file: None,
            max_history: 1000,
        }
    }
}

impl ReplConfig {
    /// Create a new config with the given prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    /// Set the history file path.
    pub fn history_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.history_file = Some(path.into());
        self
    }

    /// Set the maximum history size.
    pub fn max_history(mut self, n: usize) -> Self {
        self.max_history = n;
        self
    }
}
