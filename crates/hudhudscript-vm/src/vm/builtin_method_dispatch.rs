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
    pub(crate) fn call_method_on_value(
        &mut self,
        receiver: &Value16,
        method: &str,
        args: Vec<Value16>,
        bytecode: &Bytecode,
    ) -> CompileResult<Value16> {
        // SOP: ability dispatch on subject instances
        if let Some(inner) = receiver.as_object() {
            if inner.get("__type").and_then(|v| v.as_string()).as_deref()
                == Some("subject_instance")
            {
                let template = inner
                    .get("__template")
                    .and_then(|v| v.as_string())
                    .unwrap_or_default();

                // P2: 2-entry polymorphic inline cache for ability dispatch.
                let mut chunk_with_name: Option<(Arc<hudhudscript_bytecode::FunctionChunk>, String)> = None;
                for i in 0..2 {
                    if let (Some((ref ct, ref cm, ref ck, ref cn)), ref age) = self.ability_cache[i] {
                        if ct == &template && cm == method {
                            self.ability_cache[i].1 = age.saturating_add(1);
                            chunk_with_name = Some((Arc::clone(ck), cn.clone()));
                            break;
                        }
                    }
                }
                if chunk_with_name.is_none() {
                    let scoped_name = if !template.is_empty() { format!("ability::{}::{}", template, method) } else { String::new() };
                    let unscoped_name = format!("ability::{}", method);
                    let funcs = bytecode.functions.borrow();
                    let chunk = if !scoped_name.is_empty() { bytecode.get_function(&scoped_name) } else { None };
                    let chunk = chunk.or_else(|| bytecode.get_function(&unscoped_name));
                    let cap_chunk_name = if !scoped_name.is_empty() && bytecode.has_function(&scoped_name) { scoped_name.clone() } else { unscoped_name.clone() };
                    if let Some(ref c) = chunk {
                        let replace_idx = if self.ability_cache[0].1 <= self.ability_cache[1].1 { 0 } else { 1 };
                        self.ability_cache[replace_idx] = (Some((template.clone(), method.to_string(), Arc::clone(c), cap_chunk_name.clone())), 0);
                    }
                    chunk_with_name = chunk.map(|c| (c, cap_chunk_name));
                }

                if let Some((chunk, cap_chunk_name)) = chunk_with_name {
                    let params = chunk.params.clone();
                    let mut call_args = vec![*receiver];
                    call_args.extend(args);
                    let result = self.call_chunk_with_captures(
                        &chunk,
                        &params,
                        &call_args,
                        bytecode,
                        &cap_chunk_name,
                        &HashMap::new(),
                    )?;

                    // SOP0007: composition rule-aware view dispatch
                    let instance_id_str = inner
                        .get("__instance_id")
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    let compose_key = format!("{}::{}", template, method);
                    let rules = self.composition_rules.get(&compose_key).cloned();
                    if let Some(rules) = rules {
                        for rule in &rules {
                            match &rule.mode {
                                crate::vm::sop_types::CompositionMode::Combine(subjects) => {
                                    for view_name in subjects {
                                        let view_ability =
                                            format!("ability::{}::{}", view_name, method);
                                        if let Some(vc) =
                                            bytecode.get_function(&view_ability)
                                        {
                                            let vp = vc.params.clone();
                                            let _ = self.call_chunk_with_captures(
                                                &vc,
                                                &vp,
                                                &call_args,
                                                bytecode,
                                                &view_ability,
                                                &HashMap::new(),
                                            );
                                        }
                                    }
                                }
                                crate::vm::sop_types::CompositionMode::Override(subject) => {
                                    let view_ability = format!("ability::{}::{}", subject, method);
                                    if let Some(vc) =
                                        bytecode.get_function(&view_ability)
                                    {
                                        let vp = vc.params.clone();
                                        return self.call_chunk_with_captures(
                                            &vc,
                                            &vp,
                                            &call_args,
                                            bytecode,
                                            &view_ability,
                                            &HashMap::new(),
                                        );
                                    }
                                }
                                crate::vm::sop_types::CompositionMode::Before(subject) => {
                                    let view_ability = format!("ability::{}::{}", subject, method);
                                    if let Some(vc) =
                                        bytecode.get_function(&view_ability)
                                    {
                                        let vp = vc.params.clone();
                                        let _ = self.call_chunk_with_captures(
                                            &vc,
                                            &vp,
                                            &call_args,
                                            bytecode,
                                            &view_ability,
                                            &HashMap::new(),
                                        );
                                    }
                                }
                                crate::vm::sop_types::CompositionMode::After(subject) => {
                                    let view_ability = format!("ability::{}::{}", subject, method);
                                    if let Some(vc) =
                                        bytecode.get_function(&view_ability)
                                    {
                                        let vp = vc.params.clone();
                                        let _ = self.call_chunk_with_captures(
                                            &vc,
                                            &vp,
                                            &call_args,
                                            bytecode,
                                            &view_ability,
                                            &HashMap::new(),
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        // No composition rules: default combine-all
                        let view_abilities: Vec<String> =
                            if let Some(inst) = self.subject_instances.get(&instance_id_str) {
                                inst.views
                                    .keys()
                                    .map(|view_name| format!("ability::{}::{}", view_name, method))
                                    .collect()
                            } else {
                                Vec::new()
                            };
                        for view_ability in &view_abilities {
                            if let Some(vc) = bytecode.get_function(view_ability)
                            {
                                let vp = vc.params.clone();
                                let _ = self.call_chunk_with_captures(
                                    &vc,
                                    &vp,
                                    &call_args,
                                    bytecode,
                                    view_ability,
                                    &HashMap::new(),
                                );
                            }
                        }
                    }
                    return Ok(result);
                }

                // SOP0006: base ability not found — try view-only dispatch
                let instance_id_str = inner
                    .get("__instance_id")
                    .and_then(|v| v.as_string())
                    .unwrap_or_default();
                if let Some(inst) = self.subject_instances.get(&instance_id_str) {
                    for (view_name, _) in &inst.views {
                        let view_ability = format!("ability::{}::{}", view_name, method);
                        if let Some(vc) = bytecode.get_function(&view_ability) {
                            let vp = vc.params.clone();
                            let mut call_args = vec![*receiver];
                            call_args.extend(args.clone());
                            return self.call_chunk_with_captures(
                                &vc,
                                &vp,
                                &call_args,
                                bytecode,
                                &view_ability,
                                &HashMap::new(),
                            );
                        }
                    }
                }
            }
        }

        // Instance method dispatch
        if let Some(inst) = receiver.as_instance_data() {
            if let Some(class_data) = inst.class.as_class_data() {
                if let Some(method_val) = class_data
                    .vtable
                    .get(method)
                    .or_else(|| class_data.methods.get(method))
                {
                    if let Some(chunk_name) = method_val.as_string() {
                        if let Some(chunk) = bytecode.get_function(&chunk_name) {
                            let class_name = inst.class_name.clone();
                            // OOP0002: access modifier check
                            if let Some(access) = class_data.method_access.get(method) {
                                if *access == 1 {
                                    // Private: only callable from same class
                                    if self.class_context_stack.last().map(|c| c.as_str())
                                        != Some(&class_name)
                                    {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Cannot call private method '{}' on class '{}' from outside the class",
                                            method, class_name
                                        )));
                                    }
                                } else if *access == 2 {
                                    // Protected: only callable from same class or subclass
                                    let current =
                                        self.class_context_stack.last().map(|c| c.clone());
                                    let allowed = current.as_deref() == Some(&class_name)
                                        || self.is_subclass_of(current.as_deref(), &class_name);
                                    if !allowed {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Cannot call protected method '{}' on class '{}' from unrelated class",
                                            method, class_name
                                        )));
                                    }
                                }
                            }
                            let prev_this = self.get_var_cloned("this");
                            let prev_class = self.get_var_cloned("__current_class");
                            self.set_var("this", receiver.clone())?;
                            self.set_var("__current_class", Value16::string(class_name.clone()))?;
                            self.class_context_stack.push(class_name.clone());
                            let result = self.call_chunk(
                                &chunk,
                                &chunk.params,
                                &args,
                                bytecode,
                                &chunk_name,
                            );
                            self.class_context_stack.pop();
                            self.last_instance_mutation = self.get_var_cloned("this").map(Box::new);
                            match prev_this {
                                Some(old) => {
                                    let _ = self.set_var("this", old);
                                }
                                None => {
                                    self.remove_var("this");
                                }
                            }
                            match prev_class {
                                Some(old) => {
                                    let _ = self.set_var("__current_class", old);
                                }
                                None => {
                                    self.remove_var("__current_class");
                                }
                            }
                            return result;
                        }
                    }
                }
            }
        }
        // Array
        if receiver.as_array().is_some() {
            return self.call_array_method(*receiver, method, args, bytecode);
        }
        // String
        if let Some(s) = receiver.as_string() {
            return self.call_string_method(s.as_str(), method, args.clone());
        }
        // Object (includes instances created via NewInstance as Object)
        if let Some(obj) = receiver.as_object() {
            // P3: Identity cache checks BEFORE __module lookup (hot path)
            // Math object
            if Some(*receiver) == self.math_obj {
                return self.call_math_method(method, args.clone());
            }
            // JSON object
            if Some(*receiver) == self.json_obj {
                return self.call_json_method(method, args.clone());
            }
            // #928: Check registered modules
            if let Some(module_name) = obj.get("__module").and_then(|v| v.as_string()) {
                if let Some(result) = self
                    .module_registry
                    .call(&module_name, method, args.clone())
                {
                    return result;
                }
            }
            // http module
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("http"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_http_method(method, args_v);
            }
            // file module
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("file"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_file_method(method, args_v);
            }
            // Promise object
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("Promise"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_promise_method(method, args_v);
            }
            // linalg module
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("linalg"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_linalg_method(method, args_v);
            }
            // stats module
            if obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("stats"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_stats_method(method, args_v);
            }
            // P3: lazy clone for remaining branches that need Vec<Value16>
            let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
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
                    "tokenomics" => return self.call_tokenomics_method(method, args_v.clone()),
                    "channel" => return self.call_channel_method(method, args_v.clone()),
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
            // AGENT0004/0005: Swarm/Council dispatch
            if let Some(obj_name) = obj.get("name").and_then(|v| v.as_string()) {
                if self.swarm_names.contains_key(&obj_name)
                    && (method == "run" || method == "execute")
                {
                    let agents: Vec<String> = obj
                        .get("agents")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|a| a.as_string().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    let task = args_v.into_iter().next().unwrap_or(Value16::null());
                    return self.dispatch_swarm_run(&obj_name, &agents, &task, bytecode);
                }
                if self.swarm_names.contains_key(&obj_name) && method == "add_agent" {
                    let new_agent = args_v.into_iter().next().unwrap_or(Value16::null());
                    return self.dispatch_swarm_add_agent(&obj_name, &new_agent, bytecode);
                }
                if self.swarm_names.contains_key(&obj_name) && method == "remove_agent" {
                    let agent_name = args_v
                        .into_iter()
                        .next()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    return self.dispatch_swarm_remove_agent(&obj_name, &agent_name, bytecode);
                }
                if self.council_names.contains_key(&obj_name)
                    && (method == "decide" || method == "vote")
                {
                    let agents: Vec<String> = obj
                        .get("members")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|m| {
                                    m.as_object()
                                        .and_then(|o| o.get("agent_id").and_then(|v| v.as_string()))
                                })
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_default();
                    let task = args_v.into_iter().next().unwrap_or(Value16::null());
                    return self.dispatch_swarm_run(&obj_name, &agents, &task, bytecode);
                }
                if self.community_names.contains_key(&obj_name)
                    && (method == "run" || method == "decide")
                {
                    // Community delegates to each council
                    let council_names: Vec<String> = obj
                        .get("councils")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|c| c.as_string().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    let task = args_v.into_iter().next().unwrap_or(Value16::null());
                    let mut all_results = Vec::new();
                    for cn in &council_names {
                        let council = self.get_var_cloned(cn);
                        if let Some(co) = council.and_then(|v| v.as_object().cloned()) {
                            let council_agents: Vec<String> = co
                                .get("members")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|m| {
                                            m.as_object().and_then(|o| {
                                                o.get("agent_id").and_then(|v| v.as_string())
                                            })
                                        })
                                        .map(|s| s.to_string())
                                        .collect()
                                })
                                .unwrap_or_default();
                            let r =
                                self.dispatch_swarm_run(cn, &council_agents, &task, bytecode)?;
                            all_results.push(r);
                        }
                    }
                    return Ok(Value16::array(all_results));
                }
            }
            // AGENT0002: Agent dispatch — route method calls through agent's provider
            if let Some(agent_name) = obj.get("name").and_then(|v| v.as_string()) {
                if self.agent_names.contains_key(&agent_name) {
                    // Look up provider from agent's "provider" field
                    let provider_name = obj.get("provider").and_then(|v| v.as_string());
                    let prov_obj = provider_name.and_then(|pn| self.get_var_cloned(&pn));
                    if let Some(mut prov_obj) = prov_obj {
                        // Agent-level model override
                        if let Some(model) = obj.get("model").and_then(|v| v.as_string()) {
                            if let Some(prov_map) = prov_obj.as_object() {
                                let mut merged = prov_map.clone();
                                merged.insert(
                                    "model".to_string(),
                                    Value16::string(model.to_string()),
                                );
                                prov_obj = Value16::object(merged);
                            }
                        }
                        // Agent.call() → route to provider.call()
                        if method == "call" || method == "stream" {
                            let config = args_v.into_iter().next().unwrap_or(Value16::null());
                            let prev = self.dispatch_provider_receiver.take();
                            self.dispatch_provider_receiver = Some(prov_obj);
                            let result = dispatch_provider_call(self, &config);
                            self.dispatch_provider_receiver = prev;
                            return result;
                        }
                        // Agent.any_method() → build prompt + route to provider
                        let method_str = method.to_string();
                        let arg_strs: Vec<String> =
                            args_v.iter().map(|v| self.value_to_string(v)).collect();
                        let prompt = format!("Task: {}. Args: {}", method_str, arg_strs.join(", "));
                        let mut config = hudhudscript_bytecode::ObjMap::default();
                        config.insert("prompt".to_string(), Value16::string(prompt));
                        let prev = self.dispatch_provider_receiver.take();
                        self.dispatch_provider_receiver = Some(prov_obj);
                        let result = dispatch_provider_call(self, &Value16::object(config));
                        self.dispatch_provider_receiver = prev;
                        return result;
                    }
                }
            }
            // Provider / LLM dispatch
            let is_mcp = obj
                .get("__module")
                .map_or(false, |v| v.as_string().as_deref() == Some("mcp"));
            if (method == "call" || method == "stream") && !is_mcp {
                let config = args.into_iter().next().unwrap_or(Value16::null());
                // PROVIDER0002: set receiver so provider_get_provider can build from it
                let prev = self.dispatch_provider_receiver.take();
                self.dispatch_provider_receiver = Some(receiver.clone());
                let result = dispatch_provider_call(self, &config);
                self.dispatch_provider_receiver = prev;
                return result;
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
            // Class methods (chunk name lookup)
            if let Some(chunk_name) = obj.get(method).and_then(|v| v.as_string()) {
                if let Some(chunk) = bytecode
                    .get_function(chunk_name.as_str())
                {
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
                        obj.keys().map(|k| Value16::string(k.to_string())).collect();
                    ks.sort_by(|a, b| {
                        let sa = a.as_string().unwrap_or_default();
                        let sb = b.as_string().unwrap_or_default();
                        sa.cmp(&sb)
                    });
                    Ok(Value16::array(ks))
                }
                "values" => {
                    let mut pairs: Vec<(String, Value16)> = obj.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
                    pairs.sort_by_key(|(k, _)| k.clone());
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
        } else if let Some(inst) = receiver.as_instance_data() {
            let class_name = &inst.class_name;
            let fields = &inst.fields;
            let class = &inst.class;
            // O(1) vtable method dispatch
            if let Some(parent_cls) = class.as_class_data() {
                if let Some(chunk_name) = parent_cls.vtable.get(method).and_then(|v| v.as_string())
                {
                    if let Some(chunk) = bytecode.get_function(&chunk_name) {
                        let prev_this = self.get_var_cloned("this");
                        let prev_class = self.get_var_cloned("__current_class");
                        self.set_var("this", receiver.clone())?;
                        self.set_var("__current_class", Value16::string(class_name.clone()))?;
                        self.class_context_stack.push(class_name.clone());
                        let result =
                            self.call_chunk(&chunk, &chunk.params, &args, bytecode, &chunk_name);
                        self.class_context_stack.pop();
                        self.last_instance_mutation = self.get_var_cloned("this").map(Box::new);
                        match prev_this {
                            Some(old) => {
                                let _ = self.set_var("this", old);
                            }
                            None => {
                                self.remove_var("this");
                            }
                        }
                        match prev_class {
                            Some(old) => {
                                let _ = self.set_var("__current_class", old);
                            }
                            None => {
                                self.remove_var("__current_class");
                            }
                        }
                        return result;
                    }
                }
            }
            // Fallback: look up method in instance fields
            if let Some(chunk_name) = fields.get(method).and_then(|v| v.as_string()) {
                if let Some(chunk) = bytecode.get_function(&chunk_name) {
                    let prev_this = self.get_var_cloned("this");
                    self.set_var("this", receiver.clone())?;
                    let result =
                        self.call_chunk(&chunk, &chunk.params, &args, bytecode, &chunk_name);
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
                        fields.keys().map(|k| Value16::string(k.to_string())).collect();
                    ks.sort_by(|a, b| {
                        let sa = a.as_string().unwrap_or_default();
                        let sb = b.as_string().unwrap_or_default();
                        sa.cmp(&sb)
                    });
                    Ok(Value16::array(ks))
                }
                "values" => {
                    let mut pairs: Vec<(String, Value16)> = fields.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
                    pairs.sort_by_key(|(k, _)| k.clone());
                    Ok(Value16::array(
                        pairs.into_iter().map(|(_, v)| v.clone()).collect(),
                    ))
                }
                "length" => Ok(Value16::int(fields.len() as i64)),
                _ => Err(compile_codes::runtime_error(format!(
                    "Unknown method '{}' on instance of {}",
                    method, class_name
                ))),
            }
        } else if let Some(state) = receiver.as_promise_state() {
            match method {
                "then" => {
                    if args.is_empty() {
                        return Err(compile_codes::runtime_error(
                            "then() requires a callback".to_string(),
                        ));
                    }
                    let callback = &args[0];
                    match state {
                        hudhudscript_bytecode::PromiseState16::Resolved(val) => {
                            let result = self.call_value_as_function(
                                callback,
                                vec![(**val).clone()],
                                bytecode,
                            )?;
                            Ok(Value16::promise(
                                hudhudscript_bytecode::PromiseState16::Resolved(Box::new(result)),
                            ))
                        }
                        hudhudscript_bytecode::PromiseState16::Rejected(msg) => {
                            Ok(Value16::promise(
                                hudhudscript_bytecode::PromiseState16::Rejected(msg.clone()),
                            ))
                        }
                        _ => Ok(receiver.clone()),
                    }
                }
                "catch" => {
                    if args.is_empty() {
                        return Err(compile_codes::runtime_error(
                            "catch() requires a callback".to_string(),
                        ));
                    }
                    let callback = &args[0];
                    match state {
                        hudhudscript_bytecode::PromiseState16::Rejected(msg) => {
                            let result = self.call_value_as_function(
                                callback,
                                vec![Value16::string(msg.clone())],
                                bytecode,
                            )?;
                            Ok(Value16::promise(
                                hudhudscript_bytecode::PromiseState16::Resolved(Box::new(result)),
                            ))
                        }
                        _ => Ok(receiver.clone()),
                    }
                }
                _ => Err(compile_codes::runtime_error(format!(
                    "Unknown method '{}' on promise",
                    method
                ))),
            }
        } else if let Some(items) = receiver.as_set() {
            self.call_set_method(items, method, args.clone())
        } else if let Some(pairs) = receiver.as_map_pairs() {
            self.call_map_method(pairs, method, args.clone())
        } else if let Some(state) = receiver.as_generator_state() {
            match method {
                "next" => {
                    let next_val = crate::vm::exec::helpers::generator_advance(self, state);
                    Ok(next_val.unwrap_or(Value16::null()))
                }
                "toArray" => {
                    let all = state.lock().collect_all();
                    Ok(Value16::array(all))
                }
                _ => Err(compile_codes::runtime_error(format!(
                    "No method '{}' on generator",
                    method
                ))),
            }
        } else {
            // SOP0003: for subject instances, include role/ability info in error
            let detail = if let Some(inner) = receiver.as_object() {
                if inner.get("__type").and_then(|v| v.as_string()).as_deref()
                    == Some("subject_instance")
                {
                    let template = inner
                        .get("__template")
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    let instance_id = inner
                        .get("__id")
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    let role_info = if let Some(tmpl) = self.subject_templates.get(&template) {
                        if tmpl.roles.is_empty() {
                            format!(
                                " (subject '{}' has no roles, instance: {})",
                                template, instance_id
                            )
                        } else {
                            format!(
                                " (subject '{}' has roles: [{}], instance: {})",
                                template,
                                tmpl.roles.join(", "),
                                instance_id
                            )
                        }
                    } else {
                        String::new()
                    };
                    format!(
                        "Cannot call method '{}' on subject '{}'{}.",
                        method, template, role_info
                    )
                } else {
                    format!(
                        "Cannot call method '{}' on {}",
                        method,
                        self.type_name_of(receiver)
                    )
                }
            } else {
                format!(
                    "Cannot call method '{}' on {}",
                    method,
                    self.type_name_of(receiver)
                )
            };
            Err(compile_codes::runtime_error(detail))
        }
    }

    /// StringBuilder builtin — O(n) string concatenation via Vec<String> accumulation.
    ///
    /// Usage:
    ///   let sb = StringBuilder.new();
    ///   sb.append("hello ");
    ///   sb.append("world");
    ///   let s = sb.build();  // "hello world"
    pub(crate) fn call_string_builder_method(
        &mut self,
        method: &str,
        args: Vec<Value16>,
        receiver: &Value16,
    ) -> CompileResult<Value16> {
        match method {
            "new" => {
                // Return a builder object with an empty parts array.
                let mut obj = hudhudscript_bytecode::ObjMap::default();
                obj.insert(
                    "__module".to_string(),
                    Value16::string("StringBuilder".to_string()),
                );
                obj.insert("__parts".to_string(), Value16::array(Vec::new()));
                Ok(Value16::object(obj))
            }
            "append" => {
                if let Some(obj) = receiver.as_object() {
                    let text = args.first().map(|v| v.display_string()).unwrap_or_default();
                    if let Some(parts_val) = obj.get("__parts") {
                        if let Some(mut parts) = parts_val.as_array().cloned() {
                            // Accumulate the raw bytes for zero-copy join at build time.
                            parts.push(Value16::string(text));
                            let mut new_obj = obj.clone();
                            new_obj.insert("__parts".to_string(), Value16::array(parts));
                            // Mutate the receiver in-place via set_var
                            Ok(Value16::object(new_obj))
                        } else {
                            Err(compile_codes::runtime_error(
                                "StringBuilder: __parts is not an array".to_string(),
                            ))
                        }
                    } else {
                        Err(compile_codes::runtime_error(
                            "StringBuilder: not a valid builder instance".to_string(),
                        ))
                    }
                } else {
                    Err(compile_codes::runtime_error(
                        "StringBuilder.append: receiver is not an object".to_string(),
                    ))
                }
            }
            "build" => {
                if let Some(obj) = receiver.as_object() {
                    if let Some(parts_val) = obj.get("__parts") {
                        if let Some(parts) = parts_val.as_array() {
                            let total_len: usize =
                                parts.iter().map(|v| v.display_string().len()).sum();
                            let mut result = String::with_capacity(total_len);
                            for part in parts {
                                result.push_str(&part.display_string());
                            }
                            Ok(Value16::string(result))
                        } else {
                            Err(compile_codes::runtime_error(
                                "StringBuilder: __parts is not an array".to_string(),
                            ))
                        }
                    } else {
                        Err(compile_codes::runtime_error(
                            "StringBuilder: not a valid builder instance".to_string(),
                        ))
                    }
                } else {
                    Err(compile_codes::runtime_error(
                        "StringBuilder.build: receiver is not an object".to_string(),
                    ))
                }
            }
            _ => Err(compile_codes::runtime_error(format!(
                "StringBuilder: unknown method '{}'",
                method
            ))),
        }
    }
}
