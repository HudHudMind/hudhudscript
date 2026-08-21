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
    pub(crate) fn dispatch_builtin_group5(
        &mut self,
        name: &str,
        arg_count: u8,
        first_arg: u8,
        _bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<bool> {
        match name {
            "check_constitution_compliance" => {
                // check_constitution_compliance(action) — check against active constitution
                // Uses hudhudscript-governance's enforce_constitution engine (#888)
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "check_constitution_compliance() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let action = self.registers[first_arg as usize];
                if let Some(ref const_name) = self.active_constitution {
                    if let Some(constitution) = self.constitutions.get(const_name).cloned() {
                        let eval_ctx = Self::value_to_eval_context(&action);
                        let enforcement_result =
                            enforce_constitution(&constitution, &eval_ctx, None);

                        let violations: Vec<Value16> = enforcement_result
                            .violations
                            .iter()
                            .map(|v| Value16::string(v.clone()))
                            .collect();

                        let mut result = hudhudscript_bytecode::ObjMap::default();
                        result.insert(
                            "compliant".to_string(),
                            Value16::bool_(enforcement_result.allowed),
                        );
                        result.insert(
                            "constitution".to_string(),
                            Value16::string(const_name.clone()),
                        );
                        result.insert("action".to_string(), action);
                        result.insert("violations".to_string(), Value16::array(violations));
                        self.registers[255] = Value16::object(result);
                    } else {
                        self.registers[255] = Value16::bool_(true);
                    }
                } else {
                    self.registers[255] = Value16::bool_(true);
                }
                Ok(true)
            }
            "remember" => {
                // RAG store as a value-returning call: remember(content[, store])
                // → the new entry's id. Delegates to `VM::rag_remember`, the
                // same implementation `Instruction::Remember` uses (Kural 7).
                if arg_count == 0 || arg_count > 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "remember() expects 1 or 2 arguments (content[, store]), got {}",
                        arg_count
                    )));
                }
                let content = self.registers[first_arg as usize];
                let store_key = if arg_count == 2 {
                    let store = self.registers[first_arg as usize + 1];
                    self.resolve_rag_store_key(&store)
                } else {
                    "default".to_string()
                };
                let entry_id = self.rag_remember(content, &store_key)?;
                self.registers[255] = Value16::string(entry_id);
                Ok(true)
            }
            "forget" => {
                // RAG delete as a value-returning call: forget(target[, store])
                // → how many entries were removed. An empty target clears the
                // store. Delegates to `VM::rag_forget`, the same implementation
                // `Instruction::Forget` uses (Kural 7).
                if arg_count == 0 || arg_count > 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "forget() expects 1 or 2 arguments (target[, store]), got {}",
                        arg_count
                    )));
                }
                let target = self.registers[first_arg as usize];
                let target_str = self.value_to_string(&target);
                let store_key = if arg_count == 2 {
                    let store = self.registers[first_arg as usize + 1];
                    self.resolve_rag_store_key(&store)
                } else {
                    "default".to_string()
                };
                let removed = self.rag_forget(&target_str, &store_key);
                self.registers[255] = Value16::number(removed as f64);
                Ok(true)
            }
            "recall" => {
                // RAG recall as a value-returning call: recall(query[, store]).
                // Delegates to `VM::rag_recall`, the same implementation
                // `Instruction::Recall` uses (Kural 7 — one code path).
                if arg_count == 0 || arg_count > 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "recall() expects 1 or 2 arguments (query[, store]), got {}",
                        arg_count
                    )));
                }
                let query = self.registers[first_arg as usize];
                let query_str = self.value_to_string(&query);
                let store_key = if arg_count == 2 {
                    let store = self.registers[first_arg as usize + 1];
                    self.resolve_rag_store_key(&store)
                } else {
                    "default".to_string()
                };
                let out = self.rag_recall(&query_str, &store_key);
                self.registers[255] = out;
                Ok(true)
            }
            "mcp_call" => {
                // Shared dispatch path (Kural 7). Argument order on the stack
                // after the variadic push is: server, tool, [arguments].
                // Non-string server/tool are stringified so callers can pass
                // identifiers resolved from upstream expressions.
                if arg_count < 2 {
                    return Err(compile_codes::runtime_error(
                        "mcp_call() requires at least 2 arguments (server, tool, [args])"
                            .to_string(),
                    ));
                }
                let mut args: Vec<Value16> = Vec::with_capacity(arg_count as usize);
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                // Args are in order: server, tool, [arguments] — no reverse needed.
                let server = self.value_to_string(&args[0]);
                let tool = self.value_to_string(&args[1]);
                let tool_args = args.get(2).cloned().unwrap_or(Value16::null());
                let result = dispatch_mcp_tool_call(self, &server, &tool, &tool_args)?;
                self.registers[255] = result;

                Ok(true)
            }
            // instanceof check
            "instanceof" => {
                if arg_count != 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "instanceof() expects 2 arguments, got {}",
                        arg_count
                    )));
                }
                let class_name_val = self.registers[first_arg as usize];
                let obj_val = self.registers[(first_arg + 1) as usize];
                let class_name = self.value_to_string(&class_name_val);
                let result = if let Some(obj) = obj_val.as_object() {
                    // Check __type field and walk parent chain
                    let obj_type = obj.get("__type").and_then(|v| v.as_string());
                    if obj_type.as_deref() == Some(&class_name) {
                        true
                    } else if let Some(type_name) = obj_type {
                        // Walk parent chain
                        let mut current = type_name.to_string();
                        let mut found = false;
                        while let Some((parent, _)) = self.classes.get(&current) {
                            if let Some(ref p) = parent {
                                if p == &class_name {
                                    found = true;
                                    break;
                                }
                                current = p.clone();
                            } else {
                                break;
                            }
                        }
                        found
                    } else {
                        false
                    }
                } else {
                    false
                };
                self.registers[255] = Value16::bool_(result);
                Ok(true)
            }
            // STM builtins — Kural 7: shared `hudhudscript-stm` impl.
            // `tvar_new("name", initial)` registers (or looks up) a named
            // TVar in the shared registry. `tvar_read`/`tvar_write` route
            // through the active `current_tx` when inside an `atomically(fn)`
            // block so the shared Transaction tracks version conflicts.
            "tvar_new" => {
                if arg_count < 1 {
                    return Err(compile_codes::runtime_error(
                        "tvar_new() requires at least 1 argument".to_string(),
                    ));
                }
                let mut args: Vec<Value16> = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                let name = self.value_to_string(&args[0]);
                let initial = args.get(1).cloned().unwrap_or(Value16::null());
                self.tvars.create_with_id(name.clone(), initial);
                self.registers[255] = Value16::string(name); // Return tvar handle (name
                Ok(true)
            }
            "tvar_read" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "tvar_read() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let name_val = self.registers[first_arg as usize];
                let name = self.value_to_string(&name_val);
                let tvar = self.tvars.get(&name);

                let val = match tvar {
                    None => Value16::null(),
                    Some(tvar_arc) => {
                        if let Some(tx) = self.current_tx.as_mut() {
                            tx.read(&tvar_arc)
                        } else {
                            tvar_arc.read_committed().0
                        }
                    }
                };
                self.registers[255] = val;

                Ok(true)
            }
            "tvar_write" => {
                if arg_count != 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "tvar_write() expects 2 arguments, got {}",
                        arg_count
                    )));
                }
                let value = self.registers[first_arg as usize + 1];
                let name_val = self.registers[first_arg as usize];
                let name = self.value_to_string(&name_val);

                // Auto-register unknown TVars as Null-initialised so stray
                // writes do not silently disappear. This matches the previous
                // HashMap-insert behaviour.
                let tvar_arc = match self.tvars.get(&name) {
                    Some(t) => t,
                    None => {
                        self.tvars.create_with_id(name.clone(), Value16::null());
                        self.tvars
                            .get(&name)
                            .expect("just-created TVar must be retrievable")
                    }
                };

                if let Some(tx) = self.current_tx.as_mut() {
                    tx.write(&tvar_arc, value);
                } else {
                    // Non-transactional write: one-shot commit through the
                    // shared atomically() so the version counter advances.
                    hudhudscript_stm::atomically::<Value16, _, _>(|tx| {
                        tx.write(&tvar_arc, value.clone());
                        Ok(())
                    })
                    .map_err(|e| compile_codes::runtime_error(e.message.clone()))?;
                }
                self.registers[255] = Value16::bool_(true);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn resolve_rag_store_key(&self, store: &Value16) -> String {
        if let Some(s) = store.as_string() {
            s
        } else if let Some(obj) = store.as_object() {
            obj.get("name")
                .or_else(|| obj.get("__name"))
                .or_else(|| obj.get("id"))
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| self.value_to_string(store))
        } else {
            self.value_to_string(store)
        }
    }
}
