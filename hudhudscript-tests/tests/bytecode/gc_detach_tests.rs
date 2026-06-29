//! Detach / attach public API tests.
//!
//! Moved from `hudhudscript-bytecode/src/gc_detach.rs` inline `tests.rs` module
//! as part of I2-A3 (private inline test consolidation).
//!
//! These tests exercise only the public `gc_detach::detach` / `gc_detach::attach`
//! API and `Value16` constructors; they preserve the original assertions.

use hudhudscript_bytecode::gc_detach::{attach, detach, OwnedTree};
use hudhudscript_bytecode::{DynamicData, DynamicObject, FunctionData, ObjMap, Value16};
use hudhudscript_bytecode::sym::SymId;
use hudhudscript_bytecode::interner::intern;

#[test]
fn detach_attach_roundtrip_simple() {
    let orig = Value16::int(42);
    let graph = detach(orig).unwrap();
    let back = attach(&graph);
    assert_eq!(back.as_int(), Some(42));
}

#[test]
fn detach_attach_roundtrip_string() {
    let orig = Value16::string("hello-world-over-15-bytes!");
    let graph = detach(orig).unwrap();
    let back = attach(&graph);
    assert_eq!(back.as_str(), Some("hello-world-over-15-bytes!"));
}

#[test]
fn detach_attach_roundtrip_nested() {
    let inner = Value16::string("nested-value");
    let arr = Value16::array(vec![inner, Value16::int(7)]);
    let obj = Value16::object(std::collections::HashMap::from([
        (SymId(intern("arr").0), arr),
        (SymId(intern("num").0), Value16::int(99)),
    ]));
    let graph = detach(obj).unwrap();
    let back = attach(&graph);
    let map = back.as_object().unwrap();
    assert_eq!(map[&SymId(intern("num").0)].as_int(), Some(99));
    let arr_back = map[&SymId(intern("arr").0)].as_array().unwrap();
    assert_eq!(arr_back.len(), 2);
    assert_eq!(arr_back[0].as_str(), Some("nested-value"));
}

#[test]
fn detach_attach_preserves_shared_subgraph() {
    let shared = Value16::string("shared-leaf");
    let a = Value16::array(vec![shared, Value16::int(1)]);
    let b = Value16::array(vec![shared, Value16::int(2)]);
    let outer = Value16::array(vec![a, b]);
    let graph = detach(outer).unwrap();
    let back = attach(&graph);
    let arr = back.as_array().unwrap();
    let sub_a = arr[0].as_array().unwrap();
    let sub_b = arr[1].as_array().unwrap();
    assert_eq!(sub_a[0].as_str(), Some("shared-leaf"));
    assert_eq!(sub_b[0].as_str(), Some("shared-leaf"));
    let ptr_a = sub_a[0].0.as_ptr();
    let ptr_b = sub_b[0].0.as_ptr();
    assert_eq!(ptr_a, ptr_b, "shared sub-object must be a single copy");
}

#[test]
fn detach_survives_source_heap_drop() {
    let orig = Value16::string("survive-after-source-drop");
    let graph = detach(orig).unwrap();
    struct DummyRoot;
    impl hudhudscript_bytecode::gc::GcRootSource for DummyRoot {
        fn mark_roots(&self) {}
    }
    hudhudscript_bytecode::gc::collect(&DummyRoot);
    let back = attach(&graph);
    assert_eq!(back.as_str(), Some("survive-after-source-drop"));
}

#[test]
fn detach_unsupported_type_returns_error() {
    let func = Value16::function(FunctionData {
        name: "test".to_string(),
        params: vec![],
        chunk_name: "".to_string(),
        captures: std::collections::HashMap::new(),
    });
    let result = detach(func);
    assert!(result.is_err());
}

// ── G3-1b: Transferable type roundtrips ──

#[test]
fn detach_attach_option_some() {
    let opt = Value16::option(Some(Value16::int(42)));
    let graph = detach(opt).unwrap();
    let back = attach(&graph);
    assert!(back.as_option().is_some());
}

