use serde::{Deserialize, Serialize};

/// High-level category for a exception. Used for filtering, routing, and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExceptionCategory {
    Ai,
    Cli,
    Compile,
    Cybernetics,
    Governance,
    Io,
    Lex,
    Localization,
    Lsp,
    Native,
    Orchestration,
    Package,
    Parse,
    Promise,
    Resource,
    Runtime,
    Security,
    Storage,
    Tokenomics,
    Tool,
    Type,
    Ui,
    Validation,
}

impl ExceptionCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExceptionCategory::Ai => "ai",
            ExceptionCategory::Cli => "cli",
            ExceptionCategory::Compile => "compile",
            ExceptionCategory::Cybernetics => "cybernetics",
            ExceptionCategory::Governance => "governance",
            ExceptionCategory::Io => "io",
            ExceptionCategory::Lex => "lex",
            ExceptionCategory::Localization => "localization",
            ExceptionCategory::Lsp => "lsp",
            ExceptionCategory::Native => "native",
            ExceptionCategory::Orchestration => "orchestration",
            ExceptionCategory::Package => "package",
            ExceptionCategory::Parse => "parse",
            ExceptionCategory::Promise => "promise",
            ExceptionCategory::Resource => "resource",
            ExceptionCategory::Runtime => "runtime",
            ExceptionCategory::Security => "security",
            ExceptionCategory::Storage => "storage",
            ExceptionCategory::Tokenomics => "tokenomics",
            ExceptionCategory::Tool => "tool",
            ExceptionCategory::Type => "type",
            ExceptionCategory::Ui => "ui",
            ExceptionCategory::Validation => "validation",
        }
    }
}

impl std::fmt::Display for ExceptionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
