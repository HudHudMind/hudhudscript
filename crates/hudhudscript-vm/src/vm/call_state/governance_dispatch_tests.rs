//! G06E acceptance tests for governance dispatch and STM continuations.

use crate::vm::call_state::{
    AtomicTransactionAttemptState, ContinuationId, ContinuationResume, VmContinuation,
};
use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{FunctionChunk, Instruction, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use rustc_hash::FxHashMap;
use std::sync::Arc;

fn compile_source(source: &str) -> hudhudscript_bytecode::Bytecode {
    let ast = parse(source).expect("test source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("test source must compile")
}

fn intent_chunk() -> Arc<FunctionChunk> {
    let mut constants = Vec::new();
    let mut object = hudhudscript_bytecode::ObjMap::default();
    object.insert("score".to_string(), Value16::number(42.0));
    constants.push(Value16::object(object));
    Arc::new(FunctionChunk {
        params: vec![],
        instructions: vec![
            Instruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            Instruction::Return { src: 0 },
        ],
        constants,
        captures: vec![],
        capture_sym_ids: vec![],
        capture_slots: vec![],
        is_async: false,
        is_generator: false,
        local_count: 0,
        local_names: vec![],
        capture_cells: vec![],
        max_register: 1,
        sym_to_slot: std::sync::OnceLock::new(),
        param_slots: Box::new([]),
        is_plain_function: true,
        source_positions: vec![None],
    })
}

#[test]
fn governance_dispatch_uses_trampoline_and_merges_result() {
    let bytecode = compile_source(
        r#"
let r = dispatch_intent("go", "Sub")
"#,
    );
    bytecode.add_function("intent::Sub.go".to_string(), intent_chunk());

    let mut vm = VM::new();
    VM::reset_driver_entry_count_for_test();
    vm.execute(&bytecode).expect("dispatch_intent must execute");

    assert_eq!(
        VM::driver_entry_count_for_test(),
        1,
        "governance dispatch must stay on the canonical native driver"
    );
    let result_value = vm.get_variable_owned("r").expect("r must be published");
    let result = result_value.as_object().expect("r must be an object");

    assert_eq!(
        result.get("dispatched").and_then(Value16::as_bool),
        Some(true)
    );
    assert_eq!(result.get("score").and_then(Value16::as_number), Some(42.0));
    assert_eq!(
        result.get("intent").and_then(Value16::as_string),
        Some("go".to_string())
    );
}

fn body_chunk(return_value: Value16) -> Arc<FunctionChunk> {
    Arc::new(FunctionChunk {
        params: vec![],
        instructions: vec![Instruction::Return { src: 0 }],
        constants: vec![return_value],
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

fn atomic_state(chunk: Arc<FunctionChunk>, max_retries: usize) -> AtomicTransactionAttemptState {
    AtomicTransactionAttemptState {
        function: Value16::null(),
        chunk,
        func_sym: hudhudscript_bytecode::SymId(0),
        captures: FxHashMap::default(),
        dst: 200,
        origin_ip: 0,
        attempt: 0,
        started_at: std::time::Instant::now(),
        config: hudhudscript_stm::StmConfig {
            max_retries,
            ..Default::default()
        },
        backoff_us: 0,
    }
}

#[test]
fn atomic_continuation_preserves_retry_and_commit() {
    let mut vm = VM::new();
    vm.tvars
        .create_with_id("x".to_string(), Value16::number(0.0));
    let tvar = vm.tvars.get("x").expect("tvar must exist");

    let chunk = body_chunk(Value16::number(7.0));
    let state = atomic_state(chunk, 3);
    vm.start_atomic_transaction_attempt(state)
        .expect("first attempt must schedule");
    assert!(vm.in_stm_context);
    assert!(vm.current_tx.is_some());

    // First attempt read the tvar, then a concurrent writer bumped its
    // version — the commit must detect the conflict and retry.
    vm.current_tx
        .as_mut()
        .expect("transaction must be installed")
        .read(&tvar);
    tvar.commit_write(Value16::number(1.0));

    let id = ContinuationId(
        vm.vm_continuations
            .iter()
            .rposition(|slot| matches!(slot, VmContinuation::AtomicTransactionAttempt(_)))
            .expect("atomic continuation must exist"),
    );
    match vm.resume_continuation(id, Value16::number(7.0)).unwrap() {
        ContinuationResume::Schedule(_) => {}
        _ => panic!("conflict must schedule a retry"),
    }
    assert!(
        vm.in_stm_context,
        "retry must reinstall transaction context"
    );
    assert!(vm.current_tx.is_some(), "retry must open a new transaction");

    // The retried transaction reads no stale versions — its commit wins and
    // the callback result is delivered to dst.
    match vm.resume_continuation(id, Value16::number(7.0)).unwrap() {
        ContinuationResume::Complete { dst, value } => {
            assert_eq!(dst, 200);
            assert_eq!(value.as_number(), Some(7.0));
        }
        _ => panic!("conflict-free commit must complete"),
    }
    assert!(!vm.in_stm_context);
    assert!(vm.current_tx.is_none());
}

#[test]
fn atomic_body_error_clears_transaction_context() {
    let bytecode = compile_source(
        r#"
let r = atomically(() => { return missing_function_g06e() })
"#,
    );
    let mut vm = VM::new();
    let result: CompileResult<()> = vm.execute(&bytecode).map(|_| ());
    assert!(result.is_err(), "body error must propagate");
    assert!(
        !vm.in_stm_context,
        "failed body must clear the STM context flag"
    );
    assert!(
        vm.current_tx.is_none(),
        "failed body must drop the transaction"
    );
}

#[test]
fn atomic_continuation_state_survives_gc() {
    let mut vm = VM::new();
    let argument = Value16::string("gc-atomic-callback-dynamic-value");
    let state = AtomicTransactionAttemptState {
        function: argument,
        chunk: body_chunk(Value16::null()),
        func_sym: hudhudscript_bytecode::SymId(0),
        captures: FxHashMap::default(),
        dst: 200,
        origin_ip: 0,
        attempt: 0,
        started_at: std::time::Instant::now(),
        config: hudhudscript_stm::StmConfig::default(),
        backoff_us: 0,
    };
    vm.vm_continuations
        .push(VmContinuation::AtomicTransactionAttempt(state));

    vm.mark_from_roots();

    assert!(hudhudscript_bytecode::gc::is_marked(argument));
}
