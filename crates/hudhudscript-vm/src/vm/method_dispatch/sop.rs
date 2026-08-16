use crate::vm::call_state::{
    deferred_method_in_immediate_context, DeferredCallSite, MethodDispatchOutcome, SopCallStep,
    SopResultPolicy,
};
use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{Bytecode, FunctionChunk, SymId, Value16};
use std::sync::Arc;

fn ability_step(
    bytecode: &Bytecode,
    ability: &str,
    policy: SopResultPolicy,
) -> Option<SopCallStep> {
    let chunk = bytecode.get_function(ability)?;
    Some(SopCallStep {
        chunk,
        func_sym: SymId(hudhudscript_bytecode::interner::intern(ability).0),
        result_policy: policy,
        swallow_error: false,
    })
}

impl VM {
    pub(crate) fn dispatch_sop_method(
        &mut self,
        receiver: &Value16,
        method: &str,
        method_sym: SymId,
        args: &[Value16],
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<Option<MethodDispatchOutcome>> {
        let Some(inner) = receiver.as_object() else {
            return Ok(None);
        };
        if inner
            .get(&hudhudscript_bytecode::well_known::wk().type_)
            .and_then(|value| value.as_str())
            != Some("subject_instance")
        {
            return Ok(None);
        }

        let template = inner
            .get(&hudhudscript_bytecode::well_known::wk().template)
            .and_then(|value| value.as_string())
            .unwrap_or_default();
        let mut chunk_with_name: Option<(Arc<FunctionChunk>, String)> = None;

        for index in 0..2 {
            if let (Some((cached_template, cached_method, chunk, name)), age) =
                &mut self.ability_cache[index]
            {
                if cached_template == &template && *cached_method == method_sym {
                    *age = age.saturating_add(1);
                    chunk_with_name = Some((Arc::clone(chunk), name.clone()));
                    break;
                }
            }
        }

        if chunk_with_name.is_none() {
            let scoped_name = if template.is_empty() {
                String::new()
            } else {
                format!("ability::{}::{}", template, method)
            };
            let unscoped_name = format!("ability::{}", method);
            let chunk = if scoped_name.is_empty() {
                None
            } else {
                bytecode.get_function(&scoped_name)
            }
            .or_else(|| bytecode.get_function(&unscoped_name));
            let chunk_name = if !scoped_name.is_empty() && bytecode.has_function(&scoped_name) {
                scoped_name
            } else {
                unscoped_name
            };

            if let Some(cached_chunk) = &chunk {
                let replace = usize::from(self.ability_cache[0].1 > self.ability_cache[1].1);
                self.ability_cache[replace] = (
                    Some((
                        template.clone(),
                        method_sym,
                        Arc::clone(cached_chunk),
                        chunk_name.clone(),
                    )),
                    0,
                );
            }
            chunk_with_name = chunk.map(|chunk| (chunk, chunk_name));
        }

        let mut call_args = Vec::with_capacity(args.len() + 1);
        call_args.push(*receiver);
        call_args.extend_from_slice(args);

        if let Some((chunk, chunk_name)) = chunk_with_name {
            let instance_id = inner
                .get(&hudhudscript_bytecode::well_known::wk().instance_id)
                .and_then(|value| value.as_string())
                .unwrap_or_default();
            let rules = self
                .composition_rules
                .get(&format!("{}::{}", template, method))
                .cloned();
            let mut before = Vec::new();
            let mut after = Vec::new();
            let mut combine = Vec::new();
            let mut override_subject = None;

            if let Some(rules) = rules {
                for rule in rules {
                    match rule.mode {
                        crate::vm::sop_types::CompositionMode::Combine(subjects) => {
                            combine.extend(subjects);
                        }
                        crate::vm::sop_types::CompositionMode::Override(subject) => {
                            if override_subject.is_none() {
                                override_subject = Some(subject);
                            }
                        }
                        crate::vm::sop_types::CompositionMode::Before(subject) => {
                            before.push(subject);
                        }
                        crate::vm::sop_types::CompositionMode::After(subject) => {
                            after.push(subject);
                        }
                    }
                }
            } else if let Some(instance) = self.subject_instances.get(&instance_id) {
                combine.extend(instance.views.keys().cloned());
            }

            // Phase order is contractual: before always runs first, override
            // replaces base (and skips combine), after and effect run last.
            let mut steps: Vec<SopCallStep> = Vec::new();
            for subject in &before {
                let ability = format!("ability::{}::{}", subject, method);
                if let Some(step) = ability_step(bytecode, &ability, SopResultPolicy::Ignore) {
                    steps.push(step);
                }
            }
            let mut replaced = false;
            if let Some(subject) = override_subject {
                let ability = format!("ability::{}::{}", subject, method);
                if let Some(step) = ability_step(bytecode, &ability, SopResultPolicy::Replace) {
                    steps.push(step);
                    replaced = true;
                }
            }
            if !replaced {
                steps.push(SopCallStep {
                    chunk,
                    func_sym: SymId(hudhudscript_bytecode::interner::intern(&chunk_name).0),
                    result_policy: SopResultPolicy::Replace,
                    swallow_error: false,
                });
                for subject in &combine {
                    let ability = format!("ability::{}::{}", subject, method);
                    if let Some(step) = ability_step(bytecode, &ability, SopResultPolicy::Replace) {
                        steps.push(step);
                    }
                }
            }
            for subject in &after {
                let ability = format!("ability::{}::{}", subject, method);
                if let Some(step) = ability_step(bytecode, &ability, SopResultPolicy::Ignore) {
                    steps.push(step);
                }
            }
            if let Some(effect_name) = self.effects.get(method).cloned() {
                if let Some(mut step) =
                    ability_step(bytecode, &effect_name, SopResultPolicy::Ignore)
                {
                    step.swallow_error = true;
                    steps.push(step);
                }
            }

            let outcome = self.start_sop_ability_sequence(
                steps,
                call_args,
                call_site.dst,
                call_site.origin_ip,
            )?;
            return Ok(Some(outcome));
        }

        if let Some(instance) = self.subject_instances.get(
            &inner
                .get(&hudhudscript_bytecode::well_known::wk().instance_id)
                .and_then(|value| value.as_string())
                .unwrap_or_default(),
        ) {
            let view_names: Vec<String> = instance.views.keys().cloned().collect();
            for view_name in view_names {
                let ability = format!("ability::{}::{}", view_name, method);
                if let Some(step) = ability_step(bytecode, &ability, SopResultPolicy::Replace) {
                    let outcome = self.start_sop_ability_sequence(
                        vec![step],
                        call_args,
                        call_site.dst,
                        call_site.origin_ip,
                    )?;
                    return Ok(Some(outcome));
                }
            }
        }

        Ok(None)
    }
}
