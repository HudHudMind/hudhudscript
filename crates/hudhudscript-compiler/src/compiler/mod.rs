//! AST to Bytecode compiler

pub(super) use crate::bytecode::{
    Bytecode, FunctionChunk, FunctionData, Instruction, SymId, Value16,
};
pub(super) use crate::error::{compile_codes, CompileResult, SourcePosition};
pub(super) use hudhudscript_ast::{
    BinaryOp, ChainLinkAst, ChainTargetAst, Decl, Expr, GateBranchAst, GateTargetAst, Literal,
    LoopItemAst, Span, StepGateAst, Stmt,
};
use rustc_hash::FxHashMap;
pub(super) use std::collections::{HashMap, HashSet};
pub(super) use std::sync::Arc;

mod expr;
// FunctionCompiler eliminated (ISSUE-2) — Compiler handles function bodies directly
// mod function_compiler;
// mod function_compiler_target;
mod floop;
mod floop_analyze;
mod helpers;
pub(crate) use helpers::*;
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
    /// B8: snapshot of the outer bytecode's int/numeric constants for inliner remap.
    /// Set before function body compilation, cleared after.
    pub(super) global_int_constants: Vec<i64>,
    pub(super) global_numeric_constants: Vec<u64>,
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
    /// P5d: true when the Math global has been reassigned in this compilation unit.
    math_reassigned: bool,
    /// P3a: function registry for compiler-side inlining.
    /// Populated at function declaration time, independent of bytecode.function_names RefCell.
    inline_function_chunks: FxHashMap<String, Arc<FunctionChunk>>,
    /// P4: call-site parameter type tracking.
    /// fn_name -> Vec<(param_name, ExprType)>
    call_site_param_types: HashMap<String, Vec<(String, crate::compiler::expr::ExprType)>>,
    /// P4a: function parameter names registry.
    /// fn_name -> Vec<param_name>
    fn_param_names: HashMap<String, Vec<String>>,
    /// P4: current function being compiled (None = top-level)
    current_function_name: Option<String>,
    /// Issue #982: Track declared trait names → required method names for SOP enforcement
    known_traits: HashMap<String, Vec<String>>,
    /// SOP0003: Track declared role names → capability names for role enforcement
    known_roles: HashMap<String, Vec<String>>,
    /// FUNCTION0001: Track function names per scope for duplicate detection
    declared_fns: Vec<HashMap<String, usize>>,
    /// ISSUE-2e-1: set of top-level names referenced inside any function/closure
    /// body.  Used to build `Bytecode::main_local_shared` so the VM can later
    /// decide whether a top-level symbol is main-frame-only or shared.
    pub(super) referenced_top_level: HashSet<String>,
    /// ISSUE-2e-optimize: top-level names classified as "shared" BEFORE
    /// codegen starts, via a pre-pass.  Used by assignment.rs and core.rs
    /// to decide whether to emit StoreGlobal/DeclGlobal or stay pure register.
    pub(super) shared_top_level_names: HashSet<String>,
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
    /// FAZ F: Loop step name registry for cross-loop entry selector lookup.
    /// Maps loop_name → step_names (in order).
    pub(super) loop_step_names: HashMap<String, Vec<String>>,
    /// FAZ G: Gate declaration registry for AttachGate resolution.
    /// Maps gate_name → (branches, else_target).
    pub(super) gate_registry: HashMap<String, (Vec<GateBranchAst>, GateTargetAst)>,
    /// A2: Standalone step registry for use_step resolution.
    /// Maps step_name → (params, body, gate).
    pub(super) step_registry: HashMap<String, (Vec<String>, Vec<Stmt>, Option<StepGateAst>)>,
    /// A3: Attached steps pending injection per loop.
    /// Maps loop_name → Vec<LoopItemAst>.
    pub(super) attach_step_queue: HashMap<String, Vec<LoopItemAst>>,
    /// A3: Attached loops pending injection per chain.
    pub(super) attach_loop_queue:
        HashMap<String, Vec<(String, Option<ChainTargetAst>, Option<ChainTargetAst>)>>,
    /// Stack of active loop/switch break targets.
    pub(super) break_targets: Vec<crate::compiler::target::BreakTarget>,
    /// G12: etkin f-loop bağlamları (iç içe döngü v1'de reddedildiği için
    /// pratikte en fazla 1; yığın ileriye dönük sağlamlık içindir).
    pub(super) floop_stack: Vec<FloopCtx>,
    /// Base directory for resolving nested module imports.
    pub module_base_dir: Option<std::path::PathBuf>,
}

/// G12: tek bir f-loop bağlamı — aday isim → f-slot + geçici slot sayacı +
/// döngü-öncesi hoist edilmiş sabitler (f64 bit deseni → f-slot; sabitler
/// döngü-değişmezidir, gövde içinde FConst tekrarı YASAK — slot okunur).
pub(super) struct FloopCtx {
    pub(super) slots: HashMap<String, u8>,
    pub(super) consts: HashMap<u64, u8>,
    pub(super) temp_next: u8,
    pub(super) temp_base: u8,
}

/// Per-function-body compilation context (ISSUE-2).
#[derive(Default)]
pub(super) struct FuncCtx {
    pub(super) params: Vec<String>,
    pub(super) fn_name: Option<String>,
    pub(super) has_rest: bool,
    pub(super) referenced: Vec<String>,
    /// ADIM B: locals genuinely captured by nested closures (not pure locals)
    pub(super) nested_captured: HashSet<String>,
    pub(super) is_async: bool,
}

/// Convert an AST Span to a SourcePosition (uses the start of the span).
fn span_pos(span: &Span) -> SourcePosition {
    SourcePosition {
        line: span.start.line,
        column: span.start.column,
    }
}

pub mod decl;
mod decl_core;
mod decl_expr;
mod decl_function;
mod decl_precompute;
mod p4b_prepass;
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
