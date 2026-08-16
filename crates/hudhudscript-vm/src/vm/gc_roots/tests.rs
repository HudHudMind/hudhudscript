use super::*;
use crate::vm::{
    machine::CallFrame,
    sop_types::{SubjectInstance, SubjectTemplate},
    types::{PendingFlow, SavedFinally},
};
use hudhudscript_bytecode::{gc, GeneratorState16, SymId, Value16};
use parking_lot::{Mutex, RwLock};
use std::{ptr, sync::Arc};

fn dyn_value(label: &str) -> Value16 {
    Value16::string(format!("gc-root-{}-dynamic-value", label))
}

fn fx_map(pairs: &[(&str, Value16)]) -> FxHashMap<String, Value16> {
    let mut map = FxHashMap::default();
    for (key, value) in pairs {
        map.insert((*key).to_string(), *value);
    }
    map
}

fn dummy_frame_with_pending(value: Value16) -> CallFrame {
    CallFrame {
        chunk_ptr: ptr::null(),
        owned_chunk: None,
        packed: ptr::null(),
        func_sym: SymId(0),
        ip: 0,
        dst: 0,
        reg_base: 0,
        reg_size: 0,
        saved_finally: Some(Box::new(SavedFinally {
            try_frames: vec![],
            finally_frames: vec![],
            pending_flow: Some(PendingFlow::Throw(Box::new(value))),
        })),
        has_captures: false,
        debugger_pushed: false,
        call_depth: 0,
        owned_local_syms: false,
        class_context: false,
        return_sink: crate::vm::call_state::ReturnSink::Discard,
        receiver_context: None,
        swallow_error: false,
    }
}

fn assert_marks_root(label: &str, setup: impl FnOnce(&mut VM, Value16)) {
    let mut vm = VM::new();
    let rooted = dyn_value(label);
    setup(&mut vm, rooted);
    let unreachable = dyn_value("unreachable");

    vm.mark_from_roots();

    assert!(gc::is_marked(rooted), "{} root was not marked", label);
    assert!(
        !gc::is_marked(unreachable),
        "{} unexpectedly marked an unrelated value",
        label
    );
}

macro_rules! root_test {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            assert_marks_root(stringify!($name), $body);
        }
    };
}

root_test!(marks_register_temporary_root, |vm: &mut VM, rooted| {
    vm.registers.advance(16);
    vm.registers[3] = rooted;
});

root_test!(marks_last_return_root, |vm: &mut VM, rooted| {
    vm.last_return = rooted;
});

root_test!(marks_global_root, |vm: &mut VM, rooted| {
    vm.globals
        .insert(hudhudscript_bytecode::interner::intern("g"), rooted);
});

root_test!(marks_scope_cell_root, |vm: &mut VM, rooted| {
    let cells: Box<[Option<Arc<RwLock<Value16>>>]> =
        Box::new([Some(Arc::new(RwLock::new(rooted)))]);
    let sym_ids: Arc<[u32]> = Arc::new([42]);
    vm.scope_cells.push((cells, sym_ids));
});

root_test!(marks_iterator_root, |vm: &mut VM, rooted| {
    vm.iterators.push((vec![rooted], "item".to_string(), 0));
});

root_test!(
    marks_iterator_generator_buffer_root,
    |vm: &mut VM, rooted| {
        let generator = Arc::new(Mutex::new(GeneratorState16::from(vec![rooted])));
        assert_eq!(generator.lock().advance(), Some(rooted));
        vm.iterator_generators.push(Some(generator));
    }
);

root_test!(marks_declaration_root, |vm: &mut VM, rooted| {
    vm.declarations.insert("kind:name".to_string(), rooted);
});

root_test!(marks_relation_root, |vm: &mut VM, rooted| {
    vm.relations.insert("A_B".to_string(), rooted);
});

root_test!(
    marks_subject_template_default_root,
    |vm: &mut VM, rooted| {
        vm.subject_templates.insert(
            "Subject".to_string(),
            SubjectTemplate {
                name: "Subject".to_string(),
                of_subject: None,
                roles: vec![],
                state_defaults: fx_map(&[("field", rooted)]),
                capabilities: vec![],
                intents: vec![],
            },
        );
    }
);

