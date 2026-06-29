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
    pub(crate) fn call_env_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        // HOST-5: enforce host_access Env.* policy per method.
        let extract_key = |args: &[Value16]| -> CompileResult<String> {
            args.first()
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    compile_codes::runtime_error(
                        "Env method expects a string key as first argument".to_string(),
                    )
                })
        };
        match method {
            "get" | "has" => {
                let key = extract_key(&args)?;
                self.host_access_policy.ensure_env_read(&key)?;
            }
            "set" => {
                let key = extract_key(&args)?;
                self.host_access_policy.ensure_env_write(&key)?;
            }
            "remove" => {
                let key = extract_key(&args)?;
                self.host_access_policy.ensure_env_remove(&key)?;
            }
            "all" => {
                self.host_access_policy.ensure_env_all()?;
            }
            "all_unfiltered" => {
                self.host_access_policy.ensure_env_all_unfiltered()?;
            }
            _ => {}
        }

        {
            let id = method.parse::<hudhud_env::env_ops::EnvMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── OS info module (v0.4.38 — #622) ─────────────────────────────────

    pub(crate) fn call_os_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_os::os_ops::OsMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── Date/Time module (v0.4.38 — #593) ───────────────────────────────

    pub(crate) fn call_date_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_datetime::date::DateMethodId>()?;
            hudhud_datetime::date::dispatch(id, &args)
        }
    }

    // ── Duration module (v0.4.38 — #593) ────────────────────────────────

    pub(crate) fn call_duration_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_datetime::duration::DurationMethodId>()?;
            hudhud_datetime::duration::dispatch(id, &args)
        }
    }

    // ── Regex module (v0.4.38 — #592) ───────────────────────────────────

    pub(crate) fn call_regex_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_regex::regex_ops::RegexMethodId>()?;
            hudhud_regex::regex_ops::dispatch(id, &args)
        }
    }

    // ── Schedule module (v0.4.38 — #618) ────────────────────────────────

    pub(crate) fn call_schedule_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_scheduler::schedule_ops::ScriptMethodId>()?;
            hudhud_scheduler::schedule_ops::dispatch(id, &args)
        }
    }

    // ── IPC / Event Bus (v0.4.38 — #597) ────────────────────────────────

    pub(crate) fn call_event_bus_method(
        &mut self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_eventbus::event_bus_ops::EventBusMethodId>()?;
            hudhud_eventbus::event_bus_ops::dispatch(id, &args)
        }
    }

    // ── Plugin lifecycle (v0.4.38 — #598) — shared dispatch (Kural 7) ───

    pub(crate) fn call_plugin_method(
        &mut self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_plugin::plugin_ops::ScriptMethodId>()?;
            hudhud_plugin::plugin_ops::dispatch(id, &args)
        }
    }

    // ── MCP Server mode (v0.4.38 — #600) ────────────────────────────────
    //
    // Delegates to shared `mcp_server_ops` (Kural 7). The global server
    // registry lives in `crate::vm::mcp_server_ops`.

    pub(crate) fn call_mcp_server_method(
        &mut self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        crate::vm::mcp_server_ops::call_mcp_server_method(method, &args)
    }

    // ── HTTP Server (v0.4.38 — #602) ────────────────────────────────────
    //
    // Delegates to shared `http_server_ops` (Kural 7). Real TcpListener +
    // background accept loop lives there; VM's previous stub is eliminated.

    pub(crate) fn call_server_method(
        &mut self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        hudhud_http::http_server_ops::dispatch_str(method, &args)
    }

    // ── Per-plugin config (v0.4.38 — #610) ──────────────────────────────

    pub(crate) fn call_plugin_config_method(
        &mut self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        match method {
            "load" => {
                let plugin_name = match args.first().and_then(|v| v.as_string()) {
                    Some(s) => s,
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.load: expected plugin name string".to_string(),
                        ))
                    }
                };
                let system_path = format!("/etc/hudhud/plugins/{}.toml", plugin_name);
                let user_path = match std::env::var("HOME") {
                    Ok(home) => format!("{}/.config/hudhud/plugins/{}.toml", home, plugin_name),
                    Err(_) => format!("~/.config/hudhud/plugins/{}.toml", plugin_name),
                };
                let mut config = hudhudscript_bytecode::ObjMap::default();
                // Try loading from system and user paths using shared toml_ops
                for path in [&system_path, &user_path] {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if let Ok(parsed) =
                            hudhud_serial::toml_ops::parse(&[Value16::string(content)])
                        {
                            if let Some(obj) = parsed.as_object() {
                                for (k, v) in obj.iter() {
                                    config.insert(k.clone(), v.clone());
                                }
                            }
                        }
                    }
                }
                // Env var overrides
                let prefix = format!(
                    "HUDHUD_PLUGIN_{}_",
                    plugin_name.to_uppercase().replace('-', "_")
                );
                for (key, val) in std::env::vars() {
                    if let Some(suffix) = key.strip_prefix(&prefix) {
                        config.insert(suffix.to_lowercase(), Value16::string(val));
                    }
                }
                config.insert("__plugin".to_string(), Value16::string(plugin_name));
                config.insert("__system_path".to_string(), Value16::string(system_path));
                config.insert("__user_path".to_string(), Value16::string(user_path));
                Ok(Value16::object(config))
            }
            "get" => {
                let config = match args.first().and_then(|v| v.as_object()) {
                    Some(o) => o,
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.get: expected config object".to_string(),
                        ))
                    }
                };
                let key = match args.get(1).and_then(|v| v.as_string()) {
                    Some(s) => s,
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.get: expected key string".to_string(),
                        ))
                    }
                };
                let parts: Vec<&str> = key.split('.').collect();
                let mut current = Value16::object(config.clone());
                for part in &parts {
                    match current.as_object() {
                        Some(obj) => match obj.get(*part) {
                            Some(v) => current = v.clone(),
                            None => return Ok(Value16::null()),
                        },
                        None => return Ok(Value16::null()),
                    }
                }
                Ok(current)
            }
            "set" => {
                let mut config = match args.first().and_then(|v| v.as_object()) {
                    Some(o) => o.clone(),
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.set: expected config object".to_string(),
                        ))
                    }
                };
                let key = match args.get(1).and_then(|v| v.as_string()) {
                    Some(s) => s,
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.set: expected key string".to_string(),
                        ))
                    }
                };
                let value = args.get(2).cloned().unwrap_or(Value16::null());
                config.insert(key, value);
                Ok(Value16::object(config))
            }
            "save" => {
                let config = match args.first().and_then(|v| v.as_object()) {
                    Some(o) => o,
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.save: expected config object".to_string(),
                        ))
                    }
                };
                let path = match args.get(1).and_then(|v| v.as_string()) {
                    Some(s) => s,
                    _ => match config.get("__user_path").and_then(|v| v.as_string()) {
                        Some(s) => s,
                        _ => {
                            return Err(compile_codes::runtime_error(
                                "PluginConfig.save: no path specified".to_string(),
                            ))
                        }
                    },
                };
                // Filter out internal keys and stringify via shared toml_ops
                let filtered: hudhudscript_bytecode::ObjMap = config
                    .iter()
                    .filter(|(k, _)| !k.to_string().starts_with("__"))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                let content = match hudhud_serial::toml_ops::stringify(&[Value16::object(filtered)])
                {
                    Ok(v) => v.as_string().unwrap_or_default(),
                    Ok(_) => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.save: stringify returned non-string".to_string(),
                        ))
                    }
                    Err(e) => {
                        return Err(compile_codes::runtime_error(format!(
                            "PluginConfig.save: serialize error: {}",
                            e
                        )))
                    }
                };
                // Best-effort directory creation — write below will fail with a clear error if needed
                if let Some(parent) = std::path::Path::new(&path).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::write(&path, content).map_err(|e| {
                    compile_codes::runtime_error(format!("PluginConfig.save: write error: {}", e))
                })?;
                let mut result = hudhudscript_bytecode::ObjMap::default();
                result.insert("saved".to_string(), Value16::bool_(true));
                result.insert("path".to_string(), Value16::string(path));
                Ok(Value16::object(result))
            }
            "merge" => {
                let base = match args.first().and_then(|v| v.as_object()) {
                    Some(o) => o.clone(),
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.merge: expected base config object".to_string(),
                        ))
                    }
                };
                let overlay = match args.get(1).and_then(|v| v.as_object()) {
                    Some(o) => o,
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.merge: expected overlay config object".to_string(),
                        ))
                    }
                };
                let mut merged = base;
                for (k, v) in overlay.iter() {
                    merged.insert(k.clone(), v.clone());
                }
                Ok(Value16::object(merged))
            }
            "watch" => {
                let path = match args.first() {
                    Some(v) => match v
                        .as_object()
                        .and_then(|o| o.get("__user_path"))
                        .and_then(|v| v.as_string())
                    {
                        Some(s) => s,
                        _ => "unknown".to_string(),
                    },
                    _ => "unknown".to_string(),
                };
                let mut result = hudhudscript_bytecode::ObjMap::default();
                result.insert("watching".to_string(), Value16::bool_(true));
                result.insert("path".to_string(), Value16::string(path));
                Ok(Value16::object(result))
            }
            "defaults" => {
                let plugin_name = match args.first().and_then(|v| v.as_string()) {
                    Some(s) => s,
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.defaults: expected plugin name string".to_string(),
                        ))
                    }
                };
                let mut config = match args.get(1).and_then(|v| v.as_object()) {
                    Some(o) => o.clone(),
                    _ => hudhudscript_bytecode::ObjMap::default(),
                };
                config.insert("__plugin".to_string(), Value16::string(plugin_name));
                config.insert("__defaults_applied".to_string(), Value16::bool_(true));
                Ok(Value16::object(config))
            }
            "paths" => {
                let plugin_name = match args.first().and_then(|v| v.as_string()) {
                    Some(s) => s,
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "PluginConfig.paths: expected plugin name string".to_string(),
                        ))
                    }
                };
                let system_path = format!("/etc/hudhud/plugins/{}.toml", plugin_name);
                let user_path = match std::env::var("HOME") {
                    Ok(home) => format!("{}/.config/hudhud/plugins/{}.toml", home, plugin_name),
                    Err(_) => format!("~/.config/hudhud/plugins/{}.toml", plugin_name),
                };
                let mut result = hudhudscript_bytecode::ObjMap::default();
                result.insert("system".to_string(), Value16::string(system_path));
                result.insert("user".to_string(), Value16::string(user_path));
                Ok(Value16::object(result))
            }
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown PluginConfig method: {}",
                method
            ))),
        }
    }

}
