use super::{
    sop_types::{SubjectInstance, SubjectTemplate},
    types::PendingFlow,
    VM,
};
use hudhudscript_bytecode::{
    actor_registry::ActorMailbox, gc, DynamicObject, GeneratorState16, Value16,
};
use rustc_hash::FxHashMap;
use std::sync::Arc;

impl gc::GcRootSource for VM {
    fn mark_roots(&self) {
        self.mark_from_roots();
    }
}

fn trace_values<'a>(
    values: impl IntoIterator<Item = &'a Value16>,
    gray: &mut Vec<*mut DynamicObject>,
) {
    for value in values {
        gc::trace_value(*value, gray);
    }
}

fn trace_scope_cells(
    cells: &[FxHashMap<String, Arc<parking_lot::RwLock<Value16>>>],
    gray: &mut Vec<*mut DynamicObject>,
) {
    for scope in cells {
        for cell in scope.values() {
            gc::trace_value(*cell.read(), gray);
        }
    }
}

fn trace_subject_templates(
    templates: &FxHashMap<String, SubjectTemplate>,
    gray: &mut Vec<*mut DynamicObject>,
) {
    for template in templates.values() {
        trace_values(template.state_defaults.values(), gray);
    }
}

fn trace_subject_instances(
    instances: &FxHashMap<String, SubjectInstance>,
    gray: &mut Vec<*mut DynamicObject>,
) {
    for instance in instances.values() {
        trace_values(instance.state.values(), gray);
        for view in instance.views.values() {
            trace_values(view.values(), gray);
        }
    }
}

fn trace_pending_flow(pending: Option<&PendingFlow>, gray: &mut Vec<*mut DynamicObject>) {
    match pending {
        Some(PendingFlow::Return(value)) | Some(PendingFlow::Throw(value)) => {
            gc::trace_value(*value.as_ref(), gray);
        }
        None => {}
    }
}

fn trace_generator_buffers(
    generators: &[Option<Arc<parking_lot::Mutex<GeneratorState16>>>],
    gray: &mut Vec<*mut DynamicObject>,
) {
    for generator in generators.iter().flatten() {
        let state = generator.lock();
        trace_values(state.pending.iter(), gray);
        trace_values(state.buffered.iter(), gray);
    }
}

fn trace_actor_mailbox(mailbox: &ActorMailbox<Value16>, gray: &mut Vec<*mut DynamicObject>) {
    for message in mailbox.peek_nonblocking() {
        gc::trace_value(message.payload, gray);
    }
}

impl VM {
    pub(crate) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        trace_values(self.registers.arena().iter(), gray);
        gc::trace_value(self.last_return, gray);
        trace_values(self.globals.values(), gray);
        trace_values(self.shared_globals_vec.iter(), gray);
        trace_scope_cells(&self.scope_cells, gray);
        trace_scope_cells(&self.scope_cells_pool, gray);
        for (elements, _, _) in &self.iterators {
            trace_values(elements.iter(), gray);
        }
        trace_generator_buffers(&self.iterator_generators, gray);
        trace_values(self.declarations.values(), gray);
        trace_values(self.relations.values(), gray);
        trace_subject_templates(&self.subject_templates, gray);
        trace_subject_instances(&self.subject_instances, gray);
        gc::trace_value(self.toml_config, gray);
        if let Some(receiver) = self.dispatch_provider_receiver {
            gc::trace_value(receiver, gray);
        }
        if let Some(value) = self.last_instance_mutation.as_deref() {
            gc::trace_value(*value, gray);
        }
        trace_pending_flow(self.pending_flow.as_ref(), gray);
        for frame in &self.frame_stack {
            trace_pending_flow(
                frame
                    .saved_finally
                    .as_deref()
                    .and_then(|saved| saved.pending_flow.as_ref()),
                gray,
            );
        }
        for value in self.tvars.snapshot_values() {
            gc::trace_value(value, gray);
        }
        if let Some(tx) = self.current_tx.as_deref() {
            for value in tx.root_values() {
                gc::trace_value(value, gray);
            }
        }
        for value in self.promise_registry.cached_values() {
            gc::trace_value(value, gray);
        }
        for mailbox in self.actor_mailboxes.values() {
            trace_actor_mailbox(mailbox, gray);
        }
        if let Some(args) = self.tco_args.as_ref() {
            trace_values(args.iter(), gray);
        }
        trace_values(self.args_scratch.iter(), gray);
        // P5.1: Bytecode + FunctionChunk sabit havuzları.
        trace_values(self.gc_constant_roots.iter(), gray);
    }

    pub(crate) fn mark_from_roots(&self) {
        let mut gray = Vec::new();
        self.trace_roots(&mut gray);
        hudhudscript_bytecode::gc_pin::trace_pinned(&mut gray);
        gc::drain_gray(&mut gray);
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod read_only_tests;

#[cfg(test)]
mod runtime_bucket_tests;

#[cfg(test)]
mod collect_tests;
