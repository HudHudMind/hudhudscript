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

impl crate::vm::VM {
    pub(crate) fn check_arg_count(
        &self,
        name: &str,
        expected: u8,
        actual: u8,
    ) -> CompileResult<()> {
        if expected != actual {
            return Err(compile_codes::runtime_error(format!(
                "{} expects {} argument(s), got {}",
                name, expected, actual
            )));
        }
        Ok(())
    }

    pub(crate) fn lookup_method_in_parent_chain(
        &mut self,
        class: &Value16,
        method: &str,
        args: &[Value16],
        bytecode: &Bytecode,
    ) -> CompileResult<Value16> {
        // Extract parent from class value
        let parent = if let Some(c) = class.as_class_data() {
            &c.parent
        } else {
            return Err(compile_codes::runtime_error(
                "lookup_method_in_parent_chain: expected Class value".to_string(),
            ));
        };

        if let Some(parent_value) = parent {
            if let Some(parent_name) = parent_value.as_string() {
                // Get parent class from variables
                if let Some(parent_class) = self.get_var_cloned(&parent_name) {
                    // Check parent's vtable (v4.3: packed int value).
                    if let Some(parent_cls) = parent_class.as_class_data() {
                        if let Some(packed) =
                            parent_cls.vtable.get(method).and_then(|v| v.as_int())
                        {
                            let idx = (packed >> 32) as u32;
                            let chunk_sym = hudhudscript_bytecode::SymId(packed as u32);
                            let chunk = bytecode.get_function_by_index(idx)
                                .ok_or_else(|| crate::vm::builtin_method_dispatch::err_function_idx_missing(idx))?;
                            return self.call_chunk(
                                &chunk,
                                &chunk.params,
                                &args,
                                bytecode,
                                chunk_sym,
                            );
                        }
                    }
                    // Recursively search further up the chain
                    return self.lookup_method_in_parent_chain(
                        &parent_class,
                        method,
                        args,
                        bytecode,
                    );
                }
            }
        }

        Err(compile_codes::runtime_error(format!(
            "Method '{}' not found in class hierarchy",
            method
        )))
    }

    /// Call a method on a value (array/string/object method dispatch).
    /// Issue #1012: Eagerly drain a custom iterator (Instance or Object with a
    /// `next()` method) into a `Vec<Value>` for use in for-in loops.
    /// Calls `next()` repeatedly until it returns `Null`.
    pub(crate) fn collect_custom_iterator(
        &mut self,
        receiver: Value16,
        bytecode: &Bytecode,
    ) -> CompileResult<Vec<Value16>> {
        let limit = self.max_builtin_iter;
        let mut elements = Vec::new();
        let next_sym = hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern("next").0);
        for _ in 0..limit {
            let val = self.call_method_on_value(&receiver, "next", next_sym, vec![], bytecode)?;
            if val.is_null() {
                break;
            }
            elements.push(val);
        }
        if elements.len() >= limit {
            return Err(compile_codes::runtime_error(format!(
                "Custom iterator exceeded maximum iteration limit of {}",
                limit
            )));
        }
        Ok(elements)
    }
}
