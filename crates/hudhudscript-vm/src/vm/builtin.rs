use crate::vm::call_state::{ArrayCallbackOperation, DeferredCallSite, MethodDispatchOutcome};
use crate::vm::config_types::{OutputLocale, SandboxConfig};
use crate::vm::mcp_dispatch::{dispatch_mcp_tool_call, McpContext};
use crate::vm::prepack::PACK_SENTINEL;
use crate::vm::provider_dispatch::{dispatch_provider_call, ProviderCallConfig, ProviderContext};
use crate::vm::registry::{BuiltinFn, ModuleRegistry};
use crate::vm::util::builtin_name_set;
use crate::vm::VM;
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
use std::sync::Arc;

impl VM {
    /// O(1) hash lookup for built-in function names.
    pub(crate) fn is_builtin(&self, name: &str) -> bool {
        builtin_name_set().contains(name)
    }

    pub(crate) fn call_builtin(
        &mut self,
        name: &str,
        arg_count: u8,
        first_arg: u8,
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<()> {
        if ArrayCallbackOperation::from_name(name).is_some() {
            match self
                .start_array_callback_builtin(name, arg_count, first_arg, bytecode, call_site)?
            {
                MethodDispatchOutcome::Immediate(value) => self.registers[255] = value,
                MethodDispatchOutcome::Deferred => {}
            }
            return Ok(());
        }
        if self.dispatch_builtin_group1(name, arg_count, first_arg, bytecode)? {
            return Ok(());
        }
        if self.dispatch_builtin_group2(name, arg_count, first_arg, bytecode)? {
            return Ok(());
        }
        if self.dispatch_builtin_group3(name, arg_count, first_arg, bytecode)? {
            return Ok(());
        }
        if self.dispatch_builtin_group4(name, arg_count, first_arg, bytecode, call_site)? {
            return Ok(());
        }
        if self.dispatch_builtin_group5(name, arg_count, first_arg, bytecode)? {
            return Ok(());
        }
        if self.dispatch_builtin_group6(name, arg_count, first_arg, bytecode, call_site)? {
            return Ok(());
        }
        if self.dispatch_builtin_group7(name, arg_count, first_arg, bytecode)? {
            return Ok(());
        }
        Err(compile_codes::runtime_error(format!(
            "Unknown built-in function: {}",
            name
        )))
    }
}
