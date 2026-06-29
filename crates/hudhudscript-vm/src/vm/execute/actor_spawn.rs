#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_actor_spawn(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::Spawn {
                payload_idx,
                first_arg,
                arg_count,
            } => {
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

                let mut instance = hudhudscript_bytecode::ObjMap::default();
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
                    self.subject_instances
                        .insert(instance_id.clone(), subj_inst);

                    // SOP0006: auto-bind views — add all view subjects whose `of` targets this subject
                    let views_for_this: Vec<_> = self
                        .subject_templates
                        .iter()
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

                    instance.insert(
                        "__type".to_string(),
                        Value16::string("subject_instance".to_string()),
                    );
                    instance.insert("__instance_id".to_string(), Value16::string(instance_id));
                    instance.insert(
                        "__template".to_string(),
                        Value16::string(subject_name.clone()),
                    );
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
                    if let Some(instance_id) =
                        inner.get("__instance_id").and_then(|v| v.as_string())
                    {
                        let instance_id = instance_id.to_string();
                        self.subject_instances.remove(&instance_id);
                        self.registers[*reg as usize] = Value16::null();
                        self.registers[255] = Value16::bool_(true);
                    } else {
                        return Err(compile_codes::runtime_error(
                            "despawn: value is not a subject instance".to_string(),
                        ));
                    }
                } else {
                    return Err(compile_codes::runtime_error(
                        "despawn: value is not an object".to_string(),
                    ));
                }
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
