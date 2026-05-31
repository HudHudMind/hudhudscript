//! HudHudScript Code Formatter
//!
//! This module provides code formatting functionality. Terminal Markdown
//! rendering (with syntax highlighting and streaming support) has been
//! extracted into the [`hudhudscript_markdown`] crate and is re-exported
//! here for backward compatibility.
//!
//! # Design Decision: Why this formatter does NOT use `AstVisitor` (#894)
//!
//! The `AstVisitor` trait (in `hudhudscript-ast/src/visitor.rs`) is designed for
//! **data-collection** passes — linter diagnostics, type-checker annotations, LSP
//! symbol tables — where a visitor accumulates findings while the `walk_*`
//! functions handle structural recursion. The formatter is fundamentally different:
//! it is an **output-producing transformation** where each AST node maps to
//! formatted text with precise control over indentation and child interleaving.
//!
//! Specific reasons the visitor pattern is a poor fit here:
//!
//! 1. **Parent controls child output placement.** An `if` statement emits
//!    `"if (COND) {\n"`, then formats children at increased indent, then emits
//!    `"} else {\n"`, then more children, then `"}\n"`. The walker cannot inject
//!    text between children — it only fires `visit_stmt` / `leave_stmt` callbacks.
//!
//! 2. **Indentation is stateful and context-dependent.** `current_indent` is
//!    incremented before recursing into a block and decremented after. The walker
//!    has no mechanism to manage this per-node depth.
//!
//! 3. **Each node produces a different string format.** The match arms are not
//!    boilerplate — they encode the language's syntax. Delegating to a visitor
//!    would just move the same match arms into `visit_stmt` / `visit_expr` with
//!    extra complexity and no deduplication.
//!
//! 4. **`format_expr` is `&self` (immutable)** while `format_stmt` is `&mut self`.
//!    The visitor trait uses `&mut self` uniformly, which does not match the
//!    formatter's borrowing requirements.
//!
//! The formatter therefore keeps its manual `match` statements. If a future
//! pre-pass (e.g., import sorting, comment extraction) is needed, that pass
//! can use `AstVisitor` independently before the formatter runs.

// Re-export from the extracted `hudhudscript-markdown` crate for backward
// compatibility. Dependents can also use `hudhudscript_markdown` directly.
pub use hudhudscript_markdown::markdown;
pub use hudhudscript_markdown::streaming;
pub use hudhudscript_markdown::syntax;
pub use hudhudscript_markdown::theme;

use hudhudscript_ast::{
    AccessModifier, ArrowFunctionBody, BinaryOp, ClassDecl, ClassMember, Decl, Expr, ImportKind,
    Literal, McpServerDecl, Param, TemplateStringPart, UnaryOp, VarDecl,
};
use hudhudscript_ast::{
    ActionDecl, CatchClause, ConditionDecl, CouncilMemberDecl, CultureDecl, EnumVariant, LawDecl,
    MatchArm, MatchPattern, Stmt, SwitchCase,
};

/// Formatter configuration
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Indentation string (e.g., "  " for 2 spaces, "\t" for tab)
    pub indent: String,
    /// Maximum line length
    pub max_line_length: usize,
    /// Add semicolons
    pub semicolons: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(), // 4 spaces
            max_line_length: 100,
            semicolons: true,
        }
    }
}

/// Code formatter
pub struct Formatter {
    config: FormatterConfig,
    current_indent: usize,
}

/// Escape special characters in a string for safe output
pub fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

mod formatter_impl;
pub use formatter_impl::*;
