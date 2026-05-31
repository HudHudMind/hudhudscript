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
    pub(crate) fn require_string_arg(
        &self,
        args: &[Value16],
        idx: usize,
        method: &str,
    ) -> CompileResult<String> {
        match args.get(idx) {
            Some(s) => s.as_string().ok_or_else(|| {
                compile_codes::runtime_error(format!("{} requires a string argument", method))
            }),
            None => Err(compile_codes::runtime_error(format!(
                "{} requires an argument",
                method
            ))),
        }
    }

    // ── Promise methods ──────────────────────────────────────────────

    pub(crate) fn call_promise_method(
        &mut self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        match method {
            "resolve" => {
                let val = args.into_iter().next().unwrap_or(Value16::null());
                Ok(Value16::promise(
                    hudhudscript_bytecode::PromiseState16::Resolved(Box::new(val)),
                ))
            }
            "reject" => {
                let val = args.into_iter().next().unwrap_or(Value16::null());
                let msg = self.value_to_string(&val);
                Ok(Value16::promise(
                    hudhudscript_bytecode::PromiseState16::Rejected(msg),
                ))
            }
            "all" => {
                let arr = args.into_iter().next().unwrap_or(Value16::array(vec![]));
                if let Some(promises) = arr.as_array() {
                    // P1-4: concurrent Promise.all via shared registry.
                    // See `resolve_promise_all` for full semantics.
                    match self.resolve_promise_all(promises.to_vec()) {
                        Ok(values) => Ok(Value16::promise(
                            hudhudscript_bytecode::PromiseState16::Resolved(Box::new(
                                Value16::array(values),
                            )),
                        )),
                        Err(msg) => Ok(Value16::promise(
                            hudhudscript_bytecode::PromiseState16::Rejected(msg),
                        )),
                    }
                } else {
                    Err(compile_codes::runtime_error(
                        "Promise.all() requires an array".to_string(),
                    ))
                }
            }
            "race" => {
                let arr = args.into_iter().next().unwrap_or(Value16::array(vec![]));
                if let Some(promises) = arr.as_array() {
                    // P1-3: concurrent Promise.race via shared registry.
                    // See `resolve_promise_race` for full semantics.
                    match self.resolve_promise_race(promises.to_vec()) {
                        Ok(val) => Ok(Value16::promise(
                            hudhudscript_bytecode::PromiseState16::Resolved(Box::new(val)),
                        )),
                        Err(msg) => Ok(Value16::promise(
                            hudhudscript_bytecode::PromiseState16::Rejected(msg),
                        )),
                    }
                } else {
                    Err(compile_codes::runtime_error(
                        "Promise.race() requires an array".to_string(),
                    ))
                }
            }
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown Promise method: {}",
                method
            ))),
        }
    }

    // ── HTTP methods ─────────────────────────────────────────────────

    pub(crate) fn call_http_method(
        &mut self,
        method_name: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        // ── Sandbox permission check (#515) ──────────────────────────
        if let Some(ref sandbox) = self.sandbox {
            if !sandbox.allow_network {
                return Err(compile_codes::runtime_error(
                    "Sandbox: network access is not allowed".to_string(),
                ));
            }
        }

        // ── Sandbox: allowed hosts check (#515) ─────────────────────
        if let Some(ref sandbox) = self.sandbox {
            if !sandbox.allowed_hosts.is_empty() {
                let url_str = args.first().and_then(|v| v.as_string());
                if let Some(ref url) = url_str {
                    let host = hudhud_http::http_ops::parse_url_host(url);
                    match host {
                        Some(h) if sandbox.allowed_hosts.iter().any(|allowed| allowed == &h) => {}
                        _ => {
                            return Err(compile_codes::runtime_error(format!(
                                "Sandbox: host is not in the allowed list for URL '{}'",
                                url
                            )));
                        }
                    }
                }
            }
        }

        {
            let id = method_name.parse::<hudhud_http::http_ops::HttpMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── File methods ────────────────────────────────────────────────

    pub(crate) fn call_file_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        // ── Sandbox permission check (#515) ──────────────────────────
        if let Some(ref sandbox) = self.sandbox {
            let is_read = matches!(method, "read" | "exists" | "list");
            let is_write = matches!(method, "write" | "append" | "delete");
            if is_read && !sandbox.allow_file_read {
                return Err(compile_codes::runtime_error(
                    "Sandbox: file read access is not allowed".to_string(),
                ));
            }
            if is_write && !sandbox.allow_file_write {
                return Err(compile_codes::runtime_error(
                    "Sandbox: file write access is not allowed".to_string(),
                ));
            }
            // Path allow-list check
            if !sandbox.allowed_paths.is_empty() {
                if let Some(p) = args.first().and_then(|v| v.as_string()) {
                    let path = std::path::Path::new(&p);
                    let allowed = sandbox.allowed_paths.iter().any(|ap| path.starts_with(ap));
                    if !allowed {
                        return Err(compile_codes::runtime_error(format!(
                            "Sandbox: path '{}' is not in the allowed paths list",
                            p
                        )));
                    }
                }
            }
        }

        hudhud_fs::file_ops::dispatch(method, &args)
    }

    // ── Linear algebra methods ────────────────────────────────────────

    pub(crate) fn call_linalg_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_linalg::linalg::LinAlgMethodId>()?;
            hudhud_linalg::linalg::dispatch(id, &args)
        }
    }

    // ── Statistics methods ───────────────────────────────────────────

    pub(crate) fn call_stats_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_stats::stats::StatsMethodId>()?;
            hudhud_stats::stats::dispatch(id, &args)
        }
    }

    // ── env lookup ──────────────────────────────────────────────────

    pub(crate) fn env_lookup(key: &str) -> Value16 {
        // Check environment variable first
        if let Ok(val) = std::env::var(key) {
            return Value16::string(val);
        }
        // Search .env files
        let search_dirs = [".", "..", "../..", "../../.."];
        for dir in &search_dirs {
            let path = std::path::Path::new(dir).join(".env");
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((k, v)) = line.split_once('=') {
                        let k = k.trim();
                        let v = v.trim().trim_matches('"').trim_matches('\'');
                        if k == key {
                            return Value16::string(v.to_string());
                        }
                    }
                }
            }
        }
        Value16::null()
    }

    pub(crate) fn type_name_of(&self, value: &Value16) -> &'static str {
        if value.is_null() {
            "null"
        } else if value.as_bool().is_some() {
            "boolean"
        } else if value.as_int().is_some() || value.as_number().is_some() {
            "number"
        } else if value.as_string().is_some() {
            "string"
        } else if value.as_array().is_some() {
            "array"
        } else if value.as_object().is_some() {
            "object"
        } else if value.as_function_data().is_some() {
            "function"
        } else if value.as_promise_state().is_some() {
            "promise"
        } else if value.as_option().is_some() {
            "option"
        } else if value.as_result().is_some() {
            "result"
        } else if value.as_class_data().is_some() {
            "class"
        } else if value.as_instance_data().is_some() {
            "instance"
        } else if value.as_data_data().is_some() {
            "data"
        } else if value.as_tool_ref().is_some() {
            "tool"
        } else if value.as_resource_ref().is_some() {
            "resource"
        } else if value.as_set().is_some() {
            "set"
        } else if value.as_map_pairs().is_some() {
            "map"
        } else if value.as_generator_state().is_some() {
            "generator"
        } else {
            "unknown"
        }
    }
}
