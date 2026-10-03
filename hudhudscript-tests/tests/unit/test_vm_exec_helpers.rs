//! VM exec helpers: generator advance — precomputed generators use the
//! original `advance()`, yield-based generators attach to the caller heap.

use hudhudscript_vm::vm::exec::helpers::generator_advance;
use hudhudscript_vm::vm::VM;
use hudhudscript_bytecode::{gc_detach, GeneratorState16, Value16};
use parking_lot::Mutex;
use std::sync::Arc;

#[test]
fn advance_precomputed_uses_original_advance() {
    let mut vm = VM::new();
    let state = Arc::new(Mutex::new(GeneratorState16::from(vec![
        Value16::int(10),
        Value16::int(20),
    ])));
    // yield_id=None → uses advance()
    let v1 = generator_advance(&mut vm, &state);
    let v2 = generator_advance(&mut vm, &state);
    let v3 = generator_advance(&mut vm, &state);
    assert_eq!(v1.and_then(|v| v.as_int()), Some(10));
    assert_eq!(v2.and_then(|v| v.as_int()), Some(20));
    assert!(v3.is_none());
    // Check buffered
    let st = state.lock();
    assert_eq!(st.buffered.len(), 2);
}

#[test]
fn advance_yield_based_attaches_to_caller_heap() {
    let mut vm = VM::new();
    let (tx, rx) = std::sync::mpsc::channel();
    let tree = gc_detach::detach(Value16::string("yielded-long-string")).unwrap();
    tx.send(tree).unwrap();
    drop(tx); // close channel

    let state = Arc::new(Mutex::new({
        let mut s = GeneratorState16::new(std::sync::mpsc::sync_channel::<Value16>(0).1);
        s.yield_id = Some(0);
        s
    }));
    vm.yield_receivers.insert(0, rx);

    let val = generator_advance(&mut vm, &state);
    assert!(val.is_some());
    let v = val.unwrap();
    assert_eq!(v.as_str(), Some("yielded-long-string"));

    // Check buffered was populated
    let st = state.lock();
    assert_eq!(st.buffered.len(), 1);
    assert_eq!(st.buffered[0].as_str(), Some("yielded-long-string"));
}