root_test!(marks_subject_instance_state_root, |vm: &mut VM, rooted| {
    vm.subject_instances.insert(
        "subject-1".to_string(),
        SubjectInstance {
            template_name: "Subject".to_string(),
            instance_id: "subject-1".to_string(),
            state: fx_map(&[("field", rooted)]),
            actor_id: "actor-1".to_string(),
            views: FxHashMap::default(),
        },
    );
});

root_test!(marks_subject_instance_view_root, |vm: &mut VM, rooted| {
    let mut views = FxHashMap::default();
    views.insert("main".to_string(), fx_map(&[("field", rooted)]));
    vm.subject_instances.insert(
        "subject-1".to_string(),
        SubjectInstance {
            template_name: "Subject".to_string(),
            instance_id: "subject-1".to_string(),
            state: FxHashMap::default(),
            actor_id: "actor-1".to_string(),
            views,
        },
    );
});

root_test!(marks_toml_config_root, |vm: &mut VM, rooted| {
    vm.toml_config = rooted;
});

root_test!(
    marks_dispatch_provider_receiver_root,
    |vm: &mut VM, rooted| {
        vm.dispatch_provider_receiver = Some(rooted);
    }
);

root_test!(marks_last_instance_mutation_root, |vm: &mut VM, rooted| {
    vm.last_instance_mutation = Some(Box::new(rooted));
});

root_test!(marks_pending_flow_root, |vm: &mut VM, rooted| {
    vm.pending_flow = Some(PendingFlow::Return(Box::new(rooted)));
});

root_test!(marks_saved_finally_pending_root, |vm: &mut VM, rooted| {
    vm.frame_stack.push(dummy_frame_with_pending(rooted));
});

root_test!(marks_tvar_registry_root, |vm: &mut VM, rooted| {
    vm.tvars.create_with_id("gc-root-tvar", rooted);
});

root_test!(marks_current_transaction_root, |vm: &mut VM, rooted| {
    let tvar_id = vm.tvars.create_with_id("gc-root-tvar", Value16::null());
    let tvar = vm.tvars.get(&tvar_id).expect("tvar exists");
    let mut tx = hudhudscript_stm::Transaction::new();
    tx.write(&tvar, rooted);
    vm.current_tx = Some(Box::new(tx));
});

root_test!(marks_cached_promise_root, |vm: &mut VM, rooted| {
    let _id = vm.promise_registry.store_result(Ok(rooted));
});

root_test!(marks_tco_args_root, |vm: &mut VM, rooted| {
    vm.tco_args = Some(vec![rooted]);
});

root_test!(marks_args_scratch_root, |vm: &mut VM, rooted| {
    vm.args_scratch.push(rooted);
});

#[test]
fn marks_actor_mailbox_without_consuming_messages() {
    let mut vm = VM::new();
    let rooted = dyn_value("actor-mailbox");
    let (actor_ref, mailbox) = vm.actors.spawn();
    actor_ref
        .send_with_reply(rooted, "reply-target".to_string())
        .expect("send actor payload");
    vm.actor_mailboxes.insert(actor_ref.id.clone(), mailbox);

    vm.mark_from_roots();

    assert!(gc::is_marked(rooted));

    let snapshot = vm
        .actor_mailboxes
        .get(&actor_ref.id)
        .map(|mailbox| mailbox.peek_nonblocking())
        .expect("mailbox exists");
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].payload, rooted);
    assert_eq!(snapshot[0].reply_to.as_deref(), Some("reply-target"));

    let msg = vm
        .actor_mailboxes
        .get(&actor_ref.id)
        .and_then(|mailbox| mailbox.try_recv())
        .expect("actor mailbox contents preserved");
    assert_eq!(msg.payload, rooted);
    assert_eq!(msg.reply_to.as_deref(), Some("reply-target"));
}

