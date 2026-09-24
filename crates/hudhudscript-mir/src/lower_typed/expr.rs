//! Expression lowering for typed HIR → MIR.

use hudhudscript_types::{HirExpr, HirFunction};

use crate::mir::{MirType, ValueId};
use crate::LowerError;

use super::binary::lower_binary;
use super::builtins::lower_builtin_call;
use super::cx::FnCx;
use super::methods::{lower_array_method, lower_property_get};

pub(crate) fn lower_expr(
    hir: &HirFunction,
    cx: &mut FnCx,
    expr: &HirExpr,
) -> Result<ValueId, LowerError> {
    match expr {
        HirExpr::IntLit(n) => {
            let block = cx.current_block;
            let v = cx.builder.const_i64(block, *n);
            cx.set_ty(v, MirType::I64);
            Ok(v)
        }
        HirExpr::FloatLit(f) => {
            let block = cx.current_block;
            let v = cx.builder.const_f64(block, *f);
            cx.set_ty(v, MirType::F64);
            Ok(v)
        }
        HirExpr::BoolLit(b) => {
            let block = cx.current_block;
            let v = cx.builder.const_bool(block, *b);
            cx.set_ty(v, MirType::Bool);
            Ok(v)
        }
        HirExpr::StringLit(s) => {
            let block = cx.current_block;
            let v = cx.builder.const_string(block, s);
            cx.set_ty(v, MirType::Ref(crate::mir::RefKind::String));
            Ok(v)
        }
        HirExpr::NullLit => {
            let block = cx.current_block;
            let v = cx.builder.const_null(block);
            cx.set_ty(v, MirType::Generic);
            Ok(v)
        }
        HirExpr::ArrayLit { elements, .. } => {
            let block = cx.current_block;
            let cap = cx.builder.const_i64(block, elements.len() as i64);
            let arr = cx.builder.array_new(block, cap);
            cx.set_ty(arr, MirType::Ref(crate::mir::RefKind::Array));
            for (i, elem) in elements.iter().enumerate() {
                let v = lower_expr(hir, cx, elem)?;
                let block = cx.current_block;
                let idx = cx.builder.const_i64(block, i as i64);
                cx.builder.array_set(block, arr, idx, v);
            }
            Ok(arr)
        }
        HirExpr::ArrayIndex { array, index, .. } => {
            let arr = lower_expr(hir, cx, array)?;
            let idx = lower_expr(hir, cx, index)?;
            let block = cx.current_block;
            if cx.ty_of(idx) == Some(MirType::Ref(crate::mir::RefKind::String))
                || cx.ty_of(arr) == Some(MirType::Ref(crate::mir::RefKind::Object))
            {
                let val_ty = if let HirExpr::Local { name, .. } = array.as_ref() {
                    cx.array_elem_tys.get(name).copied().unwrap_or(MirType::I64)
                } else {
                    MirType::I64
                };
                let v = cx.builder.object_get(block, val_ty, arr, idx);
                cx.set_ty(v, val_ty);
                return Ok(v);
            }
            match cx.ty_of(arr) {
                Some(MirType::Ref(crate::mir::RefKind::String)) => {
                    let v = cx.builder.string_char_at(block, arr, idx);
                    cx.set_ty(v, MirType::Ref(crate::mir::RefKind::String));
                    Ok(v)
                }
                Some(MirType::Ref(crate::mir::RefKind::Array)) => {
                    let elem_ty = if let HirExpr::Local { name, .. } = array.as_ref() {
                        cx.array_elem_tys.get(name).copied().unwrap_or(MirType::I64)
                    } else if let HirExpr::ArrayIndex { array: inner, .. } = array.as_ref() {
                        if let HirExpr::Local { name, .. } = inner.as_ref() {
                            cx.array_inner_elem_tys.get(name).copied().unwrap_or(MirType::F64)
                        } else {
                            MirType::F64
                        }
                    } else {
                        MirType::I64
                    };
                    let v = cx.builder.array_get(block, elem_ty, arr, idx);
                    cx.set_ty(v, elem_ty);
                    Ok(v)
                }
                _ => Err(cx.err("indexing on ambiguous handle (param could be string or array)")),
            }
        }
        HirExpr::ArrayStore { array, index } => {
            let arr = lower_expr(hir, cx, array)?;
            let idx = lower_expr(hir, cx, index)?;
            let _ = (arr, idx);
            Err(cx.err("array store should be handled by Assign path"))
        }
        HirExpr::ArrayMethod { array, method, args, .. } => {
            lower_array_method(hir, cx, array, method, args)
        }
        HirExpr::ObjectLit { properties, .. } => {
            let block = cx.current_block;
            let obj = cx.builder.object_new(block);
            cx.set_ty(obj, MirType::Ref(crate::mir::RefKind::Object));
            for (name, value) in properties {
                let v = lower_expr(hir, cx, value)?;
                match cx.ty_of(v) {
                    Some(MirType::F64) => {
                        return Err(cx.err("float in object literal — f64 field lane arrives with GC"));
                    }
                    Some(MirType::Ref(crate::mir::RefKind::String)) => {}
                    _ => {}
                }
                let block = cx.current_block;
                let key = cx.builder.const_string(block, name);
                cx.set_ty(key, MirType::Ref(crate::mir::RefKind::String));
                cx.builder.object_set(block, obj, key, v);
            }
            Ok(obj)
        }
        HirExpr::PropertyGet { object, name, .. } => {
            lower_property_get(hir, cx, object, name)
        }
        HirExpr::Ternary { condition, true_expr, false_expr, .. } => {
            let c = lower_expr(hir, cx, condition)?;
            let then_blk = cx.builder.create_block();
            let else_blk = cx.builder.create_block();
            cx.builder.cond_branch(cx.current_block, c, then_blk, else_blk);

            let promote = |cx: &mut FnCx, v: ValueId| -> ValueId {
                match cx.ty_of(v) {
                    Some(MirType::Bool) => {
                        let b = cx.current_block;
                        let zero = cx.builder.const_i64(b, 0);
                        cx.builder.bin(b, crate::builder::BinOp::Add, MirType::I64, zero, v)
                    }
                    _ => v,
                }
            };

            cx.current_block = then_blk;
            let t_val = lower_expr(hir, cx, true_expr)?;
            let t_val = promote(cx, t_val);
            let then_final = cx.current_block;

            cx.current_block = else_blk;
            let f_val = lower_expr(hir, cx, false_expr)?;
            let f_val = promote(cx, f_val);
            let else_final = cx.current_block;

            let is_str = |t: Option<MirType>| t == Some(MirType::Ref(crate::mir::RefKind::String));
            let phi_ty = if is_str(cx.ty_of(t_val)) || is_str(cx.ty_of(f_val)) {
                MirType::Ref(crate::mir::RefKind::String)
            } else {
                MirType::I64
            };

            let (merge_blk, phi_vals) = cx.builder.create_block_with_params(vec![phi_ty]);
            let phi_val = phi_vals[0];
            cx.set_ty(phi_val, phi_ty);
            cx.builder.branch_with_args(then_final, merge_blk, vec![t_val]);
            cx.builder.branch_with_args(else_final, merge_blk, vec![f_val]);

            cx.current_block = merge_blk;
            Ok(phi_val)
        }
        HirExpr::Unary { op, operand, .. } => {
            let v = lower_expr(hir, cx, operand)?;
            let block = cx.current_block;
            match op {
                hudhudscript_types::HirUnOp::Neg => {
                    let ty = cx.ty_of(v).unwrap_or(MirType::I64);
                    let result = if ty == MirType::F64 {
                        let zero = cx.builder.const_f64(block, 0.0);
                        cx.builder.bin(block, crate::builder::BinOp::Sub, MirType::F64, zero, v)
                    } else {
                        let zero = cx.builder.const_i64(block, 0);
                        cx.builder.bin(block, crate::builder::BinOp::Sub, MirType::I64, zero, v)
                    };
                    cx.set_ty(result, ty);
                    Ok(result)
                }
                hudhudscript_types::HirUnOp::Not => {
                    let one = cx.builder.const_i64(block, 1);
                    let v = cx.builder.bin(block, crate::builder::BinOp::Sub, MirType::I64, one, v);
                    cx.set_ty(v, MirType::Bool);
                    Ok(v)
                }
            }
        }
        HirExpr::Local { name, .. } => match cx.bindings.get(name) {
            Some(v) if !cx.module_globals.contains_key(name) => Ok(*v),
            Some(v) if matches!(cx.ty_of(*v), Some(MirType::Ref(_))) => Ok(*v),
            Some(v) if !cx.is_init => Ok(*v),
            _ if cx.module_globals.contains_key(name) => {
                let &(slot, ty) = cx.module_globals.get(name).unwrap();
                let b = cx.current_block;
                let slot_val = cx.builder.const_i64(b, slot as i64);
                let v = cx.builder.call_native(
                    b,
                    ty,
                    crate::mir::RuntimeHelperId::GlobalGet,
                    vec![slot_val],
                );
                cx.set_ty(v, ty);
                Ok(v)
            }
            _ => Err(cx.err(&format!(
                "unbound name `{name}` (params, let-locals and module-level lets)"
            ))),
        },
        HirExpr::Binary { op, lhs, rhs, .. } => {
            lower_binary(hir, cx, *op, lhs, rhs)
        }
        HirExpr::Call { callee, args, .. } => {
            if let Some((func_id, callee_param_tys, callee_ret_ty)) = cx.module_functions.get(callee).cloned() {
                if args.len() != callee_param_tys.len() {
                    return Err(cx.err(&format!(
                        "`{callee}` expects {} args, got {}",
                        callee_param_tys.len(), args.len()
                    )));
                }
                let mut arg_vals = Vec::with_capacity(args.len());
                for a in args {
                    let v = lower_expr(hir, cx, a)?;
                    match cx.ty_of(v) {
                        Some(MirType::I64) | Some(MirType::F64) | Some(MirType::Ref(_)) | Some(MirType::Generic) => {}
                        other => {
                            return Err(cx.err(&format!(
                                "`{callee}` arg must be i64/handle, got {other:?}"
                            )))
                        }
                    }
                    arg_vals.push(v);
                }
                let block = cx.current_block;
                let ret = cx.builder.call_static(block, callee_ret_ty, func_id, arg_vals);
                cx.set_ty(ret, callee_ret_ty);
                if let Some(catch_blk) = cx.catch_target {
                    let has_ex = cx.builder.call_native(
                        block,
                        MirType::I64,
                        crate::mir::RuntimeHelperId::HasException,
                        vec![],
                    );
                    let cont_blk = cx.builder.create_block();
                    cx.builder.cond_branch(block, has_ex, catch_blk, cont_blk);
                    cx.current_block = cont_blk;
                }
                return Ok(ret);
            }
            if let Some(v) = lower_builtin_call(hir, cx, callee.as_str(), args)? {
                return Ok(v);
            }
            Err(cx.err(&format!("unknown function `{callee}` (not in module, not a builtin)")))
        }
    }
}
