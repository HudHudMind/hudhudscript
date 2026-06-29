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
    pub(crate) fn call_set_method(
        &self,
        items: &[Value16],
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        crate::vm::set::call_set_method(items, method, &args)
    }

    /// Map method dispatch — Issue #654
    /// Delegates to shared implementation (Issue #904).
    pub(crate) fn call_map_method(
        &self,
        pairs: &[(Value16, Value16)],
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        crate::vm::map::call_map_method(pairs, method, &args)
    }

    /// Array method dispatch
    pub(crate) fn call_array_method(
        &mut self,
        mut receiver: Value16,
        method: &str,
        args: Vec<Value16>,
        bytecode: &Bytecode,
    ) -> CompileResult<Value16> {
        // Mutating methods: handle in-place (O(1)), compute return value
        // before/during mutation, store modified array in last_instance_mutation
        // for WriteBackReceiver writeback, and return early.
        match method {
            "push" => {
                if let Some(vec) = receiver.as_array_mut() {
                    for arg in &args {
                        vec.push(*arg);
                    }
                }
                self.last_instance_mutation = Some(Box::new(receiver));
                return Ok(receiver);
            }
            "pop" => {
                let ret = if let Some(vec) = receiver.as_array_mut() {
                    if vec.is_empty() {
                        return Err(compile_codes::runtime_error(
                            "Cannot pop from empty array".to_string(),
                        ));
                    }
                    vec.pop().unwrap_or(Value16::null())
                } else {
                    Value16::null()
                };
                self.last_instance_mutation = Some(Box::new(receiver));
                return Ok(ret);
            }
            "shift" => {
                let ret = if let Some(vec) = receiver.as_array_mut() {
                    if vec.is_empty() {
                        Value16::null()
                    } else {
                        vec.remove(0)
                    }
                } else {
                    Value16::null()
                };
                self.last_instance_mutation = Some(Box::new(receiver));
                return Ok(ret);
            }
            "unshift" => {
                if let Some(vec) = receiver.as_array_mut() {
                    let new_front: Vec<Value16> = args.iter().cloned().collect();
                    let old = std::mem::replace(vec, new_front);
                    vec.extend(old);
                }
                self.last_instance_mutation = Some(Box::new(receiver));
                return Ok(receiver);
            }
            _ => {}
        }

        // Non-mutating methods: borrow arr only after the mutation block is done.
        let arr: &[Value16] = receiver.as_array().map(|v| v.as_slice()).unwrap_or(&[]);

        // Non-callback methods (length/join/slice/concat/reverse/flat/indexOf/contains).
        if let Some(result) = crate::vm::array::call_array_method(arr, method, &args) {
            return result;
        }

        // Callback-dependent methods (map/filter/reduce/forEach/find/
        // some/every) route through the same shared implementation (Kural 7).
        let mut invoker = crate::vm::callback::VmCallbackInvoker { vm: self, bytecode };
        if let Some(result) =
            crate::vm::array::call_array_method_with_callback(arr, method, &args, &mut invoker)
        {
            return result;
        }

        Err(compile_codes::runtime_error(format!(
            "Unknown array method: {}",
            method
        )))
    }

    /// String method dispatch
    pub(crate) fn call_string_method(
        &self,
        s: &str,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        crate::vm::string::call_string_method(s, method, &args)
    }

    /// Call a Value as a function (used by map/filter/reduce/etc.)
    pub(crate) fn call_value_as_function(
        &mut self,
        func: &Value16,
        args: Vec<Value16>,
        bytecode: &Bytecode,
    ) -> CompileResult<Value16> {
        if let Some(func_data) = func.as_function_data() {
            let chunk_name = &func_data.chunk_name;
            let params = &func_data.params;
            let captures = &func_data.captures;
            let chunk = bytecode
                .get_function(chunk_name)
                .ok_or_else(|| {
                    compile_codes::runtime_error(format!(
                        "Function chunk not found: {}",
                        chunk_name
                    ))
                })?
                .clone();
            self.call_chunk_with_captures(&chunk, params, &args, bytecode, chunk_name, captures)
        } else {
            Err(compile_codes::runtime_error(format!(
                "Expected function, got {:?}",
                func
            )))
        }
    }

    // ── Math methods ──────────────────────────────────────────────────

    pub(crate) fn call_math_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_math::math::MathMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── JSON methods ─────────────────────────────────────────────────

    pub(crate) fn call_json_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_http::json::JsonMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── TOML methods (v0.4.38 — #650) ─────────────────────────────────
}
