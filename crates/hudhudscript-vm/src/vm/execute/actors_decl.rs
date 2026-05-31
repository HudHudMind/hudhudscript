#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline]
    pub(crate) fn step_actors_decl(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let bytecode = ctx.bytecode;

        match instr {
            Instruction::DeclStore { payload_idx, src, .. } => {
                // CROSS-2d: two-sym payload — first = kind symbol,
                // second = name symbol.
                let payload = bytecode.get_two_sym_payload(*payload_idx as u32);
                let kind = bytecode.resolve_symbol(payload.first);
                let name = bytecode.resolve_symbol(payload.second);
                let mut fields = self.registers[*src as usize];
                let key = format!("{}:{}", kind, name);
                self.declarations.insert(key, fields.clone());
                // #351 — register effects in effect registry
                if kind == "effect" {
                    let fields_v = fields;
                    if let Some(obj) = fields_v.as_object() {
                        if let Some(event) = obj.get("event").and_then(|v| v.as_string()) {
                            if let Some(func) =
                                obj.get("handler").and_then(|v| v.as_function_data())
                            {
                                self.effects.insert(event, func.chunk_name.clone());
                            }
                        }
                    }
                }
                // #352 — register relations in relation store
                if kind == "relation" {
                    self.relations.insert(name.clone(), fields.clone());
                }
                // SOP: subject template registration
                if kind == "subject" {
                    if let Some(obj) = fields.as_object() {
                        let mut state_defaults = rustc_hash::FxHashMap::default();
                        if let Some(state_obj) = obj.get("state").and_then(|v| v.as_object()) {
                            for (k, v) in &*state_obj {
                                state_defaults.insert(k.clone(), *v);
                            }
                        }
                        let roles = obj.get("roles")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_string()).collect())
                            .unwrap_or_default();
                        let capabilities = obj.get("can")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_string()).collect())
                            .unwrap_or_default();
                        let of_subject = obj.get("__of_subject").and_then(|v| v.as_string()).map(|s| s.to_string());
                        self.subject_templates.insert(name.clone(), crate::vm::sop_types::SubjectTemplate {
                            name: name.clone(),
                            of_subject,
                            roles,
                            state_defaults,
                            capabilities,
                            intents: vec![],
                        });
                    }
                }
                // SOP0007: composition rules registration
                if kind == "compose" {
                    if let Some(obj) = fields.as_object() {
                        if let Some(rules_arr) = obj.get("rules").and_then(|v| v.as_array()) {
                            for rule_val in rules_arr {
                                if let Some(rule_obj) = rule_val.as_object() {
                                    let ability = rule_obj.get("ability").and_then(|v| v.as_string()).unwrap_or_default();
                                    let mode_str = rule_obj.get("mode").and_then(|v| v.as_string()).unwrap_or_default();
                                    let mode = match mode_str.as_str() {
                                        "combine" => {
                                            let subjects: Vec<String> = rule_obj.get("subjects")
                                                .and_then(|v| v.as_array())
                                                .map(|arr| arr.iter().filter_map(|v| v.as_string()).collect())
                                                .unwrap_or_default();
                                            crate::vm::sop_types::CompositionMode::Combine(subjects)
                                        }
                                        "override" => {
                                            let s = rule_obj.get("subject").and_then(|v| v.as_string()).unwrap_or_default();
                                            crate::vm::sop_types::CompositionMode::Override(s.to_string())
                                        }
                                        "before" => {
                                            let s = rule_obj.get("subject").and_then(|v| v.as_string()).unwrap_or_default();
                                            crate::vm::sop_types::CompositionMode::Before(s.to_string())
                                        }
                                        "after" => {
                                            let s = rule_obj.get("subject").and_then(|v| v.as_string()).unwrap_or_default();
                                            crate::vm::sop_types::CompositionMode::After(s.to_string())
                                        }
                                        _ => crate::vm::sop_types::CompositionMode::Combine(vec![]),
                                    };
                                    let key = format!("{}::{}", name, ability);
                                    self.composition_rules.entry(key).or_default().push(
                                        crate::vm::sop_types::CompositionRule { ability_name: ability.to_string(), mode }
                                    );
                                }
                            }
                        }
                        // SOP0009: field correspondence rules
                        if let Some(field_rules_arr) = obj.get("field_rules").and_then(|v| v.as_array()) {
                            for fr_val in field_rules_arr {
                                if let Some(fr) = fr_val.as_object() {
                                    if let Some(field_name) = fr.get("field").and_then(|v| v.as_string()) {
                                        let corr = match fr.get("mode").and_then(|v| v.as_string()).as_deref() {
                                            Some("correspond") => crate::vm::sop_types::FieldCorrespondence::Correspond,
                                            _ => crate::vm::sop_types::FieldCorrespondence::Separate,
                                        };
                                        let key = format!("{}::state::{}", name, field_name);
                                        self.field_correspondences.insert(key, corr);
                                    }
                                }
                            }
                        }
                    }
                }
                // SOP: event schema registration
                if kind == "event_schema" {
                    let mut schema_fields = Vec::new();
                    if let Some(obj) = fields.as_object() {
                        for (k, v) in &*obj {
                            let type_hint = v.as_string().unwrap_or_else(|| "Any".to_string());
                            schema_fields.push((k.clone(), type_hint.to_string()));
                        }
                    }
                    self.event_schemas.insert(name.clone(), crate::vm::sop_types::EventSchema {
                        name: name.clone(),
                        fields: schema_fields,
                    });
                }
                // #347 / Kural 2: `constitution Name { ... }` must
                // populate the typed constitution store and activate
                // it, exactly like the interpreter's `register + activate`
                // sequence.  Without this the VM's governance methods
                // (has_active_constitution / check_constitution_compliance)
                // never see the declaration and tests that use the
                // constitution block directly (rather than calling
                // `register_constitution(...)` as a builtin) silently
                // succeed against an empty constitution set.
                if kind == "constitution" {
                    let typed = Self::value_to_constitution(&name, &fields);
                    self.constitutions.insert(name.clone(), typed);
                    self.active_constitution = Some(name.clone());
                }
                // MCP-001 / Kural 7: an `mcp server Name { ... }` block
                // must actually spawn an McpClient so later
                // `mcp.Name.tool(args)` dispatches can reach a real
                // server — previously the VM only stored the config
                // object and left `mcp_clients` empty, which made
                // every dispatch fail with "MCP server not found".
                // Client lifecycle (transport → initialize → response
                // handler) lives in the shared dispatcher so both
                // runtimes use the same code path.
                if kind == "mcp_server" {
                    let fields_v = fields;
                    if let Some(obj) = fields_v.as_object() {
                        use crate::vm::mcp_dispatch::{
                            create_mcp_client_from_config, McpTransportKind,
                        };
                        let transport_str = match obj.get("transport").and_then(|v| v.as_string()) {
                            Some(s) => s,
                            _ => "stdio".to_string(),
                        };
                        let transport_kind = match transport_str.as_str() {
                            "sse" | "SSE" => McpTransportKind::Sse,
                            _ => McpTransportKind::Stdio,
                        };
                        let command = match obj.get("command").and_then(|v| v.as_string()) {
                            Some(s) => Some(s),
                            _ => None,
                        };
                        let args_vec: Vec<String> = match obj.get("args").and_then(|v| v.as_array())
                        {
                            Some(arr) => arr.iter().filter_map(|v| v.as_string()).collect(),
                            _ => Vec::new(),
                        };
                        let url = match obj.get("url").and_then(|v| v.as_string()) {
                            Some(s) => Some(s),
                            _ => None,
                        };

                        // Sandbox gate: MCP spawn == process / network
                        // access (Issue #33 parity with the interpreter).
                        if let Some(sandbox) = &self.sandbox {
                            match transport_kind {
                                McpTransportKind::Sse => {
                                    if !sandbox.allow_network {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Sandbox: network access denied for mcp server '{}'",
                                            name
                                        )));
                                    }
                                }
                                McpTransportKind::Stdio => {
                                    // stdio spawns a subprocess; if the
                                    // sandbox disallows both fs and net
                                    // we also deny subprocess execution.
                                    if !sandbox.allow_file_read
                                        && !sandbox.allow_file_write
                                        && !sandbox.allow_network
                                    {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Sandbox: process execution denied for mcp server '{}'",
                                            name
                                        )));
                                    }
                                }
                            }
                        }

                        match create_mcp_client_from_config(
                            &name,
                            transport_kind,
                            command.as_deref(),
                            &args_vec,
                            url.as_deref(),
                        ) {
                            Ok(client) => {
                                self.register_mcp_client(name.clone(), client);
                            }
                            Err(e) => {
                                return Err(compile_codes::runtime_error(e));
                            }
                        }
                    }
                }
                // ENV0004: merge hudhud.toml [providers.*] defaults
                // ENV0002: resolve env() markers (script + toml) — ONCE at DeclStore
                if kind == "provider" {
                    if let Some(obj) = fields.as_object() {
                        let mut merged = obj.clone();
                        // 1. Toml'dan eksik field'ları doldur (script kazanır)
                        if let Some(toml_fields) = self.toml_providers.get(&name) {
                            for (k, v) in toml_fields {
                                merged.entry(k.clone()).or_insert_with(|| {
                                    if let Some(env_name) = v.strip_prefix("${").and_then(|s| s.strip_suffix("}")) {
                                        Value16::string(format!("__hudhud_env__:{}", env_name))
                                    } else {
                                        Value16::string(v.clone())
                                    }
                                });
                            }
                        }
                        // 2. Tüm env marker'ları TEK SEFER resolve et
                        for (_, val) in merged.iter_mut() {
                            if let Some(s) = val.as_string() {
                                if let Some(env_name) = s.strip_prefix("__hudhud_env__:") {
                                    *val = match std::env::var(env_name) {
                                        Ok(v) => Value16::string(v),
                                        Err(_) => Value16::null(),
                                    };
                                }
                            }
                        }
                        fields = Value16::object(merged);
                    }
                }
                // AGENT0003: track agent names for dispatch
                if kind == "agent" {
                    self.agent_names.insert(name.clone(), ());
                }
                if kind == "swarm" {
                    self.swarm_names.insert(name.clone(), ());
                }
                if kind == "council" {
                    self.council_names.insert(name.clone(), ());
                }
                if kind == "community" {
                    self.community_names.insert(name.clone(), ());
                }
                self.set_var(&name, fields)?;
            }

            Instruction::Spawn { payload_idx, first_arg, arg_count } => {
                // CROSS-2c: resolve the call payload from the side table.
                let payload = bytecode.get_call_payload(*payload_idx as u32);
                let subject_name_sym = payload.sym;
                let subject_name = bytecode.resolve_symbol(subject_name_sym.0);
                // Register-based: read args from registers[first_arg..+n]
                let n = *arg_count as usize;
                let first = *first_arg as usize;
                let args: Vec<Value16> = (0..n).map(|i| self.registers[first + i]).collect();

                let (actor_ref, mailbox) = self.actors.spawn();
                let actor_id = actor_ref.id.clone();
                self.actor_mailboxes.insert(actor_id.clone(), mailbox);

                let mut instance = HashMap::new();
                instance.insert("__type".to_string(), Value16::string(subject_name.clone()));
                instance.insert("__actor_id".to_string(), Value16::string(actor_id.clone()));

                // SOP: if this is a subject template, create SubjectInstance with state
                if let Some(template) = self.subject_templates.get(&subject_name).cloned() {
                    let instance_id = format!("{}_{}", subject_name, actor_id);
                    let mut instance_state = template.state_defaults.clone();
                    let subj_inst = crate::vm::sop_types::SubjectInstance {
                        template_name: subject_name.clone(),
                        instance_id: instance_id.clone(),
                        state: instance_state.clone(),
                        actor_id: actor_id.clone(),
                        views: rustc_hash::FxHashMap::default(),
                    };
                    self.subject_instances.insert(instance_id.clone(), subj_inst);

                    // SOP0006: auto-bind views — add all view subjects whose `of` targets this subject
                    let views_for_this: Vec<_> = self.subject_templates.iter()
                        .filter(|(_, tmpl)| tmpl.of_subject.as_deref() == Some(&subject_name))
                        .map(|(view_name, tmpl)| (view_name.clone(), tmpl.state_defaults.clone()))
                        .collect();
                    if !views_for_this.is_empty() {
                        if let Some(inst) = self.subject_instances.get_mut(&instance_id) {
                            for (view_name, view_defaults) in views_for_this {
                                inst.views.insert(view_name, view_defaults);
                            }
                        }
                    }

                    instance.insert("__type".to_string(), Value16::string("subject_instance".to_string()));
                    instance.insert("__instance_id".to_string(), Value16::string(instance_id));
                    instance.insert("__template".to_string(), Value16::string(subject_name.clone()));
                    instance.insert("name".to_string(), Value16::string(subject_name.clone()));
                    // Expose state fields at top level for property access
                    for (k, v) in &instance_state {
                        instance.insert(k.clone(), *v);
                    }
                }

                instance.insert("__args".to_string(), Value16::array(args));
                instance.insert(
                    "__kind__".to_string(),
                    Value16::string("actor_ref".to_string()),
                );
                self.registers[255] = Value16::object(instance);

            }
            Instruction::Despawn { reg } => {
                let obj = self.registers[*reg as usize];
                if let Some(inner) = obj.as_object() {
                    if let Some(instance_id) = inner.get("__instance_id").and_then(|v| v.as_string()) {
                        let instance_id = instance_id.to_string();
                        self.subject_instances.remove(&instance_id);
                        self.registers[*reg as usize] = Value16::null();
                        self.registers[255] = Value16::bool_(true);
                    } else {
                        return Err(compile_codes::runtime_error(
                            "despawn: value is not a subject instance".to_string()
                        ));
                    }
                } else {
                    return Err(compile_codes::runtime_error(
                        "despawn: value is not an object".to_string()
                    ));
                }
            }
            Instruction::ViewAs { obj, view_sym } => {
                let view_name = bytecode.resolve_symbol(*view_sym as u32);
                let obj_val = self.registers[*obj as usize];
                if let Some(map) = obj_val.as_object() {
                    if map.get("__type").and_then(|v| v.as_string()).as_deref() == Some("subject_instance") {
                        if let Some(instance_id) = map.get("__instance_id").and_then(|v| v.as_string()) {
                            if let Some(view_tmpl) = self.subject_templates.get(&view_name) {
                                if let Some(inst) = self.subject_instances.get_mut(&instance_id) {
                                    inst.views.insert(view_name.clone(), view_tmpl.state_defaults.clone());
                                }
                            }
                            let mut new_map = map.clone();
                            new_map.insert("__view_name".to_string(), Value16::string(view_name.clone()));
                            self.registers[255] = Value16::object(new_map);
                        }
                    }
                }
            }
            Instruction::Send { message: msg_reg, target: tgt_reg } => {
                let target = self.registers[*tgt_reg as usize];
                let message = self.registers[*msg_reg as usize];

                let resolved = match target.as_string() {
                    Some(name) => self.get_var_cloned(&name).unwrap_or_else(|| target.clone()),
                    None => target.clone(),
                };

                let resolved_v = resolved;
                let actor_id = if let Some(obj) = resolved_v.as_object() {
                    obj.get("__actor_id").and_then(|v| {
                        if let Some(s) = v.as_string() {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                if let Some(id) = actor_id {
                    let actor_ref = self.actors.get(&id).ok_or_else(|| {
                        compile_codes::runtime_error(format!("Actor not found: {}", id))
                    })?;
                    actor_ref
                        .send(message.clone())
                        .map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
                }
                self.registers[255] = message;

            }
            Instruction::Receive { var_sym_idx: var_name_sym, src, .. } => {
                let var_name = bytecode.resolve_symbol(*var_name_sym as u32);
                let source = self.registers[*src as usize];

                let resolved = match source.as_string() {
                    Some(name) => self.get_var_cloned(&name).unwrap_or_else(|| source.clone()),
                    None => source.clone(),
                };

                let resolved_v = resolved;
                let actor_id = if let Some(obj) = resolved_v.as_object() {
                    obj.get("__actor_id").and_then(|v| {
                        if let Some(s) = v.as_string() {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                let received = match actor_id {
                    Some(id) => {
                        let mailbox = self.actor_mailboxes.get(&id).ok_or_else(|| {
                            compile_codes::runtime_error(format!("Actor not found: {}", id))
                        })?;
                        mailbox
                            .try_recv()
                            .map(|m| m.payload)
                            .unwrap_or(Value16::null())
                    }
                    None => {
                        let resolved_v = resolved;
                        if let Some(_) = resolved_v.as_string() {
                            // Gap 3 (interpreter parity) — a String
                            // source that didn't resolve to a live
                            // actor binds Null (matches `Stmt::Receive`
                            // in the interpreter for the typical
                            // `receive msg from "agent1"` pattern
                            // before the actor has been spawned).
                            Value16::null()
                        } else {
                            // Any other shape (Number / non-actor
                            // Object / etc.) is a hard usage error —
                            // `test_parity_receive_non_actor_fails`
                            // pushes a raw `Number(7.0)` and asserts
                            // the returned error contains "valid actor
                            // reference".
                            return Err(compile_codes::runtime_error(format!(
                                "Receive target must be a valid actor reference, got {}",
                                Self::bytecode_value_type_name(&resolved)
                            )));
                        }
                    }
                };

                self.set_var(&var_name, received)?;
            }
            Instruction::Require { src, .. } => {
                let condition = self.registers[*src as usize];
                if !condition.is_truthy() {
                    // Gap 3 (interpreter parity) — match interpreter's
                    // error text so tests asserting
                    // `err.contains("require")` / `err.contains("condition")`
                    // pass.  Previously the VM emitted
                    // "Requirement not met" (no lowercase "require").
                    return Err(compile_codes::runtime_error(
                        "require condition not met".to_string(),
                    ));
                }
            }
            Instruction::Perform { .. } => {
                return Err(Self::runtime_error_with_pos(
                    "legacy Perform instruction is unsupported; compiler must emit action call",
                    bytecode, ctx.ip,
                ));
            }

            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
