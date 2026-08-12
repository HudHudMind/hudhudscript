#![allow(unused_imports)]

use crate::vm::config_types::{OutputLocale, SandboxConfig};
use crate::vm::mcp_dispatch::{dispatch_mcp_tool_call, McpContext};
use crate::vm::prepack::PACK_SENTINEL;
use crate::vm::provider_dispatch::{dispatch_provider_call, ProviderCallConfig, ProviderContext};
use crate::vm::registry::{BuiltinFn, ModuleRegistry};
use crate::vm::util::evaluate_condition_static;
use crate::vm::VM;
use crate::vm::{
    max_stack_size, num_as_f64, num_ref_as_f64, numeric_slot, EndFinallyAction, FinallyStep,
    GenStep, NumericSlot, PackedResult, PendingFlow, SavedFinally, StepAction,
};
use hudhudscript_bytecode::cache_utils::{enforce_cache_limit, MAX_MCP_CACHE, MAX_RAG_STORE_CACHE};
use hudhudscript_bytecode::error::{compile_codes, CompileError, CompileResult, SourcePosition};
use hudhudscript_bytecode::packed_instruction;
use hudhudscript_bytecode::shared_value::{
    num_add, num_div, num_eq, num_ge, num_gt, num_le, num_lt, num_mod, num_mul, num_neg, num_sub,
};
use hudhudscript_bytecode::{
    Bytecode, ClassData, FunctionChunk, FunctionData, GeneratorState16, InstanceData, Instruction,
    PromiseState16, ReprTag, Value16,
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
use std::sync::Arc;

mod actor_core;
mod actor_messaging;
mod actor_misc;
mod actor_spawn;
mod actors_decl;
pub(crate) mod branch;
mod call_load;
mod class_ops;
mod classes_modules;
mod collections_calls;
mod collections_fast;
mod control_flow;
mod indexing;
mod int_arith;
mod int_cmp;
mod int_slot_super;
mod literals_locals;
mod methods_async_generator;
mod methods_generator;
mod module_ops;
mod num_arith;
mod rag;
mod step;
mod string_ops;
mod super_instructions;
mod variables;

pub(crate) struct StepContext<'a> {
    pub(crate) instructions: &'a [Instruction],
    pub(crate) constants: &'a [Value16],
    pub(crate) bytecode: &'a Bytecode,
    pub(crate) ip: usize,
    pub(crate) ip_ref: &'a mut usize,
    /// Raw pointer to the current FunctionChunk (null for top-level bytecode).
    /// Used by IC fast-paths to access call_ic_slots / prop_ic_slots.
    pub(crate) chunk_ptr: *const FunctionChunk,
}
