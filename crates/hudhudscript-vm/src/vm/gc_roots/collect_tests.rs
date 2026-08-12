use super::*;
use crate::vm::{
    machine::CallFrame,
    types::{PendingFlow, SavedFinally},
};
use hudhudscript_bytecode::interner;
use hudhudscript_bytecode::{gc, GeneratorState16, SymId, Value16};
use parking_lot::{Mutex, RwLock};
use rustc_hash::FxHashMap;
use std::{ptr, sync::Arc};

fn dyn_value(label: &str) -> Value16 {
    Value16::string(format!("gc-collect-{}-dynamic-value", label))
}

fn value_string(value: Value16) -> String {
    value.as_string().expect("root value is still readable")
}

fn assert_collect_preserves_root(
    label: &str,
    install: impl FnOnce(&mut VM, Value16),
    read: impl FnOnce(&VM) -> Value16,
) {
    let mut vm = VM::new();
    let rooted = dyn_value(label);
    let expected = value_string(rooted);
    install(&mut vm, rooted);
    let garbage = dyn_value("unreachable");
    let before = gc::heap_object_count();

    gc::collect(&vm);

    assert!(gc::heap_object_count() < before);
    assert_eq!(value_string(read(&vm)), expected);
    let _ = garbage;
}

fn dummy_frame_with_pending(value: Value16) -> CallFrame {
    CallFrame {
        chunk_ptr: ptr::null(),
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
    }
}

#[test]
fn collect_drops_unreachable_object_and_resets_gc_accounting() {
    let vm = VM::new();
    let _garbage = dyn_value("only-garbage");
    let before = gc::heap_object_count();

    gc::collect(&vm);

    assert!(gc::heap_object_count() < before);
    assert_eq!(gc::bytes_allocated(), 0);
    assert!(gc::next_gc_threshold() >= gc::heap_object_count());
}

#[test]
fn collect_preserves_active_frame_temporary_root() {
    assert_collect_preserves_root(
        "register-temporary",
        |vm, rooted| {
            vm.registers.advance(16);
            vm.registers[3] = rooted;
        },
        |vm| vm.registers[3],
    );
}

#[test]
fn collect_preserves_last_return_root() {
    assert_collect_preserves_root(
        "last-return",
        |vm, rooted| vm.last_return = rooted,
        |vm| vm.last_return,
    );
}

#[test]
fn collect_preserves_global_root() {
    assert_collect_preserves_root(
        "global",
        |vm, rooted| {
            vm.globals.insert(interner::intern("g"), rooted);
        },
        |vm| {
            *vm.globals
                .get(&interner::intern("g"))
                .expect("global root exists")
        },
    );
}

#[test]
fn collect_preserves_scope_cell_root() {
    assert_collect_preserves_root(
        "scope-cell",
        |vm, rooted| {
            let cells: Box<[Option<Arc<RwLock<Value16>>>]> =
                Box::new([Some(Arc::new(RwLock::new(rooted)))]);
            let sym_ids: Arc<[u32]> = Arc::new([42]);
            vm.scope_cells.push((cells, sym_ids));
        },
        |vm| *vm.scope_cells[0].0[0].as_ref().expect("cell exists").read(),
    );
}

#[test]
fn collect_preserves_iterator_root() {
    assert_collect_preserves_root(
        "iterator",
        |vm, rooted| vm.iterators.push((vec![rooted], "item".to_string(), 0)),
        |vm| vm.iterators[0].0[0],
    );
}

#[test]
fn collect_preserves_iterator_generator_buffer_root() {
    assert_collect_preserves_root(
        "iterator-generator",
        |vm, rooted| {
            let generator = Arc::new(Mutex::new(GeneratorState16::from(vec![rooted])));
            assert_eq!(generator.lock().advance(), Some(rooted));
            vm.iterator_generators.push(Some(generator));
        },
        |vm| {
            vm.iterator_generators[0]
                .as_ref()
                .expect("generator exists")
                .lock()
                .buffered[0]
        },
    );
}

#[test]
fn collect_preserves_declaration_root() {
    assert_collect_preserves_root(
        "declaration",
        |vm, rooted| {
            vm.declarations.insert("kind:name".to_string(), rooted);
        },
        |vm| {
            *vm.declarations
                .get("kind:name")
                .expect("declaration exists")
        },
    );
}

#[test]
fn collect_preserves_relation_root() {
    assert_collect_preserves_root(
        "relation",
        |vm, rooted| {
            vm.relations.insert("A_B".to_string(), rooted);
        },
        |vm| *vm.relations.get("A_B").expect("relation exists"),
    );
}

