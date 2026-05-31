use thiserror::Error;

use crate::{Diagnostic, SourcePosition};

/// Top-level error enum that wraps errors from all compiler phases.
///
/// This enables a single `Result<T, HudHudError>` for pipelines that span
/// multiple phases (lex -> parse -> compile -> run).
#[derive(Debug, Error)]
pub enum HudHudError {
    /// An error originating from the lexer.
    #[error("lex error: {message}")]
    Lex {
        message: String,
        position: Option<SourcePosition>,
    },

    /// An error originating from the parser.
    #[error("parse error: {message}")]
    Parse {
        message: String,
        position: Option<SourcePosition>,
    },

    /// An error originating from the type checker.
    #[error("type error: {message}")]
    Type {
        message: String,
        position: Option<SourcePosition>,
    },

    /// An error originating from the compiler (bytecode generation).
    #[error("compile error: {message}")]
    Compile {
        message: String,
        position: Option<SourcePosition>,
    },

    /// An error originating from the runtime / interpreter.
    #[error("runtime error: {message}")]
    Runtime { message: String },

    /// A collection of diagnostics (useful for reporting multiple errors).
    #[error("{} diagnostic(s)", .0.len())]
    Diagnostics(Vec<Diagnostic>),

    /// Catch-all for IO and other non-compiler errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl HudHudError {
    /// Convert this error into a list of [`Diagnostic`]s.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        match self {
            HudHudError::Diagnostics(diags) => diags,
            HudHudError::Lex { message, position } => {
                let mut d = Diagnostic::error(message).with_code("E_LEX");
                if let Some(pos) = position {
                    d = d.at(pos);
                }
                vec![d]
            }
            HudHudError::Parse { message, position } => {
                let mut d = Diagnostic::error(message).with_code("E_PARSE");
                if let Some(pos) = position {
                    d = d.at(pos);
                }
                vec![d]
            }
            HudHudError::Type { message, position } => {
                let mut d = Diagnostic::error(message).with_code("E_TYPE");
                if let Some(pos) = position {
                    d = d.at(pos);
                }
                vec![d]
            }
            HudHudError::Compile { message, position } => {
                let mut d = Diagnostic::error(message).with_code("E_COMPILE");
                if let Some(pos) = position {
                    d = d.at(pos);
                }
                vec![d]
            }
            HudHudError::Runtime { message } => {
                vec![Diagnostic::error(message).with_code("E_RUNTIME")]
            }
            HudHudError::Io(e) => {
                vec![Diagnostic::error(e.to_string()).with_code("E_IO")]
            }
        }
    }
}
