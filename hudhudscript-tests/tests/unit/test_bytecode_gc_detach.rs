//! Tests for hudhudscript-bytecode gc_detach: detach/attach round-trips.

use hudhudscript_bytecode::gc_detach::{attach, detach};
use hudhudscript_bytecode::num_bigint;
use hudhudscript_bytecode::{DataData, ObjMap, SymId, Value16};

#[test]
fn round_trip_string() {
    let v = Value16::string("hello world over 15 chars");
    let g = detach(v).unwrap();
    let v2 = attach(&g);
    assert_eq!(v2.as_str(), Some("hello world over 15 chars"));
}

#[test]
fn round_trip_int() {
    let v = Value16::int(42);
    let g = detach(v).unwrap();
    let v2 = attach(&g);
    assert_eq!(v2.as_int(), Some(42));
}

#[test]
fn round_trip_bigint() {
    let big = num_bigint::BigInt::from(2u64).pow(100);
    let v = Value16::bigint(big.clone());
    let g = detach(v).unwrap();
    let v2 = attach(&g);
    assert!(v2.is_bigint());
    assert_eq!(*v2.as_bigint().unwrap(), big);
}

#[test]
fn round_trip_result_ok() {
    let v = Value16::result(Ok(Value16::int(99)));
    let g = detach(v).unwrap();
    let v2 = attach(&g);
    let res = v2.as_result().unwrap();
    assert!(res.is_ok());
    assert_eq!(res.unwrap().as_int(), Some(99));
}

#[test]
fn round_trip_data() {
    let mut m = ObjMap::default();
    m.insert(SymId::from("x"), Value16::int(1));
    let v = Value16::data(DataData {
        type_name: "Test".into(),
        fields: m,
    });
    let g = detach(v).unwrap();
    let v2 = attach(&g);
    let d = v2.as_data_data().unwrap();
    assert_eq!(d.type_name, "Test");
    assert_eq!(d.fields.get(&SymId::from("x")).unwrap().as_int(), Some(1));
}