#[test]
fn collect_preserves_subject_template_root() {
    assert_collect_preserves_root(
        "subject-template",
        |vm, rooted| {
            vm.subject_templates.insert(
                "Subject".to_string(),
                SubjectTemplate {
                    name: "Subject".to_string(),
                    of_subject: None,
                    roles: vec![],
                    state_defaults: FxHashMap::from_iter([("field".to_string(), rooted)]),
                    capabilities: vec![],
                    intents: vec![],
                },
            );
        },
        |vm| vm.subject_templates["Subject"].state_defaults["field"],
    );
}

#[test]
fn collect_preserves_subject_instance_state_root() {
    assert_collect_preserves_root(
        "subject-instance-state",
        |vm, rooted| {
            vm.subject_instances.insert(
                "subject-1".to_string(),
                SubjectInstance {
                    template_name: "Subject".to_string(),
                    instance_id: "subject-1".to_string(),
                    state: FxHashMap::from_iter([("field".to_string(), rooted)]),
                    actor_id: "actor-1".to_string(),
                    views: FxHashMap::default(),
                },
            );
        },
        |vm| vm.subject_instances["subject-1"].state["field"],
    );
}

#[test]
fn collect_preserves_subject_instance_view_root() {
    assert_collect_preserves_root(
        "subject-instance-view",
        |vm, rooted| {
            let mut views = FxHashMap::default();
            views.insert(
                "main".to_string(),
                FxHashMap::from_iter([("field".to_string(), rooted)]),
            );
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
        },
        |vm| vm.subject_instances["subject-1"].views["main"]["field"],
    );
}

#[test]
fn collect_preserves_toml_config_root() {
    assert_collect_preserves_root(
        "toml-config",
        |vm, rooted| vm.toml_config = rooted,
        |vm| vm.toml_config,
    );
}

#[test]
fn collect_preserves_dispatch_provider_receiver_root() {
    assert_collect_preserves_root(
        "dispatch-provider",
        |vm, rooted| vm.dispatch_provider_receiver = Some(rooted),
        |vm| vm.dispatch_provider_receiver.expect("receiver exists"),
    );
}

#[test]
fn collect_preserves_last_instance_mutation_root() {
    assert_collect_preserves_root(
        "last-instance-mutation",
        |vm, rooted| vm.last_instance_mutation = Some(Box::new(rooted)),
        |vm| **vm.last_instance_mutation.as_ref().expect("mutation exists"),
    );
}

#[test]
fn collect_preserves_pending_flow_root() {
    assert_collect_preserves_root(
        "pending-flow",
        |vm, rooted| vm.pending_flow = Some(PendingFlow::Return(Box::new(rooted))),
        |vm| match vm.pending_flow.as_ref().expect("pending flow exists") {
            PendingFlow::Return(value) | PendingFlow::Throw(value) => *value.as_ref(),
        },
    );
}

#[test]
fn collect_preserves_saved_finally_pending_root() {
    assert_collect_preserves_root(
        "saved-finally",
        |vm, rooted| vm.frame_stack.push(dummy_frame_with_pending(rooted)),
        |vm| {
            let pending = vm.frame_stack[0]
                .saved_finally
                .as_ref()
                .and_then(|saved| saved.pending_flow.as_ref())
                .expect("saved pending flow exists");
            match pending {
                PendingFlow::Return(value) | PendingFlow::Throw(value) => *value.as_ref(),
            }
        },
    );
}

#[test]
fn collect_preserves_tvar_registry_root() {
    assert_collect_preserves_root(
        "tvar-registry",
        |vm, rooted| {
            vm.tvars.create_with_id("gc-collect-tvar", rooted);
        },
        |vm| vm.tvars.read("gc-collect-tvar").expect("tvar root exists"),
    );
}

#[test]
fn collect_preserves_current_transaction_root() {
    assert_collect_preserves_root(
        "current-tx",
        |vm, rooted| {
            let tvar_id = vm
                .tvars
                .create_with_id("gc-collect-tx-tvar", Value16::null());
            let tvar = vm.tvars.get(&tvar_id).expect("tvar exists");
            let mut tx = hudhudscript_stm::Transaction::new();
            tx.write(&tvar, rooted);
            vm.current_tx = Some(Box::new(tx));
        },
        |vm| {
            vm.current_tx
                .as_ref()
                .expect("current transaction exists")
                .root_values()[0]
        },
    );
}

#[test]
fn collect_preserves_cached_promise_root() {
    assert_collect_preserves_root(
        "promise-cache",
        |vm, rooted| {
            let _promise_id = vm.promise_registry.store_result(Ok(rooted));
        },
        |vm| vm.promise_registry.cached_values()[0],
    );
}

