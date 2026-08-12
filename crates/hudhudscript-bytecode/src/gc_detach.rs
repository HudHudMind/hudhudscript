//! V2-C1: Thread-safe value transfer — detach from source heap, attach to caller's heap.
//!
//! `gc::detach` + `gc::attach` = exhaustive, cycle-safe, preserves object identity sharing.
//!
//! G3-1a: detach returns Result — unsupported types produce catalog error E0311.
//!
//! G2 design: values are represented as a `DetachedGraph` where each dynamic object
//! lives exactly once in `nodes` and is referenced by `OwnedTree::Ref(idx)`.  This
//! eliminates the previous root/pool clone overhead.

use crate::{DynamicData, DynamicKind, DynamicObject, Value16};
use hudhudscript_errors::catalog::ErrorCode;
use crate::{DataData, InstanceData, ObjMap, SymId};
use std::collections::HashMap;

/// Error for unsupported thread transfer types (G3-1a).
const E_UNSUPPORTED: ErrorCode = ErrorCode(311);

// ── OwnedTree: heap-independent value representation ──────────────

/// Heap-independent owned value graph. Can be sent across threads safely
/// because it owns all its data (no pointers into any GC heap).
#[derive(Debug, Clone)]
pub enum OwnedTree {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    BigInt(Vec<u8>),
    Array(Vec<OwnedTree>),
    Object(Vec<(String, OwnedTree)>),
    Set(Vec<OwnedTree>),
    Map(Vec<(OwnedTree, OwnedTree)>),
    Option(Option<Box<OwnedTree>>),
    /// V2-C2: Preserve Result type through detach/attach round-trip.
    Ok(Box<OwnedTree>),
    Err(String),
    /// V2-C2: Preserve Data type (tagged struct) through round-trip.
    Data { type_name: String, fields: Vec<(String, OwnedTree)> },
    /// V2-C2: Preserve Instance type through round-trip.
    Instance { class_name: String, fields: Vec<(String, OwnedTree)> },
    Ref(usize),
}

/// Detached value graph: each dynamic object lives exactly once in `nodes`,
/// referenced by `Ref(idx)`. The root value is at `nodes[root_idx]`.
#[derive(Debug, Clone, Default)]
pub struct DetachedGraph {
    pub root_idx: usize,
    pub nodes: Vec<OwnedTree>,
}

// ── detach: extract from current heap ──────────────────────────────

/// Move a value OUT of the current thread's GC heap into a
/// `DetachedGraph`. Cycle-safe via `seen` map.
/// Returns Err for types that cannot be transferred (Function, Class,
/// Generator, Tool, Resource — G3-1a).
pub fn detach(value: Value16) -> Result<DetachedGraph, ErrorCode> {
    let mut graph = DetachedGraph::default();
    let mut seen: HashMap<*const DynamicObject, usize> = HashMap::new();
    let root = detach_value(value, &mut graph, &mut seen)?;

    if let OwnedTree::Ref(idx) = root {
        graph.root_idx = idx;
    } else {
        // Primitive roots never got a pool node; store them explicitly.
        graph.root_idx = graph.nodes.len();
        graph.nodes.push(root);
    }
    Ok(graph)
}

