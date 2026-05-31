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
    pub(crate) fn dispatch_builtin_group6(
        &mut self,
        name: &str,
        arg_count: u8,
        first_arg: u8,
        bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<bool> {
        match name {
            "atomically" => {
                // Shared STM (Kural 7 / #859). The retry loop, version
                // tracking, conflict detection and backoff policy all live
                // in `hudhudscript-stm` — this handler just installs a
                // `Transaction` on `self.current_tx` around each attempt and
                // defers commit/retry decisions to the shared layer.
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "atomically() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let func = self.registers[first_arg as usize];

                if let Some(func_data) = func.as_function_data() {
                    let chunk_name = &func_data.chunk_name;
                    let params = &func_data.params;
                    let captures = &func_data.captures;
                    if let Some(chunk) = bytecode.functions.borrow().get(chunk_name.as_str()).cloned() {
                        let params = params.clone();
                        let captures = captures.clone();

                        let config = hudhudscript_stm::StmConfig::default();
                        let start = std::time::Instant::now();
                        let mut backoff_us = config.initial_backoff_us;
                        let mut final_val: Option<Value16> = None;

                        for _attempt in 0..config.max_retries {
                            let elapsed_ms = start.elapsed().as_millis() as u64;
                            if elapsed_ms > config.timeout_ms {
                                return Err(compile_codes::runtime_error(
                                    hudhudscript_stm::err_timeout(config.timeout_ms, elapsed_ms)
                                        .message,
                                ));
                            }

                            // Install a fresh Transaction for this attempt.
                            self.current_tx = Some(Box::new(hudhudscript_stm::Transaction::new()));
                            self.in_stm_context = true;

                            let result = self.call_chunk_with_captures(
                                &chunk,
                                &params,
                                &[],
                                bytecode,
                                "atomically",
                                &captures,
                            );

                            self.in_stm_context = false;
                            let tx = self.current_tx.take();

                            match result {
                                Ok(val) => {
                                    let committed = tx.map(|tx| tx.try_commit()).unwrap_or(true);
                                    if committed {
                                        final_val = Some(val);
                                        break;
                                    }
                                    // Conflict — back off and retry.
                                    if backoff_us < 10 {
                                        std::thread::yield_now();
                                    } else {
                                        std::thread::sleep(std::time::Duration::from_micros(
                                            backoff_us,
                                        ));
                                    }
                                    backoff_us = (backoff_us * 2).min(config.max_backoff_us);
                                }
                                Err(e) => {
                                    // Body failed — drop transaction, propagate.
                                    return Err(e);
                                }
                            }
                        }

                        match final_val {
                            Some(v) => self.registers[255] = v,
                            None => {
                                return Err(compile_codes::runtime_error(
                                    hudhudscript_stm::err_max_retries_exceeded(config.max_retries)
                                        .message,
                                ));
                            }
                        }
                    } else {
                        self.registers[255] = Value16::null();
                    }
                } else {
                    return Err(compile_codes::runtime_error(
                        "atomically() expects a function argument".to_string(),
                    ));
                }
                Ok(true)
            }
            "spawn_actor" => {
                // Shared actor registry (Kural 7). Previously this handler
                // built a fake `{name, mailbox: [], alive: true}` object
                // that looked like an actor but had no real channel —
                // every `send_actor` / `receive_actor` against it silently
                // became a no-op. Now we mint a live actor via the shared
                // registry and store the receive-side mailbox on this VM
                // so `receive_actor` can drain it later, matching
                // `Instruction::Spawn` semantics so script-level
                // `spawn Agent(...)` and builtin `spawn_actor("name")`
                // share the same execution path.
                if arg_count < 1 {
                    return Err(compile_codes::runtime_error(
                        "spawn_actor() requires at least 1 argument".to_string(),
                    ));
                }
                let mut args: Vec<Value16> = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let actor_name = self.value_to_string(&args[0]);

                let (actor_ref, mailbox) = self.actors.spawn();
                let actor_id = actor_ref.id.clone();
                self.actor_mailboxes.insert(actor_id.clone(), mailbox);

                let mut actor = HashMap::new();
                actor.insert("__type".to_string(), Value16::string("actor".to_string()));
                actor.insert(
                    "__kind__".to_string(),
                    Value16::string("actor_ref".to_string()),
                );
                actor.insert("__actor_id".to_string(), Value16::string(actor_id));
                actor.insert("name".to_string(), Value16::string(actor_name));
                actor.insert("alive".to_string(), Value16::bool_(true));
                self.registers[255] = Value16::object(actor);
                Ok(true)
            }

            // Higher-order array functions (Issue #357) — standalone call form
            "map" | "filter" | "reduce" | "forEach" | "find" | "some" | "every" => {
                // Standalone higher-order form (Issue #357): `map(arr, fn)`
                // rather than `arr.map(fn)`. Both forms now flow through the
                // single shared callback dispatcher in
                // `crate::vm::array::call_array_method_with_callback`
                // via the `VmCallbackInvoker` adapter — Kural 7.
                //
                // Stack layout after the variadic pop+reverse: [arr, callback, …extras].
                // `reduce(arr, fn, initial)` exposes its initial value as the
                // second trailing arg so the shared reducer picks it up at
                // args[1] (the shared impl's rule: `args[0]` is the callback,
                // `args[1]` (optional) is the initial accumulator).
                let builtin_name = name.to_string();
                let mut args: Vec<Value16> = Vec::with_capacity(arg_count as usize);
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                if args.is_empty() {
                    return Err(compile_codes::runtime_error(format!(
                        "{}() requires at least 1 argument",
                        builtin_name
                    )));
                }
                let receiver = args.remove(0);
                let arr = match receiver.as_array() {
                    Some(a) => a.clone(),
                    None => {
                        return Err(compile_codes::runtime_error(format!(
                            "{}() requires an array as first argument, got {}",
                            builtin_name,
                            Self::bytecode_value_type_name(&receiver),
                        )));
                    }
                };

                let mut invoker = crate::vm::callback::VmCallbackInvoker { vm: self, bytecode };
                let result = crate::vm::array::call_array_method_with_callback(
                    &arr,
                    &builtin_name,
                    &args,
                    &mut invoker,
                )
                .ok_or_else(|| {
                    compile_codes::runtime_error(format!(
                        "{}() is not a callback-based array method",
                        builtin_name
                    ))
                })??;
                self.registers[255] = result;
                Ok(true)
            }

            // sleep/delay (v0.4.38 — #655)
            "sleep" | "delay" => {
                self.check_arg_count(name, 1, arg_count)?;
                let val = self.registers[first_arg as usize];
                let ms = if let Some(n) = val.as_number() {
                    if n < 0.0 {
                        return Err(compile_codes::runtime_error(
                            "sleep duration must be non-negative".to_string(),
                        ));
                    }
                    n as u64
                } else {
                    return Err(compile_codes::runtime_error(format!(
                        "{}() expects a number argument",
                        name
                    )));
                };
                std::thread::sleep(std::time::Duration::from_millis(ms));
                self.registers[255] = Value16::null();
                Ok(true)
            }

            // stdin functions (INPUT0001: delegate to hudhud_term stdin_ops)
            "input" | "oku" | "gir" | "eingabe" | "leer" | "lire" => {
                let mut args = Vec::new();
                for _i in 0..arg_count { args.push(self.registers[first_arg as usize + _i as usize]); }
                args.reverse();
                self.registers[255] = hudhud_term::stdin_ops::dispatch(
                    hudhud_term::stdin_ops::StdinMethodId::Read, &args,
                )?;
                Ok(true)
            }
            "input_hidden" => {
                let mut args = Vec::new();
                for _i in 0..arg_count { args.push(self.registers[first_arg as usize + _i as usize]); }
                args.reverse();
                self.registers[255] = hudhud_term::stdin_ops::dispatch(
                    hudhud_term::stdin_ops::StdinMethodId::Password, &args,
                )?;
                Ok(true)
            }
            "confirm" => {
                let mut args = Vec::new();
                for _i in 0..arg_count { args.push(self.registers[first_arg as usize + _i as usize]); }
                args.reverse();
                self.registers[255] = hudhud_term::stdin_ops::dispatch(
                    hudhud_term::stdin_ops::StdinMethodId::Confirm, &args,
                )?;
                Ok(true)
            }

            _ => Ok(false),
        }
    }
}
