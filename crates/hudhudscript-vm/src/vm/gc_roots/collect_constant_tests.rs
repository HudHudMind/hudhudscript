use super::*;
use hudhudscript_bytecode::interner;
use hudhudscript_bytecode::{gc, GeneratorState16, Value16};
use parking_lot::Mutex;
use std::sync::Arc;

#[test]
fn collect_preserves_unconsumed_generator_values() {
    let mut vm = VM::new();
    let first_value = Value16::string("gen-pending-value-one!");
    let second_value = Value16::string("gen-pending-value-two!");
    let state = Arc::new(Mutex::new(GeneratorState16::from(vec![
        first_value,
        second_value,
    ])));
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
    let mut vm = VM::new();
    let long_strings: Vec<Value16> = (0..1000)
        .map(|index| Value16::string(format!("literal_longer_than_15_chars_{:04}", index)))
        .collect();
    for (index, value) in long_strings.iter().enumerate() {
        vm.globals
            .insert(interner::intern(&format!("k{}", index)), *value);
    }
    vm.gc_constant_roots.extend(long_strings.clone());

    gc::collect(&vm);

    for (index, value) in long_strings.iter().enumerate() {
        let key = interner::intern(&format!("k{}", index));
        let actual = vm.globals.get(&key).expect("key survived");
        assert_eq!(
            actual.as_str(),
            value.as_str(),
            "literal {} should survive collect",
            index
        );
    }
}

#[test]
fn collect_preserves_function_chunk_constants_via_gc_constant_roots() {
    let mut vm = VM::new();
    let chunk_const = Value16::string("function_chunk_literal_over_15bytes!");
    let chunk_sym = interner::intern("chunk_literal");
    vm.globals.insert(chunk_sym, chunk_const);
    vm.gc_constant_roots.push(chunk_const);

    gc::collect(&vm);

    let actual = vm.globals.get(&chunk_sym).expect("global survived");
    assert_eq!(
        actual.as_str(),
        Some("function_chunk_literal_over_15bytes!"),
        "chunk literal should not be freed"
    );
}

#[test]
fn collect_increments_stats() {
    let stats_before = gc::stats();
    let mut vm = VM::new();
    let value = Value16::string("gc-stats-test-string");
    vm.globals.insert(interner::intern("k"), value);

    gc::collect(&vm);

    let stats_after = gc::stats();
    assert!(stats_after.collections > stats_before.collections);
    assert!(
        stats_after.live_objects >= 1,
        "at least our global survives"
    );
}