fn detach_value(
    value: Value16,
    graph: &mut DetachedGraph,
    seen: &mut HashMap<*const DynamicObject, usize>,
) -> Result<OwnedTree, ErrorCode> {
    if let Some(s) = value.as_str() {
        return Ok(OwnedTree::String(s.to_string()));
    }
    if value.is_null() {
        return Ok(OwnedTree::Null);
    }
    if let Some(b) = value.as_bool() {
        return Ok(OwnedTree::Bool(b));
    }
    if let Some(i) = value.as_int() {
        return Ok(OwnedTree::Int(i));
    }
    if let Some(b) = value.as_bigint() {
        return Ok(OwnedTree::BigInt(b.to_signed_bytes_le()));
    }
    if let Some(n) = value.as_number() {
        return Ok(OwnedTree::Float(n));
    }
    if let Some(ptr) = value.0.as_ptr() {
        let obj = unsafe { &*(ptr as *const DynamicObject) };
        let raw = obj as *const DynamicObject;
        if let Some(&idx) = seen.get(&raw) {
            return Ok(OwnedTree::Ref(idx));
        }

        let idx = graph.nodes.len();
        graph.nodes.push(OwnedTree::Null);
        seen.insert(raw, idx);

        let tree = match &obj.data {
            DynamicData::String(s) => OwnedTree::String(s.clone()),
            DynamicData::Array(arr) => {
                let mut items = Vec::with_capacity(arr.len());
                for v in arr { items.push(detach_value(*v, graph, seen)?); }
                OwnedTree::Array(items)
            }
            DynamicData::Object(map) => {
                let mut fields = Vec::with_capacity(map.len());
                for (k, v) in map { fields.push((k.to_string(), detach_value(*v, graph, seen)?)); }
                OwnedTree::Object(fields)
            }
            DynamicData::Set(vec) => {
                let mut items = Vec::new();
                for v in vec { items.push(detach_value(*v, graph, seen)?); }
                OwnedTree::Set(items)
            }
            DynamicData::Map(pairs) => {
                let mut out = Vec::with_capacity(pairs.len());
                for (k, v) in pairs {
                    out.push((detach_value(*k, graph, seen)?, detach_value(*v, graph, seen)?));
                }
                OwnedTree::Map(out)
            }
            DynamicData::BigInt(b) => OwnedTree::BigInt(b.to_signed_bytes_le()),
            DynamicData::Option(opt) => {
                OwnedTree::Option(
                    opt.as_ref()
                        .map(|b| detach_value(**b, graph, seen))
                        .transpose()?
                        .map(Box::new),
                )
            }
            DynamicData::Result(res) => {
                match res.as_ref() {
                    Ok(boxed) => OwnedTree::Ok(Box::new(detach_value(**boxed, graph, seen)?)),
                    Err(s) => OwnedTree::Err(s.clone()),
                }
            }
            DynamicData::Data(data) => {
                let fields: Vec<(String, OwnedTree)> = data.fields.iter()
                    .map(|(k, v)| Ok((k.to_string(), detach_value(*v, graph, seen)?)))
                    .collect::<Result<_, ErrorCode>>()?;
                OwnedTree::Data { type_name: data.type_name.clone(), fields }
            }
            DynamicData::Instance(inst) => {
                let fields: Vec<(String, OwnedTree)> = inst.fields.iter()
                    .map(|(k, v)| Ok((k.to_string(), detach_value(*v, graph, seen)?)))
                    .collect::<Result<_, ErrorCode>>()?;
                OwnedTree::Instance { class_name: inst.class_name.clone(), fields }
            }
            DynamicData::Function(_)
            | DynamicData::Promise(_)
            | DynamicData::Class(_)
            | DynamicData::Generator(_)
            | DynamicData::Tool(_)
            | DynamicData::Resource(_) => {
                return Err(E_UNSUPPORTED);
            }
        };
        graph.nodes[idx] = tree;
        Ok(OwnedTree::Ref(idx))
    } else {
        Ok(OwnedTree::Null)
    }
}

// ── attach: allocate on caller's heap ──────────────────────────────

