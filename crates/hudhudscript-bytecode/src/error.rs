//! Bytecode compiler error types.
//!
//! v0.4.48 — TAM CONSOLIDATION (Anayasa Kural 1 İstisna authorized).
//! `CompileError` is now a type alias for the unified
//! [`hudhudscript_errors::Error`]. The eight former variants
//! (`Generic`, `GenericAt`, `UnsupportedFeature`, `UnsupportedFeatureAt`,
//! `InvalidBytecode`, `InvalidBytecodeAt`, `RuntimeError`, `RuntimeErrorAt`)
//! are constructed via the [`compile_codes`] module. Downstream code that
//! used to match on enum variants now matches on `error.code` against
//! `ErrorCode::Compile*`, and reads variant fields via `error.context_get`.

/// Source position information for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl SourcePosition {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

pub type CompileResult<T> = hudhudscript_errors::HudHudResult<T>;

/// Bytecode compiler error — type alias for the unified
/// [`hudhudscript_errors::Error`].
pub type CompileError = hudhudscript_errors::Error;

/// Constructor functions for compile-stage errors.
pub mod compile_codes {
    use super::SourcePosition as LocalPos;
    use hudhudscript_errors::{Error, ErrorCode, SourcePosition};

    fn local_to_source(p: LocalPos) -> SourcePosition {
        SourcePosition::new(p.line, p.column, 0)
    }

    pub fn generic(s: impl Into<String>) -> Error {
        let s = s.into();
        Error::new(
            ErrorCode::CompileGeneric,
            format!("Compilation error: {}", s),
        )
        .with_context("message", s)
    }

    pub fn generic_at(msg: impl Into<String>, pos: LocalPos) -> Error {
        let msg = msg.into();
        Error::new(ErrorCode::CompileGenericAt, format!("{} at {}", msg, pos))
            .at(local_to_source(pos))
            .with_context("message", msg)
    }

    pub fn unsupported_feature(s: impl Into<String>) -> Error {
        let s = s.into();
        Error::new(
            ErrorCode::CompileUnsupportedFeature,
            format!("Unsupported feature: {}", s),
        )
        .with_context("message", s)
    }

    /// ISSUE-1: Duplicate function/declaration error — proper code, not "unsupported feature".
    pub fn duplicate_declaration(s: impl Into<String>) -> Error {
        let s = s.into();
        Error::new(
            ErrorCode::CompileGeneric,
            format!("Duplicate declaration: {}", s),
        )
        .with_context("message", s)
    }

    pub fn type_error(s: impl Into<String>) -> Error {
        let s = s.into();
        Error::new(
            ErrorCode::CompileGeneric,
            format!("Type error: {}", s),
        )
        .with_context("message", s)
    }

    pub fn unsupported_feature_at(msg: impl Into<String>, pos: LocalPos) -> Error {
        let msg = msg.into();
        Error::new(
            ErrorCode::CompileUnsupportedFeatureAt,
            format!("Unsupported feature at {}: {}", pos, msg),
        )
        .at(local_to_source(pos))
        .with_context("message", msg)
    }

    pub fn invalid_bytecode(s: impl Into<String>) -> Error {
        let s = s.into();
        Error::new(
            ErrorCode::CompileInvalidBytecode,
            format!("Invalid bytecode: {}", s),
        )
        .with_context("message", s)
    }

    pub fn invalid_bytecode_at(msg: impl Into<String>, pos: LocalPos) -> Error {
        let msg = msg.into();
        Error::new(
            ErrorCode::CompileInvalidBytecodeAt,
            format!("Invalid bytecode at {}: {}", pos, msg),
        )
        .at(local_to_source(pos))
        .with_context("message", msg)
    }

    pub fn runtime_error(s: impl Into<String>) -> Error {
        let s = s.into();
        Error::new(
            ErrorCode::CompileRuntimeError,
            format!("Runtime error: {}", s),
        )
        .with_context("message", s)
    }

    pub fn runtime_error_at(msg: impl Into<String>, pos: LocalPos) -> Error {
        let msg = msg.into();
        Error::new(
            ErrorCode::CompileRuntimeErrorAt,
            format!("Runtime error at {}: {}", pos, msg),
        )
        .at(local_to_source(pos))
        .with_context("message", msg)
    }
}
