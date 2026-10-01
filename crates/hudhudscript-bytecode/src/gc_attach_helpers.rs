//! Attach-side helpers (`resolve`/`attach_placeholder`/`attach_ref_node`/
//! `attach_leaf`) split from gc_detach.rs.

use crate::gc_detach::OwnedTree;
use crate::{DynamicData, DynamicKind, ObjMap, SymId, Value16};

pub(super) fn resolve(t: &OwnedTree, nodes: &Vec<OwnedTree>, pool: &Vec<Value16>) -> Value16 {
    match t {
        OwnedTree::Ref(idx) => pool[*idx],
        other => attach_leaf(other, nodes, pool),
    }
}

/// Create a preliminary node with correct DynamicKind, children uninitialized.
pub(super) fn attach_placeholder(tree: &OwnedTree) -> Value16 {
    match tree {
        OwnedTree::Array(_) => Value16::array(vec![]),
        OwnedTree::Object(_) => Value16::object(ObjMap::default()),
        OwnedTree::Set(_) => Value16::set(vec![]),
        OwnedTree::Map(_) => crate::gc::alloc(DynamicKind::Map, DynamicData::Map(vec![])),
        OwnedTree::Option(_) => Value16::option(None::<Value16>),
        OwnedTree::Ok(_) => Value16::result(Ok(Value16::null())),
        OwnedTree::Err(s) => Value16::result(Err(s.clone())),
        OwnedTree::Data { type_name, .. } => {
            let data = crate::DataData {
                type_name: type_name.clone(),
                fields: ObjMap::default(),
            };
            Value16::data(data)
        }
        OwnedTree::Instance { class_name, .. } => {
            let inst = crate::InstanceData {
                class_name: class_name.clone(),
                fields: ObjMap::default(),
                class: Value16::null(),
            };
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
            for (k, t) in fields {
                map.insert(SymId::from(k.as_str()), r(t));
            }
            Value16::object(map)
        }
        OwnedTree::Set(items) => Value16::set(items.iter().map(|t| r(t)).collect()),
        OwnedTree::Map(pairs) => {
            let vals: Vec<(Value16, Value16)> = pairs.iter().map(|(k, v)| (r(k), r(v))).collect();
            crate::gc::alloc(DynamicKind::Map, DynamicData::Map(vals))
        }
        OwnedTree::Option(opt) => {
            if let Some(boxed) = opt {
                Value16::option(Some(r(boxed)))
            } else {
                Value16::option(None::<Value16>)
            }
        }
        OwnedTree::Ok(boxed) => Value16::result(Ok(r(boxed))),
        OwnedTree::Err(s) => Value16::result(Err(s.clone())),
        OwnedTree::Data { type_name, fields } => {
            let mut m = ObjMap::default();
            for (k, t) in fields {
                m.insert(SymId::from(k.as_str()), r(t));
            }
            Value16::data(crate::DataData {
                type_name: type_name.clone(),
                fields: m,
            })
        }
        OwnedTree::Instance { class_name, fields } => {
            let mut m = ObjMap::default();
            for (k, t) in fields {
                m.insert(SymId::from(k.as_str()), r(t));
            }
            Value16::instance(crate::InstanceData {
                class_name: class_name.clone(),
                fields: m,
                class: Value16::null(),
            })
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
            for (k, t) in fields {
                map.insert(SymId::from(k.as_str()), r(t));
            }
            Value16::object(map)
        }
        OwnedTree::Set(items) => Value16::set(items.iter().map(|t| r(t)).collect()),
        OwnedTree::Map(pairs) => {
            let vals: Vec<(Value16, Value16)> = pairs.iter().map(|(k, v)| (r(k), r(v))).collect();
            crate::gc::alloc(DynamicKind::Map, DynamicData::Map(vals))
        }
        OwnedTree::Option(opt) => {
            if let Some(boxed) = opt {
                Value16::option(Some(r(boxed)))
            } else {
                Value16::option(None::<Value16>)
            }
        }
        OwnedTree::Ok(boxed) => Value16::result(Ok(r(boxed))),
        OwnedTree::Err(s) => Value16::result(Err(s.clone())),
        OwnedTree::Data { type_name, fields } => {
            let mut m = ObjMap::default();
            for (k, t) in fields {
                m.insert(SymId::from(k.as_str()), r(t));
            }
            Value16::data(crate::DataData {
                type_name: type_name.clone(),
                fields: m,
            })
        }
        OwnedTree::Instance { class_name, fields } => {
            let mut m = ObjMap::default();
            for (k, t) in fields {
                m.insert(SymId::from(k.as_str()), r(t));
            }
            Value16::instance(crate::InstanceData {
                class_name: class_name.clone(),
                fields: m,
                class: Value16::null(),
            })
        }
        OwnedTree::Ref(idx) => pool[*idx],
    }
}
