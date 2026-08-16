use crate::vm::call_state::{DeferredCallSite, MethodDispatchOutcome};
use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{Bytecode, Value16};

impl VM {
    pub(crate) fn dispatch_native_method(
        &mut self,
        receiver: &Value16,
        method: &str,
        args: &[Value16],
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<Option<MethodDispatchOutcome>> {
        if let Some(string) = receiver.as_str() {
            return self
                .call_string_method(string, method, args, receiver.is_dynamic_string_ascii())
                .map(|value| Some(MethodDispatchOutcome::Immediate(value)));
        }
        if let Some(value) = self.dispatch_native_object(receiver, method, args)? {
            return Ok(Some(MethodDispatchOutcome::Immediate(value)));
        }
        if let Some(state) = receiver.as_promise_state() {
            return self
                .dispatch_promise_value(receiver, state, method, args, bytecode, Some(call_site))
                .map(Some);
        }
        if let Some(items) = receiver.as_set() {
            return self
                .call_set_method(items, method, args.to_vec())
                .map(|value| Some(MethodDispatchOutcome::Immediate(value)));
        }
        if let Some(pairs) = receiver.as_map_pairs() {
            return self
                .call_map_method(pairs, method, args.to_vec())
                .map(|value| Some(MethodDispatchOutcome::Immediate(value)));
        }
        if let Some(state) = receiver.as_generator_state() {
            let value = match method {
                "next" => crate::vm::exec::helpers::generator_advance(self, state)
                    .unwrap_or(Value16::null()),
                "toArray" => Value16::array(state.lock().collect_all()),
                _ => {
                    return Err(compile_codes::runtime_error(format!(
                        "No method '{}' on generator",
                        method
                    )))
                }
            };
            return Ok(Some(MethodDispatchOutcome::Immediate(value)));
        }
        Ok(None)
    }

    fn dispatch_native_object(
        &mut self,
        receiver: &Value16,
        method: &str,
        args: &[Value16],
    ) -> CompileResult<Option<Value16>> {
        let Some(object) = receiver.as_object() else {
            return Ok(None);
        };
        if Some(*receiver) == self.math_obj {
            return self.call_math_method(method, args.to_vec()).map(Some);
        }
        if Some(*receiver) == self.json_obj {
            return self.call_json_method(method, args.to_vec()).map(Some);
        }

        let module = object
            .get(&hudhudscript_bytecode::well_known::wk().module)
            .and_then(|value| value.as_string());
        if let Some(module_name) = &module {
            if let Some(result) = self
                .module_registry
                .call(module_name, method, args.to_vec())
            {
                return result.map(Some);
            }
        }

        match module.as_deref() {
            Some("http") => self.call_http_method(method, args.to_vec()).map(Some),
            Some("file") => self.call_file_method(method, args.to_vec()).map(Some),
            Some("Promise") => self.call_promise_method(method, args.to_vec()).map(Some),
            Some("linalg") => self.call_linalg_method(method, args.to_vec()).map(Some),
            Some("stats") => self.call_stats_method(method, args.to_vec()).map(Some),
            Some("TOML") => self.call_toml_method(method, args.to_vec()).map(Some),
            Some("YAML") => self.call_yaml_method(method, args.to_vec()).map(Some),
            Some("CSV") => self.call_csv_method(method, args.to_vec()).map(Some),
            Some("INI") => self.call_ini_method(method, args.to_vec()).map(Some),
            Some("Base64") => self.call_base64_method(method, args.to_vec()).map(Some),
            Some("Hex") => self.call_hex_method(method, args.to_vec()).map(Some),
            Some("URL") => self.call_url_method(method, args.to_vec()).map(Some),
            Some("uuid") => self.call_uuid_method(method, args.to_vec()).map(Some),
            Some("Path") => self.call_path_method(method, args.to_vec()).map(Some),
            Some("Temp") => self.call_temp_method(method, args.to_vec()).map(Some),
            Some("URLParser") => self.call_url_parser_method(method, args.to_vec()).map(Some),
            Some("Glob") => self.call_glob_method(method, args.to_vec()).map(Some),
            Some("Set") => self.call_set_module_method(method, args.to_vec()).map(Some),
            Some("Map") => self.call_map_module_method(method, args.to_vec()).map(Some),
            Some("stdin") => self.call_stdin_method(method, args.to_vec()).map(Some),
            Some("Terminal") => self.call_terminal_method(method, args.to_vec()).map(Some),
            Some("log") => self.call_log_method(method, args.to_vec()).map(Some),
            Some("exec") => self.call_exec_method(method, args.to_vec()).map(Some),
            Some("tcp") => self.call_tcp_method(method, args.to_vec()).map(Some),
            Some("udp") => self.call_udp_method(method, args.to_vec()).map(Some),
            Some("unix") => self.call_unix_method(method, args.to_vec()).map(Some),
            Some("ws") => self.call_ws_method(method, args.to_vec()).map(Some),
            Some("daemon") => self.call_daemon_method(method, args.to_vec()).map(Some),
            Some("fs") => self.call_fs_method(method, args.to_vec()).map(Some),
            Some("Env") => self.call_env_method(method, args.to_vec()).map(Some),
            Some("tokenomics") => self.call_tokenomics_method(method, args.to_vec()).map(Some),
            Some("channel") => self.call_channel_method(method, args.to_vec()).map(Some),
            Some("os") => self.call_os_method(method, args.to_vec()).map(Some),
            Some("Date") => self.call_date_method(method, args.to_vec()).map(Some),
            Some("Duration") => self.call_duration_method(method, args.to_vec()).map(Some),
            Some("regex") => self.call_regex_method(method, args.to_vec()).map(Some),
            Some("schedule") => self.call_schedule_method(method, args.to_vec()).map(Some),
            Some("EventBus") => self.call_event_bus_method(method, args.to_vec()).map(Some),
            Some("Plugin") => self.call_plugin_method(method, args.to_vec()).map(Some),
            Some("McpServer") => self.call_mcp_server_method(method, args.to_vec()).map(Some),
            Some("Server") => self.call_server_method(method, args.to_vec()).map(Some),
            Some("PluginConfig") => self
                .call_plugin_config_method(method, args.to_vec())
                .map(Some),
            Some("StringBuilder") => self
                .call_string_builder_method(method, args.to_vec(), receiver)
                .map(Some),
            _ => Ok(None),
        }
    }

    fn schedule_promise_callback(
        &mut self,
        callback: Value16,
        argument: Value16,
        bytecode: &Bytecode,
        site: DeferredCallSite,
    ) -> CompileResult<MethodDispatchOutcome> {
        let function = callback.as_function_data().ok_or_else(|| {
            compile_codes::runtime_error(format!("Expected function, got {:?}", callback))
        })?;
        let chunk = bytecode.get_function(&function.chunk_name).ok_or_else(|| {
            compile_codes::runtime_error(format!(
                "Function chunk not found: {}",
                function.chunk_name
            ))
        })?;
        let captures = function
            .captures
            .iter()
            .map(|(name, cell)| (name.clone(), std::sync::Arc::clone(cell)))
            .collect();
        let state = crate::vm::call_state::PromiseCallbackState {
            dst: site.dst,
            callback,
            origin_ip: site.origin_ip,
            argument,
            chunk,
            func_sym: hudhudscript_bytecode::SymId(function.chunk_sym),
            captures,
        };
        let request = state.request();
        self.schedule_vm_call_with_continuation(
            crate::vm::call_state::VmContinuation::PromiseCallback(state),
            request,
        )?;
        Ok(MethodDispatchOutcome::Deferred)
    }

    fn dispatch_promise_value(
        &mut self,
        receiver: &Value16,
        state: &hudhudscript_bytecode::PromiseState16,
        method: &str,
        args: &[Value16],
        bytecode: &Bytecode,
        call_site: Option<DeferredCallSite>,
    ) -> CompileResult<MethodDispatchOutcome> {
        use hudhudscript_bytecode::PromiseState16;
        if args.is_empty() && matches!(method, "then" | "catch") {
            return Err(compile_codes::runtime_error(format!(
                "{}() requires a callback",
                method
            )));
        }
        match (method, state) {
            ("then", PromiseState16::Resolved(value)) => {
                let site = call_site.ok_or_else(|| {
                    crate::vm::call_state::deferred_method_in_immediate_context("promise-then")
                })?;
                self.schedule_promise_callback(args[0], **value, bytecode, site)
            }
            ("then", PromiseState16::Rejected(message)) => Ok(MethodDispatchOutcome::Immediate(
                Value16::promise(PromiseState16::Rejected(message.clone())),
            )),
            ("then", _) | ("catch", PromiseState16::Resolved(_)) => {
                Ok(MethodDispatchOutcome::Immediate(*receiver))
            }
            ("catch", PromiseState16::Rejected(message)) => {
                let site = call_site.ok_or_else(|| {
                    crate::vm::call_state::deferred_method_in_immediate_context("promise-catch")
                })?;
                self.schedule_promise_callback(
                    args[0],
                    Value16::string(message.clone()),
                    bytecode,
                    site,
                )
            }
            ("catch", _) => Ok(MethodDispatchOutcome::Immediate(*receiver)),
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown method '{}' on promise",
                method
            ))),
        }
    }

    /// StringBuilder builtin: O(n) concatenation via accumulated parts.
    pub(crate) fn call_string_builder_method(
        &mut self,
        method: &str,
        args: Vec<Value16>,
        receiver: &Value16,
    ) -> CompileResult<Value16> {
        match method {
            "new" => {
                let mut object = hudhudscript_bytecode::ObjMap::default();
                object.insert(
                    "__module".to_string(),
                    Value16::string("StringBuilder".to_string()),
                );
                object.insert("__parts".to_string(), Value16::array(Vec::new()));
                Ok(Value16::object(object))
            }
            "append" => {
                let object = receiver.as_object().ok_or_else(|| {
                    compile_codes::runtime_error(
                        "StringBuilder.append: receiver is not an object".to_string(),
                    )
                })?;
                let parts_value = object.get("__parts").ok_or_else(|| {
                    compile_codes::runtime_error(
                        "StringBuilder: not a valid builder instance".to_string(),
                    )
                })?;
                let mut parts = parts_value.as_array().cloned().ok_or_else(|| {
                    compile_codes::runtime_error(
                        "StringBuilder: __parts is not an array".to_string(),
                    )
                })?;
                parts.push(Value16::string(
                    args.first()
                        .map(Value16::display_string)
                        .unwrap_or_default(),
                ));
                let mut updated = object.clone();
                updated.insert("__parts".to_string(), Value16::array(parts));
                Ok(Value16::object(updated))
            }
            "build" => {
                let object = receiver.as_object().ok_or_else(|| {
                    compile_codes::runtime_error(
                        "StringBuilder.build: receiver is not an object".to_string(),
                    )
                })?;
                let parts_value = object.get("__parts").ok_or_else(|| {
                    compile_codes::runtime_error(
                        "StringBuilder: not a valid builder instance".to_string(),
                    )
                })?;
                let parts = parts_value.as_array().ok_or_else(|| {
                    compile_codes::runtime_error(
                        "StringBuilder: __parts is not an array".to_string(),
                    )
                })?;
                let total = parts.iter().map(|part| part.display_string().len()).sum();
                let mut result = String::with_capacity(total);
                for part in parts {
                    result.push_str(&part.display_string());
                }
                Ok(Value16::string(result))
            }
            _ => Err(compile_codes::runtime_error(format!(
                "StringBuilder: unknown method '{}'",
                method
            ))),
        }
    }
}
