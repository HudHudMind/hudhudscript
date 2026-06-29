#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_class_ops(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::ClassDecl(payload_idx) => {
                // CROSS-2a: payload lives in `bytecode.class_decl_payloads`.
                let payload = &bytecode.class_decl_payloads[*payload_idx as usize];
                let name = bytecode.resolve_symbol(payload.name.0);
                let parent: Option<String> = payload
                    .parent
                    .as_ref()
                    .map(|s| bytecode.resolve_symbol(s.0));
                let methods: Vec<String> = payload
                    .methods
                    .iter()
                    .map(|s| bytecode.resolve_symbol(s.0))
                    .collect();
                let method_access: std::collections::HashMap<String, u8> = methods
                    .iter()
                    .enumerate()
                    .map(|(i, m)| {
                        (
                            m.clone(),
                            payload.method_access.get(i).copied().unwrap_or(0),
                        )
                    })
                    .collect();
                self.classes
                    .insert(name.clone(), (parent.clone(), methods.clone()));
                let mut method_map = hudhudscript_bytecode::ObjMap::default();
                for method in &methods {
                    let chunk_name = format!("{}::{}", name, method);
                    method_map.insert(method.clone(), Value16::string(chunk_name));
                }
                let mut vtable = hudhudscript_bytecode::ObjMap::default();
                if let Some(ref p) = parent {
                    if let Some(parent_v) = self.get_var_cloned(p) {
                        if let Some(parent_cls) = parent_v.as_class_data() {
                            vtable = parent_cls.vtable.clone();
                        }
                    }
                }
                for (k, v) in &method_map {
                    vtable.insert(k.clone(), v.clone());
                }
                let method_map_v: hudhudscript_bytecode::ObjMap =
                    method_map.into_iter().map(|(k, v)| (k, v)).collect();
                let vtable_v: hudhudscript_bytecode::ObjMap =
                    vtable.into_iter().map(|(k, v)| (k, v)).collect();
                let class_val = Value16::class(ClassData {
                    name: name.clone(),
                    methods: method_map_v,
                    fields: hudhudscript_bytecode::ObjMap::default(),
                    parent: parent.as_ref().map(|p| Value16::string(p.clone())),
                    vtable: vtable_v,
                    method_access,
                    is_abstract: payload.is_abstract,
                });
                self.set_var(&name, class_val)?;
            }
            // Issue #982: SOP trait/protocol enforcement
            Instruction::TraitCheck(payload_idx) => {
                // CROSS-2a: payload lives in `bytecode.trait_check_payloads`.
                let payload = &bytecode.trait_check_payloads[*payload_idx as usize];
                let class_name = bytecode.resolve_symbol(payload.class_name.0);
                let trait_name = bytecode.resolve_symbol(payload.trait_name.0);
                let required_methods: Vec<String> = payload
                    .required_methods
                    .iter()
                    .map(|s| bytecode.resolve_symbol(s.0))
                    .collect();
                let class_methods: Vec<String> = payload
                    .class_methods
                    .iter()
                    .map(|s| bytecode.resolve_symbol(s.0))
                    .collect();
                if let Err(missing) = hudhud_sop::sop_ops::check_trait_implementation(
                    &class_name,
                    &trait_name,
                    &required_methods,
                    &class_methods,
                ) {
                    return Err(compile_codes::runtime_error(
                        hudhud_sop::sop_ops::trait_not_implemented_error(
                            &class_name,
                            &trait_name,
                            &missing,
                        ),
                    ));
                }
            }
            Instruction::NewInstance {
                payload_idx,
                first_arg,
                arg_count,
            } => {
                // CROSS-2c: resolve the call payload from the side table.
                let payload = bytecode.get_call_payload(*payload_idx as u32);
                let class_name_sym = payload.sym;
                let class_name = bytecode.resolve_symbol(class_name_sym.0);

                // OOP0004: reject instantiation of abstract classes
                if let Some(class_val) = self.get_var_cloned(&class_name) {
                    if let Some(class_data) = class_val.as_class_data() {
                        if class_data.is_abstract {
                            return Err(compile_codes::runtime_error(format!(
                                "Cannot instantiate abstract class '{}'",
                                class_name
                            )));
                        }
                    }
                }

                let n = *arg_count as usize;
                let first = *first_arg as usize;
                let args: Vec<Value16> = (0..n).map(|i| self.registers[first + i]).collect();
                let mut instance = hudhudscript_bytecode::ObjMap::default();
                instance.insert("__type".to_string(), Value16::string(class_name.clone()));

                // Copy inherited methods from parent chain
                self.copy_parent_methods(&class_name, &mut instance);

                // Copy class methods
                if let Some((_parent, methods)) = self.classes.get(&class_name).cloned() {
                    for method in &methods {
                        let chunk_name = format!("{}::{}", class_name, method);
                        instance.insert(method.clone(), Value16::string(chunk_name));
                    }
                }

                // Retrieve the class Value for the Instance
                let class_val = self.get_var_cloned(&class_name).unwrap_or(Value16::null());

                // Issue #1016: Error chaining — set message, cause, stack, name
                // for Error and Error subclasses using shared logic (Kural 7).
                let is_error = class_name == "Error" || self.class_extends_error(&class_name);
                if is_error {
                    let error_fields = hudhudscript_bytecode::shared_value::construct_error_fields(
                        &class_name,
                        &args,
                    );
                    instance.extend(error_fields);
                }

                // Run constructor if available
                let ctor_name = format!("{}::constructor", class_name);
                if let Some(chunk) = bytecode.get_function(&ctor_name) {
                    self.set_var("this", Value16::object(instance.clone()))?;
                    self.set_var("__current_class", Value16::string(class_name.clone()))?;
                    self.class_context_stack.push(class_name.clone());
                    let _result =
                        self.call_chunk(&chunk, &chunk.params, &args, bytecode, &ctor_name)?;
                    self.class_context_stack.pop();
                    instance = self.collect_this_fields(instance);
                }
                let inst_data = InstanceData {
                    class_name,
                    fields: instance.into_iter().map(|(k, v)| (k, v)).collect(),
                    class: class_val,
                };
                self.registers[255] = Value16::instance(inst_data);
            }

            // ── Property access (Issue #343) ────────────────────────
            // Issue #1021: Move values out of owned object instead of cloning
            // P2-9: Missing-property error parity — body extracted into
            // `get_property_op` to keep the main match frame small.
            // GetProperty: register-based property access with inline cache
            Instruction::GetProperty { dst, obj, prop_sym } => {
                let obj_val = self.registers[*obj as usize];
                let result = self.get_property_ic(
                    obj_val,
                    &hudhudscript_bytecode::SymId(*prop_sym as u32),
                    ctx.ip,
                )?;
                self.registers[*dst as usize] = result;
            }

            Instruction::SetProperty {
                dst,
                obj,
                val,
                prop_sym,
            } => {
                let field = bytecode.resolve_symbol(*prop_sym as u32);
                let prop_sym_id = hudhudscript_bytecode::SymId(*prop_sym as u32);
                let value = self.registers[*val as usize];
                let obj_val = self.registers[*obj as usize];
                let ip = ctx.ip;
                let result = if let Some(inst) = obj_val.as_instance_data() {
                    let mut new_fields = inst.fields.clone();
                    self.set_property_ic(&mut new_fields, &prop_sym_id, value, ip);
                    Value16::instance(InstanceData {
                        class_name: inst.class_name.clone(),
                        fields: new_fields,
                        class: inst.class,
                    })
                } else if let Some(map) = obj_val.as_object() {
                    // SOP: if subject_instance, also update live instance state
                    if map.get("__type").and_then(|v| v.as_string()).as_deref()
                        == Some("subject_instance")
                    {
                        if let Some(id) = map.get("__instance_id").and_then(|v| v.as_string()) {
                            if let Some(inst) = self.subject_instances.get_mut(&id) {
                                // SOP0009: field correspondence check
                                let template = inst.template_name.clone();
                                let compose_key = format!("{}::state::{}", template, field);
                                if let Some(corr) =
                                    self.field_correspondences.get(&compose_key).copied()
                                {
                                    if corr == crate::vm::sop_types::FieldCorrespondence::Separate {
                                        // Default: only write to the relevant location
                                    }
                                }
                                if inst.state.contains_key(&field) {
                                    inst.state.insert(field.clone(), value);
                                } else {
                                    // SOP0006: view state write
                                    let mut written = false;
                                    for (_view_name, view_state) in inst.views.iter_mut() {
                                        if view_state.contains_key(&field) {
                                            view_state.insert(field.clone(), value);
                                            written = true;
                                            break;
                                        }
                                    }
                                    if !written {
                                        inst.state.insert(field.clone(), value);
                                    }
                                }
                            } else {
                                // SOP0005: despawned subject write attempt
                                return Err(compile_codes::runtime_error(format!(
                                    "Cannot set property '{}' on despawned subject '{}'",
                                    field, id
                                )));
                            }
                        }
                    }
                    let mut new_map = map.clone();
                    self.set_property_ic(&mut new_map, &prop_sym_id, value, ip);
                    Value16::object(new_map)
                } else {
                    return Err(compile_codes::runtime_error(format!(
                        "Cannot set property '{}' on non-object",
                        field
                    )));
                };
                self.registers[*dst as usize] = result;
            }
            Instruction::PropertySubAssign { obj, prop_sym, src } => {
                let field = bytecode.resolve_symbol(*prop_sym as u32);
                let sub = self.registers[*src as usize];
                let obj_val = self.registers[*obj as usize];
                let prop_val = if let Some(inst) = obj_val.as_instance_data() {
                    inst.fields.get(&field).cloned().unwrap_or(Value16::int(0))
                } else if let Some(map) = obj_val.as_object() {
                    map.get(&field).cloned().unwrap_or(Value16::int(0))
                } else {
                    return Err(compile_codes::runtime_error(format!("PropertySubAssign: '{}' on non-object", field)));
                };
                let result = crate::vm::bigint_arith::int_sub(prop_val, sub).map_err(|code| {
                    compile_codes::runtime_error(code.to_string())
                })?;
                let new_obj = if let Some(inst) = obj_val.as_instance_data() {
                    let mut f = inst.fields.clone(); f.insert(field.clone(), result);
                    Value16::instance(InstanceData { class_name: inst.class_name.clone(), fields: f, class: inst.class })
                } else if let Some(mut map) = obj_val.as_object().cloned() {
                    map.insert(field.clone(), result); Value16::object(map)
                } else {
                    return Err(compile_codes::runtime_error("PropertySubAssign: not an object"));
                };
                self.registers[*obj as usize] = new_obj;
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