#[test]
fn detach_attach_option_none() {
    let opt = Value16::option(None::<Value16>);
    let graph = detach(opt).unwrap();
    let back = attach(&graph);
    assert!(back.as_option().is_some());
}

#[test]
fn detach_attach_instance_fields() {
    use hudhudscript_bytecode::InstanceData;
    let mut fields = ObjMap::default();
    fields.insert(
        "name".to_string(),
        Value16::string("test-instance-name-over-15"),
    );
    fields.insert("age".to_string(), Value16::int(30));
    let inst = hudhudscript_bytecode::gc::alloc(
        hudhudscript_bytecode::DynamicKind::Instance,
        hudhudscript_bytecode::DynamicData::Instance(InstanceData {
            class_name: "TestClass".to_string(),
            fields,
            class: Value16::null(),
        }),
    );
    let graph = detach(inst).unwrap();
    let back = attach(&graph);
    assert!(back.is_dynamic());
}

#[test]
fn detach_attach_data_fields() {
    let mut fields = ObjMap::default();
    fields.insert("x".to_string(), Value16::int(10));
    let data = hudhudscript_bytecode::gc::alloc(
        hudhudscript_bytecode::DynamicKind::Data,
        hudhudscript_bytecode::DynamicData::Data(hudhudscript_bytecode::payloads::DataData {
            type_name: "Point".to_string(),
            fields,
        }),
    );
    let graph = detach(data).unwrap();
    let back = attach(&graph);
    assert!(back.is_dynamic());
}

// ── G2: shape-level detach structure tests ─────────────────

#[test]
fn shared_heap_subobject_uses_single_pool_node() {
    // Use a shared dynamic sub-array (not a string) so that the identity is preserved
    // by Ref(idx) instead of being copied by value.
    let shared = Value16::array(vec![Value16::int(42)]);
    let a = Value16::array(vec![shared, Value16::int(1)]);
    let b = Value16::array(vec![shared, Value16::int(2)]);
    let outer = Value16::array(vec![a, b]);

    let graph = detach(outer).unwrap();

    let OwnedTree::Array(ref root_items) = graph.nodes[graph.root_idx] else {
        panic!("expected root Array");
    };
    assert!(matches!(root_items[0], OwnedTree::Ref(_)));
    assert!(matches!(root_items[1], OwnedTree::Ref(_)));

    let array_count = graph
        .nodes
        .iter()
        .filter(|n| matches!(n, OwnedTree::Array(_)))
        .count();
    assert_eq!(
        array_count, 4,
        "root + two shared children + shared subobject = four arrays"
    );

    let OwnedTree::Ref(idx_a_root) = root_items[0] else {
        unreachable!()
    };
    let OwnedTree::Ref(idx_b_root) = root_items[1] else {
        unreachable!()
    };

    let OwnedTree::Array(ref child_a) = graph.nodes[idx_a_root] else {
        panic!("expected first child to be an Array");
    };
    let OwnedTree::Array(ref child_b) = graph.nodes[idx_b_root] else {
        panic!("expected second child to be an Array");
    };
    assert!(
        matches!(child_a[0], OwnedTree::Ref(_)),
        "first element of child_a must be a Ref to the shared subobject"
    );
    assert!(
        matches!(child_b[0], OwnedTree::Ref(_)),
        "first element of child_b must be a Ref to the shared subobject"
    );

    let OwnedTree::Ref(idx_a) = child_a[0] else {
        unreachable!()
    };
    let OwnedTree::Ref(idx_b) = child_b[0] else {
        unreachable!()
    };
    assert_eq!(idx_a, idx_b, "both Refs must target the same pool node");
}

