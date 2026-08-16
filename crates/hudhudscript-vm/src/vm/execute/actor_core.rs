#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_actor_core(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::DeclStore {
                payload_idx, src, ..
            } => {
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
                                state_defaults.insert(k.to_string(), *v);
                            }
                        }
                        let roles = obj
                            .get("roles")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_string()).collect())
                            .unwrap_or_default();
                        let capabilities = obj
                            .get("can")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_string()).collect())
                            .unwrap_or_default();
                        let of_subject = obj
                            .get("__of_subject")
                            .and_then(|v| v.as_string())
                            .map(|s| s.to_string());
                        let intents = obj
                            .get("intents")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_string()).collect())
                            .unwrap_or_default();
                        self.subject_templates.insert(
                            name.clone(),
                            crate::vm::sop_types::SubjectTemplate {
                                name: name.clone(),
                                of_subject,
                                roles,
                                state_defaults,
                                capabilities,
                                intents,
                            },
                        );
                    }
                }
                // SOP0007: composition rules registration
                if kind == "compose" {
                    if let Some(obj) = fields.as_object() {
                        if let Some(rules_arr) = obj.get("rules").and_then(|v| v.as_array()) {
                            for rule_val in rules_arr {
                                if let Some(rule_obj) = rule_val.as_object() {
                                    let ability = rule_obj
                                        .get("ability")
                                        .and_then(|v| v.as_string())
                                        .unwrap_or_default();
                                    let mode_str = rule_obj
                                        .get("mode")
                                        .and_then(|v| v.as_string())
                                        .unwrap_or_default();
                                    let mode = match mode_str.as_str() {
                                        "combine" => {
                                            let subjects: Vec<String> = rule_obj
                                                .get("subjects")
                                                .and_then(|v| v.as_array())
                                                .map(|arr| {
                                                    arr.iter()
                                                        .filter_map(|v| v.as_string())
                                                        .collect()
                                                })
                                                .unwrap_or_default();
                                            crate::vm::sop_types::CompositionMode::Combine(subjects)
                                        }
                                        "override" => {
                                            let s = rule_obj
                                                .get("subject")
                                                .and_then(|v| v.as_string())
                                                .unwrap_or_default();
                                            crate::vm::sop_types::CompositionMode::Override(
                                                s.to_string(),
                                            )
                                        }
                                        "before" => {
                                            let s = rule_obj
                                                .get("subject")
                                                .and_then(|v| v.as_string())
                                                .unwrap_or_default();
                                            crate::vm::sop_types::CompositionMode::Before(
                                                s.to_string(),
                                            )
                                        }
                                        "after" => {
                                            let s = rule_obj
                                                .get("subject")
                                                .and_then(|v| v.as_string())
                                                .unwrap_or_default();
                                            crate::vm::sop_types::CompositionMode::After(
                                                s.to_string(),
                                            )
                                        }
                                        _ => crate::vm::sop_types::CompositionMode::Combine(vec![]),
                                    };
                                    let key = format!("{}::{}", name, ability);
                                    self.composition_rules.entry(key).or_default().push(
                                        crate::vm::sop_types::CompositionRule {
                                            ability_name: ability.to_string(),
                                            mode,
                                        },
                                    );
                                }
                            }
                        }
                        // SOP0009: field correspondence rules
                        if let Some(field_rules_arr) =
                            obj.get("field_rules").and_then(|v| v.as_array())
                        {
                            for fr_val in field_rules_arr {
                                if let Some(fr) = fr_val.as_object() {
                                    if let Some(field_name) =
                                        fr.get("field").and_then(|v| v.as_string())
                                    {
                                        let mode_opt = fr.get("mode").and_then(|v| v.as_string());
                                        let corr = match mode_opt.as_deref() {
                                            Some("correspond") => crate::vm::sop_types::FieldCorrespondence::Correspond,
                                            _ => crate::vm::sop_types::FieldCorrespondence::Separate,
                                        };
                                        let key = format!("{}::state::{}", name, field_name);
                                        let corr_str = match corr {
                                            crate::vm::sop_types::FieldCorrespondence::Correspond => "correspond",
                                            crate::vm::sop_types::FieldCorrespondence::Separate => "separate",
                                        };
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
                            schema_fields.push((k.to_string(), type_hint.to_string()));
                        }
                    }
                    self.event_schemas.insert(
                        name.clone(),
                        crate::vm::sop_types::EventSchema {
                            name: name.clone(),
                            fields: schema_fields,
                        },
                    );
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
                        // No silent fallback (Kural 7c): an unrecognised
                        // transport used to be treated as stdio, which quietly
                        // demanded `allow_process` and spawned a process for a
                        // declaration that asked for something else.
                        let transport_kind = match transport_str.as_str() {
                            "sse" | "SSE" => McpTransportKind::Sse,
                            "stdio" | "STDIO" => McpTransportKind::Stdio,
                            other => {
                                return Err(compile_codes::runtime_error(format!(
                                    "mcp server '{}': unknown transport '{}' (expected \"stdio\" or \"sse\")",
                                    name, other
                                )));
                            }
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

                        // MCP-40: Sandbox gate — process/network access must
                        // be explicitly granted via allow_process / allow_network.
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
                                    if !sandbox.allow_process {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Sandbox: process execution denied for mcp server '{}'. Set allow_process: true in sandbox config.",
                                            name
                                        )));
                                    }
                                    // MCP-40: Command allowlist/denylist check.
                                    if let Some(cmd) = command.as_ref() {
                                        let cmd_base = std::path::Path::new(cmd)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or(cmd);
                                        for denied in &sandbox.denied_commands {
                                            if cmd_base == denied.as_str() || cmd.contains(denied) {
                                                return Err(compile_codes::runtime_error(format!(
                                                    "Sandbox: command '{}' is denied for mcp server '{}'",
                                                    cmd, name
                                                )));
                                            }
                                        }
                                        if !sandbox.allowed_commands.is_empty() {
                                            let allowed =
                                                sandbox.allowed_commands.iter().any(|a| {
                                                    cmd_base == a.as_str()
                                                        || cmd.contains(a.as_str())
                                                });
                                            if !allowed {
                                                return Err(compile_codes::runtime_error(format!(
                                                    "Sandbox: command '{}' not in allowed_commands for mcp server '{}'",
                                                    cmd, name
                                                )));
                                            }
                                        }
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
                            self.allow_insecure_http,
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
                        // 1. Toml'dan eksik field'ları doldur
                        if let Some(toml_fields) = self.toml_providers.get(&name) {
                            for (k, v) in toml_fields {
                                let is_timeout_key =
                                    k == "timeout" || k == "timeout_secs" || k == "timeout_seconds";
                                if is_timeout_key
                                    && (merged.contains_key("timeout")
                                        || merged.contains_key("timeout_secs")
                                        || merged.contains_key("timeout_seconds"))
                                {
                                    continue;
                                }
                                merged.entry(k.clone()).or_insert_with(|| {
                                    if let Some(env_name) =
                                        v.strip_prefix("${").and_then(|s| s.strip_suffix("}"))
                                    {
                                        Value16::string(format!("__hudhud_env__:{}", env_name))
                                    } else if is_timeout_key {
                                        if let Ok(n) = v.parse::<f64>() {
                                            Value16::number(n)
                                        } else {
                                            Value16::string(v.clone())
                                        }
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
                        let key = format!("{}:{}", kind, name);
                        self.declarations.insert(key, fields.clone());
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

            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