#[test]
fn mark_from_roots_marks_vm_reachable_values() {
    let mut vm = VM::new();

    let register_root = dyn_value("register");
    vm.registers.arena_mut()[7] = register_root;
    let last_return_root = dyn_value("last-return");
    vm.last_return = last_return_root;
    let global_root = dyn_value("global");
    vm.globals
        .insert(hudhudscript_bytecode::interner::intern("g"), global_root);
    let scope_root = dyn_value("scope");
    let cells: Box<[Option<Arc<RwLock<Value16>>>]> =
        Box::new([Some(Arc::new(RwLock::new(scope_root)))]);
    let sym_ids: Arc<[u32]> = Arc::new([42]);
    vm.scope_cells.push((cells, sym_ids));
    let iter_root = dyn_value("iterator");
    vm.iterators.push((vec![iter_root], "item".to_string(), 0));
    let generator_root = dyn_value("iterator-generator");
    let generator = Arc::new(Mutex::new(GeneratorState16::from(vec![generator_root])));
    assert_eq!(generator.lock().advance(), Some(generator_root));
    vm.iterator_generators.push(Some(generator));
    let declaration_root = dyn_value("declaration");
    vm.declarations
        .insert("kind:name".to_string(), declaration_root);
    let relation_root = dyn_value("relation");
    vm.relations.insert("A_B".to_string(), relation_root);
    let template_root = dyn_value("template");
    vm.subject_templates.insert(
        "Subject".to_string(),
        SubjectTemplate {
            name: "Subject".to_string(),
            of_subject: None,
            roles: vec![],
            state_defaults: fx_map(&[("field", template_root)]),
            capabilities: vec![],
            intents: vec![],
        },
    );
    let subject_state_root = dyn_value("subject-state");
    let subject_view_root = dyn_value("subject-view");
    let mut views = FxHashMap::default();
    views.insert("main".to_string(), fx_map(&[("field", subject_view_root)]));
    vm.subject_instances.insert(
        "subject-1".to_string(),
        SubjectInstance {
            template_name: "Subject".to_string(),
            instance_id: "subject-1".to_string(),
            state: fx_map(&[("field", subject_state_root)]),
            actor_id: "actor-1".to_string(),
            views,
        },
    );
    let toml_root = dyn_value("toml");
    vm.toml_config = toml_root;
    let dispatch_root = dyn_value("dispatch-provider");
    vm.dispatch_provider_receiver = Some(dispatch_root);
    let last_mutation_root = dyn_value("last-mutation");
    vm.last_instance_mutation = Some(Box::new(last_mutation_root));
    let pending_root = dyn_value("pending");
    vm.pending_flow = Some(PendingFlow::Return(Box::new(pending_root)));
    let saved_finally_root = dyn_value("saved-finally");
    vm.frame_stack
        .push(dummy_frame_with_pending(saved_finally_root));
    let tvar_root = dyn_value("tvar");
    let tvar_id = vm.tvars.create_with_id("gc-root-tvar", tvar_root);
    let staged_tx_root = dyn_value("current-tx");
    let tvar = vm.tvars.get(&tvar_id).expect("tvar exists");
    let mut tx = hudhudscript_stm::Transaction::new();
    tx.write(&tvar, staged_tx_root);
    vm.current_tx = Some(Box::new(tx));
    let promise_root = dyn_value("promise-cache");
    let _promise_id = vm.promise_registry.store_result(Ok(promise_root));
    let (actor_ref, mailbox) = vm.actors.spawn();
    let actor_root = dyn_value("actor-mailbox");
    actor_ref
        .send_with_reply(actor_root, "reply-target".to_string())
        .expect("send actor payload");
    vm.actor_mailboxes.insert(actor_ref.id.clone(), mailbox);
    let tco_root = dyn_value("tco");
    vm.tco_args = Some(vec![tco_root]);
    let scratch_root = dyn_value("scratch");
    vm.args_scratch.push(scratch_root);
    let unreachable = dyn_value("unreachable");

    vm.mark_from_roots();

    for value in [
        register_root,
        last_return_root,
        global_root,
        scope_root,
        iter_root,
        generator_root,
        declaration_root,
        relation_root,
        template_root,
        subject_state_root,
        subject_view_root,
        toml_root,
        dispatch_root,
        last_mutation_root,
        pending_root,
        saved_finally_root,
        tvar_root,
        staged_tx_root,
        promise_root,
        actor_root,
        tco_root,
        scratch_root,
    ] {
        assert!(gc::is_marked(value));
    }
    assert!(!gc::is_marked(unreachable));
}
