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
    pub(crate) fn dispatch_builtin_group4(
        &mut self,
        name: &str,
        arg_count: u8,
        first_arg: u8,
        bytecode: &hudhudscript_bytecode::Bytecode,
        call_site: crate::vm::call_state::DeferredCallSite,
    ) -> hudhudscript_bytecode::error::CompileResult<bool> {
        match name {
            "dispatch_intent" => {
                // dispatch_intent(subject_name, intent_name, ...args)
                if arg_count < 2 {
                    return Err(compile_codes::runtime_error(
                        "dispatch_intent() requires at least 2 arguments".to_string(),
                    ));
                }
                let mut args: Vec<Value16> = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let subject_name = self.value_to_string(&args[0]);
                let intent_name = self.value_to_string(&args[1]);
                let intent_args = if args.len() > 2 {
                    args[2..].to_vec()
                } else {
                    vec![]
                };
                let mut result = hudhudscript_bytecode::ObjMap::default();
                result.insert("subject".to_string(), Value16::string(subject_name.clone()));
                result.insert("intent".to_string(), Value16::string(intent_name.clone()));
                result.insert("args".to_string(), Value16::array(intent_args.clone()));

                // SOP: try to dispatch via subject instances and action registry
                let chunk_name = format!("intent::{}.{}", subject_name, intent_name);
                if let Some(chunk) = bytecode.get_function(&chunk_name) {
                    use crate::vm::call_state::{
                        GovernanceDispatchState, ReturnSink, VmCallRequest, VmContinuation,
                    };
                    let request = Box::new(VmCallRequest {
                        chunk,
                        func_sym: hudhudscript_bytecode::SymId(
                            hudhudscript_bytecode::interner::intern(&chunk_name).0,
                        ),
                        args: intent_args,
                        captures: rustc_hash::FxHashMap::default(),
                        dst: call_site.dst,
                        origin_ip: call_site.origin_ip,
                        receiver_context: None,
                        return_sink: ReturnSink::Discard,
                        swallow_error: false,
                    });
                    self.schedule_vm_call_with_continuation(
                        VmContinuation::GovernanceDispatch(GovernanceDispatchState {
                            dst: call_site.dst,
                            response: result,
                        }),
                        request,
                    )?;
                    return Ok(true);
                } else {
                    result.insert("dispatched".to_string(), Value16::bool_(false));
                    result.insert(
                        "error".to_string(),
                        Value16::string(format!(
                            "No handler for intent '{}' on subject '{}'",
                            intent_name, subject_name
                        )),
                    );
                }
                self.registers[255] = Value16::object(result);

                Ok(true)
            }
            "get_relation" => {
                // get_relation(subject_a, subject_b) — works with names or subject instances
                if arg_count != 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "get_relation() expects 2 arguments, got {}",
                        arg_count
                    )));
                }
                let mut args: Vec<Value16> = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let a_val = &args[0];
                let b_val = &args[1];
                let a_name = self.resolve_subject_name(&a_val);
                let b_name = self.resolve_subject_name(&b_val);
                let key = format!("{}_{}", a_name, b_name);
                let relation = self
                    .relations
                    .get(&key)
                    .cloned()
                    .or_else(|| {
                        let rev_key = format!("{}_{}", b_name, a_name);
                        self.relations.get(&rev_key).cloned()
                    })
                    .unwrap_or(Value16::null());
                self.registers[255] = relation;
                Ok(true)
            }
            "update_relation" => {
                // update_relation(subject_a, subject_b, field, value)
                if arg_count != 4 {
                    return Err(compile_codes::runtime_error(format!(
                        "update_relation() expects 4 arguments, got {}",
                        arg_count
                    )));
                }
                let a = self.registers[first_arg as usize];
                let b = self.registers[(first_arg + 1) as usize];
                let field = self.registers[(first_arg + 2) as usize];
                let value = self.registers[(first_arg + 3) as usize];
                let key = format!("{}_{}", self.value_to_string(&a), self.value_to_string(&b));
                let field_name = self.value_to_string(&field);
                let relation = self
                    .relations
                    .entry(key)
                    .or_insert_with(|| Value16::object(hudhudscript_bytecode::ObjMap::default()));
                let mut new_obj = if let Some(obj) = relation.as_object() {
                    obj.clone()
                } else {
                    hudhudscript_bytecode::ObjMap::default()
                };
                new_obj.insert(field_name, value);
                *relation = Value16::object(new_obj);
                self.registers[255] = Value16::bool_(true);
                Ok(true)
            }
            "enforce_relation" => {
                // enforce_relation(subject_a, subject_b, constraint)
                if arg_count != 3 {
                    return Err(compile_codes::runtime_error(format!(
                        "enforce_relation() expects 3 arguments, got {}",
                        arg_count
                    )));
                }
                let a = self.registers[first_arg as usize];
                let b = self.registers[(first_arg + 1) as usize];
                let constraint = self.registers[(first_arg + 2) as usize];
                let key = format!("{}_{}", self.value_to_string(&a), self.value_to_string(&b));
                let valid = if let Some(rel_v) = self.relations.get(&key) {
                    if let Some(rel) = rel_v.as_object() {
                        if let Some(field) = constraint.as_string() {
                            rel.contains_key(&field)
                        } else {
                            constraint.is_truthy()
                        }
                    } else {
                        constraint.is_truthy()
                    }
                } else {
                    false
                };
                self.registers[255] = Value16::bool_(valid);
                Ok(true)
            }
            "invoke_protocol_hook" => {
                // invoke_protocol_hook(protocol_name, hook_name, ...args)
                if arg_count < 2 {
                    return Err(compile_codes::runtime_error(
                        "invoke_protocol_hook() requires at least 2 arguments".to_string(),
                    ));
                }
                let mut args: Vec<Value16> = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let protocol_name = self.value_to_string(&args[0]);
                let hook_name = self.value_to_string(&args[1]);
                // Look up protocol declaration
                let decl_key = format!("protocol:{}", protocol_name);
                let protocol = self.declarations.get(&decl_key).cloned();
                let mut result = hudhudscript_bytecode::ObjMap::default();
                result.insert("protocol".to_string(), Value16::string(protocol_name));
                result.insert("hook".to_string(), Value16::string(hook_name));
                result.insert("found".to_string(), Value16::bool_(protocol.is_some()));
                if let Some(p) = protocol {
                    result.insert("declaration".to_string(), p);
                }
                self.registers[255] = Value16::object(result);

                Ok(true)
            }
            // Governance builtins (backed by hudhudscript-governance crate)
            "register_constitution" => {
                if arg_count != 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "register_constitution() expects 2 arguments, got {}",
                        arg_count
                    )));
                }
                let name_val = self.registers[first_arg as usize];
                let constitution_val = self.registers[first_arg as usize + 1];
                let name = self.value_to_string(&name_val);
                let typed = Self::value_to_constitution(&name, &constitution_val);
                self.constitutions.insert(name, typed);
                self.registers[255] = Value16::bool_(true);
                Ok(true)
            }
            "activate_constitution" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "activate_constitution() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let name_val = self.registers[first_arg as usize];
                let name = self.value_to_string(&name_val);
                if self.constitutions.contains_key(&name) {
                    self.active_constitution = Some(name);
                    self.registers[255] = Value16::bool_(true);
                } else {
                    self.registers[255] = Value16::bool_(false);
                }
                Ok(true)
            }
            "deactivate_constitution" => {
                self.active_constitution = None;
                // Consume args
                for _ in 0..arg_count {
                    self.registers[255];
                }
                self.registers[255] = Value16::bool_(true);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// SOP: resolve a value to a subject name (for relation lookups).
    /// Handles both string names and subject_instance objects.
    fn resolve_subject_name(&self, val: &Value16) -> String {
        if let Some(inner) = val.as_object() {
            if let Some(name) = inner.get("__template").and_then(|v| v.as_string()) {
                return name;
            }
            if let Some(name) = inner.get("name").and_then(|v| v.as_string()) {
                return name;
            }
        }
        self.value_to_string(val)
    }

    /// SOP: check if a subject instance has a role (for ability gating).
    pub(crate) fn subject_has_role(&self, instance_id: &str, role: &str) -> bool {
        if let Some(inst) = self.subject_instances.get(instance_id) {
            if let Some(tmpl) = self.subject_templates.get(&inst.template_name) {
                return tmpl.roles.iter().any(|r| r == role);
            }
        }
        false
    }

    /// SOP: get relation data between two subject instances.
    pub(crate) fn get_subject_relation(
        &self,
        a_instance_id: &str,
        b_instance_id: &str,
    ) -> Option<Value16> {
        let a = self.subject_instances.get(a_instance_id)?;
        let b = self.subject_instances.get(b_instance_id)?;
        let key = format!("{}_{}", a.template_name, b.template_name);
        self.relations.get(&key).cloned()
    }

    /// OOP0002: check if `child` class is a subclass of `parent` class via parent chain walk.
    pub(crate) fn is_subclass_of(&self, child: Option<&str>, parent: &str) -> bool {
        let mut current = child;
        while let Some(c) = current {
            if c == parent {
                return true;
            }
            if let Some((parent_name, _)) = self.classes.get(c) {
                current = parent_name.as_deref();
            } else {
                break;
            }
        }
        false
    }
}
