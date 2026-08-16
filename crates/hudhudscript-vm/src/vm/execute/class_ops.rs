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
                let method_access: std::collections::HashMap<hudhudscript_bytecode::SymId, u8> =
                    methods
                        .iter()
                        .enumerate()
                        .map(|(i, m)| {
                            (
                                hudhudscript_bytecode::SymId(
                                    hudhudscript_bytecode::interner::intern(m).0,
                                ),
                                payload.method_access.get(i).copied().unwrap_or(0),
                            )
                        })
                        .collect();
                self.classes
                    .insert(name.clone(), (parent.clone(), methods.clone()));
                let mut method_map = hudhudscript_bytecode::ObjMap::default();
                for method in &methods {
                    let chunk_name = format!("{}::{}", name, method);
                    let chunk_sym = hudhudscript_bytecode::interner::intern(&chunk_name);
                    let idx = bytecode.get_function_idx(&chunk_name).ok_or_else(|| {
                        Self::runtime_error_with_pos(
                            &format!(
                                "Compiler invariant: method chunk '{}' not registered",
                                chunk_name
                            ),
                            bytecode,
                            ip,
                        )
                    })?;
                    let packed = ((idx as i64) << 32) | (chunk_sym.0 as i64);
                    method_map.insert(method.clone(), Value16::int(packed));
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

                // Run constructor if available — G06A: deferred frame with a
                // constructor continuation; no nested driver, no direct `this`
                // global setup (the receiver context owns `this`).
                let ctor_name = format!("{}::constructor", class_name);
                if let Some(chunk) = bytecode.get_function(&ctor_name) {
                    let ctor_sym = hudhudscript_bytecode::SymId(
                        hudhudscript_bytecode::interner::intern(&ctor_name).0,
                    );
                    self.schedule_constructor_call(
                        chunk,
                        ctor_sym,
                        args,
                        class_name,
                        class_val,
                        Value16::object(instance),
                        ip,
                    )?;
                    return Ok(StepAction::DeferredCall);
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
                #[cfg(feature = "telemetry")]
                {
                    self.telemetry.site_property_count += 1;
                }
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
                let prop_sym_id = hudhudscript_bytecode::SymId(*prop_sym as u32);
                let value = self.registers[*val as usize];
                let ip = ctx.ip;
                // REFSEM: mutate in-place via mutable register access (no clone)
                // Handle SOP subject_instance state update BEFORE mutable borrow
                let obj_val = self.registers[*obj as usize];
                if let Some(map) = obj_val.as_object() {
                    if map
                        .get(&hudhudscript_bytecode::well_known::wk().type_)
                        .and_then(|v| v.as_string())
                        .as_deref()
                        == Some("subject_instance")
                    {
                        let field = bytecode.resolve_symbol(*prop_sym as u32);
                        if let Some(id) = map
                            .get(&hudhudscript_bytecode::well_known::wk().instance_id)
                            .and_then(|v| v.as_string())
                        {
                            if let Some(inst) = self.subject_instances.get_mut(&id) {
                                // (... SOP field correspondence logic unchanged ...)
                                let template = inst.template_name.clone();
                                let compose_key = format!("{}::state::{}", template, field);
                                let corr = self
                                    .field_correspondences
                                    .get(&compose_key)
                                    .copied()
                                    .unwrap_or(crate::vm::sop_types::FieldCorrespondence::Separate);
                                let active_view = map
                                    .get(&hudhudscript_bytecode::well_known::wk().view_name)
                                    .and_then(|v| v.as_string())
                                    .map(|s| s.to_string());
                                match corr {
                                    crate::vm::sop_types::FieldCorrespondence::Correspond => {
                                        if inst.state.contains_key(&field) {
                                            inst.state.insert(field.clone(), value);
                                        }
                                        for (_, view_state) in inst.views.iter_mut() {
                                            if view_state.contains_key(&field) {
                                                view_state.insert(field.clone(), value);
                                            }
                                        }
                                    }
                                    crate::vm::sop_types::FieldCorrespondence::Separate => {
                                        if let Some(ref view_name) = active_view {
                                            if let Some(view_state) = inst.views.get_mut(view_name)
                                            {
                                                view_state.insert(field.clone(), value);
                                            } else if inst.state.contains_key(&field) {
                                                inst.state.insert(field.clone(), value);
                                            } else {
                                                inst.state.insert(field.clone(), value);
                                            }
                                        } else if inst.state.contains_key(&field) {
                                            inst.state.insert(field.clone(), value);
                                        } else {
                                            let mut view_names: Vec<String> =
                                                inst.views.keys().cloned().collect();
                                            view_names.sort();
                                            let mut written = false;
                                            for view_name in &view_names {
                                                if let Some(view_state) =
                                                    inst.views.get_mut(view_name)
                                                {
                                                    if view_state.contains_key(&field) {
                                                        view_state.insert(field.clone(), value);
                                                        written = true;
                                                        break;
                                                    }
                                                }
                                            }
                                            if !written {
                                                inst.state.insert(field.clone(), value);
                                            }
                                        }
                                    }
                                }
                            } else {
                                return Err(compile_codes::runtime_error(format!(
                                    "Cannot set property '{}' on despawned subject '{}'",
                                    field, id
                                )));
                            }
                        }
                    }
                }
                // Now mutate the register in-place
                let obj_mut = &mut self.registers[*obj as usize];
                if let Some(inst) = obj_mut.as_instance_mut() {
                    inst.fields.insert(prop_sym_id, value);
                } else if obj_mut.as_object_mut().is_some() {
                    obj_mut.as_object_mut().unwrap().insert(prop_sym_id, value);
                } else {
                    return Err(compile_codes::runtime_error(
                        "Cannot set property on non-object".to_string(),
                    ));
                }
                self.registers[*dst as usize] = *obj_mut;
            }
            Instruction::PropertySubAssign { obj, prop_sym, src } => {
                let field = bytecode.resolve_symbol(*prop_sym as u32);
                let sub = self.registers[*src as usize];
                let obj_val = self.registers[*obj as usize];
                // B1: SOP subject instance — read/write via subject_instances, not ObjMap
                if let Some(map) = obj_val.as_object() {
                    if map
                        .get(&hudhudscript_bytecode::well_known::wk().type_)
                        .and_then(|v| v.as_string())
                        .as_deref()
                        == Some("subject_instance")
                    {
                        if let Some(id) = map
                            .get(&hudhudscript_bytecode::well_known::wk().instance_id)
                            .and_then(|v| v.as_string())
                        {
                            if let Some(inst) = self.subject_instances.get_mut(&id) {
                                let current =
                                    inst.state.get(&field).copied().unwrap_or(Value16::int(0));
                                let result = crate::vm::bigint_arith::int_sub(current, sub)
                                    .map_err(|code| {
                                        compile_codes::runtime_error(code.to_string())
                                    })?;
                                inst.state.insert(field.clone(), result);
                                // Also update proxy ObjMap so GetProperty sees the new value
                                let obj_mut = &mut self.registers[*obj as usize];
                                if let Some(obj_map) = obj_mut.as_object_mut() {
                                    obj_map.insert(field.clone(), result);
                                }
                                return Ok(StepAction::Advance);
                            }
                        }
                    }
                }
                let prop_val = if let Some(inst) = obj_val.as_instance_data() {
                    inst.fields.get(&field).cloned().unwrap_or(Value16::int(0))
                } else if let Some(map) = obj_val.as_object() {
                    map.get(&field).cloned().unwrap_or(Value16::int(0))
                } else {
                    return Err(compile_codes::runtime_error(format!(
                        "PropertySubAssign: '{}' on non-object",
                        field
                    )));
                };
                let result = crate::vm::bigint_arith::int_sub(prop_val, sub)
                    .map_err(|code| compile_codes::runtime_error(code.to_string()))?;
                #[cfg(feature = "telemetry")]
                if !prop_val.is_bigint() && !sub.is_bigint() && result.is_bigint() {
                    self.telemetry.bigint_promotion += 1;
                    self.telemetry.bigint_alloc += 1;
                }
                let new_obj = if let Some(inst) = obj_val.as_instance_data() {
                    let mut f = inst.fields.clone();
                    f.insert(field.clone(), result);
                    Value16::instance(InstanceData {
                        class_name: inst.class_name.clone(),
                        fields: f,
                        class: inst.class,
                    })
                } else if let Some(mut map) = obj_val.as_object().cloned() {
                    map.insert(field.clone(), result);
                    Value16::object(map)
                } else {
                    return Err(compile_codes::runtime_error(
                        "PropertySubAssign: not an object",
                    ));
                };
                self.registers[*obj as usize] = new_obj;
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