pub fn attach(graph: &DetachedGraph) -> Value16 {
    let mut pool = vec![Value16::null(); graph.nodes.len()];
    // Two-pass attach (V2-C2):
    // Pass 1: create preliminary nodes with correct DynamicKind
    // Pass 2: rebuild each Ref node with resolved children, replacing pool entries
    // Cycles are preserved — a node may reference itself or its ancestors.
    for (idx, node) in graph.nodes.iter().enumerate() {
        pool[idx] = attach_placeholder(node);
    }
    // Pass 2: fill children into pre-allocated node for EVERY aggregate node.
    // Node table entries are always aggregate types (not Ref — only children are Ref).
    for idx in 0..graph.nodes.len() {
        let node = &graph.nodes[idx];
        match node {
            OwnedTree::Array(items) => {
                let children: Vec<Value16> = items.iter().map(|t| resolve(t, &graph.nodes, &pool)).collect();
                if let Some(dst) = pool[idx].as_array_mut() { *dst = children; }
            }
            OwnedTree::Object(fields) => {
                let mut map = ObjMap::default();
                for (k, t) in fields { map.insert(SymId::from(k.as_str()), resolve(t, &graph.nodes, &pool)); }
                if let Some(dst) = pool[idx].as_object_mut() { *dst = map; }
            }
            OwnedTree::Set(items) => {
                let children: Vec<Value16> = items.iter().map(|t| resolve(t, &graph.nodes, &pool)).collect();
                drop(items);
                if let Some(v) = pool[idx].as_set_mut() { *v = children; }
            }
            OwnedTree::Map(pairs) => {
                let vals: Vec<(Value16, Value16)> = pairs.iter()
                    .map(|(k, v)| (resolve(k, &graph.nodes, &pool), resolve(v, &graph.nodes, &pool))).collect();
                drop(pairs);
                if let Some(v) = pool[idx].as_map_mut() { *v = vals; }
            }
            OwnedTree::Option(Some(boxed)) => {
                let val = resolve(boxed, &graph.nodes, &pool);
                if let Some(v) = pool[idx].as_option_mut() { *v = Some(Box::new(val)); }
            }
            OwnedTree::Ok(boxed) => {
                let val = resolve(boxed, &graph.nodes, &pool);
                if let Some(v) = pool[idx].as_result_mut() { *v = Ok(Box::new(val)); }
            }
            OwnedTree::Err(_) => {}
            OwnedTree::Data { fields, .. } => {
                let pairs: Vec<(SymId, Value16)> = fields.iter()
                    .map(|(k, t)| (SymId::from(k.as_str()), resolve(t, &graph.nodes, &pool))).collect();
                if let Some(d) = pool[idx].as_data_mut() {
                    d.fields = pairs.into_iter().collect();
                }
            }
            OwnedTree::Instance { fields, .. } => {
                let pairs: Vec<(SymId, Value16)> = fields.iter()
                    .map(|(k, t)| (SymId::from(k.as_str()), resolve(t, &graph.nodes, &pool))).collect();
                if let Some(i) = pool[idx].as_instance_mut() {
                    i.fields = pairs.into_iter().collect();
                }
            }
            _ => { /* primitive/leaf node: already correct */ }
        }
    }
    // Return root from pool
    pool[graph.root_idx]
}

fn resolve(t: &OwnedTree, nodes: &Vec<OwnedTree>, pool: &Vec<Value16>) -> Value16 {
    match t {
        OwnedTree::Ref(idx) => pool[*idx],
        other => attach_leaf(other, nodes, pool),
    }
}

/// Create a preliminary node with correct DynamicKind, children uninitialized.
fn attach_placeholder(tree: &OwnedTree) -> Value16 {
    match tree {
        OwnedTree::Array(_) => Value16::array(vec![]),
        OwnedTree::Object(_) => Value16::object(ObjMap::default()),
        OwnedTree::Set(_) => Value16::set(vec![]),
        OwnedTree::Map(_) => crate::gc::alloc(DynamicKind::Map, DynamicData::Map(vec![])),
        OwnedTree::Option(_) => Value16::option(None::<Value16>),
        OwnedTree::Ok(_) => Value16::result(Ok(Value16::null())),
        OwnedTree::Err(s) => Value16::result(Err(s.clone())),
        OwnedTree::Data { type_name, .. } => {
            let data = crate::DataData { type_name: type_name.clone(), fields: ObjMap::default() };
            Value16::data(data)
        }
        OwnedTree::Instance { class_name, .. } => {
            let inst = crate::InstanceData { class_name: class_name.clone(), fields: ObjMap::default(), class: Value16::null() };
            Value16::instance(inst)
        }
        _ => attach_leaf(tree, &vec![], &vec![]),
    }
}

