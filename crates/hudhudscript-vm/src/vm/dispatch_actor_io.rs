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
        call_site: crate::vm::call_state::DeferredCallSite,
    ) -> hudhudscript_bytecode::error::CompileResult<bool> {
        match name {
            "atomically" => {
                // Shared STM (Kural 7 / #859). The retry loop, version
                // tracking, conflict detection and backoff policy all live
                // in `hudhudscript-stm` — this handler installs a fresh
                // `Transaction` on `self.current_tx` around each attempt on
                // the trampoline and defers commit/retry decisions to the
                // AtomicTransactionAttempt continuation.
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "atomically() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let func = self.registers[first_arg as usize];

                if let Some(func_data) = func.as_function_data() {
                    if let Some(chunk) = bytecode.get_function(func_data.chunk_name.as_str()) {
                        let config = hudhudscript_stm::StmConfig::default();
                        let initial_backoff_us = config.initial_backoff_us;
                        let captures: rustc_hash::FxHashMap<
                            String,
                            Arc<parking_lot::RwLock<Value16>>,
                        > = func_data
                            .captures
                            .iter()
                            .map(|(name, cell)| (name.clone(), Arc::clone(cell)))
                            .collect();
                        let state = crate::vm::call_state::AtomicTransactionAttemptState {
                            function: func,
                            chunk,
                            func_sym: hudhudscript_bytecode::SymId(func_data.chunk_sym),
                            captures,
                            dst: call_site.dst,
                            origin_ip: call_site.origin_ip,
                            attempt: 0,
                            started_at: std::time::Instant::now(),
                            config,
                            backoff_us: initial_backoff_us,
                        };
                        self.start_atomic_transaction_attempt(state)?;
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

                let mut actor = hudhudscript_bytecode::ObjMap::default();
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
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                self.registers[255] = hudhud_term::stdin_ops::dispatch(
                    hudhud_term::stdin_ops::StdinMethodId::Read,
                    &args,
                )?;
                Ok(true)
            }
            "input_hidden" => {
                let mut args = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                self.registers[255] = hudhud_term::stdin_ops::dispatch(
                    hudhud_term::stdin_ops::StdinMethodId::Password,
                    &args,
                )?;
                Ok(true)
            }
            "confirm" => {
                let mut args = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                self.registers[255] = hudhud_term::stdin_ops::dispatch(
                    hudhud_term::stdin_ops::StdinMethodId::Confirm,
                    &args,
                )?;
                Ok(true)
            }

            _ => Ok(false),
        }
    }
}
