//! Coverage: `Value16` public operations (`value16_impl.rs` and the inline
//! extractors it builds on). Exercises every dynamic-kind accessor (string,
//! array, object, set, map, option, result, bigint, function, instance,
//! class, data, promise, generator, tool, resource) plus the scalar
//! extractors (as_int/as_number/as_string/as_bool), split_tag payloads and
//! the `values_equal`-backed `PartialEq` semantics across repr kinds.

use hudhudscript_bytecode::num_bigint::BigInt;
use hudhudscript_bytecode::{
    ClassData, DataData, FunctionData, GeneratorState16, InstanceData, ObjMap, ReprTag,
    ResourceRef, ToolRef, Value16,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::{mpsc, Arc};

// ── Int extractors and i64 boundaries ───────────────────────────────────────

#[test]
fn int_extractors_and_boundaries() {
    for v in [0i64, 1, -1, 255, 256, i64::MIN, i64::MAX] {
        let x = Value16::int(v);
        assert!(x.is_int(), "is_int for {v}");
        assert_eq!(x.as_int(), Some(v));
        assert_eq!(x.as_int_fast(), Some(v));
        assert_eq!(x.as_int_unchecked(), v);
    }
    // Int widens to f64 through both number extractors.
    assert_eq!(Value16::int(7).as_number(), Some(7.0)); assert_eq!(Value16::int(7).as_number_fast(), Some(7.0));
    assert_eq!(Value16::int(i64::MIN).as_number(), Some(i64::MIN as f64)); assert_eq!(Value16::int(i64::MAX).as_number(), Some(i64::MAX as f64));
    // split_tag exposes the raw tag + payload (two's complement for negatives).
    assert_eq!(Value16::int(5).split_tag(), (ReprTag::Int, 5)); assert_eq!(Value16::int(-1).split_tag(), (ReprTag::Int, u64::MAX));
}

// ── Number extractors and f64 specials ──────────────────────────────────────

#[test]
fn number_extractors_and_f64_specials() {
    let nan = Value16::number(f64::NAN);
    assert!(nan.is_number()); assert!(nan.as_number().unwrap().is_nan());
    assert_eq!(nan.as_int(), None); // NaN is neither Int nor BigInt
    let m = Value16::number(-0.0).as_number().unwrap();
    assert_eq!(m, 0.0); assert!(m.is_sign_negative(), "-0.0 keeps its sign bit");
    assert_eq!(Value16::number(f64::MAX).as_number(), Some(f64::MAX));
    assert_eq!(Value16::number(f64::MIN_POSITIVE).as_number(), Some(f64::MIN_POSITIVE));
    assert_eq!(Value16::number(f64::INFINITY).as_number(), Some(f64::INFINITY));
    // Raw payload is exactly the f64 bit pattern.
    assert_eq!(Value16::number(1.5).split_tag(), (ReprTag::Number, 1.5f64.to_bits()));
    assert_eq!(Value16::number(-0.0).split_tag(), (ReprTag::Number, 0x8000_0000_0000_0000));
    // A Number 5.0 never decodes as an Int (no implicit narrowing).
    assert_eq!(Value16::number(5.0).as_int(), None); assert_eq!(Value16::number(5.0).as_int_fast(), None);
    assert_eq!(Value16::number(5.0).as_number_unchecked(), 5.0); assert_eq!(Value16::number(5.0).as_number_fast(), Some(5.0));
    // Strings never coerce to numbers through the fast path either.
    assert_eq!(Value16::string("1").as_number_fast(), None); assert_eq!(Value16::string("1").as_int_fast(), None);
}

// ── Bool / null / Default ───────────────────────────────────────────────────

#[test]
fn bool_null_and_default_semantics() {
    assert_eq!(Value16::bool_(true).as_bool(), Some(true));
    assert_eq!(Value16::boolean(false).as_bool(), Some(false)); // alias constructor
    assert!(Value16::bool_(true).is_bool()); assert!(Value16::null().is_null());
    assert!(Value16::default().is_null()); assert_eq!(Value16::default(), Value16::null());
    // Wrong-kind accessors: bool/null are neither numbers nor strings.
    assert_eq!(Value16::bool_(true).as_int(), None); assert_eq!(Value16::bool_(true).as_number(), None);
    assert_eq!(Value16::null().as_bool(), None); assert_eq!(Value16::null().as_int(), None); assert_eq!(Value16::null().as_string(), None);
}

// ── String reprs: inline (<=15 bytes), heap ASCII, heap unicode ─────────────

#[test]
fn string_reprs_inline_ascii_unicode() {
    // Short strings live inline in the 124-bit payload.
    let s = Value16::string("hello");
    assert_eq!(s.as_str(), Some("hello")); assert_eq!(s.as_string(), Some("hello".to_string()));
    assert!(s.is_string()); assert_eq!(Value16::string("").as_str(), Some(""));
    // Multi-byte UTF-8 up to 15 bytes is still inline; length is in bytes.
    assert_eq!(Value16::string("héllo").as_str(), Some("héllo")); assert_eq!(Value16::string("héllo").str_char_len(), Some(6));
    // 16+ ASCII bytes go to the heap as StringAscii.
    let heap_ascii = Value16::string("abcdefghijklmnop"); // 16 bytes
    assert_eq!(heap_ascii.as_str(), Some("abcdefghijklmnop"));
    assert!(heap_ascii.is_dynamic_string_ascii()); assert_eq!(heap_ascii.str_char_len(), Some(16));
    // 16+ byte non-ASCII goes to the heap as String; length is in bytes.
    let heap_uni = Value16::string("ğğğğğğğğğğğğğğğ"); // 15 chars, 30 bytes
    assert_eq!(heap_uni.as_str(), Some("ğğğğğğğğğğğğğğğ"));
    assert!(!heap_uni.is_dynamic_string_ascii()); assert_eq!(heap_uni.str_char_len(), Some(30));
    // Cached single-ASCII-char strings.
    assert_eq!(Value16::string_ascii(b'Z').as_str(), Some("Z"));
    assert_eq!(Value16::string_ascii(b'a').as_string(), Some("a".to_string()));
    // Unchecked access agrees with checked access on both reprs.
    assert_eq!(s.as_str_unchecked(), "hello"); assert_eq!(heap_uni.as_str_unchecked(), "ğğğğğğğğğğğğğğğ");
    // Non-strings never decode as strings.
    assert_eq!(Value16::int(5).as_str(), None); assert_eq!(Value16::null().as_string(), None);
    assert_eq!(Value16::array(vec![]).as_str(), None);
}

#[test]
fn string_mutation_and_ascii_downgrade() {
    let mut heap = Value16::string("aaaaaaaaaaaaaaaa"); // 16 ASCII bytes
    heap.as_string_mut().unwrap().push('q');
    assert_eq!(heap.as_str(), Some("aaaaaaaaaaaaaaaaq"));
    assert!(heap.is_dynamic_string_ascii()); assert_eq!(heap.str_char_len(), Some(17)); // kind unchanged by mutation
    // Appending non-ASCII keeps the StringAscii kind until downgraded (P4).
    heap.as_string_mut().unwrap().push('é');
    assert!(heap.is_dynamic_string_ascii());
    heap.downgrade_string_ascii(); // StringAscii -> String
    assert!(!heap.is_dynamic_string_ascii()); assert_eq!(heap.as_str(), Some("aaaaaaaaaaaaaaaaqé"));
    assert_eq!(heap.str_char_len(), Some(19)); // byte length: 17 ASCII + 2
    // Downgrade on a plain unicode String kind is a no-op.
    let mut uni = Value16::string("ğğğğğğğğğğğğğğğ");
    uni.downgrade_string_ascii();
    assert_eq!(uni.as_str(), Some("ğğğğğğğğğğğğğğğ"));
    // Inline strings are immutable; ints have no char length.
    assert_eq!(Value16::string("short").as_string_mut(), None); assert_eq!(Value16::int(1).as_string_mut(), None);
    assert_eq!(Value16::string("short").str_char_len(), Some(5));
    assert_eq!(Value16::int(1).str_char_len(), None); assert_eq!(Value16::array(vec![]).str_char_len(), None);
}

// ── Arrays: read, unchecked access, mutation, auto-extend ──────────────────

#[test]
fn array_read_and_unchecked_access() {
    let arr = Value16::array(vec![Value16::int(10), Value16::string("x"), Value16::bool_(false)]);
    let a = arr.as_array().expect("array value"); assert_eq!(a.len(), 3);
    assert_eq!(a[0].as_int(), Some(10)); assert_eq!(a[1].as_str(), Some("x"));
    assert_eq!(a[2].as_bool(), Some(false));
    assert_eq!(arr.as_array_unchecked().len(), 3); assert_eq!(arr.as_array_unchecked()[0].as_int(), Some(10));
    assert_eq!(arr.array_get(1).unwrap().as_str(), Some("x")); assert_eq!(arr.array_get(2).unwrap().as_bool(), Some(false));
    assert!(arr.array_get(3).is_none());
}

#[test]
fn array_mutation_grow_and_set() {
    let mut arr = Value16::array(vec![Value16::int(1), Value16::int(2)]);
    arr.as_array_mut().unwrap().push(Value16::int(3));
    assert_eq!(arr.as_array().unwrap().len(), 3);
    arr.as_array_mut_unchecked()[0] = Value16::bool_(true); // writes through
    assert_eq!(arr.as_array().unwrap()[0].as_bool(), Some(true));
    assert!(arr.array_set(1, Value16::string("y"))); // in-range overwrite
    assert_eq!(arr.array_get(1).unwrap().as_str(), Some("y"));
    assert!(arr.array_set(4, Value16::int(9))); // auto-extends with nulls
    assert_eq!(arr.as_array().unwrap().len(), 5);
    // Index 2 keeps the pushed element; only index 3 is a null growth slot.
    assert_eq!(arr.array_get(2).unwrap().as_int(), Some(3)); assert!(arr.array_get(3).unwrap().is_null());
    assert_eq!(arr.array_get(4).unwrap().as_int(), Some(9));
    // Empty arrays are valid and grow element-by-element.
    let mut empty = Value16::array(vec![]);
    assert_eq!(empty.as_array().unwrap().len(), 0); assert!(empty.array_set(0, Value16::int(1)));
    assert_eq!(empty.as_array().unwrap().len(), 1);
}

// ── Objects, sets, maps ─────────────────────────────────────────────────────

#[test]
fn object_read_write() {
    let obj = Value16::object(vec![("a", Value16::int(1)), ("b", Value16::string("z"))]);
    let o = obj.as_object().expect("object value"); assert_eq!(o.len(), 2);
    assert_eq!(o.get("a").unwrap().as_int(), Some(1)); assert_eq!(o.get("b").unwrap().as_str(), Some("z"));
    assert!(o.get("missing").is_none());
    let mut obj = obj;
    obj.as_object_mut().unwrap().insert("c", Value16::bool_(true));
    assert_eq!(obj.as_object().unwrap().len(), 3);
    assert_eq!(obj.as_object().unwrap().get("c").unwrap().as_bool(), Some(true));
    obj.as_object_mut_unchecked().insert("d", Value16::int(4)); // unchecked path
    assert_eq!(obj.as_object().unwrap().len(), 4);
}

#[test]
fn set_and_map_access() {
    let s = Value16::set(vec![Value16::int(1), Value16::int(2)]);
    assert_eq!(s.as_set().expect("set value").len(), 2); assert_eq!(s.as_set().unwrap()[1].as_int(), Some(2));
    let mut s = s;
    s.as_set_mut().unwrap().push(Value16::int(3));
    assert_eq!(s.as_set().unwrap().len(), 3);
    let m = Value16::map(vec![(Value16::string("k"), Value16::int(5))]);
    let mp = m.as_map_pairs().expect("map value");
    assert_eq!(mp.len(), 1); assert_eq!(mp[0].0.as_str(), Some("k")); assert_eq!(mp[0].1.as_int(), Some(5));
    let mut m = m;
    m.as_map_mut().unwrap().push((Value16::string("j"), Value16::int(6)));
    assert_eq!(m.as_map_pairs().unwrap().len(), 2);
    assert_eq!(m.as_map_pairs().unwrap()[1].1.as_int(), Some(6));
}

// ── Option and Result wrappers ──────────────────────────────────────────────

#[test]
fn option_and_result_access() {
    let some = Value16::option(Some(Value16::int(3)));
    assert_eq!(some.as_option(), Some(Some(&Value16::int(3))));
    assert_eq!(Value16::option(None).as_option(), Some(None));
    let mut some = some;
    *some.as_option_mut().unwrap() = Some(Box::new(Value16::bool_(true)));
    assert_eq!(some.as_option(), Some(Some(&Value16::bool_(true))));

    let ok = Value16::result(Ok(Value16::int(5)));
    assert_eq!(ok.as_result(), Some(Ok(&Value16::int(5))));
    let mut err = Value16::result(Err("boom".to_string()));
    assert_eq!(err.as_result(), Some(Err(&"boom".to_string())));
    *err.as_result_mut().unwrap() = Ok(Box::new(Value16::int(7)));
    assert_eq!(err.as_result(), Some(Ok(&Value16::int(7))));
    // Result is not an Option and vice versa.
    assert!(ok.as_option().is_none()); assert!(some.as_result().is_none());
}

// ── BigInt: demotion, exact access, f64 conversion ─────────────────────────

#[test]
fn bigint_demotion_and_access() {
    // Small BigInts demote to the Int tag at construction.
    let small = Value16::bigint(BigInt::from(5));
    assert!(small.is_int()); assert!(!small.is_bigint()); assert_eq!(small.as_int(), Some(5));
    // bigint_no_demote keeps the BigInt tag even when the value fits i64.
    let kept = Value16::bigint_no_demote(BigInt::from(5));
    assert!(kept.is_bigint()); assert_eq!(kept.as_int(), Some(5)); // to_i64 fallback path
    assert_eq!(kept.as_bigint(), Some(&BigInt::from(5))); assert_eq!(kept.as_bigint_unchecked(), &BigInt::from(5));
    // 2^70 exceeds i64: stays BigInt and converts exactly to f64 (power of 2).
    let big = Value16::bigint(BigInt::from(1u128 << 70));
    assert!(big.is_bigint()); assert_eq!(big.as_int(), None);
    assert_eq!(big.as_number(), Some((1u128 << 70) as f64));
    assert_eq!(big.as_bigint().unwrap().to_string(), "1180591620717411303424");
    assert_eq!(big.as_bigint_unchecked().to_string(), "1180591620717411303424");
    // to_bigint_value converts Ints directly and clones BigInts.
    assert_eq!(Value16::int(-7).to_bigint_value(), Some(BigInt::from(-7)));
    assert_eq!(big.to_bigint_value(), Some(BigInt::from(1u128 << 70)));
    assert_eq!(Value16::string("5").to_bigint_value(), None); assert!(Value16::int(5).as_bigint().is_none());
}

// ── Function values: data access and pointer identity ───────────────────────

fn fn_data(name: &str, sym: u32) -> FunctionData {
    FunctionData {
        name: name.to_string(), params: vec!["a".to_string(), "b".to_string()],
        chunk_name: format!("{name}_chunk"), chunk_sym: sym, captures: HashMap::new(),
    }
}

#[test]
fn function_identity_and_access() {
    let f = Value16::function(fn_data("add", 77));
    assert!(f.is_function());
    assert!(!Value16::int(1).is_function()); assert!(!Value16::string("aaaaaaaaaaaaaaaa").is_function());
    let d = f.as_function_data().unwrap();
    assert_eq!(d.name, "add"); assert_eq!(d.chunk_name, "add_chunk"); assert_eq!(d.chunk_sym, 77);
    assert_eq!(d.params, vec!["a".to_string(), "b".to_string()]);
    // Copies share heap identity; distinct values have distinct identities.
    let copy = f;
    assert_ne!(copy.get_func_ptr(), 0); assert_eq!(copy.get_func_ptr(), f.get_func_ptr());
    let g = Value16::function(fn_data("sub", 1));
    assert_ne!(f.get_func_ptr(), g.get_func_ptr()); assert_eq!(Value16::int(1).get_func_ptr(), 0);
    // Raw FunctionData pointer matches the borrowed view.
    let p1 = f.as_function_data_ptr().unwrap() as usize;
    let p2 = f.as_function_data().unwrap() as *const FunctionData as usize;
    assert_eq!(p1, p2);
}

// ── Instance / class / data records ─────────────────────────────────────────

#[test]
fn instance_class_and_data_access() {
    let mut inst = Value16::instance(InstanceData {
        class_name: "Point".to_string(), fields: ObjMap::with_one("x", Value16::int(1)), class: Value16::null(),
    });
    let i = inst.as_instance_data().unwrap();
    assert_eq!(i.class_name, "Point"); assert_eq!(i.fields.len(), 1);
    assert_eq!(i.fields.get("x").unwrap().as_int(), Some(1));
    inst.as_instance_mut().unwrap().fields.insert("y", Value16::int(2));
    assert_eq!(inst.as_instance_data().unwrap().fields.len(), 2);

    let cls = Value16::class(ClassData {
        name: "Point".to_string(), methods: ObjMap::with_one("run", Value16::null()), fields: ObjMap::new(),
        parent: None, vtable: ObjMap::new(), method_access: HashMap::new(), is_abstract: true,
    });
    let c = cls.as_class_data().unwrap();
    assert_eq!(c.name, "Point"); assert_eq!(c.methods.len(), 1);
    assert!(c.is_abstract); assert!(c.parent.is_none());

    let mut dat = Value16::data(DataData { type_name: "Pair".to_string(), fields: ObjMap::with_one("l", Value16::int(1)) });
    assert_eq!(dat.as_data_data().unwrap().type_name, "Pair");
    assert_eq!(dat.as_data_data().unwrap().fields.len(), 1);
    dat.as_data_mut().unwrap().fields.insert("r", Value16::int(2));
    assert_eq!(dat.as_data_data().unwrap().fields.len(), 2);
    // Cross-kind rejects.
    assert!(cls.as_instance_data().is_none()); assert!(inst.as_class_data().is_none()); assert!(Value16::int(1).as_data_data().is_none());
}

// ── Promise / generator / tool / resource ───────────────────────────────────

#[test]
fn promise_generator_tool_resource_access() {
    use hudhudscript_bytecode::PromiseState16 as P;
    assert_eq!(Value16::promise(P::Pending).as_promise_state(), Some(&P::Pending));
    assert_eq!(Value16::promise(P::Resolved(Box::new(Value16::int(9)))).as_promise_state(),
        Some(&P::Resolved(Box::new(Value16::int(9)))));
    assert_eq!(Value16::promise(P::Rejected("boom".to_string())).as_promise_state(),
        Some(&P::Rejected("boom".to_string())));
    let (_tx, rx) = mpsc::channel::<Value16>();
    let gen = Value16::generator(Arc::new(Mutex::new(GeneratorState16::new(rx))));
    let mut guard = gen.as_generator_state().expect("generator state").lock();
    assert!(guard.pending.is_empty()); assert!(guard.buffered.is_empty());
    assert!(!guard.is_done()); assert!(guard.yield_id.is_none());
    guard.pending.push_back(Value16::int(5)); drop(guard);
    assert_eq!(gen.as_generator_state().unwrap().lock().pending.len(), 1);
    let tool = Value16::tool(ToolRef { server: "srv".to_string(), tool_name: "echo".to_string() });
    assert_eq!(tool.as_tool_ref().unwrap().server, "srv"); assert_eq!(tool.as_tool_ref().unwrap().tool_name, "echo");
    let rsrc = Value16::resource(ResourceRef { server: "srv".to_string(), uri: "mem://x".to_string() });
    assert_eq!(rsrc.as_resource_ref().unwrap().server, "srv"); assert_eq!(rsrc.as_resource_ref().unwrap().uri, "mem://x");
}

// ── Equality semantics (PartialEq is values_equal) ─────────────────────────

#[test]
fn equality_scalars_and_cross_type_numerics() {
    assert_eq!(Value16::int(5), Value16::int(5)); assert_ne!(Value16::int(5), Value16::int(6));
    assert_eq!(Value16::number(2.5), Value16::number(2.5));
    assert_eq!(Value16::bool_(true), Value16::bool_(true));
    assert_ne!(Value16::bool_(true), Value16::bool_(false));
    assert_eq!(Value16::null(), Value16::null());
    // Cross-type numeric equality (JS/Lua style): 5 == 5.0.
    assert_eq!(Value16::int(5), Value16::number(5.0)); assert_eq!(Value16::number(5.0), Value16::int(5));
    assert_ne!(Value16::int(5), Value16::number(5.5));
    // Bool never equals Int; null never equals false; strings never coerce.
    assert_ne!(Value16::bool_(true), Value16::int(1)); assert_ne!(Value16::null(), Value16::bool_(false));
    assert_ne!(Value16::int(5), Value16::string("5"));
    // IEEE semantics through the value layer: -0.0 == 0.0, NaN != NaN.
    assert_eq!(Value16::number(-0.0), Value16::number(0.0));
    assert_ne!(Value16::number(f64::NAN), Value16::number(f64::NAN));
}

#[test]
fn equality_strings_and_collections() {
    // Inline vs heap strings with equal content compare equal.
    let mut heap = Value16::string("aaaaaaaaaaaaaaaa"); // 16 bytes -> heap
    assert_eq!(heap.as_string_mut().unwrap().pop(), Some('a')); // 15 bytes, still heap-resident
    assert_eq!(heap.as_str(), Some("aaaaaaaaaaaaaaa")); assert_eq!(heap, Value16::string("aaaaaaaaaaaaaaa")); // inline twin
    assert_ne!(heap, Value16::string("aaaaaaaaaaaaaaab"));
    // Arrays: element-wise, order-sensitive, length-sensitive, recursive.
    let a1 = Value16::array(vec![Value16::int(1), Value16::int(2)]);
    assert_eq!(a1, Value16::array(vec![Value16::int(1), Value16::int(2)]));
    assert_ne!(a1, Value16::array(vec![Value16::int(2), Value16::int(1)]));
    assert_ne!(a1, Value16::array(vec![Value16::int(1)]));
    let nested = Value16::array(vec![Value16::array(vec![Value16::int(1)]), Value16::int(2)]);
    assert_eq!(nested, Value16::array(vec![Value16::array(vec![Value16::int(1)]), Value16::int(2)]));
    assert_ne!(nested, Value16::array(vec![Value16::array(vec![Value16::int(9)]), Value16::int(2)]));
    // Sets are order-insensitive; maps compare as unordered pair sets.
    assert_eq!(Value16::set(vec![Value16::int(1), Value16::int(2)]), Value16::set(vec![Value16::int(2), Value16::int(1)]));
    assert_ne!(Value16::set(vec![Value16::int(1)]), Value16::set(vec![Value16::int(1), Value16::int(2)]));
    let m = Value16::map(vec![(Value16::string("a"), Value16::int(1))]);
    assert_eq!(m, Value16::map(vec![(Value16::string("a"), Value16::int(1))]));
    assert_ne!(m, Value16::map(vec![(Value16::string("a"), Value16::int(2))]));
    // Objects compare by key set + values.
    assert_eq!(Value16::object(vec![("k", Value16::int(1))]), Value16::object(vec![("k", Value16::int(1))]));
    assert_ne!(Value16::object(vec![("k", Value16::int(1))]), Value16::object(vec![("k", Value16::int(1)), ("j", Value16::int(2))]));
    // Cross-kind dynamic values are never equal.
    assert_ne!(a1, Value16::set(vec![Value16::int(1), Value16::int(2)]));
    assert_ne!(a1, Value16::object(vec![("1", Value16::int(1)), ("2", Value16::int(2))]));
    // BigInts compare by numeric value.
    assert_eq!(Value16::bigint(BigInt::from(2u128 << 70)), Value16::bigint(BigInt::from(2u128 << 70)));
    assert_ne!(Value16::bigint(BigInt::from(2u128 << 70)), Value16::bigint(BigInt::from((2u128 << 70) + 1)));
}

// ── Wrong-kind rejection sweep across every dynamic accessor ───────────────

#[test]
fn wrong_kind_accessors_all_reject() {
    let mut n = Value16::int(42);
    assert!(n.as_str().is_none()); assert!(n.as_string().is_none()); assert!(n.as_string_mut().is_none());
    assert!(!n.is_dynamic_string_ascii()); assert!(n.str_char_len().is_none());
    assert!(n.as_array().is_none()); assert!(n.as_array_mut().is_none()); assert!(n.array_get(0).is_none());
    assert!(!n.array_set(0, Value16::null()));
    assert!(n.as_object().is_none()); assert!(n.as_object_mut().is_none());
    assert!(n.as_set().is_none()); assert!(n.as_set_mut().is_none());
    assert!(n.as_map_pairs().is_none()); assert!(n.as_map_mut().is_none());
    assert!(n.as_option().is_none()); assert!(n.as_option_mut().is_none());
    assert!(n.as_result().is_none()); assert!(n.as_result_mut().is_none());
    assert!(n.as_function_data().is_none()); assert_eq!(n.get_func_ptr(), 0);
    assert!(n.as_instance_data().is_none()); assert!(n.as_instance_mut().is_none());
    assert!(n.as_promise_state().is_none()); assert!(n.as_generator_state().is_none());
    assert!(n.as_class_data().is_none()); assert!(n.as_data_data().is_none()); assert!(n.as_data_mut().is_none());
    assert!(n.as_tool_ref().is_none()); assert!(n.as_resource_ref().is_none());
    // Unchecked accessors are only ever called on proven kinds (see the
    // array/string/function sections above); calling them on an Int would
    // reinterpret the raw payload as a heap pointer, so they are excluded.

    let mut s = Value16::string("aaaaaaaaaaaaaaaa");
    assert!(s.as_array().is_none()); assert!(s.as_array_mut().is_none());
    assert!(!s.array_set(0, Value16::null())); assert!(s.array_get(0).is_none());
    assert!(s.as_object().is_none()); assert!(s.as_object_mut().is_none());
    assert!(s.as_set().is_none()); assert!(s.as_set_mut().is_none());
    assert!(s.as_map_pairs().is_none()); assert!(s.as_map_mut().is_none());
    assert!(s.as_option().is_none()); assert!(s.as_result().is_none());
    assert!(s.as_option_mut().is_none()); assert!(s.as_result_mut().is_none());
    assert!(s.as_bigint().is_none()); assert!(s.as_function_data().is_none());
    assert!(!s.is_function()); assert_eq!(s.str_char_len(), Some(16));
    assert!(s.as_instance_data().is_none()); assert!(s.as_instance_mut().is_none());
    assert!(s.as_promise_state().is_none()); assert!(s.as_class_data().is_none());
    assert!(s.as_data_data().is_none()); assert!(s.as_data_mut().is_none()); assert!(s.as_generator_state().is_none());
    assert!(s.as_tool_ref().is_none()); assert!(s.as_resource_ref().is_none());
    assert!(s.is_truthy()); // non-empty heap string
}
