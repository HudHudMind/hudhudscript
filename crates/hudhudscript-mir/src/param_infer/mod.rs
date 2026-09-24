//! Parametre ve modül tip çıkarımı (kullanım ve çağrı-tabanlı).
//!
//! Fonksiyon içi ipuçları (ArrayStore, push/pop vs. substring) ve
//! modül genelindeki çağrı noktalarından (call-site) argüman tipleri toplanarak
//! parametre, global ve dönüş tipleri belirlenir.

use std::collections::{HashMap, HashSet};
use hudhudscript_types::{HirFunction, HirModule};
use crate::mir::{MirType, RefKind};

mod call_hints;
mod elem_infer;
mod globals;
mod usages;

pub use elem_infer::infer_module_array_elem_types;
pub use globals::{infer_module_globals, infer_module_return_types, infer_module_return_types_with_hints};

pub fn infer_param_types(f: &HirFunction) -> HashMap<String, MirType> {
    let empty_hints = HashMap::new();
    infer_func_params(f, &empty_hints)
}

pub fn infer_module_param_types(module: &HirModule) -> HashMap<String, HashMap<String, MirType>> {
    let globals = infer_module_globals(module);
    let mut return_tys = infer_module_return_types(module);
    let mut all: HashMap<String, HashMap<String, MirType>> = HashMap::new();
    for _ in 0..5 {
        let call_hints = call_hints::collect_call_hints(module, &globals, &return_tys, &all);
        let mut changed = false;
        for (name, f) in &module.functions {
            let f_hints = call_hints.get(name.as_str());
            let empty = HashMap::new();
            let new_params = infer_func_params(f, f_hints.unwrap_or(&empty));
            if all.get(name) != Some(&new_params) {
                all.insert(name.clone(), new_params);
                changed = true;
            }
        }
        return_tys = globals::infer_module_return_types_with_hints(module, &all);
        if !changed {
            break;
        }
    }
    all
}

fn infer_func_params(
    f: &HirFunction,
    call_hints: &HashMap<usize, HashSet<MirType>>,
) -> HashMap<String, MirType> {
    let params: HashSet<String> = f.params.iter().map(|p| p.name.clone()).collect();
    let mut usages: HashMap<String, usages::ParamUsage> = HashMap::new();
    for p in &f.params {
        usages.insert(p.name.clone(), usages::ParamUsage::default());
    }
    usages::collect_usages_in_stmts(&f.body, &params, &mut usages);

    let mut out = HashMap::new();
    for (idx, p) in f.params.iter().enumerate() {
        let u = usages.get(&p.name).unwrap();
        let hint_ty = call_hints.get(&idx).and_then(|set| {
            if set.len() == 1 {
                set.iter().next().copied()
            } else {
                None
            }
        });

        let ty = if u.mutated_as_array || u.array_method {
            MirType::Ref(RefKind::Array)
        } else if u.string_method {
            MirType::Ref(RefKind::String)
        } else if u.used_as_object {
            MirType::Ref(RefKind::Object)
        } else if u.read_indexed {
            if let Some(h) = hint_ty {
                h
            } else {
                MirType::Generic
            }
        } else if let Some(h) = hint_ty {
            h
        } else {
            MirType::I64
        };
        out.insert(p.name.clone(), ty);
    }
    out
}
