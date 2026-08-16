use super::{ReturnSink, VmCallRequest};
use crate::vm::VM;
use hudhudscript_bytecode::{Bytecode, FunctionChunk, Instruction, SymId, Value16};
use rustc_hash::FxHashMap;
use std::sync::{Arc, OnceLock};

fn returning_chunk(value: Value16) -> Arc<FunctionChunk> {
    Arc::new(FunctionChunk {
        params: vec![],
        instructions: vec![
            Instruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            Instruction::Return { src: 0 },
        ],
        constants: vec![value],
        captures: vec![],
        capture_sym_ids: vec![],
        capture_slots: vec![],
        is_async: false,
        is_generator: false,
        local_count: 0,
        local_names: vec![],
        capture_cells: vec![],
        max_register: 0,
        sym_to_slot: OnceLock::new(),
        param_slots: Box::new([]),
        is_plain_function: true,
        source_positions: vec![None, None],
    })
}

fn request(value: Value16, dst: u8) -> Box<VmCallRequest> {
    Box::new(VmCallRequest {
        chunk: returning_chunk(value),
        func_sym: SymId(hudhudscript_bytecode::interner::intern("deferred_test").0),
        args: vec![],
        captures: FxHashMap::default(),
        dst,
        origin_ip: 0,
        receiver_context: None,
        return_sink: ReturnSink::Register(dst),
        swallow_error: false,
    })
}

#[test]
fn deferred_call_pushes_frame_without_nested_driver() {
    let mut vm = VM::new();
    let bytecode = Bytecode::new();
    VM::reset_driver_entry_count_for_test();
    vm.schedule_vm_call(request(Value16::int(7), 4)).unwrap();

    let returned = vm.run_frame_loop(&bytecode, &[], 0).unwrap();

    assert!(returned);
    assert_eq!(VM::driver_entry_count_for_test(), 1);
    assert!(vm.frame_stack.is_empty());
    assert!(vm.pending_vm_call.is_none());
}

#[test]
fn deferred_call_result_reaches_destination_register() {
    let mut vm = VM::new();
    let bytecode = Bytecode::new();
    vm.schedule_vm_call(request(Value16::int(42), 17)).unwrap();

    vm.run_frame_loop(&bytecode, &[], 0).unwrap();

    assert_eq!(vm.registers[17].as_int(), Some(42));
}

#[test]
fn deferred_call_depth_limit_returns_runtime_error() {
    let mut vm = VM::new();
    let bytecode = Bytecode::new();
    vm.with_max_call_depth(0);
    vm.schedule_vm_call(request(Value16::int(1), 0)).unwrap();

    let error = vm.run_frame_loop(&bytecode, &[], 0).unwrap_err();

    assert!(error.to_string().contains("Maximum call depth exceeded"));
    assert!(vm.frame_stack.is_empty());
}
