use super::*;
use crate::vm::types::PendingFlow;
use hudhudscript_bytecode::{gc, GeneratorState16, Value16};
use parking_lot::{Mutex, RwLock};
use rustc_hash::FxHashMap;
use std::sync::Arc;

fn dyn_value(label: &str) -> Value16 {
    Value16::string(format!("gc-runtime-bucket-{}-dynamic-value", label))
}

fn assert_runtime_bucket_marks_root(label: &str, install: impl FnOnce(&mut VM, Value16)) {
    let mut vm = VM::new();
    let rooted = dyn_value(label);
    let unrelated = dyn_value("unrelated");

    install(&mut vm, rooted);
    vm.mark_from_roots();

    assert!(gc::is_marked(rooted), "{} root was not marked", label);
    assert!(
        !gc::is_marked(unrelated),
        "{} unexpectedly marked an unrelated value",
        label
    );
}

#[test]
fn g5_marks_tco_args_call_temporary_root() {
    assert_runtime_bucket_marks_root("tco-args", |vm, rooted| {
        vm.tco_args = Some(vec![rooted]);
    });
}

#[test]
fn g5_marks_args_scratch_call_temporary_root() {
    assert_runtime_bucket_marks_root("args-scratch", |vm, rooted| {
        vm.args_scratch.push(rooted);
    });
}

#[test]
fn g5_marks_scope_cell_closure_root() {
    assert_runtime_bucket_marks_root("scope-cell", |vm, rooted| {
        vm.scope_cells.push(FxHashMap::from_iter([(
            "captured".to_string(),
            Arc::new(RwLock::new(rooted)),
        )]));
    });
}

#[test]
fn g5_marks_scope_cell_pool_closure_root() {
    assert_runtime_bucket_marks_root("scope-cell-pool", |vm, rooted| {
        vm.scope_cells_pool.push(FxHashMap::from_iter([(
            "pooled".to_string(),
            Arc::new(RwLock::new(rooted)),
        )]));
    });
}

#[test]
fn g5_marks_promise_registry_async_root() {
    assert_runtime_bucket_marks_root("promise-registry", |vm, rooted| {
        let _promise_id = vm.promise_registry.store_result(Ok(rooted));
    });
}

#[test]
fn g5_marks_tvar_registry_stm_root() {
    assert_runtime_bucket_marks_root("tvar-registry", |vm, rooted| {
        vm.tvars.create_with_id("gc-runtime-bucket-tvar", rooted);
    });
}

#[test]
fn g5_marks_current_transaction_staged_stm_root() {
    assert_runtime_bucket_marks_root("current-tx", |vm, rooted| {
        let tvar_id = vm
            .tvars
            .create_with_id("gc-runtime-bucket-tx-tvar", Value16::null());
        let tvar = vm.tvars.get(&tvar_id).expect("tvar exists");
        let mut tx = hudhudscript_stm::Transaction::new();
        tx.write(&tvar, rooted);
        vm.current_tx = Some(Box::new(tx));
    });
}

#[test]
fn g5_marks_last_return_root() {
    assert_runtime_bucket_marks_root("last-return", |vm, rooted| {
        vm.last_return = rooted;
    });
}

#[test]
fn g5_marks_pending_flow_return_root() {
    assert_runtime_bucket_marks_root("pending-flow-return", |vm, rooted| {
        vm.pending_flow = Some(PendingFlow::Return(Box::new(rooted)));
    });
}

#[test]
fn g5_marks_pending_flow_throw_root() {
    assert_runtime_bucket_marks_root("pending-flow-throw", |vm, rooted| {
        vm.pending_flow = Some(PendingFlow::Throw(Box::new(rooted)));
    });
}

#[test]
fn g5_marks_iterator_generator_buffer_root() {
    assert_runtime_bucket_marks_root("iterator-generator", |vm, rooted| {
        let generator = Arc::new(Mutex::new(GeneratorState16::from(vec![rooted])));
        assert_eq!(generator.lock().advance(), Some(rooted));
        vm.iterator_generators.push(Some(generator));
    });
}
