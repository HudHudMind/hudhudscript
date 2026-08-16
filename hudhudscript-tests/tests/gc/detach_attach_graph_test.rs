//! A2: GC detach/attach nested aggregate regression tests.
//! Verifies that nested arrays/objects survive detach→attach round-trip.

use hudhudscript_bytecode::gc_detach;
use hudhudscript_bytecode::Value16;

#[test]
fn nested_array_round_trip() {
    let inner = Value16::array(vec![Value16::int(1), Value16::int(2)]);
    let outer = Value16::array(vec![inner]);
    let g = gc_detach::detach(outer).unwrap();
    let v2 = gc_detach::attach(&g);
    let arr = v2.as_array().unwrap();
    assert_eq!(arr.len(), 1, "outer array should have 1 element");
    let inner_arr = arr[0].as_array().unwrap();
    assert_eq!(inner_arr.len(), 2, "inner array should have 2 elements");
    assert_eq!(inner_arr[0].as_int(), Some(1));
    assert_eq!(inner_arr[1].as_int(), Some(2));
}

#[test]
fn double_nested_array() {
    // [[[1,2],[3,4]]]
    let a = Value16::array(vec![Value16::int(1), Value16::int(2)]);
    let b = Value16::array(vec![Value16::int(3), Value16::int(4)]);
    let mid = Value16::array(vec![a, b]);
    let outer = Value16::array(vec![mid]);
    let g = gc_detach::detach(outer).unwrap();
    let v2 = gc_detach::attach(&g);
    let outer_arr = v2.as_array().unwrap();
    let mid_arr = outer_arr[0].as_array().unwrap();
    assert_eq!(mid_arr[0].as_array().unwrap()[0].as_int(), Some(1));
    assert_eq!(mid_arr[1].as_array().unwrap()[1].as_int(), Some(4));
}

#[test]
fn nested_object_round_trip() {
    let mut inner = hudhudscript_bytecode::ObjMap::default();
    inner.insert(hudhudscript_bytecode::SymId::from("x"), Value16::int(10));
    let inner_obj = Value16::object(inner);
    let mut outer_map = hudhudscript_bytecode::ObjMap::default();
    outer_map.insert(hudhudscript_bytecode::SymId::from("inner"), inner_obj);
    let outer_obj = Value16::object(outer_map);
    let g = gc_detach::detach(outer_obj).unwrap();
    let v2 = gc_detach::attach(&g);
    let obj = v2.as_object().unwrap();
    let inner = obj
        .get(&hudhudscript_bytecode::SymId::from("inner"))
        .unwrap();
    assert_eq!(
        inner
            .as_object()
            .unwrap()
            .get(&hudhudscript_bytecode::SymId::from("x"))
            .unwrap()
            .as_int(),
        Some(10)
    );
}

#[test]
fn shared_child_identity() {
    let child = Value16::array(vec![Value16::int(42)]);
    let parent = Value16::array(vec![child, child]);
    let g = gc_detach::detach(parent).unwrap();
    let v2 = gc_detach::attach(&g);
    let arr = v2.as_array().unwrap();
    let c0 = arr[0].as_array().unwrap();
    let c1 = arr[1].as_array().unwrap();
    assert_eq!(c0[0].as_int(), Some(42));
    assert_eq!(c1[0].as_int(), Some(42));
}

#[test]
fn plain_array_regression_guard() {
    let v = Value16::array(vec![Value16::int(1), Value16::int(2), Value16::int(3)]);
    let g = gc_detach::detach(v).unwrap();
    let v2 = gc_detach::attach(&g);
    let arr = v2.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0].as_int(), Some(1));
    assert_eq!(arr[2].as_int(), Some(3));
}
