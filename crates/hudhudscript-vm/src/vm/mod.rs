//! Virtual Machine for executing bytecode
//!
//! # Arc reference cycles (#463)
//!
//! The VM stores `Arc<FunctionChunk>` references from the bytecode's function table.
//! Currently there is no risk of reference cycles because function chunks do not hold
//! `Arc` references back to the VM or to other chunks. If closures or first-class
//! function values are ever changed to hold `Arc<FunctionChunk>` directly (rather than
//! looking up by name), care must be taken to use `Weak<FunctionChunk>` for any
//! back-references to avoid leaking memory through reference cycles.
//!
//! # TODO(#472): String interning for property names
//!
//! Property name strings (used in `LoadVar`, `StoreVar`, `GetProperty`,
//! `SetProperty`, `BindVar`, etc.) are currently heap-allocated `String`s.
//! Every instruction that references a variable or property name stores its own
//! copy, leading to many duplicate allocations for common names like `"name"`,
//! `"type"`, `"value"`, etc.
//!
//! Plan: introduce a `StringInterner` (e.g. `lasso` or `string-interner` crate)
//! that maps each unique string to a compact `SymbolId` (u32). Benefits:
//!   - Reduced memory: each unique property name stored once, instructions hold u32
//!   - Faster comparison: symbol equality is integer comparison, not string comparison
//!   - Cache-friendly: small inline IDs instead of pointer-chasing String refs
//!
//! Migration:
//!   1. Add `StringInterner` to `Bytecode` and `VM`
//!   2. Change `Instruction` variants to use `SymbolId` instead of `String`
//!   3. Intern strings at compile time (`Compiler::compile_expr`)
//!   4. Resolve symbols at VM execution time via interner lookup

// S1.1 (PERF-20): CompactValue is no longer imported — the VM now stores
// `Value` directly on the operand stack, eliminating the Value↔CompactValue
// bridge that allocated an `Arc<HeapObject>` on every heap-variant push.
use crate::vm::mcp_dispatch::{dispatch_mcp_tool_call, McpContext};
use crate::vm::provider_dispatch::{dispatch_provider_call, ProviderCallConfig, ProviderContext};
use hudhudscript_bytecode::cache_utils::{enforce_cache_limit, MAX_MCP_CACHE, MAX_RAG_STORE_CACHE};
use hudhudscript_bytecode::error::{compile_codes, CompileError, CompileResult, SourcePosition};
use hudhudscript_bytecode::packed_instruction;
use hudhudscript_bytecode::shared_value::{
    num_add, num_div, num_eq, num_ge, num_gt, num_le, num_lt, num_mod, num_mul, num_neg, num_sub,
};
use hudhudscript_bytecode::{
    Bytecode, ClassData, FunctionChunk, FunctionData, GeneratorState16, InstanceData, Instruction,
    PromiseState16, Value16,
};
use hudhudscript_debug::Debugger;
use hudhudscript_errors::HudHudResult;
use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};
use hudhudscript_governance::{Condition, Constitution};
use hudhudscript_mcp::{McpClient, Tool as McpToolDefinition};
use hudhudscript_rag::{
    DistanceMetric, EmbeddingProvider, SimpleEmbedding, VectorStore, VectorStoreConfig,
};
use hudhudscript_runtime::provider::ProviderRegistry;
use hudhudscript_tools::ToolRegistry;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
// Note: AtomicU64/Ordering no longer imported at module scope; the only
// remaining atomic is `AtomicBool` via the fully-qualified path used in
// the cancellation token field (see `cancellation_token` below).
use std::sync::Arc;
pub(crate) mod types;
pub(crate) use crate::vm::types::*;

// ── Packed-instruction opcode constants for fast dispatch ────────────
// These MUST stay in sync with hudhudscript_bytecode::packed_instruction.
// They are duplicated here (not re-exported) because the canonical
// constants are intentionally `pub(crate)` in the bytecode crate.
pub(crate) mod api;
pub mod array;
pub(crate) mod bigint_arith;
pub(crate) mod builtin;
pub(crate) mod builtin_aliases;
pub(crate) mod builtin_core;
pub(crate) mod builtin_method_dispatch;
pub(crate) mod builtin_object;
pub(crate) mod builtin_object_dispatch;
pub(crate) mod builtin_object_extra;
pub(crate) mod builtin_system;
pub(crate) mod builtin_system_extra;
pub(crate) mod builtin_value;
pub(crate) mod callback;
pub(crate) mod chunk;
pub(crate) mod class_ops;
pub(crate) mod config_types;
pub(crate) mod ctor;
pub(crate) mod dense_ops;
pub(crate) mod dense_spec;
pub(crate) mod dispatch;
pub(crate) mod dispatch_actor_io;
pub(crate) mod dispatch_core;
pub(crate) mod dispatch_env;
pub(crate) mod dispatch_general;
pub(crate) mod dispatch_governance;
pub(crate) mod dispatch_int_arith;
pub(crate) mod dispatch_mcp_stm;
pub(crate) mod dispatch_num_arith;
pub(crate) mod dispatch_string_result;
pub(crate) mod dispatch_table;
pub(crate) mod dispatch_terminal;
pub(crate) mod error_helpers;
pub(crate) mod exception_value;
pub(crate) mod exec;
pub(crate) mod execute;
pub(crate) mod execute_instructions;
pub(crate) mod fastcall;
pub(crate) mod finally;
pub(crate) mod format;
pub(crate) mod fuel;
pub(crate) mod gc_roots;
pub(crate) mod globals;
pub(crate) mod globals_api;
pub(crate) mod governance;
pub mod governance_ops;
pub mod host_access;
pub(crate) mod index_helpers;
pub use host_access::{AccessDecision as HostAccessDecision, HostAccessPolicy};
pub mod json;
pub mod map;
pub(crate) mod mcp;
pub mod mcp_dispatch;
pub mod mcp_server_ops;
pub mod operations;
pub mod packed_ops;
pub(crate) mod prepack;
pub(crate) mod promise;
pub(crate) mod property;
pub(crate) mod provider;
pub mod provider_dispatch;
pub(crate) mod register_arena;
pub mod registry;
pub(crate) mod run;
pub(crate) mod scope;
pub mod set;
pub mod sop_types;
pub mod string;
pub(crate) mod trampoline;
pub(crate) mod util;

pub use crate::vm::config_types::{OutputLocale, SandboxConfig};
use crate::vm::prepack::{prepack_instructions, PACK_SENTINEL};
use crate::vm::registry::{BuiltinFn, ModuleRegistry};

pub mod machine;
mod machine_types;
pub use machine::VM;