/// Rebuild a Ref node fully with resolved children, then replace pool entry.
fn attach_ref_node(tree: &OwnedTree, nodes: &Vec<OwnedTree>, pool: &Vec<Value16>) -> Value16 {
    let idx = match tree {
        OwnedTree::Ref(i) => *i,
        _ => return attach_leaf(tree, nodes, pool),
    };
    let node = &nodes[idx];
    let r = |t: &OwnedTree| resolve(t, nodes, pool);
    match node {
        OwnedTree::Array(items) => Value16::array(items.iter().map(|t| r(t)).collect()),
        OwnedTree::Object(fields) => {
            let mut map = ObjMap::default();
            for (k, t) in fields { map.insert(SymId::from(k.as_str()), r(t)); }
            Value16::object(map)
        }
        OwnedTree::Set(items) => Value16::set(items.iter().map(|t| r(t)).collect()),
        OwnedTree::Map(pairs) => {
            let vals: Vec<(Value16, Value16)> = pairs.iter().map(|(k, v)| (r(k), r(v))).collect();
            crate::gc::alloc(DynamicKind::Map, DynamicData::Map(vals))
        }
        OwnedTree::Option(opt) => {
            if let Some(boxed) = opt { Value16::option(Some(r(boxed))) }
            else { Value16::option(None::<Value16>) }
        }
        OwnedTree::Ok(boxed) => Value16::result(Ok(r(boxed))),
        OwnedTree::Err(s) => Value16::result(Err(s.clone())),
        OwnedTree::Data { type_name, fields } => {
            let mut m = ObjMap::default();
            for (k, t) in fields { m.insert(SymId::from(k.as_str()), r(t)); }
            Value16::data(crate::DataData { type_name: type_name.clone(), fields: m })
        }
        OwnedTree::Instance { class_name, fields } => {
            let mut m = ObjMap::default();
            for (k, t) in fields { m.insert(SymId::from(k.as_str()), r(t)); }
            Value16::instance(crate::InstanceData { class_name: class_name.clone(), fields: m, class: Value16::null() })
        }
        _ => attach_leaf(node, nodes, pool),
    }
}

/// Resolve a leaf (non-Ref) tree to a final Value16.
fn attach_leaf(tree: &OwnedTree, nodes: &Vec<OwnedTree>, pool: &Vec<Value16>) -> Value16 {
    let r = |t: &OwnedTree| resolve(t, nodes, pool);
    match tree {
        OwnedTree::Null => Value16::null(),
        OwnedTree::Bool(b) => Value16::bool_(*b),
        OwnedTree::Int(i) => Value16::int(*i),
        OwnedTree::Float(f) => Value16::number(*f),
        OwnedTree::String(s) => Value16::string(s.clone()),
        OwnedTree::BigInt(s) => Value16::bigint(num_bigint::BigInt::from_signed_bytes_le(s)),
        OwnedTree::Array(items) => Value16::array(items.iter().map(|t| r(t)).collect()),
        OwnedTree::Object(fields) => {
            let mut map = ObjMap::default();
            for (k, t) in fields { map.insert(SymId::from(k.as_str()), r(t)); }
            Value16::object(map)
        }
        OwnedTree::Set(items) => Value16::set(items.iter().map(|t| r(t)).collect()),
        OwnedTree::Map(pairs) => {
            let vals: Vec<(Value16, Value16)> = pairs.iter().map(|(k, v)| (r(k), r(v))).collect();
            crate::gc::alloc(DynamicKind::Map, DynamicData::Map(vals))
        }
        OwnedTree::Option(opt) => {
            if let Some(boxed) = opt { Value16::option(Some(r(boxed))) }
            else { Value16::option(None::<Value16>) }
        }
        OwnedTree::Ok(boxed) => Value16::result(Ok(r(boxed))),
        OwnedTree::Err(s) => Value16::result(Err(s.clone())),
        OwnedTree::Data { type_name, fields } => {
            let mut m = ObjMap::default();
            for (k, t) in fields { m.insert(SymId::from(k.as_str()), r(t)); }
            Value16::data(crate::DataData { type_name: type_name.clone(), fields: m })
        }
        OwnedTree::Instance { class_name, fields } => {
            let mut m = ObjMap::default();
            for (k, t) in fields { m.insert(SymId::from(k.as_str()), r(t)); }
            Value16::instance(crate::InstanceData { class_name: class_name.clone(), fields: m, class: Value16::null() })
        }
        OwnedTree::Ref(idx) => pool[*idx],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let v = Value16::data(crate::DataData { type_name: "Test".into(), fields: m });
        let g = detach(v).unwrap();
        let v2 = attach(&g);
        let d = v2.as_data_data().unwrap();
        assert_eq!(d.type_name, "Test");
        assert_eq!(d.fields.get(&SymId::from("x")).unwrap().as_int(), Some(1));
    }
}


