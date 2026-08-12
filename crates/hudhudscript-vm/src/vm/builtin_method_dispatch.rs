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

/// v4.3 cold-path error — outlined to avoid bloating the hot vtable unpack.
#[cold] #[inline(never)]
pub(crate) fn err_vtable_not_packed() -> hudhudscript_errors::Error {
    compile_codes::runtime_error("Compiler invariant: vtable value not packed int".to_string())
}

#[cold] #[inline(never)]
pub(crate) fn err_function_idx_missing(idx: u32) -> hudhudscript_errors::Error {
    compile_codes::runtime_error(format!("Compiler invariant: function idx {} missing", idx))
}

impl crate::vm::VM {
                            // set current class for this call (stack-only, T5.3).
    /// All 5 copies of this pattern now call this single helper.
    #[inline(always)]
    pub(crate) fn with_this_context<F, R>(
        &mut self,
        receiver: &Value16,
        class_name_sym: Option<hudhudscript_bytecode::SymId>,
        f: F,
    ) -> CompileResult<R>
    where
        F: FnOnce(&mut Self) -> CompileResult<R>,
    {
        let prev_this = self.get_var_cloned_by_sym(self.this_sym);
        self.set_var_by_sym(self.this_sym, "this", receiver.clone())?;
        if let Some(csym) = class_name_sym {
            self.class_context_stack.push(csym);
        }
        let result = f(self);
        if let Some(_) = class_name_sym {
            self.class_context_stack.pop();
        }
        if let Some(mutated_this) = self.get_var_cloned_by_sym(self.this_sym) {
            if mutated_this != *receiver {
                self.last_instance_mutation = Some(Box::new(mutated_this));
            }
        }
        match prev_this {
            Some(old) => { self.set_var_by_sym(self.this_sym, "this", old)?; }
            None => { self.remove_var_by_sym(self.this_sym); }
        }
        result
    }

