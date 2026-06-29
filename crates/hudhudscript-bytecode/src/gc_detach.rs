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
use crate::{ObjMap, SymId};
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
                    Ok(boxed) => OwnedTree::Array(vec![detach_value(**boxed, graph, seen)?]),
                    Err(s) => OwnedTree::String(format!("Err({})", s)),
                }
            }
            DynamicData::Data(data) => {
                let fields: Vec<(String, OwnedTree)> = data.fields.iter()
                    .map(|(k, v)| Ok((k.to_string(), detach_value(*v, graph, seen)?)))
                    .collect::<Result<_, ErrorCode>>()?;
                OwnedTree::Object(fields)
            }
            DynamicData::Instance(inst) => {
                let fields: Vec<(String, OwnedTree)> = inst.fields.iter()
                    .map(|(k, v)| Ok((k.to_string(), detach_value(*v, graph, seen)?)))
                    .collect::<Result<_, ErrorCode>>()?;
                OwnedTree::Object(fields)
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
    let mut visiting = std::collections::HashSet::new();
    attach_tree(
        &graph.nodes[graph.root_idx], &graph.nodes, &mut pool, &mut visiting)
}

fn attach_tree(
    tree: &OwnedTree,
    nodes: &Vec<OwnedTree>,
    pool: &mut Vec<Value16>,
    visiting: &mut std::collections::HashSet<usize>,
) -> Value16 {
    match tree {
        OwnedTree::Null => Value16::null(),
        OwnedTree::Bool(b) => Value16::bool_(*b),
        OwnedTree::Int(i) => Value16::int(*i),
        OwnedTree::Float(f) => Value16::number(*f),
        OwnedTree::String(s) => Value16::string(s.clone()),
        OwnedTree::BigInt(s) => Value16::bigint(num_bigint::BigInt::from_signed_bytes_le(s)),
        OwnedTree::Array(items) => {
            let values: Vec<Value16> = items
                .iter()
                .map(|t| attach_tree(t, nodes, pool, visiting))
                .collect();
            Value16::array(values)
        }
        OwnedTree::Object(fields) => {
            let mut map = ObjMap::default();
            for (k, v) in fields { map.insert(SymId::from(k.as_str()), attach_tree(v, nodes, pool, visiting)); }
            Value16::object(map)
        }
        OwnedTree::Set(items) => {
            let values: Vec<Value16> = items
                .iter()
                .map(|t| attach_tree(t, nodes, pool, visiting))
                .collect();
            crate::gc::alloc(DynamicKind::Set, DynamicData::Set(values))
        }
        OwnedTree::Map(pairs) => {
            let values: Vec<(Value16, Value16)> = pairs
                .iter()
                .map(|(k, v)| (attach_tree(k, nodes, pool, visiting), attach_tree(v, nodes, pool, visiting)))
                .collect();
            crate::gc::alloc(DynamicKind::Map, DynamicData::Map(values))
        }
        OwnedTree::Option(opt) => {
            if let Some(boxed) = opt {
                Value16::option(Some(attach_tree(boxed, nodes, pool, visiting)))
            } else {
                Value16::option(None::<Value16>)
            }
        }
        OwnedTree::Ref(idx) => {
            if let Some(v) = pool.get(*idx) {
                if !v.is_null() {
                    return *v;
                }
            }
            if !visiting.insert(*idx) {
                // Cycle: the target is already being constructed. Return null to break the loop.
                // Full cycle preservation requires a two-pass attach implementation (G2-A3).
                return Value16::null();
            }
            let node = nodes.get(*idx).expect("Invalid Ref index");
            let built = attach_tree(node, nodes, pool, visiting);
            if *idx < pool.len() {
                pool[*idx] = built;
            }
            visiting.remove(idx);
            built
        }
    }
}


