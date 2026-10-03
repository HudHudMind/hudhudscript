//! Tests for hudhudscript-bytecode GC: allocation accounting, tracing, BigInt.

use hudhudscript_bytecode::gc::{bytes_allocated, heap_object_count, trace_children, trace_value};
use hudhudscript_bytecode::num_bigint;
use hudhudscript_bytecode::{
    ClassData, DataData, DynamicObject, FunctionData, GeneratorState16, InstanceData, ObjMap,
    PromiseState16, SymId, Value16,
};
use parking_lot::{Mutex, RwLock};
use std::{collections::HashMap, mem, sync::Arc};

#[test]
fn constructors_register_allocations() {
    let before_count = heap_object_count();
    let before_bytes = bytes_allocated();

    let _string = Value16::string("0123456789abcdef");
    let _array = Value16::array(vec![]);
    let _option = Value16::option(None);

    assert_eq!(heap_object_count(), before_count + 3);
    assert_eq!(
        bytes_allocated(),
        before_bytes + (3 * mem::size_of::<DynamicObject>())
    );
}

#[test]
fn tracing_marks_reachable_dynamic_graph() {
    fn marked(value: Value16) -> bool {
        let ptr = value.0.as_ptr().expect("dynamic value");
        unsafe { (*(ptr as *const DynamicObject)).marked.get() }
    }

    let leaf = Value16::string("leaf-node-over-15");
    let promise = Value16::promise(PromiseState16::Resolved(Box::new(leaf)));

    let generator_state = Arc::new(Mutex::new(GeneratorState16::from(vec![leaf])));
    assert_eq!(generator_state.lock().advance(), Some(leaf));
    let generator = Value16::generator(generator_state);

    let data = Value16::data(DataData {
        type_name: "TraceData".to_string(),
        fields: {
            let mut m = ObjMap::default();
            m.insert(SymId::from("promise"), promise);
            m
        },
    });

    let class = Value16::class(ClassData {
        name: "TraceClass".to_string(),
        methods: {
            let mut m = ObjMap::default();
            m.insert(SymId::from("method"), generator);
            m
        },
        fields: {
            let mut m = ObjMap::default();
            m.insert(SymId::from("data"), data);
            m
        },
        parent: Some(leaf),
        vtable: {
            let mut m = ObjMap::default();
            m.insert(SymId::from("method"), generator);
            m
        },
        method_access: HashMap::new(),
        is_abstract: false,
    });

    let instance = Value16::instance(InstanceData {
        class_name: "TraceClass".to_string(),
        fields: {
            let mut m = ObjMap::default();
            m.insert(SymId::from("leaf"), leaf);
            m
        },
        class,
    });

    let captured = Arc::new(RwLock::new(instance));
    let function = Value16::function(FunctionData {
        name: "traceFn".to_string(),
        params: vec![],
        chunk_name: "trace_chunk".to_string(),
        chunk_sym: hudhudscript_bytecode::interner::intern("trace_chunk").0,
        captures: HashMap::from([("captured".to_string(), captured)]),
    });

    let option = Value16::option(Some(function));
    let result = Value16::result(Ok(class));
    let root = {
        let mut m = ObjMap::default();
        m.insert(SymId::from("option"), option);
        m.insert(SymId::from("result"), result);
        m.insert(SymId::from("map"), Value16::map(vec![(leaf, data)]));
        m.insert(SymId::from("set"), Value16::set(vec![generator]));
        Value16::object(m)
    };
    let unreachable = Value16::string("unreachable-over-15");

    let mut gray = Vec::new();
    trace_value(root, &mut gray);
    while let Some(ptr) = gray.pop() {
        unsafe { trace_children(&*ptr, &mut gray) };
    }

    assert!(marked(root));
    assert!(marked(option));
    assert!(marked(result));
    assert!(marked(function));
    assert!(marked(instance));
    assert!(marked(class));
    assert!(marked(data));
    assert!(marked(generator));
    assert!(marked(promise));
    assert!(marked(leaf));
    assert!(!marked(unreachable));
}

#[test]
fn bigint_constructor_normalizes_down_to_int_for_small_values() {
    let five = Value16::bigint(num_bigint::BigInt::from(5u8));
    assert!(five.is_int(), "bigint(5) should be Int (fast path)");
    assert_eq!(five.as_int(), Some(5));
}

#[test]
fn bigint_constructor_allocates_for_large_values() {
    let big = Value16::bigint(num_bigint::BigInt::from(2u32).pow(100));
    assert!(big.is_bigint(), "bigint(2^100) should be BigInt (heap)");
    assert!(!big.is_int());
}

#[test]
fn bigint_equality_is_content_based() {
    let big_val = num_bigint::BigInt::from(2u32).pow(100);
    let a = Value16::bigint(big_val.clone());
    let b = Value16::bigint(big_val);
    assert!(a.values_equal(&b), "same-content BigInt should be equal");
    // BigInt and Int should NOT be equal (different types).
    let c = Value16::int(100);
    assert!(!a.values_equal(&c), "BigInt(2^100) != Int(100)");
}

#[test]
fn bigint_as_number_converts_to_f64() {
    let big = Value16::bigint(num_bigint::BigInt::from(42u8));
    let n = big.as_number().expect("BigInt should convert to f64");
    assert!((n - 42.0).abs() < 0.001);
}
