use crate::vm::call_state::{DeferredCallSite, MethodDispatchOutcome, ReceiverContext};
use crate::vm::mcp_dispatch::dispatch_mcp_tool_call;
use crate::vm::provider_dispatch::dispatch_provider_call;
use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{Bytecode, SymId, Value16};
use rustc_hash::FxHashMap;
use std::sync::Arc;

impl VM {
    pub(crate) fn dispatch_agent_object_method(
        &mut self,
        receiver: &Value16,
        method: &str,
        args: &[Value16],
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<Option<MethodDispatchOutcome>> {
        let Some(object) = receiver.as_object() else {
            return Ok(None);
        };

        if let Some(name) = object.get("name").and_then(|value| value.as_string()) {
            if self.swarm_names.contains_key(&name) && matches!(method, "run" | "execute") {
                let agents = object_string_array(object.get("agents"));
                let task = args.first().copied().unwrap_or(Value16::null());
                return self
                    .dispatch_swarm_run(&name, &agents, &task, bytecode)
                    .map(|value| Some(MethodDispatchOutcome::Immediate(value)));
            }
            if self.swarm_names.contains_key(&name) && method == "add_agent" {
                let agent = args.first().copied().unwrap_or(Value16::null());
                return self
                    .dispatch_swarm_add_agent(&name, &agent, bytecode)
                    .map(|value| Some(MethodDispatchOutcome::Immediate(value)));
            }
            if self.swarm_names.contains_key(&name) && method == "remove_agent" {
                let agent = args
                    .first()
                    .and_then(|value| value.as_string())
                    .unwrap_or_default();
                return self
                    .dispatch_swarm_remove_agent(&name, &agent, bytecode)
                    .map(|value| Some(MethodDispatchOutcome::Immediate(value)));
            }
            if self.council_names.contains_key(&name) && matches!(method, "decide" | "vote") {
                let agents = council_agent_names(object.get("members"));
                let task = args.first().copied().unwrap_or(Value16::null());
                return self
                    .dispatch_swarm_run(&name, &agents, &task, bytecode)
                    .map(|value| Some(MethodDispatchOutcome::Immediate(value)));
            }
            if self.community_names.contains_key(&name) && matches!(method, "run" | "decide") {
                let councils = object_string_array(object.get("councils"));
                let task = args.first().copied().unwrap_or(Value16::null());
                let mut results = Vec::with_capacity(councils.len());
                for council_name in councils {
                    let Some(council) = self
                        .get_var_cloned(&council_name)
                        .and_then(|value| value.as_object().cloned())
                    else {
                        continue;
                    };
                    let agents = council_agent_names(council.get("members"));
                    results.push(self.dispatch_swarm_run(
                        &council_name,
                        &agents,
                        &task,
                        bytecode,
                    )?);
                }
                return Ok(Some(MethodDispatchOutcome::Immediate(Value16::array(
                    results,
                ))));
            }
        }

        if let Some(agent_name) = object.get("name").and_then(|value| value.as_string()) {
            if self.agent_names.contains_key(&agent_name) {
                return self
                    .dispatch_registered_agent(
                        receiver,
                        &agent_name,
                        method,
                        args,
                        bytecode,
                        call_site,
                    )
                    .map(Some);
            }
        }

        let is_mcp = object
            .get(&hudhudscript_bytecode::well_known::wk().module)
            .is_some_and(|value| value.as_str() == Some("mcp"));
        if matches!(method, "call" | "stream") && !is_mcp {
            let config = args.first().copied().unwrap_or(Value16::null());
            let previous = self.dispatch_provider_receiver.replace(*receiver);
            let result = dispatch_provider_call(self, &config);
            self.dispatch_provider_receiver = previous;
            return result.map(|value| Some(MethodDispatchOutcome::Immediate(value)));
        }

        if is_mcp {
            if let Some(server) = object.get("__server").and_then(|value| value.as_string()) {
                let tool_args = match args {
                    [] => Value16::null(),
                    [only] => *only,
                    many => Value16::array(many.to_vec()),
                };
                return dispatch_mcp_tool_call(self, &server, method, &tool_args)
                    .map(|value| Some(MethodDispatchOutcome::Immediate(value)));
            }
            if method == "call" {
                if args.len() < 2 {
                    return Err(compile_codes::runtime_error(
                        "mcp.call() requires (server, tool, [args])".to_string(),
                    ));
                }
                let server = self.value_to_string(&args[0]);
                let tool = self.value_to_string(&args[1]);
                let tool_args = args.get(2).copied().unwrap_or(Value16::null());
                return dispatch_mcp_tool_call(self, &server, &tool, &tool_args)
                    .map(|value| Some(MethodDispatchOutcome::Immediate(value)));
            }
        }

        Ok(None)
    }

    fn dispatch_registered_agent(
        &mut self,
        receiver: &Value16,
        agent_name: &str,
        method: &str,
        args: &[Value16],
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<MethodDispatchOutcome> {
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
            let action_sym = SymId(hudhudscript_bytecode::interner::intern(&action_name).0);
            if action_chunk.is_async {
                let previous_this = self.get_var_cloned_by_sym(self.this_sym);
                self.set_var_by_sym(self.this_sym, "this", *receiver)?;
                let promise = self.spawn_async_chunk(
                    Arc::clone(&action_chunk),
                    &action_chunk.params,
                    args,
                    bytecode,
                    &action_name,
                    None,
                );
                match previous_this {
                    Some(value) => self.set_var_by_sym(self.this_sym, "this", value)?,
                    None => self.remove_var_by_sym(self.this_sym),
                }
                return Ok(MethodDispatchOutcome::Immediate(promise));
            }
            let context = ReceiverContext::new(*receiver, None, true);
            return self.schedule_deferred_chunk_call(
                action_chunk,
                action_sym,
                args.to_vec(),
                FxHashMap::default(),
                Some(context),
                call_site,
            );
        }

        let provider_value = receiver
            .as_object()
            .and_then(|object| object.get("provider"));
        let mut provider_object = provider_value.and_then(|value| {
            if value.is_object() {
                Some(*value)
            } else {
                None
            }
        });
        if provider_object.is_none() {
            if let Some(provider_name) = provider_value.and_then(|value| value.as_string()) {
                provider_object = self.resolve_agent_provider(&provider_name);
            }
        }
        if provider_object.is_none() {
            let unresolved = provider_value.copied().unwrap_or(Value16::null());
            return Err(compile_codes::runtime_error(format!(
                "Agent '{}' provider did not resolve: provider field is {}",
                agent_name, unresolved
            )));
        }

        let config = if matches!(method, "call" | "stream") {
            args.first().copied().unwrap_or(Value16::null())
        } else {
            let rendered: Vec<String> = args
                .iter()
                .map(|value| self.value_to_string(value))
                .collect();
            let mut config = hudhudscript_bytecode::ObjMap::default();
            config.insert(
                "prompt".to_string(),
                Value16::string(format!("Task: {}. Args: {}", method, rendered.join(", "))),
            );
            Value16::object(config)
        };
        let previous = self.dispatch_provider_receiver.replace(*receiver);
        let result = dispatch_provider_call(self, &config);
        self.dispatch_provider_receiver = previous;
        result.map(MethodDispatchOutcome::Immediate)
    }

    fn resolve_agent_provider(&self, provider_name: &str) -> Option<Value16> {
        let mut current = self.get_var_cloned(provider_name);
        if current.is_some() {
            return current;
        }
        if !provider_name.contains('.') {
            return None;
        }

        let mut parts = provider_name.split('.');
        current = parts.next().and_then(|name| self.get_var_cloned(name));
        for part in parts {
            current = current.and_then(|value| {
                value
                    .as_object()
                    .and_then(|object| object.get(part))
                    .copied()
            });
        }
        current.filter(Value16::is_object)
    }
}

fn object_string_array(value: Option<&Value16>) -> Vec<String> {
    value
        .and_then(Value16::as_array)
        .map(|items| items.iter().filter_map(|item| item.as_string()).collect())
        .unwrap_or_default()
}

fn council_agent_names(value: Option<&Value16>) -> Vec<String> {
    value
        .and_then(Value16::as_array)
        .map(|members| {
            members
                .iter()
                .filter_map(|member| {
                    member
                        .as_object()
                        .and_then(|object| object.get("agent_id"))
                        .and_then(Value16::as_string)
                })
                .collect()
        })
        .unwrap_or_default()
}