    pub(crate) fn call_method_on_value(
        &mut self,
        receiver: &Value16,
        method: &str,
        method_sym: hudhudscript_bytecode::SymId,
        args: Vec<Value16>,
        bytecode: &Bytecode,
    ) -> CompileResult<Value16> {
        // SOP: ability dispatch on subject instances
        if let Some(inner) = receiver.as_object() {
            if inner.get(&hudhudscript_bytecode::well_known::wk().type_).and_then(|v| v.as_str())
                == Some("subject_instance")
            {
                let template = inner
                    .get(&hudhudscript_bytecode::well_known::wk().template)
                    .and_then(|v| v.as_string())
                    .unwrap_or_default();

                // P2: 2-entry polymorphic inline cache for ability dispatch.
                let mut chunk_with_name: Option<(
                    Arc<hudhudscript_bytecode::FunctionChunk>,
                    String,
                )> = None;
                for i in 0..2 {
                    if let (Some((ref ct, ref cm, ref ck, ref cn)), ref age) = self.ability_cache[i]
                    {
                        if ct == &template && *cm == method_sym {
                            self.ability_cache[i].1 = age.saturating_add(1);
                            chunk_with_name = Some((Arc::clone(ck), cn.clone()));
                            break;
                        }
                    }
                }
                if chunk_with_name.is_none() {
                    let scoped_name = if !template.is_empty() {
                        format!("ability::{}::{}", template, method)
                    } else {
                        String::new()
                    };
                    let unscoped_name = format!("ability::{}", method);
                    let funcs = bytecode.functions.borrow();
                    let chunk = if !scoped_name.is_empty() {
                        bytecode.get_function(&scoped_name)
                    } else {
                        None
                    };
                    let chunk = chunk.or_else(|| bytecode.get_function(&unscoped_name));
                    let cap_chunk_name =
                        if !scoped_name.is_empty() && bytecode.has_function(&scoped_name) {
                            scoped_name.clone()
                        } else {
                            unscoped_name.clone()
                        };
                    if let Some(ref c) = chunk {
                        let replace_idx = if self.ability_cache[0].1 <= self.ability_cache[1].1 {
                            0
                        } else {
                            1
                        };
                        self.ability_cache[replace_idx] = (
                            Some((
                                template.clone(),
                                method_sym,
                                Arc::clone(c),
                                cap_chunk_name.clone(),
                            )),
                            0,
                        );
                    }
                    chunk_with_name = chunk.map(|c| (c, cap_chunk_name));
                }

                if let Some((chunk, cap_chunk_name)) = chunk_with_name {
                    let params = chunk.params.clone();
                    let mut call_args = vec![*receiver];
                    call_args.extend(args);

                    // SOP0007: composition rule-aware view dispatch
                    // Order invariant: before -> base -> combine(reducer) -> after.
                    // If an override rule exists, base never runs and only the
                    // override view's ability is returned.
                    let instance_id_str = inner
                        .get(&hudhudscript_bytecode::well_known::wk().instance_id)
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    let compose_key = format!("{}::{}", template, method);
                    let rules = self.composition_rules.get(&compose_key).cloned();

                    // Separate rules by mode so we can enforce the fixed order.
                    let mut before_subjects: Vec<String> = Vec::new();
                    let mut after_subjects: Vec<String> = Vec::new();
                    let mut combine_subjects: Vec<String> = Vec::new();
                    let mut override_subject: Option<String> = None;

                    if let Some(ref rules) = rules {
                        for rule in rules {
                            match &rule.mode {
                                crate::vm::sop_types::CompositionMode::Combine(subjects) => {
                                    combine_subjects.extend(subjects.iter().cloned());
                                }
                                crate::vm::sop_types::CompositionMode::Override(subject) => {
                                    // Only the first override is honored; base never runs.
                                    if override_subject.is_none() {
                                        override_subject = Some(subject.clone());
                                    }
                                }
                                crate::vm::sop_types::CompositionMode::Before(subject) => {
                                    before_subjects.push(subject.clone());
                                }
                                crate::vm::sop_types::CompositionMode::After(subject) => {
                                    after_subjects.push(subject.clone());
                                }
                            }
                        }
                    } else {
                        // No composition rules: default combine-all.
                        if let Some(inst) = self.subject_instances.get(&instance_id_str) {
                            combine_subjects.extend(inst.views.keys().cloned());
                        }
                    }

                    // Override short-circuits everything (base does not run).
                    if let Some(subject) = override_subject {
                        let view_ability = format!("ability::{}::{}", subject, method);
                        if let Some(vc) = bytecode.get_function(&view_ability) {
                            let vp = vc.params.clone();
                            return self.call_chunk_with_captures(
                                &vc,
                                &vp,
                                &call_args,
                                bytecode,
                                hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&view_ability).0),
                                &HashMap::new(),
                            );
                        }
                    }

                    // before hooks (run before base, errors propagate).
                    for view_name in &before_subjects {
                        let view_ability = format!("ability::{}::{}", view_name, method);
                        if let Some(vc) = bytecode.get_function(&view_ability) {
                            let vp = vc.params.clone();
                            self.call_chunk_with_captures(
                                &vc,
                                &vp,
                                &call_args,
                                bytecode,
                                hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&view_ability).0),
                                &HashMap::new(),
                            )?;
                        }
                    }

