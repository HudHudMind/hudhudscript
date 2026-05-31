//! Interactive REPL state manager.

use std::collections::HashMap;

use crate::repl::command::{ReplAction, ReplCommand};
use crate::repl::config::ReplConfig;

/// Interactive REPL state manager.
pub struct Repl {
    /// REPL configuration.
    pub config: ReplConfig,
    /// Registered special commands.
    commands: HashMap<String, ReplCommand>,
    /// Command history (most recent last).
    history_entries: Vec<String>,
}

impl Repl {
    /// Create a new REPL with the given configuration.
    pub fn new(config: ReplConfig) -> Self {
        Self {
            config,
            commands: HashMap::new(),
            history_entries: Vec::new(),
        }
    }

    /// Create a new REPL with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ReplConfig::default())
    }

    /// Register a built-in REPL command (e.g. `:help`, `:quit`).
    pub fn add_command<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        handler: F,
    ) where
        F: Fn(&[&str]) -> ReplAction + 'static,
    {
        let name = name.into();
        self.commands.insert(
            name.clone(),
            ReplCommand {
                name,
                description: description.into(),
                handler: Box::new(handler),
            },
        );
    }

    /// Process a single input line.
    ///
    /// - Lines starting with `:` are treated as REPL commands.
    /// - Empty lines are ignored.
    /// - Everything else is returned as `Continue(Some(line))` for evaluation
    ///   by the caller.
    pub fn run_line(&mut self, line: &str) -> ReplAction {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            return ReplAction::Continue(None);
        }

        // Record in history
        self.push_history(trimmed.to_string());

        // Check for REPL commands
        if let Some(rest) = trimmed.strip_prefix(':') {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if let Some(&cmd_name) = parts.first() {
                if let Some(cmd) = self.commands.get(cmd_name) {
                    let args = if parts.len() > 1 { &parts[1..] } else { &[] };
                    return (cmd.handler)(args);
                } else {
                    return ReplAction::Error(format!("unknown command ':{}'", cmd_name));
                }
            }
        }

        ReplAction::Continue(Some(trimmed.to_string()))
    }

    /// Return a slice of the history entries.
    pub fn history(&self) -> &[String] {
        &self.history_entries
    }

    /// Get the number of history entries.
    pub fn history_len(&self) -> usize {
        self.history_entries.len()
    }

    /// Push a line into history, respecting `max_history`.
    fn push_history(&mut self, line: String) {
        // Skip duplicates of the last entry
        if self.history_entries.last().map(|s| s.as_str()) == Some(&line) {
            return;
        }
        self.history_entries.push(line);
        if self.history_entries.len() > self.config.max_history {
            self.history_entries.remove(0);
        }
    }

    /// List registered commands (for `:help` output).
    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        let mut cmds: Vec<(&str, &str)> = self
            .commands
            .values()
            .map(|c| (c.name.as_str(), c.description.as_str()))
            .collect();
        cmds.sort_by_key(|(name, _)| *name);
        cmds
    }
}

impl std::fmt::Debug for Repl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repl")
            .field("config", &self.config)
            .field("commands", &self.commands.keys().collect::<Vec<_>>())
            .field("history_len", &self.history_entries.len())
            .finish()
    }
}
