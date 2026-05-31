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
    pub(crate) fn call_object_dispatch(
        &mut self,
        receiver: &Value16,
        method: &str,
        args: Vec<Value16>,
        bytecode: &Bytecode,
    ) -> CompileResult<Value16> {
        let obj = receiver.as_object().unwrap();
        let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
            let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
            // #928: Check registered modules first
            if let Some(module_name) = obj.get("__module").and_then(|v| v.as_string()) {
                if let Some(result) = self
                    .module_registry
                    .call(&module_name, method, args.clone())
                {
                    return result;
                }
            }
            // Math object
            if obj.contains_key("PI") && obj.contains_key("E") {
                return self.call_math_method(method, args.clone());
            }
            // JSON object
            if (method == "parse" || method == "stringify") && !obj.contains_key("__module") {
                return self.call_json_method(method, args.clone());
            }
            // http module
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("http"))
            {
                return self.call_http_method(method, args_v.clone());
            }
            // file module
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("file"))
            {
                return self.call_file_method(method, args_v.clone());
            }
            // Promise object
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("Promise"))
            {
                return self.call_promise_method(method, args_v.clone());
            }
            // linalg module
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("linalg"))
            {
                return self.call_linalg_method(method, args_v.clone());
            }
            // stats module
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("stats"))
            {
                return self.call_stats_method(method, args_v.clone());
            }
            // Serialization modules
            if let Some(module_name) = obj.get("__module").and_then(|v| v.as_string()) {
                match module_name.as_str() {
                    "TOML" => return self.call_toml_method(method, args.clone()),
                    "YAML" => return self.call_yaml_method(method, args_v.clone()),
                    "CSV" => return self.call_csv_method(method, args_v.clone()),
                    "INI" => return self.call_ini_method(method, args_v.clone()),
                    "Base64" => return self.call_base64_method(method, args_v.clone()),
                    "Hex" => return self.call_hex_method(method, args_v.clone()),
                    "URL" => return self.call_url_method(method, args_v.clone()),
                    "uuid" => return self.call_uuid_method(method, args_v.clone()),
                    "Path" => return self.call_path_method(method, args_v.clone()),
                    "Temp" => return self.call_temp_method(method, args_v.clone()),
                    "URLParser" => return self.call_url_parser_method(method, args_v.clone()),
                    "Glob" => return self.call_glob_method(method, args_v.clone()),
                    "Set" => return self.call_set_module_method(method, args_v.clone()),
                    "Map" => return self.call_map_module_method(method, args_v.clone()),
                    "stdin" => return self.call_stdin_method(method, args_v.clone()),
                    "Terminal" => return self.call_terminal_method(method, args_v.clone()),
                    "log" => return self.call_log_method(method, args_v.clone()),
                    "exec" => return self.call_exec_method(method, args_v.clone()),
                    "tcp" => return self.call_tcp_method(method, args_v.clone()),
                    "udp" => return self.call_udp_method(method, args_v.clone()),
                    "unix" => return self.call_unix_method(method, args.clone()),
                    "ws" => return self.call_ws_method(method, args_v.clone()),
                    "daemon" => return self.call_daemon_method(method, args_v.clone()),
                    "fs" => return self.call_fs_method(method, args_v.clone()),
                    "Env" => return self.call_env_method(method, args_v.clone()),
                    "os" => return self.call_os_method(method, args_v.clone()),
                    "Date" => return self.call_date_method(method, args_v.clone()),
                    "Duration" => return self.call_duration_method(method, args_v.clone()),
                    "regex" => return self.call_regex_method(method, args_v.clone()),
                    "schedule" => return self.call_schedule_method(method, args_v.clone()),
                    "EventBus" => return self.call_event_bus_method(method, args_v.clone()),
                    "Plugin" => return self.call_plugin_method(method, args_v.clone()),
                    "McpServer" => return self.call_mcp_server_method(method, args_v.clone()),
                    "Server" => return self.call_server_method(method, args_v.clone()),
                    "PluginConfig" => {
                        return self.call_plugin_config_method(method, args_v.clone())
                    }
                    "StringBuilder" => {
                        return self.call_string_builder_method(method, args_v.clone(), receiver)
                    }
                    _ => {}
                }
            }
            // Provider / LLM dispatch
            let is_mcp = obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("mcp"));
            if (method == "call" || method == "stream") && !is_mcp {
                let config = args.into_iter().next().unwrap_or(Value16::null());
                return dispatch_provider_call(self, &config);
            }
            // MCP proxy dispatch
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("mcp"))
            {
                if let Some(server_name) = obj.get("__server").and_then(|v| v.as_string()) {
                    let server_name = server_name.to_string();
                    let tool_args = if args.len() == 1 {
                        args.into_iter().next().unwrap_or(Value16::null())
                    } else if args.is_empty() {
                        Value16::null()
                    } else {
                        Value16::array(args)
                    };
                    return dispatch_mcp_tool_call(self, &server_name, method, &tool_args);
                }
                if method == "call" {
                    if args.len() < 2 {
                        return Err(compile_codes::runtime_error(
                            "mcp.call() requires (server, tool, [args])".to_string(),
                        ));
                    }
                    let server = self.value_to_string(&args[0]);
                    let tool = self.value_to_string(&args[1]);
                    let tool_args = args.get(2).cloned().unwrap_or(Value16::null());
                    return dispatch_mcp_tool_call(self, &server, &tool, &tool_args);
                }
            }
            // Class methods
            if let Some(chunk_name) = obj.get(method).and_then(|v| v.as_string()) {
                if let Some(chunk) = bytecode.functions.borrow().get(chunk_name.as_str()).cloned() {
                    let prev_this = self.get_var_cloned("this");
                    self.set_var("this", receiver.clone())?;
                    let result = self.call_chunk(
                        &chunk,
                        &chunk.params,
                        &args,
                        bytecode,
                        chunk_name.as_str(),
                    );
                    self.last_instance_mutation = self.get_var_cloned("this").map(Box::new);
                    match prev_this {
                        Some(old) => {
                            let _ = self.set_var("this", old);
                        }
                        None => {
                            self.remove_var("this");
                        }
                    }
                    return result;
                }
            }
            // Property access fallback
            match method {
                "keys" => {
                    let mut ks: Vec<Value16> =
                        obj.keys().map(|k| Value16::string(k.clone())).collect();
                    ks.sort_by(|a, b| {
                        let sa = a.as_string().unwrap_or_default();
                        let sb = b.as_string().unwrap_or_default();
                        sa.cmp(&sb)
                    });
                    Ok(Value16::array(ks))
                }
                "values" => {
                    let mut pairs: Vec<(&String, &Value16)> = obj.iter().collect();
                    pairs.sort_by_key(|(k, _)| k.as_str());
                    Ok(Value16::array(
                        pairs.into_iter().map(|(_, v)| v.clone()).collect(),
                    ))
                }
                "length" => Ok(Value16::int(obj.len() as i64)),
                _ => Err(compile_codes::runtime_error(format!(
                    "Unknown method '{}' on object",
                    method
                ))),
            }
    }
}