#[test]
fn collect_preserves_actor_mailbox_root() {
    assert_collect_preserves_root(
        "actor-mailbox",
        |vm, rooted| {
            let (actor_ref, mailbox) = vm.actors.spawn();
            actor_ref.send(rooted).expect("send actor payload");
            vm.actor_mailboxes.insert(actor_ref.id.clone(), mailbox);
        },
        |vm| {
            vm.actor_mailboxes
                .values()
                .next()
                .expect("mailbox exists")
                .peek_nonblocking()[0]
                .payload
        },
    );
}

#[test]
fn collect_preserves_tco_args_root() {
    assert_collect_preserves_root(
        "tco-args",
        |vm, rooted| vm.tco_args = Some(vec![rooted]),
        |vm| vm.tco_args.as_ref().expect("tco args exist")[0],
    );
}

#[test]
fn collect_preserves_args_scratch_root() {
    assert_collect_preserves_root(
        "args-scratch",
        |vm, rooted| vm.args_scratch.push(rooted),
        |vm| vm.args_scratch[0],
    );
}

#[test]
fn collect_preserves_unconsumed_generator_values() {
    // P4 regresyon: pending'deki (henüz advance edilmemiş) değerler collect'ten sağ çıkar.
    let mut vm = VM::new();
    let v1 = Value16::string("gen-pending-value-one!");
    let v2 = Value16::string("gen-pending-value-two!");
    let state = Arc::new(Mutex::new(GeneratorState16::from(vec![v1, v2])));
    let generator = Value16::generator(Arc::clone(&state));
    vm.globals.insert(interner::intern("g"), generator);

    gc::collect(&vm);

    let first = state.lock().advance().expect("first pending value");
    assert_eq!(first.as_str(), Some("gen-pending-value-one!"));
    let second = state.lock().advance().expect("second pending value");
    assert_eq!(second.as_str(), Some("gen-pending-value-two!"));
}

#[test]
fn collect_preserves_bytecode_string_constants() {
    // P5.1 regresyon: >15B string literal'ler bytecode.constants'ta yaşar;
    // ilk collect onları free etmemeli. gc_constant_roots izlenmeli.
    let mut vm = VM::new();
    // 16+ byte literal → heap alloc yoluna girer.
    let long_strings: Vec<Value16> = (0..1000)
        .map(|i| Value16::string(format!("literal_longer_than_15_chars_{:04}", i)))
        .collect();
    for (i, s) in long_strings.iter().enumerate() {
        vm.globals.insert(interner::intern(&format!("k{}", i)), *s);
    }
    // constants taklidi: gc_constant_roots'a ekle (execute()'nin yaptığı iş).
    vm.gc_constant_roots.extend(long_strings.clone());

    gc::collect(&vm);

    for (i, s) in long_strings.iter().enumerate() {
        let key_str = format!("k{}", i);
        let key = interner::intern(&key_str);
        let got = vm.globals.get(&key).expect("key survived");
        assert_eq!(
            got.as_str(),
            s.as_str(),
            "literal {} should survive collect",
            i
        );
    }
}

#[test]
fn collect_preserves_function_chunk_constants_via_gc_constant_roots() {
    // P5.1: gc_constant_roots'taki değerler collect'ten sağ çıkmalı
    // (trace_roots'un onları gördüğünün doğrudan kanıtı).
    let mut vm = VM::new();
    let chunk_const = Value16::string("function_chunk_literal_over_15bytes!");
    let chunk_sym = interner::intern("chunk_literal");
    vm.globals.insert(chunk_sym, chunk_const);
    // gc_constant_roots ile trace_roots'a dahil et: chunk cache yüklenmiş gibi.
    vm.gc_constant_roots.push(chunk_const);

    gc::collect(&vm);

    let got = vm.globals.get(&chunk_sym).expect("global survived");
    assert_eq!(
        got.as_str(),
        Some("function_chunk_literal_over_15bytes!"),
        "chunk literal should not be freed"
    );
}

#[test]
fn collect_increments_stats() {
    // P8: Tek collect sonrası collections >= 1, total_freed >= 0.
    let st_before = gc::stats();
    let mut vm = VM::new();
    let s = Value16::string("gc-stats-test-string");
    vm.globals.insert(interner::intern("k"), s);
    gc::collect(&vm);
    let st_after = gc::stats();
    assert!(
        st_after.collections > st_before.collections,
        "collections should increment"
    );
    assert!(st_after.live_objects >= 1, "at least our global survives");
}
