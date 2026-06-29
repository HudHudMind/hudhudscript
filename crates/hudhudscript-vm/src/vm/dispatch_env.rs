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
    pub(crate) fn dispatch_builtin_group3(
        &mut self,
        name: &str,
        arg_count: u8,
        first_arg: u8,
        _bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<bool> {
        match name {
            "is_some" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "is_some() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let val_v = val;
                if let Some(opt) = val_v.as_option() {
                    self.registers[255] = Value16::bool_(opt.is_some());
                } else if val_v.is_null() {
                    self.registers[255] = Value16::bool_(false);
                } else {
                    return Err(compile_codes::runtime_error(
                        "is_some() requires Option".to_string(),
                    ));
                }
                Ok(true)
            }
            "is_none" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "is_none() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let val_v = val;
                if let Some(opt) = val_v.as_option() {
                    self.registers[255] = Value16::bool_(opt.is_none());
                } else if val_v.is_null() {
                    self.registers[255] = Value16::bool_(true);
                } else {
                    return Err(compile_codes::runtime_error(
                        "is_none() requires Option".to_string(),
                    ));
                }
                Ok(true)
            }
            "is_ok" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "is_ok() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let val_v = val;
                if let Some(res) = val_v.as_result() {
                    self.registers[255] = Value16::bool_(res.is_ok());
                } else {
                    return Err(compile_codes::runtime_error(
                        "is_ok() requires Result".to_string(),
                    ));
                }
                Ok(true)
            }
            "is_err" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "is_err() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let val_v = val;
                if let Some(res) = val_v.as_result() {
                    self.registers[255] = Value16::bool_(res.is_err());
                } else {
                    return Err(compile_codes::runtime_error(
                        "is_err() requires Result".to_string(),
                    ));
                }
                Ok(true)
            }

            // exception() constructor — canonical exception type (lowercase)
            "exception" | "istisna" => {
                if arg_count < 2 {
                    return Err(compile_codes::runtime_error(
                        "exception() requires at least 2 arguments: code, title".to_string(),
                    ));
                }
                let code = self.registers[first_arg as usize]
                    .as_string()
                    .ok_or_else(|| {
                        compile_codes::runtime_error("exception() code must be string".to_string())
                    })?;
                let title = self.registers[first_arg as usize + 1]
                    .as_string()
                    .ok_or_else(|| {
                        compile_codes::runtime_error("exception() title must be string".to_string())
                    })?;
                let description = if arg_count >= 3 {
                    self.registers[first_arg as usize + 2]
                        .as_string()
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                let value = if arg_count >= 4 {
                    self.registers[first_arg as usize + 3]
                } else {
                    Value16::null()
                };
                self.registers[255] =
                    crate::vm::exception_value::make_exception(&code, &title, &description, value);
                Ok(true)
            }

            // env() function
            "env" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "env() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                if let Some(key) = val.as_string() {
                    // HOST-5: enforce env read policy before lookup.
                    self.host_access_policy.ensure_env_read(&key)?;
                    let result = self.env_lookup(&key);
                    self.registers[255] = result;
                } else {
                    return Err(compile_codes::runtime_error(
                        "env() requires a string key".to_string(),
                    ));
                }
                Ok(true)
            }

            // Promise constructors and combinators
            "Promise.resolve" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "Promise.resolve() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let val_v = val;
                self.registers[255] = Value16::promise(
                    hudhudscript_bytecode::PromiseState::Resolved(Box::new(val_v)),
                );
                Ok(true)
            }
            "Promise.reject" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "Promise.reject() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let msg = self.value_to_string(&val);
                self.registers[255] =
                    Value16::promise(hudhudscript_bytecode::PromiseState::Rejected(msg));
                Ok(true)
            }
            "Promise.all" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "Promise.all() expects 1 argument (array), got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                if let Some(promises) = val.as_array() {
                    let promises_v: Vec<Value16> = promises.clone();
                    // P1-4: concurrent Promise.all — awaits every
                    // AsyncPending entry on its own OS thread via the
                    // shared registry, so wall-clock is max(d_i) rather
                    // than sum(d_i). Matches the interpreter's
                    // `eval_promise_all_async` semantics.
                    match self.resolve_promise_all(promises_v) {
                        Ok(values) => {
                            self.registers[255] =
                                Value16::promise(hudhudscript_bytecode::PromiseState::Resolved(
                                    Box::new(Value16::array(values)),
                                ));
                        }
                        Err(msg) => {
                            self.registers[255] = Value16::promise(
                                hudhudscript_bytecode::PromiseState::Rejected(msg),
                            );
                        }
                    }
                } else {
                    return Err(compile_codes::runtime_error(
                        "Promise.all() requires an array".to_string(),
                    ));
                }
                Ok(true)
            }
            "Promise.race" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "Promise.race() expects 1 argument (array), got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                if let Some(promises) = val.as_array() {
                    // P1-3: concurrent Promise.race — races every
                    // AsyncPending entry on its own OS thread via the
                    // shared registry, returning the first to settle
                    // (resolve or reject). Matches the interpreter's
                    // `eval_promise_race_async` semantics. An empty
                    // array rejects with the canonical message rather
                    // than hanging.
                    match self.resolve_promise_race(promises.clone()) {
                        Ok(val) => {
                            self.registers[255] = Value16::promise(
                                hudhudscript_bytecode::PromiseState16::Resolved(Box::new(val)),
                            )
                        }
                        Err(msg) => {
                            self.registers[255] = Value16::promise(
                                hudhudscript_bytecode::PromiseState16::Rejected(msg),
                            )
                        }
                    }
                } else {
                    return Err(compile_codes::runtime_error(
                        "Promise.race() requires an array".to_string(),
                    ));
                }
                Ok(true)
            }

            "Promise.allSettled" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "Promise.allSettled() expects 1 argument (array), got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                if let Some(promises) = val.as_array() {
                    let mut results = Vec::with_capacity(promises.len());
                    for p in promises.iter() {
                        // Kural 7b: actually await AsyncPending entries
                        // rather than treating them as silently fulfilled.
                        if let Some(ps) = p.as_promise_state() {
                            match ps {
                                hudhudscript_bytecode::PromiseState16::Resolved(v) => {
                                    let mut obj = hudhudscript_bytecode::ObjMap::default();
                                    obj.insert(
                                        "status".to_string(),
                                        Value16::string("fulfilled".to_string()),
                                    );
                                    obj.insert("value".to_string(), (**v).clone());
                                    results.push(Value16::object(obj));
                                }
                                hudhudscript_bytecode::PromiseState16::Rejected(reason) => {
                                    let mut obj = hudhudscript_bytecode::ObjMap::default();
                                    obj.insert(
                                        "status".to_string(),
                                        Value16::string("rejected".to_string()),
                                    );
                                    obj.insert(
                                        "reason".to_string(),
                                        Value16::string(reason.clone()),
                                    );
                                    results.push(Value16::object(obj));
                                }
                                hudhudscript_bytecode::PromiseState16::AsyncPending(id) => {
                                    let async_pending = Value16::promise(
                                        hudhudscript_bytecode::PromiseState16::AsyncPending(
                                            id.clone(),
                                        ),
                                    );
                                    let mut obj = hudhudscript_bytecode::ObjMap::default();
                                    match self.resolve_promise_value(async_pending) {
                                        Ok(val) => {
                                            obj.insert(
                                                "status".to_string(),
                                                Value16::string("fulfilled".to_string()),
                                            );
                                            obj.insert("value".to_string(), val);
                                        }
                                        Err(reason) => {
                                            obj.insert(
                                                "status".to_string(),
                                                Value16::string("rejected".to_string()),
                                            );
                                            obj.insert(
                                                "reason".to_string(),
                                                Value16::string(reason),
                                            );
                                        }
                                    }
                                    results.push(Value16::object(obj));
                                }
                                hudhudscript_bytecode::PromiseState16::Pending => {
                                    // Original behaviour: bare Pending fell through
                                    // to the `other =>` arm and surfaced as
                                    // `fulfilled` with the Pending promise itself
                                    // as the value. Preserve that here to stay in
                                    // lock-step with the interpreter.
                                    let mut obj = hudhudscript_bytecode::ObjMap::default();
                                    obj.insert(
                                        "status".to_string(),
                                        Value16::string("fulfilled".to_string()),
                                    );
                                    obj.insert(
                                        "value".to_string(),
                                        Value16::promise(
                                            hudhudscript_bytecode::PromiseState16::Pending,
                                        ),
                                    );
                                    results.push(Value16::object(obj));
                                }
                            }
                        } else {
                            let mut obj = hudhudscript_bytecode::ObjMap::default();
                            obj.insert(
                                "status".to_string(),
                                Value16::string("fulfilled".to_string()),
                            );
                            obj.insert("value".to_string(), p.clone());
                            results.push(Value16::object(obj));
                        }
                    }
                    self.registers[255] =
                        Value16::promise(hudhudscript_bytecode::PromiseState16::Resolved(
                            Box::new(Value16::array(results)),
                        ));
                } else {
                    return Err(compile_codes::runtime_error(
                        "Promise.allSettled() requires an array".to_string(),
                    ));
                }
                Ok(true)
            }

            // SOP Runtime builtins
            _ => Ok(false),
        }
    }
}