                    // Base ability result.
                    let mut result = self.call_chunk_with_captures(
                        &chunk,
                        &params,
                        &call_args,
                        bytecode,
                        hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&cap_chunk_name).0),
                        &HashMap::new(),
                    )?;

                    // combine reducer: default is "last result wins".
                    fn default_combine_reducer(_acc: Value16, next: Value16) -> Value16 {
                        next
                    }
                    for view_name in &combine_subjects {
                        let view_ability = format!("ability::{}::{}", view_name, method);
                        if let Some(vc) = bytecode.get_function(&view_ability) {
                            let vp = vc.params.clone();
                            let view_result = self.call_chunk_with_captures(
                                &vc,
                                &vp,
                                &call_args,
                                bytecode,
                                hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&view_ability).0),
                                &HashMap::new(),
                            )?;
                            result = default_combine_reducer(result, view_result);
                        }
                    }

                    // after hooks (run after base, errors propagate).
                    for view_name in &after_subjects {
                        let view_ability = format!("ability::{}::{}", view_name, method);
                        if let Some(vc) = bytecode.get_function(&view_ability) {
                            let vp = vc.params.clone();
                            self.call_chunk_with_captures(
                                &vc,
                                &vp,
                                &call_args,
                                bytecode,
                                hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&view_ability).0),
                                &HashMap::new(),
                            )?;
                        }
                    }

                    // B1: invoke registered effects after ability dispatch
                    if let Some(effect_chunk_name) = self.effects.get(method).cloned() {
                        if let Some(effect_chunk) = bytecode.get_function(&effect_chunk_name) {
                            let effect_params = effect_chunk.params.clone();
                            let _ = self.call_chunk_with_captures(
                                &effect_chunk,
                                &effect_params,
                                &call_args,
                                bytecode,
                                hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&effect_chunk_name).0),
                                &HashMap::new(),
                            );
                        }
                    }

                    return Ok(result);
                }

                // SOP0006: base ability not found — try view-only dispatch
                let instance_id_str = inner
                    .get(&hudhudscript_bytecode::well_known::wk().instance_id)
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
                                hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&view_ability).0),
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
                    .get(&method_sym)
                    .or_else(|| class_data.methods.get(&method_sym))
                {
                    // v4.3: unpack packed (idx<<32)|sym from vtable int value.
                    let packed = method_val.as_int()
                        .ok_or_else(err_vtable_not_packed)?;
                    let idx = (packed >> 32) as u32;
                    let chunk_sym = hudhudscript_bytecode::SymId(packed as u32);
                    let chunk = bytecode.get_function_by_index(idx)
                        .ok_or_else(|| err_function_idx_missing(idx))?;
                            let class_name: &str = &inst.class_name;
                            let class_name_sym = hudhudscript_bytecode::SymId(
                                hudhudscript_bytecode::interner::intern(class_name).0);
                            // OOP0002: access modifier check
                            if let Some(access) = class_data.method_access.get(&method_sym) {
                                if *access == 1 {
                                    // Private: only callable from same class
                                    if self.class_context_stack.last().copied()
                                        != Some(class_name_sym)
                                    {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Cannot call private method '{}' on class '{}' from outside the class",
                                            method, class_name
                                        )));
                                    }
                                } else if *access == 2 {
                                    // Protected: only callable from same class or subclass
                                    let current =
                                        self.class_context_stack.last().copied();
                                    let current_str = current.map(|s| hudhudscript_bytecode::interner::resolve(
                                        hudhudscript_bytecode::interner::SymbolId(s.0)));
                                    let allowed = current == Some(class_name_sym)
                                        || self.is_subclass_of(current_str.as_deref(), class_name);
                                    if !allowed {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Cannot call protected method '{}' on class '{}' from unrelated class",
                                            method, class_name
                                        )));
                                    }
                                }
                            }
                            return self.with_this_context(
                                receiver, Some(class_name_sym),
                                |this| {
                                    this.call_chunk(&chunk, &chunk.params, &args, bytecode, chunk_sym)
                                },
                            );
                        }
            }
        }
        // Array
        if receiver.as_array().is_some() {
            return self.call_array_method(*receiver, method, args, bytecode);
        }
        // String
        if let Some(s) = receiver.as_str() {
            let is_ascii = receiver.is_dynamic_string_ascii();
            return self.call_string_method(s, method, &args, is_ascii);
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
            if let Some(module_name) = obj.get(&hudhudscript_bytecode::well_known::wk().module).and_then(|v| v.as_string()) {
                if let Some(result) = self
                    .module_registry
                    .call(&module_name, method, args.clone())
                {
                    return result;
                }
            }
            // http module
            if obj
                .get(&hudhudscript_bytecode::well_known::wk().module)
                .map_or(false, |v| v.as_str() == Some("http"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_http_method(method, args_v);
            }
            // file module
            if obj
                .get(&hudhudscript_bytecode::well_known::wk().module)
                .map_or(false, |v| v.as_str() == Some("file"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_file_method(method, args_v);
            }
            // Promise object
            if obj
                .get(&hudhudscript_bytecode::well_known::wk().module)
                .map_or(false, |v| v.as_str() == Some("Promise"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_promise_method(method, args_v);
            }
            // linalg module
            if obj
                .get(&hudhudscript_bytecode::well_known::wk().module)
                .map_or(false, |v| v.as_str() == Some("linalg"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_linalg_method(method, args_v);
            }
            // stats module
            if obj
                .get(&hudhudscript_bytecode::well_known::wk().module)
                .map_or(false, |v| v.as_str() == Some("stats"))
            {
                let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
                return self.call_stats_method(method, args_v);
            }
            // P3: lazy clone for remaining branches that need Vec<Value16>
            let args_v: Vec<Value16> = args.iter().map(|v| *v).collect();
            // Serialization modules
            if let Some(module_name) = obj.get(&hudhudscript_bytecode::well_known::wk().module).and_then(|v| v.as_string()) {
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
                    let action_name = format!("{}.{}", agent_name, method);
                    let action_chunk = bytecode
                        .action_registry
                        .borrow()
                        .get(action_name.as_str())
                        .cloned();
                    if let Some(action_chunk) = action_chunk {
                        if action_chunk.params.len() != args.len() {
                            return Err(compile_codes::runtime_error(format!(
                                "Action {} expects {} arguments, got {}",
                                action_name,
                                action_chunk.params.len(),
                                args.len()
                            )));
                        }

                        let action_sym = hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&action_name).0);
                        return self.with_this_context(receiver, None, |this| {
                            if action_chunk.is_async {
                                Ok(this.spawn_async_chunk(
                                    Arc::clone(&action_chunk),
                                    &action_chunk.params,
                                    &args,
                                    bytecode,
                                    &action_name,
                                    None,
                                ))
                            } else {
                                this.call_chunk(
                                    &action_chunk,
                                    &action_chunk.params,
                                    &args,
                                    bytecode,
                                    action_sym,
                                )
                            }
                        });
                    }

                    // Look up provider from agent's "provider" field
                    let provider_value = obj.get("provider");
                    let mut prov_obj = None;

                    if let Some(pv) = provider_value {
                        if pv.is_object() {
                            prov_obj = Some(pv.clone());
                        } else if let Some(pn) = pv.as_string() {
                            // Backward compatibility
                            prov_obj = self.get_var_cloned(&pn);
                            if prov_obj.is_none() && pn.contains('.') {
                                let parts: Vec<&str> = pn.split('.').collect();
                                if let Some(mut current) = self.get_var_cloned(parts[0]) {
                                    let mut resolved = true;
                                    for part in &parts[1..] {
                                        if let Some(obj) = current.as_object() {
                                            if let Some(next) = obj.get(*part) {
                                                current = next.clone();
                                            } else {
                                                resolved = false;
                                                break;
                                            }
                                        } else {
                                            resolved = false;
                                            break;
                                        }
                                    }
                                    if resolved && current.is_object() {
                                        prov_obj = Some(current);
                                    }
                                }
                            }
                        }
                    }

                    if prov_obj.is_none() {
                        return Err(compile_codes::runtime_error(
                            format!("Agent '{}' provider did not resolve: provider field is {}", agent_name, provider_value.unwrap_or(&Value16::null())),
                        ));
                    }

                    if prov_obj.is_some() {
                        // Agent.call() → route to provider.call()
                        if method == "call" || method == "stream" {
                            let config = args_v.into_iter().next().unwrap_or(Value16::null());
                            let prev = self.dispatch_provider_receiver.take();
                            self.dispatch_provider_receiver = Some(receiver.clone());
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
                        self.dispatch_provider_receiver = Some(receiver.clone());
                        let result = dispatch_provider_call(self, &Value16::object(config));
                        self.dispatch_provider_receiver = prev;
                        return result;
                    }
                }
            }
            // Provider / LLM dispatch
            let is_mcp = obj
                .get(&hudhudscript_bytecode::well_known::wk().module)
                .map_or(false, |v| v.as_str() == Some("mcp"));
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
                .get(&hudhudscript_bytecode::well_known::wk().module)
                .map_or(false, |v| v.as_str() == Some("mcp"))
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
            if let Some(chunk_name) = obj.get(&method_sym).and_then(|v| v.as_string()) {
                if let Some(chunk) = bytecode.get_function(chunk_name.as_str()) {
                    let func_sym = hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&chunk_name).0);
                    return self.with_this_context(receiver, None, |this| {
                        this.call_chunk(&chunk, &chunk.params, &args, bytecode, func_sym)
                    });
                }
            }
            // Property holding a function value — module namespace functions
            if let Some(f) = Self::property_function_value(obj.get(&method_sym)) {
                return self.call_property_function(receiver, f, args, bytecode);
            }
            // Property access fallback
            match crate::vm::builtin_method::lookup_method(method_sym) {
                Some(crate::vm::builtin_method::BuiltinMethod::Keys) => {
                    let mut ks: Vec<Value16> =
                        obj.keys().map(|k| Value16::string(k.to_string())).collect();
                    ks.sort_by(|a, b| {
                        let sa = a.as_string().unwrap_or_default();
                        let sb = b.as_string().unwrap_or_default();
                        sa.cmp(&sb)
                    });
                    Ok(Value16::array(ks))
                }
                Some(crate::vm::builtin_method::BuiltinMethod::Values) => {
                    let mut pairs: Vec<(String, Value16)> = obj
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.clone()))
                        .collect();
                    pairs.sort_by_key(|(k, _)| k.clone());
                    Ok(Value16::array(
                        pairs.into_iter().map(|(_, v)| v.clone()).collect(),
                    ))
                }
                Some(crate::vm::builtin_method::BuiltinMethod::Length) => Ok(Value16::int(obj.len() as i64)),
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
                if let Some(packed) = parent_cls.vtable.get(&method_sym).and_then(|v| v.as_int())
                {
                    let idx = (packed >> 32) as u32;
                    let chunk_sym = hudhudscript_bytecode::SymId(packed as u32);
                    let chunk = bytecode.get_function_by_index(idx)
                        .ok_or_else(|| err_function_idx_missing(idx))?;
                    let cn_sym = hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(class_name).0);
                    return self.with_this_context(
                        receiver,
                        Some(cn_sym),
                        |this| {
                            this.call_chunk(&chunk, &chunk.params, &args, bytecode, chunk_sym)
                            },
                        );
                    }
            }
            // Fallback: look up method in instance fields
            if let Some(chunk_name) = fields.get(&method_sym).and_then(|v| v.as_string()) {
                if let Some(chunk) = bytecode.get_function(&chunk_name) {
                    let func_sym = hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(&chunk_name).0);
                    return self.with_this_context(receiver, None, |this| {
                        this.call_chunk(&chunk, &chunk.params, &args, bytecode, func_sym)
                    });
                }
            }
            // Property access fallback
            match crate::vm::builtin_method::lookup_method(method_sym) {
                Some(crate::vm::builtin_method::BuiltinMethod::Keys) => {
                    let mut ks: Vec<Value16> = fields
                        .keys()
                        .map(|k| Value16::string(k.to_string()))
                        .collect();
                    ks.sort_by(|a, b| {
                        let sa = a.as_string().unwrap_or_default();
                        let sb = b.as_string().unwrap_or_default();
                        sa.cmp(&sb)
                    });
                    Ok(Value16::array(ks))
                }
                Some(crate::vm::builtin_method::BuiltinMethod::Values) => {
                    let mut pairs: Vec<(String, Value16)> = fields
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.clone()))
                        .collect();
                    pairs.sort_by_key(|(k, _)| k.clone());
                    Ok(Value16::array(
                        pairs.into_iter().map(|(_, v)| v.clone()).collect(),
                    ))
                }
                Some(crate::vm::builtin_method::BuiltinMethod::Length) => Ok(Value16::int(fields.len() as i64)),
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
                if inner.get(&hudhudscript_bytecode::well_known::wk().type_).and_then(|v| v.as_str())
                    == Some("subject_instance")
                {
                    let template = inner
                        .get(&hudhudscript_bytecode::well_known::wk().template)
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
