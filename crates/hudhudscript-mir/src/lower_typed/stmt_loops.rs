//! While and Try/Catch statement lowering for typed MIR.

use hudhudscript_types::{HirExpr, HirFunction, HirStmt};

use crate::mir::{MirType, RuntimeHelperId, ValueId};
use crate::LowerError;
use super::cx::{truthy, FnCx};
use super::expr::lower_expr;
use super::phi_helpers::{body_assigns_float, collect_reassigned, promote_to_phi};
use super::stmt::{lower_stmts, StmtFlow};

pub(crate) fn lower_while(
    hir: &HirFunction,
    cx: &mut FnCx,
    cond: &HirExpr,
    body: &[HirStmt],
) -> Result<Option<StmtFlow>, LowerError> {
    let blk = cx.current_block;
    let saved = cx.bindings.clone();
    let reassigned = collect_reassigned(body, &saved);

    let phi_tys: Vec<MirType> = reassigned.iter()
        .map(|n| {
            let init_ty = cx.ty_of(saved[n]).unwrap_or(MirType::I64);
            if init_ty == MirType::F64 || body_assigns_float(n, body, cx) {
                MirType::F64
            } else {
                init_ty
            }
        })
        .collect();
    let (header_blk, phi_vals) = cx.builder.create_block_with_params(phi_tys.clone());
    for (i, v) in phi_vals.iter().enumerate() {
        cx.set_ty(*v, phi_tys[i]);
    }

    let entry_args: Vec<ValueId> = reassigned.iter().enumerate()
        .map(|(i, n)| promote_to_phi(cx, blk, saved[n], phi_tys[i]))
        .collect();
    cx.builder.branch_with_args(blk, header_blk, entry_args);

    for (i, name) in reassigned.iter().enumerate() {
        cx.bindings.insert(name.clone(), phi_vals[i]);
    }

    cx.current_block = header_blk;
    let c = lower_expr(hir, cx, cond)?;
    let c = truthy(hir, cx, c)?;

    let body_blk = cx.builder.create_block();
    let (exit_blk, exit_phi_vals) =
        cx.builder.create_block_with_params(phi_tys.clone());
    for (i, v) in exit_phi_vals.iter().enumerate() {
        cx.set_ty(*v, phi_tys[i]);
    }
    let exit_phis: Vec<String> = reassigned.clone();
    let cur = cx.current_block;
    let exit_args_now: Vec<ValueId> = exit_phis.iter().enumerate()
        .map(|(i, n)| {
            let v = cx.bindings.get(n).copied().unwrap_or(saved[n]);
            promote_to_phi(cx, cur, v, phi_tys[i])
        })
        .collect();
    cx.builder.cond_branch_with_args(cur, c, body_blk, vec![], exit_blk, exit_args_now);

    cx.current_block = body_blk;
    let prev_loop = cx.loop_exit.replace((exit_blk, exit_phis.clone()));
    let prev_hdr = cx.loop_header.replace((header_blk, reassigned.clone()));
    let body_flow = lower_stmts(hir, cx, body)?;
    cx.loop_header = prev_hdr;
    cx.loop_exit = prev_loop;
    let body_final = cx.current_block;
    let body_bindings = cx.bindings.clone();

    if !matches!(body_flow, StmtFlow::Broke | StmtFlow::Returned(_)) {
        let back_args: Vec<ValueId> = reassigned.iter().enumerate()
            .map(|(i, n)| {
                let v = body_bindings.get(n).copied().unwrap_or(saved[n]);
                promote_to_phi(cx, body_final, v, phi_tys[i])
            })
            .collect();
        cx.builder.branch_with_args(body_final, header_blk, back_args);
    }

    cx.bindings = saved.clone();
    for (i, name) in reassigned.iter().enumerate() {
        cx.bindings.insert(name.clone(), exit_phi_vals[i]);
    }
    cx.current_block = exit_blk;

    if matches!(body_flow, StmtFlow::Returned(_)) {
        let v = cx.builder.const_i64(exit_blk, 0);
        cx.builder.ret(exit_blk, v);
        return Ok(Some(StmtFlow::Returned(v)));
    }
    Ok(None)
}