#[test]
fn attach_reconstructs_single_shared_pointer() {
    let shared = Value16::array(vec![Value16::int(42)]);
    let a = Value16::array(vec![shared, Value16::int(1)]);
    let b = Value16::array(vec![shared, Value16::int(2)]);
    let outer = Value16::array(vec![a, b]);

    let graph = detach(outer).unwrap();
    let back = attach(&graph);
    let arr = back.as_array().unwrap();
    let sub_a = arr[0].as_array().unwrap();
    let sub_b = arr[1].as_array().unwrap();

    let ptr_a = sub_a[0].0.as_ptr();
    let ptr_b = sub_b[0].0.as_ptr();
    assert_eq!(
        ptr_a, ptr_b,
        "shared sub-object must be reconstructed as a single heap pointer"
    );
}

#[test]
fn cycle_detach_does_not_stack_overflow() {
    let leaf = Value16::string("cycle-leaf-over-15-chars");
    let a = Value16::array(vec![leaf, Value16::null()]);
    let b = Value16::array(vec![Value16::null(), Value16::int(1)]);
    unsafe {
        let a_ptr = a.0.as_ptr().unwrap() as *mut DynamicObject;
        if let DynamicData::Array(ref mut a_vec) = (*a_ptr).data {
            a_vec[1] = b;
        }
        let b_ptr = b.0.as_ptr().unwrap() as *mut DynamicObject;
        if let DynamicData::Array(ref mut b_vec) = (*b_ptr).data {
            b_vec[0] = a;
        }
    }

    // Detach must complete without stack overflow and produce a finite graph.
    let graph = detach(a).unwrap();
    let OwnedTree::Array(ref a_items) = graph.nodes[graph.root_idx] else {
        panic!("expected Array root");
    };
    assert!(matches!(a_items[0], OwnedTree::String(_)));
    assert!(matches!(a_items[1], OwnedTree::Ref(_)));

    // Attach must also terminate; cycle edge currently returns null (G2-A3).
    let back = attach(&graph);
    let arr_back = back.as_array().unwrap();
    assert_eq!(arr_back[0].as_str(), Some("cycle-leaf-over-15-chars"));
    assert!(arr_back[1].is_null() || arr_back[1].as_array().is_some());
}

#[test]
fn detach_attach_cycle_safe() {
    let leaf = Value16::string("leaf");
    let a = Value16::array(vec![leaf, Value16::null()]);
    let b = Value16::array(vec![Value16::null(), Value16::int(1)]);
    unsafe {
        let a_ptr = a.0.as_ptr().unwrap() as *mut DynamicObject;
        if let DynamicData::Array(ref mut a_vec) = (*a_ptr).data {
            a_vec[1] = b;
        }
        let b_ptr = b.0.as_ptr().unwrap() as *mut DynamicObject;
        if let DynamicData::Array(ref mut b_vec) = (*b_ptr).data {
            b_vec[0] = a;
        }
    }
    let graph = detach(a).unwrap();
    let back = attach(&graph);
    let arr_back = back.as_array().unwrap();
    assert_eq!(arr_back[0].as_str(), Some("leaf"));
    assert!(arr_back[1].as_array().is_some());
}

#[test]
fn root_tree_is_single_instance_in_pool() {
    let arr = Value16::array(vec![Value16::int(1), Value16::int(2)]);
    let graph = detach(arr).unwrap();

    let OwnedTree::Array(ref root_items) = &graph.nodes[graph.root_idx] else {
        panic!("expected Array root");
    };
    assert_eq!(root_items.len(), 2);

    let root_in_pool = graph
        .nodes
        .iter()
        .find(|n| matches!(n, OwnedTree::Array(items) if items.len() == 2))
        .expect("root Array must live in pool.nodes");
    assert!(
        matches!(root_in_pool, OwnedTree::Array(items)
            if matches!(&items[0], OwnedTree::Int(1)) && matches!(&items[1], OwnedTree::Int(2))),
        "pool must contain the single root instance"
    );

    // There should be exactly one Array node (the root). Ints are inline and do not
    // get their own pool nodes.
    let array_nodes = graph
        .nodes
        .iter()
        .filter(|n| matches!(n, OwnedTree::Array(_)))
        .count();
    assert_eq!(array_nodes, 1, "only root array node should exist");
}
