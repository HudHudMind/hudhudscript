use super::*;
use crate::vm::call_state::{ReceiverContext, ReturnSink, VmCallRequest, VmContinuation};
use crate::vm::machine::CallFrame;
use hudhudscript_bytecode::{gc, FunctionChunk, Instruction, SymId, Value16};
use rustc_hash::FxHashMap;
use std::{ptr, sync::Arc};

fn dynamic_value(label: &str) -> Value16 {
    Value16::string(format!("gc-call-state-{}-dynamic-value", label))
}

fn return_chunk() -> Arc<FunctionChunk> {
    Arc::new(FunctionChunk {
        params: vec![],
        instructions: vec![Instruction::Return { src: 0 }],
        constants: vec![],
        captures: vec![],
        capture_sym_ids: vec![],
        capture_slots: vec![],
        is_async: false,
        is_generator: false,
        local_count: 0,
        local_names: vec![],
        capture_cells: vec![],
        max_register: 0,
        sym_to_slot: std::sync::OnceLock::new(),
        param_slots: Box::new([]),
        is_plain_function: true,
        source_positions: vec![None],
    })
}

fn pending_request(value: Value16) -> Box<VmCallRequest> {
    Box::new(VmCallRequest {
        chunk: return_chunk(),
        func_sym: SymId(0),
        args: vec![value],
        captures: FxHashMap::default(),
        dst: 0,
        origin_ip: 0,
        receiver_context: None,
        return_sink: ReturnSink::Discard,
        swallow_error: false,
    })
}

fn frame_with_receiver(receiver: Value16) -> CallFrame {
    CallFrame {
        chunk_ptr: ptr::null(),
        owned_chunk: None,
        packed: ptr::null(),
        func_sym: SymId(0),
        ip: 0,
        dst: 0,
        reg_base: 0,
        reg_size: 0,
        saved_finally: None,
        has_captures: false,
        debugger_pushed: false,
        call_depth: 0,
        owned_local_syms: false,
        class_context: false,
        return_sink: ReturnSink::Discard,
        receiver_context: Some(Box::new(ReceiverContext::new(receiver, None, false))),
        swallow_error: false,
    }
}

fn assert_call_state_root(label: &str, install: impl FnOnce(&mut VM, Value16)) {
    let mut vm = VM::new();
    let rooted = dynamic_value(label);
    install(&mut vm, rooted);
    let unreachable = dynamic_value("unreachable");

    vm.mark_from_roots();

    assert!(gc::is_marked(rooted), "{} root was not marked", label);
    assert!(!gc::is_marked(unreachable));
}

#[test]
fn marks_pending_vm_call_root() {
    assert_call_state_root("pending-request", |vm, rooted| {
        vm.pending_vm_call = Some(pending_request(rooted));
    });
}

#[test]
fn marks_continuation_request_root() {
    assert_call_state_root("continuation-request", |vm, rooted| {
        vm.pending_vm_call = Some(pending_request(rooted));
        vm.vm_continuations.push(VmContinuation::GovernanceDispatch(
            crate::vm::call_state::GovernanceDispatchState {
                dst: 0,
                response: {
                    let mut map = hudhudscript_bytecode::ObjMap::default();
                    map.insert("root".to_string(), rooted);
                    map
                },
            },
        ));
    });
}

#[test]
fn marks_frame_receiver_context_root() {
    assert_call_state_root("frame-receiver", |vm, rooted| {
        vm.frame_stack.push(frame_with_receiver(rooted));
    });
}