pub(crate) fn lower_try(
    hir: &HirFunction,
    cx: &mut FnCx,
    try_body: &[HirStmt],
    catch_param: Option<&String>,
    catch_body: &[HirStmt],
    finally_body: &[HirStmt],
) -> Result<(), LowerError> {
    let try_blk = cx.builder.create_block();
    let catch_blk = cx.builder.create_block();
    let saved = cx.bindings.clone();
    let saved_catch = cx.catch_target;

    cx.builder.branch(cx.current_block, try_blk);

    cx.current_block = try_blk;
    cx.catch_target = Some(catch_blk);
    let try_flow = lower_stmts(hir, cx, try_body)?;
    let try_final = cx.current_block;
    let try_bindings = cx.bindings.clone();

    cx.current_block = catch_blk;
    cx.catch_target = saved_catch;
    cx.bindings = saved.clone();

    let ex_val = cx.builder.call_native(catch_blk, MirType::Generic, RuntimeHelperId::Catch, vec![]);
    cx.set_ty(ex_val, MirType::Generic);
    if let Some(param) = catch_param {
        cx.bindings.insert(param.clone(), ex_val);
    }

    let catch_flow = lower_stmts(hir, cx, catch_body)?;
    let catch_final = cx.current_block;
    let catch_bindings = cx.bindings.clone();

    cx.catch_target = saved_catch;

    if !finally_body.is_empty() {
        let _ = lower_stmts(hir, cx, finally_body)?;
    }

    let mut phi_names: Vec<String> = Vec::new();
    let mut phi_tys: Vec<MirType> = Vec::new();
    for (name, pre_v) in &saved {
        let tv = try_bindings.get(name);
        let cv = catch_bindings.get(name);
        if tv != Some(pre_v) || cv != Some(pre_v) {
            let pre_ty = cx.ty_of(*pre_v).unwrap_or(MirType::I64);
            let t_ty = tv.and_then(|v| cx.ty_of(*v));
            let c_ty = cv.and_then(|v| cx.ty_of(*v));
            let ty = if pre_ty == MirType::F64 || t_ty == Some(MirType::F64) || c_ty == Some(MirType::F64) {
                MirType::F64
            } else {
                pre_ty
            };
            phi_names.push(name.clone());
            phi_tys.push(ty);
        }
    }

    let (merge_blk, phi_vals) = cx.builder.create_block_with_params(phi_tys.clone());
    for (i, v) in phi_vals.iter().enumerate() {
        cx.set_ty(*v, phi_tys[i]);
    }

    if !matches!(try_flow, StmtFlow::Returned(_) | StmtFlow::Broke) {
        let args: Vec<ValueId> = phi_names.iter().enumerate()
            .map(|(i, n)| {
                let v = try_bindings.get(n).copied().unwrap_or(saved[n]);
                promote_to_phi(cx, try_final, v, phi_tys[i])
            })
            .collect();
        cx.builder.branch_with_args(try_final, merge_blk, args);
    }

    if !matches!(catch_flow, StmtFlow::Returned(_) | StmtFlow::Broke) {
        let args: Vec<ValueId> = phi_names.iter().enumerate()
            .map(|(i, n)| {
                let v = catch_bindings.get(n).copied().unwrap_or(saved[n]);
                promote_to_phi(cx, catch_final, v, phi_tys[i])
            })
            .collect();
        cx.builder.branch_with_args(catch_final, merge_blk, args);
    }

    for (i, name) in phi_names.iter().enumerate() {
        cx.bindings.insert(name.clone(), phi_vals[i]);
    }

    cx.current_block = merge_blk;
    Ok(())
}
