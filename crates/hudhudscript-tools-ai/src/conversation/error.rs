/// Errors that can occur during conversation operations.
#[derive(Debug)]
pub enum ConversationError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Empty,
}

impl std::fmt::Display for ConversationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ConversationError::Io(e) => write!(f, "IO error: {}", e),
            ConversationError::Serialization(e) => write!(f, "Serialization error: {}", e),
            ConversationError::Empty => write!(f, "Conversation is empty"),
        }
    }
}

impl std::error::Error for ConversationError {}

impl From<std::io::Error> for ConversationError {
    fn from(e: std::io::Error) -> Self {
        ConversationError::Io(e)
    }
}

impl From<serde_json::Error> for ConversationError {
    fn from(e: serde_json::Error) -> Self {
        ConversationError::Serialization(e)
    }
}

impl ConversationError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ConversationError::Empty => hudhudscript_errors::ErrorCode::ConversationEmpty,
            ConversationError::Io(..) => hudhudscript_errors::ErrorCode::ConversationIo,
            ConversationError::Serialization(..) => {
                hudhudscript_errors::ErrorCode::ConversationSerialization
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<ConversationError> for hudhudscript_errors::Error {
    fn from(e: ConversationError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
