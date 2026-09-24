//! Typed HIR → MIR lowering pipeline (AŞAMA-0).
//!
//! Source params carry no annotations in the current AST, so the MIR
//! pipeline drives typed lowering from a caller-supplied type environment
//! (`param_types`). This is the bridge where future inference or JIT-tiering
//! profiles will plug in.
//!
//! §18 Contract:
//! - Pure functions only in typed lane (I64 / F64 / Ref / Generic).
//! - Math library calls (sin, cos, sqrt, floor, abs, pow, min, max) lower to CallNative.
//! - Calls between functions in the same module lower to CallStatic.
//! - Module globals lower to ObjectGet/Set on the runtime globals handle.

use std::collections::HashMap;

use hudhudscript_types::HirFunction;

use crate::builder::MirFunctionBuilder;
use crate::mir::{MirFunction, MirType};
use crate::LowerError;

pub(crate) mod binary;
pub(crate) mod builtins;
pub(crate) mod cx;
pub(crate) mod expr;
pub(crate) mod methods;
pub(crate) mod phi_helpers;
pub(crate) mod stmt;
pub(crate) mod stmt_loops;
#[cfg(test)]
mod tests;

pub(crate) use cx::FnCx;
pub(crate) use stmt::{lower_stmts, StmtFlow};

/// Lower `hir` with explicit param types (`name -> MirType`). Every param
/// of `hir` must be present in `param_types`.
pub fn lower_function_typed(
    hir: &HirFunction,
    param_types: &HashMap<String, MirType>,
) -> Result<MirFunction, LowerError> {
    lower_function_typed_in_module(hir, param_types, &HashMap::new(), &HashMap::new())
}

pub fn lower_function_typed_in_module(
    hir: &HirFunction,
    param_types: &HashMap<String, MirType>,
    module_functions: &HashMap<String, (crate::mir::FunctionId, Vec<MirType>, MirType)>,
    module_globals: &HashMap<String, (u32, MirType)>,
) -> Result<MirFunction, LowerError> {
    lower_function_typed_full(hir, param_types, module_functions, module_globals, &HashMap::new())
}

pub fn lower_function_typed_full(
    hir: &HirFunction,
    param_types: &HashMap<String, MirType>,
    module_functions: &HashMap<String, (crate::mir::FunctionId, Vec<MirType>, MirType)>,
    module_globals: &HashMap<String, (u32, MirType)>,
    array_elem_tys: &HashMap<String, MirType>,
) -> Result<MirFunction, LowerError> {
    let mut param_tys = Vec::with_capacity(hir.params.len());
    for p in &hir.params {
        let ty = param_types.get(&p.name).ok_or_else(|| LowerError::Unsupported {
            function: hir.name.clone(),
            reason: format!("no MIR type provided for param `{}` (typed lane requires explicit types)", p.name),
        })?;
        if !matches!(ty, MirType::I64 | MirType::F64 | MirType::Ref(_) | MirType::Generic) {
            return Err(LowerError::Unsupported {
                function: hir.name.clone(),
                reason: format!("param `{}` has type {ty}; the native lane accepts i64 and f64 params", p.name),
            });
        }
        param_tys.push(*ty);
    }

    let return_ty = module_functions
        .get(&hir.name)
        .map(|(_, _, ret)| *ret)
        .unwrap_or_else(|| crate::lower::machine_return_ty(&hir.return_type).unwrap_or(MirType::I64));
    let b = MirFunctionBuilder::new(&hir.name, param_tys.clone(), return_ty);
    let entry = b.entry();

    let mut cx = FnCx {
        builder: b,
        name: hir.name.clone(),
        value_tys: Vec::new(),
        bindings: HashMap::new(),
        current_block: entry,
        module_functions: module_functions.clone(),
        loop_exit: None,
        module_globals: module_globals.clone(),
        is_init: hir.name == "_hudhud_init",
        loop_header: None,
        array_elem_tys: array_elem_tys.clone(),
        array_inner_elem_tys: HashMap::new(),
        catch_target: None,
    };
    for (i, p) in hir.params.iter().enumerate() {
        let v = cx.builder.param(entry, i as u32);
        cx.set_ty(v, param_tys[i]);
        cx.bindings.insert(p.name.clone(), v);
    }

    let flow = lower_stmts(hir, &mut cx, &hir.body)?;
    // Erken Return'ler zaten bloklarında terminator emit etti;
    // fall-through ise final_blk'ta Return gerekir.
    match flow {
        StmtFlow::Returned(_) => {
            // Blokların hepsi terminator'lu; ek Return gerekmez
            // (builder.finish() zaten tamamlar)
        }
        StmtFlow::Last(v) => {
            let final_blk = cx.current_block;
            if return_ty == MirType::Unit {
                cx.builder.ret_void(final_blk);
            } else {
                cx.builder.ret(final_blk, v);
            }
        }
        StmtFlow::Broke => {
            // En dış döngüden break: exit bloğu current_block olarak geldi
            let final_blk = cx.current_block;
            let v = cx.builder.const_null(final_blk);
            if return_ty == MirType::Unit {
                cx.builder.ret_void(final_blk);
            } else {
                cx.builder.ret(final_blk, v);
            }
        }
    }
    let f = cx.builder.finish();
    crate::verify_function(&f).map_err(|e| LowerError::Unsupported {
        function: hir.name.clone(),
        reason: format!("lowering produced invalid MIR: {e}"),
    })?;
    Ok(f)
}

/// Bir HirModule'ın TÜM fonksiyonlarını tipli olarak MIR'a düşür.
/// Fonksiyonlar arası CallStatic referansları otomatik bağlanır.
/// `all_param_types`: fonksiyon_adı → (param_adı → MirType)
pub fn lower_module_typed(
    module: &hudhudscript_types::HirModule,
    all_param_types: &HashMap<String, HashMap<String, MirType>>,
) -> Result<crate::mir::MirModule, LowerError> {
    let module_globals = crate::param_infer::infer_module_globals(module);
    let return_types = crate::param_infer::infer_module_return_types_with_hints(module, all_param_types);
    let array_elem_types = crate::param_infer::infer_module_array_elem_types(module, &return_types);

    let names: Vec<&String> = module.functions.keys().collect();
    let mut module_functions: HashMap<String, (crate::mir::FunctionId, Vec<MirType>, MirType)> =
        HashMap::new();
    for (i, name) in names.iter().enumerate() {
        let hir = &module.functions[*name];
        let ptys: Vec<MirType> = hir
            .params
            .iter()
            .map(|p| {
                all_param_types
                    .get(*name)
                    .and_then(|m| m.get(&p.name))
                    .copied()
                    .unwrap_or(MirType::I64)
            })
            .collect();
        let ret_ty = return_types.get(*name).copied().unwrap_or_else(|| {
            crate::lower::machine_return_ty(&hir.return_type).unwrap_or(MirType::I64)
        });
        module_functions.insert((*name).clone(), (crate::mir::FunctionId(i as u32), ptys, ret_ty));
    }

    let mut mir_module = crate::mir::MirModule::default();
    for name in &names {
        let hir = &module.functions[name.as_str()];
        let ptys = all_param_types
            .get(name.as_str())
            .cloned()
            .unwrap_or_default();
        let elem_tys = array_elem_types.get(name.as_str()).cloned().unwrap_or_default();
        let mir = lower_function_typed_full(hir, &ptys, &module_functions, &module_globals, &elem_tys)?;
        mir_module.functions.push(mir);
    }
    Ok(mir_module)
}
