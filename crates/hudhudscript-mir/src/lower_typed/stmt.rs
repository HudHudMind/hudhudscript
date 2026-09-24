use hudhudscript_types::{HirBinOp, HirExpr, HirFunction, HirStmt};

use crate::mir::{MirType, RefKind, RuntimeHelperId, ValueId};
use crate::LowerError;

use super::cx::{emit_global_store, truthy, FnCx};
use super::expr::lower_expr;
use super::phi_helpers::{promote_to_phi, track_assigned_array_types};

/// Akış sonucu: erken Return mü, normal son değer mi
pub enum StmtFlow {
    Returned(#[allow(dead_code)] ValueId),
    /// break ile döngüden çıkıldı (arg'lar exit philerine verilmiş)
    Broke,
    Last(ValueId),
}

pub(crate) fn lower_stmts(
    hir: &HirFunction,
    cx: &mut FnCx,
    stmts: &[HirStmt],
) -> Result<StmtFlow, LowerError> {
    let mut last: Option<ValueId> = None;
    for stmt in stmts {
        let blk = cx.current_block;
        match stmt {
            HirStmt::Return(Some(expr)) => {
                let v = lower_expr(hir, cx, expr)?;
                // Kontrol-akış ifadeleri (ternary) current_block'u değiştirir;
                // Return her zaman LOWERING SONRASI güncel bloğa yazılmalı.
                cx.builder.ret(cx.current_block, v);
                return Ok(StmtFlow::Returned(v));
            }
            HirStmt::Return(None) => {
                let v = cx.builder.const_null(blk);
                cx.builder.ret_void(blk);
                return Ok(StmtFlow::Returned(v));
            }
            HirStmt::Expr(expr) => last = Some(lower_expr(hir, cx, expr)?),
            HirStmt::Let { name, value, .. } => {
                let v = lower_expr(hir, cx, value)?;
                cx.bindings.insert(name.clone(), v);
                track_assigned_array_types(cx, name, value, v);
                if cx.is_init && cx.module_globals.contains_key(name) {
                    emit_global_store(cx, name, v);
                }
                last = Some(v);
            }
            HirStmt::Assign { name, value } => {
                let v = if let HirExpr::Binary { op: HirBinOp::Add, lhs, rhs, .. } = value {
                    if let HirExpr::Local { name: ref n, .. } = **lhs {
                        let prev_v = cx.bindings.get(name).copied();
                        if n == name && prev_v.is_some() && cx.ty_of(prev_v.unwrap()) == Some(MirType::Ref(RefKind::String)) {
                            let l = prev_v.unwrap();
                            let r = lower_expr(hir, cx, rhs)?;
                            let block = cx.current_block;
                            let rs = if cx.ty_of(r) == Some(MirType::Ref(RefKind::String)) {
                                r
                            } else if cx.ty_of(r) == Some(MirType::F64) {
                                cx.builder.float_to_string(block, r)
                            } else {
                                cx.builder.int_to_string(block, r)
                            };
                            let app = cx.builder.call_native(
                                block,
                                MirType::Ref(RefKind::String),
                                RuntimeHelperId::StringAppend,
                                vec![l, rs],
                            );
                            cx.set_ty(app, MirType::Ref(RefKind::String));
                            app
                        } else {
                            lower_expr(hir, cx, value)?
                        }
                    } else {
                        lower_expr(hir, cx, value)?
                    }
                } else {
                    lower_expr(hir, cx, value)?
                };
                track_assigned_array_types(cx, name, value, v);
                if cx.module_globals.contains_key(name) && !cx.bindings.contains_key(name) {
                    // modül-global'e yazma (fonksiyon içinden)
                    emit_global_store(cx, name, v);
                    last = Some(v);
                    continue;
                }
                cx.bindings.insert(name.clone(), v);
                if cx.is_init && cx.module_globals.contains_key(name) {
                    emit_global_store(cx, name, v);
                }
                last = Some(v);
            }
            HirStmt::Continue => {
                let (header, names) = cx
                    .loop_header
                    .clone()
                    .ok_or_else(|| cx.err("continue outside of a loop"))?;
                let cur = cx.current_block;
                let args: Vec<ValueId> = names.iter()
                    .map(|n| cx.bindings.get(n).copied().unwrap_or_else(|| ValueId(0)))
                    .collect();
                cx.builder.branch_with_args(cur, header, args);
                return Ok(StmtFlow::Broke);
            }
            HirStmt::Break => {
                let (exit, names) = cx
                    .loop_exit
                    .clone()
                    .ok_or_else(|| cx.err("break outside of a loop"))?;
                let args: Vec<ValueId> = names.iter()
                    .map(|n| cx.bindings.get(n).copied()
                        .ok_or_else(|| cx.err(&format!("break: unbound `{n}`"))).unwrap())
                    .collect();
                let cur = cx.current_block;
                cx.builder.branch_with_args(cur, exit, args);
                return Ok(StmtFlow::Broke);
            }
            HirStmt::ArrayStore { array, index, value } => {
                let arr = lower_expr(hir, cx, array)?;
                let idx = lower_expr(hir, cx, index)?;
                let v = lower_expr(hir, cx, value)?;
                let b = cx.current_block;
                if cx.ty_of(idx) == Some(MirType::Ref(crate::mir::RefKind::String))
                    || cx.ty_of(arr) == Some(MirType::Ref(crate::mir::RefKind::Object))
                {
                    cx.builder.object_set(b, arr, idx, v);
                    if let Some(val_ty) = cx.ty_of(v) {
                        if let HirExpr::Local { name, .. } = array {
                            cx.array_elem_tys.insert(name.clone(), val_ty);
                        }
                    }
                } else {
                    cx.builder.array_set(b, arr, idx, v);
                    if let Some(elem_ty) = cx.ty_of(v) {
                        if let HirExpr::Local { name, .. } = array {
                            cx.array_elem_tys.insert(name.clone(), elem_ty);
                        }
                    }
                }
            }
            HirStmt::PropertySet { object, name, value } => {
                let obj = lower_expr(hir, cx, object)?;
                let v = lower_expr(hir, cx, value)?;
                let b = cx.current_block;
                let key = cx.builder.const_string(b, name);
                cx.set_ty(key, MirType::Ref(crate::mir::RefKind::String));
                cx.builder.object_set(b, obj, key, v);
            }
            HirStmt::If { cond, then_branch, else_branch } => {
                let c = lower_expr(hir, cx, cond)?;
                let c = truthy(hir, cx, c)?;
                let then_blk = cx.builder.create_block();
                let else_blk = cx.builder.create_block();
                // cond içinde ternary olabilir → güncel blok değişmiş olabilir
                cx.builder.cond_branch(cx.current_block, c, then_blk, else_blk);

                // Then dalı (merge'i ŞİMDİ DEĞİL — phi parametrelerini
                // bilmeden oluşturamayız; önce dalları düşürüyoruz)
                let saved = cx.bindings.clone();
                cx.current_block = then_blk;
                let then_flow = lower_stmts(hir, cx, then_branch)?;
                let then_final = cx.current_block;
                let then_bindings = cx.bindings.clone();
                cx.bindings = saved.clone();

                // Else dalı
                cx.current_block = else_blk;
                let else_flow = lower_stmts(hir, cx, else_branch)?;
                let else_final = cx.current_block;
                let else_bindings = cx.bindings.clone();
                cx.bindings = saved.clone();

                // Phi analizi: dallarda değişen değişkenler
                let mut phi_names: Vec<String> = Vec::new();
                let mut phi_tys: Vec<MirType> = Vec::new();
                for (name, pre_v) in &saved {
                    let then_v = then_bindings.get(name);
                    let else_v = else_bindings.get(name);
                    if then_v != Some(pre_v) || else_v != Some(pre_v) {
                        let pre_ty = cx.ty_of(*pre_v).unwrap_or(MirType::I64);
                        let then_ty = then_v.and_then(|v| cx.ty_of(*v));
                        let else_ty = else_v.and_then(|v| cx.ty_of(*v));
                        let ty = if pre_ty == MirType::F64 || then_ty == Some(MirType::F64) || else_ty == Some(MirType::F64) {
                            MirType::F64
                        } else {
                            pre_ty
                        };
                        phi_names.push(name.clone());
                        phi_tys.push(ty);
                    }
                }

                // Merge bloğu phi parametreleriyle oluştur
                let (merge_blk, phi_vals) = cx.builder.create_block_with_params(phi_tys.clone());
                for (i, v) in phi_vals.iter().enumerate() {
                    cx.set_ty(*v, phi_tys[i]);
                }

                // Then'den merge'e: phi değerlerini geçir
                if !matches!(then_flow, StmtFlow::Returned(_) | StmtFlow::Broke) {
                    let args: Vec<ValueId> = phi_names.iter().enumerate()
                        .map(|(i, n)| {
                            let v = then_bindings.get(n).copied().unwrap_or(saved[n]);
                            promote_to_phi(cx, then_final, v, phi_tys[i])
                        })
                        .collect();
                    cx.builder.branch_with_args(then_final, merge_blk, args);
                }

                // Else'den merge'e
                if !matches!(else_flow, StmtFlow::Returned(_) | StmtFlow::Broke) {
                    let args: Vec<ValueId> = phi_names.iter().enumerate()
                        .map(|(i, n)| {
                            let v = else_bindings.get(n).copied().unwrap_or(saved[n]);
                            promote_to_phi(cx, else_final, v, phi_tys[i])
                        })
                        .collect();
                    cx.builder.branch_with_args(else_final, merge_blk, args);
                }

                // Merge bağlamaları: phi değerlerini kullan
                for (i, name) in phi_names.iter().enumerate() {
                    cx.bindings.insert(name.clone(), phi_vals[i]);
                }

                // Her iki dal erken dönüş yaptıysa merge UNREACHABLE
                let both_early = matches!(then_flow, StmtFlow::Returned(_) | StmtFlow::Broke)
                    && matches!(else_flow, StmtFlow::Returned(_) | StmtFlow::Broke);
                if both_early {
                    cx.current_block = merge_blk;
                    let v = cx.builder.const_i64(merge_blk, 0);
                    cx.builder.ret(merge_blk, v);
                    return Ok(StmtFlow::Returned(v));
                }

                // Devam merge bloğunda
                cx.current_block = merge_blk;
            }
            HirStmt::While { cond, body } => {
                if let Some(flow) = super::stmt_loops::lower_while(hir, cx, cond, body)? {
                    return Ok(flow);
                }
            }
            HirStmt::Throw(expr) => {
                let v = lower_expr(hir, cx, expr)?;
                let blk = cx.current_block;
                let _ = cx.builder.call_native(blk, MirType::Unit, RuntimeHelperId::Throw, vec![v]);
                if let Some(catch_target) = cx.catch_target {
                    cx.builder.branch(cx.current_block, catch_target);
                    return Ok(StmtFlow::Broke);
                } else {
                    let zero = cx.builder.const_i64(cx.current_block, 0);
                    cx.builder.ret(cx.current_block, zero);
                    return Ok(StmtFlow::Returned(zero));
                }
            }
            HirStmt::Try { try_body, catch_param, catch_body, finally_body } => {
                super::stmt_loops::lower_try(hir, cx, try_body, catch_param.as_ref(), catch_body, finally_body)?;
            }
        }
    }
    let v = last.unwrap_or_else(|| cx.builder.const_null(cx.current_block));
    Ok(StmtFlow::Last(v))
}
