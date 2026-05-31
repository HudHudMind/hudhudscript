//! AST to Bytecode compiler

pub(super) use crate::bytecode::{
    Bytecode, FunctionChunk, FunctionData, Instruction, SymId, Value16,
};
pub(super) use crate::error::{compile_codes, CompileResult, SourcePosition};
pub(super) use hudhudscript_ast::{BinaryOp, Decl, Expr, Literal, Span, Stmt};
pub(super) use std::collections::{HashMap, HashSet};
pub(super) use std::sync::Arc;

mod expr;
// FunctionCompiler eliminated (ISSUE-2) — Compiler handles function bodies directly
// mod function_compiler;
// mod function_compiler_target;
mod helpers;
mod regalloc;
pub(crate) use regalloc::RegAlloc;
mod stmt_shared;
mod target;
// FunctionCompiler eliminated (ISSUE-2) — Compiler handles function bodies directly
// use function_compiler::FunctionCompiler;

pub(crate) use expr::{emit_numeric_literal, is_numeric};
use helpers::*;
use stmt_shared::compile_stmt_shared;
pub use target::CompileTarget;

pub struct Compiler {
    bytecode: Bytecode,
    /// Current scope nesting depth (0 = global)
    scope_depth: usize,
    /// True only for the outermost compilation (not inner function bodies)
    in_top_level: bool,
    /// Names declared at top-level (for closure capture filtering)
    top_level_names: HashSet<String>,
    /// Local variable declarations in scope order
    locals: Vec<Local>,
    /// Track declared class names so we can emit GetStatic for ClassName.member
    known_classes: HashSet<String>,
    /// Track declared generator function names so we can emit MakeGenerator
    known_generators: HashSet<String>,
    /// Issue #982: Track declared trait names → required method names for SOP enforcement
    known_traits: HashMap<String, Vec<String>>,
    /// SOP0003: Track declared role names → capability names for role enforcement
    known_roles: HashMap<String, Vec<String>>,
    /// FUNCTION0001: Track function names per scope for duplicate detection
    declared_fns: Vec<HashMap<String, usize>>,
    /// Pending statement-boundary source position consumed by the next
    /// `ct_emit`. Populated by `ct_mark_stmt_pos` at the start of every
    /// statement; drained when the first instruction of that statement
    /// is emitted, so `source_positions[ip]` matches the instruction
    /// at `ip`. Enables the VM's DAP `on_statement` hook to resolve
    /// ip → (line, col) in O(1).
    pending_source_pos: Option<(usize, usize)>,
    /// PERF-B1: Top-level local variable slot names (index = slot number).
    /// Populated by `declare_local`; emitted as `Bytecode::main_local_names`.
    pub(super) local_slot_names: Vec<String>,
    /// ISSUE-2: Temporary function body context.
    /// Set during `compile_function_body_named_async`, None otherwise.
    fn_ctx: Option<FuncCtx>,
    /// ISSUE-1: Last allocated match pattern register (for patch sites)
    pub(super) last_match_reg: u8,
    /// CLAUDE_BENCHMARK OPT1: Highest register index emitted so far for the
    /// current function body.  Written into FunctionChunk::max_register at
    /// the end of compilation so the VM can size register save/restore.
    pub(super) current_max_register: u8,
    /// K1-1: Next local-variable register index (params + locals).
    /// Parameters start at r0, locals continue sequentially.
    pub(super) next_local_reg: u8,
}

/// Per-function-body compilation context (ISSUE-2).
#[derive(Default)]
pub(super) struct FuncCtx {
    pub(super) params: Vec<String>,
    pub(super) fn_name: Option<String>,
    pub(super) has_rest: bool,
    pub(super) referenced: Vec<String>,
    pub(super) is_async: bool,
}

/// Convert an AST Span to a SourcePosition (uses the start of the span).
fn span_pos(span: &Span) -> SourcePosition {
    SourcePosition {
        line: span.start.line,
        column: span.start.column,
    }
}

mod decl;
mod decl_core;
mod decl_expr;
mod decl_function;
mod target_impl;

pub use decl::*;
pub use decl_core::*;
pub use decl_expr::*;
pub use decl_function::*;
pub use target_impl::*;

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

// Tests moved to hudhud-script-tests/tests/compiler_test_inline.rs
