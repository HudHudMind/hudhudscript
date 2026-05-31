//! REPL commands and actions.

/// A callable REPL command (registered via `add_command`).
pub struct ReplCommand {
    /// The command name (without leading colon/slash).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// The handler function.
    pub(crate) handler: Box<dyn Fn(&[&str]) -> ReplAction>,
}

impl std::fmt::Debug for ReplCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplCommand")
            .field("name", &self.name)
            .field("description", &self.description)
            .finish()
    }
}

/// The outcome of evaluating a single REPL input line.
#[derive(Debug, Clone, PartialEq)]
pub enum ReplAction {
    /// Continue the REPL loop (possibly with output text).
    Continue(Option<String>),
    /// Exit the REPL.
    Exit,
    /// An error occurred; display the message but keep running.
    Error(String),
}
