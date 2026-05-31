//! AUTO-GENERATED error catalog.
//!
//! Single source of truth for every error/exception code in HudHudScript.
//! Edit `crates/hudhudscript-errors/tools/gen_rust.py` and the JSON content
//! files, then regenerate. Do not hand-edit entries.
//!
//! 323 entries across 23 categories.

use serde::{Deserialize, Serialize};

/// High-level category for a error. Used for filtering, routing, and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
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

impl ErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::Ai => "ai",
            ErrorCategory::Cli => "cli",
            ErrorCategory::Compile => "compile",
            ErrorCategory::Cybernetics => "cybernetics",
            ErrorCategory::Governance => "governance",
            ErrorCategory::Io => "io",
            ErrorCategory::Lex => "lex",
            ErrorCategory::Localization => "localization",
            ErrorCategory::Lsp => "lsp",
            ErrorCategory::Native => "native",
            ErrorCategory::Orchestration => "orchestration",
            ErrorCategory::Package => "package",
            ErrorCategory::Parse => "parse",
            ErrorCategory::Promise => "promise",
            ErrorCategory::Resource => "resource",
            ErrorCategory::Runtime => "runtime",
            ErrorCategory::Security => "security",
            ErrorCategory::Storage => "storage",
            ErrorCategory::Tokenomics => "tokenomics",
            ErrorCategory::Tool => "tool",
            ErrorCategory::Type => "type",
            ErrorCategory::Ui => "ui",
            ErrorCategory::Validation => "validation",
        }
    }
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
