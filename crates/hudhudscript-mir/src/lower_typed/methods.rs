//! Method and property lowering for typed HIR → MIR.

use hudhudscript_types::{HirExpr, HirFunction};

use crate::mir::{MirType, RefKind, RuntimeHelperId, ValueId};
use crate::LowerError;

use super::cx::FnCx;
use super::expr::lower_expr;

pub(crate) fn lower_array_method(
    hir: &HirFunction,
    cx: &mut FnCx,
    array: &HirExpr,
    method: &str,
    args: &[HirExpr],
) -> Result<ValueId, LowerError> {
    let arr = lower_expr(hir, cx, array)?;
    match cx.ty_of(arr) {
        Some(MirType::Ref(RefKind::String)) => match method {
            "length" | "len" => {
                let block = cx.current_block;
                let len = cx.builder.string_len(block, arr);
                cx.set_ty(len, MirType::I64);
                Ok(len)
            }
            "substring" | "slice" => {
                if args.len() < 2 {
                    return Err(cx.err("substring requires start and end arguments"));
                }
                let start = lower_expr(hir, cx, &args[0])?;
                let end = lower_expr(hir, cx, &args[1])?;
                let block = cx.current_block;
                let v = cx.builder.string_substring(block, arr, start, end);
                cx.set_ty(v, MirType::Ref(RefKind::String));
                Ok(v)
            }
            "split" => {
                let delim = if args.is_empty() {
                    let s = cx.builder.const_string(cx.current_block, "");
                    cx.set_ty(s, MirType::Ref(RefKind::String));
                    s
                } else {
                    lower_expr(hir, cx, &args[0])?
                };
                let block = cx.current_block;
                let v = cx.builder.call_native(
                    block,
                    MirType::Ref(RefKind::Array),
                    RuntimeHelperId::StringSplit,
                    vec![arr, delim],
                );
                cx.set_ty(v, MirType::Ref(RefKind::Array));
                Ok(v)
            }
            "indexOf" => {
                if args.is_empty() {
                    return Err(cx.err("indexOf requires an argument"));
                }
                let needle = lower_expr(hir, cx, &args[0])?;
                let block = cx.current_block;
                let v = cx.builder.call_native(
                    block,
                    MirType::I64,
                    RuntimeHelperId::StringIndexOf,
                    vec![arr, needle],
                );
                cx.set_ty(v, MirType::I64);
                Ok(v)
            }
            other => Err(cx.err(&format!("string method .{other} not supported in this lane"))),
        },
        _ => match method {
            "push" => {
                if args.len() != 1 {
                    return Err(cx.err("push takes exactly 1 argument"));
                }
                let v = lower_expr(hir, cx, &args[0])?;
                let block = cx.current_block;
                cx.builder.array_push(block, arr, v);
                cx.set_ty(arr, MirType::Ref(RefKind::Array));
                if let Some(elem_ty) = cx.ty_of(v) {
                    if let HirExpr::Local { name, .. } = array {
                        cx.array_elem_tys.insert(name.clone(), elem_ty);
                        if elem_ty == MirType::Ref(RefKind::Array) {
                            if let HirExpr::Local { name: val_name, .. } = &args[0] {
                                if let Some(inner) = cx.array_elem_tys.get(val_name).copied() {
                                    cx.array_inner_elem_tys.insert(name.clone(), inner);
                                }
                            }
                        }
                    }
                }
                Ok(arr)
            }
            "fill" => {
                if args.len() == 2 {
                    let len = lower_expr(hir, cx, &args[0])?;
                    let val = lower_expr(hir, cx, &args[1])?;
                    let block = cx.current_block;
                    cx.builder.call_native(
                        block,
                        MirType::Ref(RefKind::Array),
                        RuntimeHelperId::ArrayFill,
                        vec![arr, len, val],
                    );
                    Ok(arr)
                } else if args.len() == 1 {
                    let val = lower_expr(hir, cx, &args[0])?;
                    let block = cx.current_block;
                    let len = cx.builder.array_len(block, arr);
                    cx.set_ty(len, MirType::I64);
                    cx.builder.call_native(
                        block,
                        MirType::Ref(RefKind::Array),
                        RuntimeHelperId::ArrayFill,
                        vec![arr, len, val],
                    );
                    Ok(arr)
                } else {
                    Err(cx.err("fill takes 1 or 2 arguments"))
                }
            }
            "pop" => {
                if !args.is_empty() {
                    return Err(cx.err("pop takes no arguments"));
                }
                let block = cx.current_block;
                let v = cx.builder.array_pop(block, arr);
                cx.set_ty(v, MirType::I64);
                Ok(v)
            }
            "length" | "len" => {
                let block = cx.current_block;
                let len = cx.builder.array_len(block, arr);
                cx.set_ty(len, MirType::I64);
                Ok(len)
            }
            "join" => {
                let block = cx.current_block;
                let sep = if args.is_empty() {
                    let s = cx.builder.const_string(block, ",");
                    cx.set_ty(s, MirType::Ref(RefKind::String));
                    s
                } else {
                    lower_expr(hir, cx, &args[0])?
                };
                let v = cx.builder.array_join(block, arr, sep);
                cx.set_ty(v, MirType::Ref(RefKind::String));
                Ok(v)
            }
            other => Err(cx.err(&format!("array method .{other} not supported in this lane"))),
        },
    }
}

pub(crate) fn lower_property_get(
    hir: &HirFunction,
    cx: &mut FnCx,
    object: &HirExpr,
    name: &str,
) -> Result<ValueId, LowerError> {
    let obj = lower_expr(hir, cx, object)?;
    let block = cx.current_block;
    // Tip bazlı dispatch: array handle'ına object ABI çağrısı
    // yapılamaz (type confusion → UB); tipler MIR'de ayrık.
    match cx.ty_of(obj) {
        Some(MirType::Ref(RefKind::Array)) => match name {
            "pop" => {
                let v = cx.builder.array_pop(block, obj);
                cx.set_ty(v, MirType::I64);
                Ok(v)
            }
            "length" | "len" => {
                let v = cx.builder.array_len(block, obj);
                cx.set_ty(v, MirType::I64);
                Ok(v)
            }
            other => Err(cx.err(&format!(
                "array property .{other} not supported in this lane (arrays support .length; .push is a statement)"
            ))),
        },
        Some(MirType::Ref(RefKind::String)) => match name {
            "length" | "len" => {
                let v = cx.builder.string_len(block, obj);
                cx.set_ty(v, MirType::I64);
                Ok(v)
            }
            other => Err(cx.err(&format!(
                "string property .{other} not supported in this lane (strings support .length)"
            ))),
        },
        _ => {
            let key = cx.builder.const_string(block, name);
            cx.set_ty(key, MirType::Ref(RefKind::String));
            let v = cx.builder.object_get(block, MirType::I64, obj, key);
            cx.set_ty(v, MirType::I64);
            Ok(v)
        }
    }
}
