//! Builtin function calls (Date, Math, print) for typed HIR → MIR lowering.

use hudhudscript_types::{HirExpr, HirFunction};

use crate::mir::{MirType, RuntimeHelperId, ValueId};
use crate::LowerError;

use super::cx::FnCx;
use super::expr::lower_expr;

pub(crate) fn lower_builtin_call(
    hir: &HirFunction,
    cx: &mut FnCx,
    callee: &str,
    args: &[HirExpr],
) -> Result<Option<ValueId>, LowerError> {
    match callee {
        "Date.to_millis" => {
            if !args.is_empty() {
                return Err(cx.err("Date.to_millis takes no arguments"));
            }
            let block = cx.current_block;
            let v = cx.builder.call_native(
                block,
                MirType::I64,
                RuntimeHelperId::DateMillis,
                vec![],
            );
            cx.set_ty(v, MirType::I64);
            Ok(Some(v))
        }
        "Array.fill" => {
            if args.len() != 2 {
                return Err(cx.err("Array.fill takes 2 arguments: (length, value)"));
            }
            let len = lower_expr(hir, cx, &args[0])?;
            let val = lower_expr(hir, cx, &args[1])?;
            let block = cx.current_block;
            let v = cx.builder.call_native(
                block,
                MirType::Ref(crate::mir::RefKind::Array),
                RuntimeHelperId::ArrayFilled,
                vec![len, val],
            );
            cx.set_ty(v, MirType::Ref(crate::mir::RefKind::Array));
            Ok(Some(v))
        }
        "Math.sin" | "Math.sqrt" | "Math.cos" | "Math.floor" | "Math.abs"
        | "Math.pow" | "Math.min" | "Math.max" => {
            if args.len() == 2 {
                let a = lower_expr(hir, cx, &args[0])?;
                let b = lower_expr(hir, cx, &args[1])?;
                let block = cx.current_block;
                let (a64, b64) = (f64_of(cx, a)?, f64_of(cx, b)?);
                let helper = match callee {
                    "Math.pow" => RuntimeHelperId::MathPow,
                    "Math.min" => RuntimeHelperId::MathMin,
                    _ => RuntimeHelperId::MathMax,
                };
                let r = cx.builder.call_native_f2(block, helper, a64, b64);
                cx.set_ty(r, MirType::F64);
                return Ok(Some(r));
            }
            if args.len() != 1 {
                return Err(cx.err(&format!("{callee} takes exactly one argument")));
            }
            let v = lower_expr(hir, cx, &args[0])?;
            let block = cx.current_block;
            let vf = match cx.ty_of(v) {
                Some(MirType::F64) => v,
                Some(MirType::I64) => {
                    let f = cx.builder.int_to_float(block, v);
                    cx.set_ty(f, MirType::F64);
                    f
                }
                _ => return Err(cx.err(&format!("{callee} needs a float operand"))),
            };
            let helper = match callee {
                "Math.sin" => RuntimeHelperId::MathSin,
                "Math.sqrt" => RuntimeHelperId::MathSqrt,
                "Math.cos" => RuntimeHelperId::MathCos,
                "Math.floor" => RuntimeHelperId::MathFloor,
                _ => RuntimeHelperId::MathAbs,
            };
            let r = cx.builder.call_native(block, MirType::F64, helper, vec![vf]);
            cx.set_ty(r, MirType::F64);
            Ok(Some(r))
        }
        "len" => {
            if args.len() != 1 {
                return Err(cx.err("len takes exactly one argument"));
            }
            let v = lower_expr(hir, cx, &args[0])?;
            let block = cx.current_block;
            match cx.ty_of(v) {
                Some(MirType::Ref(crate::mir::RefKind::String)) => {
                    let r = cx.builder.string_len(block, v);
                    cx.set_ty(r, MirType::I64);
                    Ok(Some(r))
                }
                Some(MirType::Ref(crate::mir::RefKind::Array)) => {
                    let r = cx.builder.array_len(block, v);
                    cx.set_ty(r, MirType::I64);
                    Ok(Some(r))
                }
                _ => Err(cx.err("len() expects a string or array")),
            }
        }
        "input" | "confirm" => {
            // M6: 0 veya 1 arg (string istem). input → string handle,
            // confirm → 0/1. Çok-dilli alias'lar canonical_builtin'ten
            // gelir (oku → input).
            let prompt = if args.is_empty() {
                let b = cx.current_block;
                let z = cx.builder.const_null(b);
                cx.set_ty(z, MirType::I64);
                z
            } else {
                let v = lower_expr(hir, cx, &args[0])?;
                let b = cx.current_block;
                match cx.ty_of(v) {
                    Some(MirType::Ref(crate::mir::RefKind::String)) => v,
                    _ => {
                        let z = cx.builder.const_null(b);
                        cx.set_ty(z, MirType::I64);
                        z
                    }
                }
            };
            let h = if callee == "input" { RuntimeHelperId::Input } else { RuntimeHelperId::Confirm };
            let block = cx.current_block;
            let r = cx.builder.call_native(block, MirType::Generic, h, vec![prompt]);
            cx.set_ty(r, MirType::Generic);
            Ok(Some(r))
        }
        "__hudhud_dyn_call" => {
            // M5: [alıcı, yöntem adı(string), ...arg'lar] — ad const string
            // handle'ı; hepsi i64 şeridinde CallNative'e iner.
            if args.is_empty() {
                return Err(cx.err("dyn_call needs method name"));
            }
            let name = lower_expr(hir, cx, &args[0])?;
            let zero = {
                let b = cx.current_block;
                let z = cx.builder.const_null(b);
                cx.set_ty(z, MirType::I64);
                z
            };
            let mut vals = vec![zero, name];
            for a in &args[1..] {
                vals.push(lower_expr(hir, cx, a)?);
            }
            let block = cx.current_block;
            let r = cx.builder.call_native(block, MirType::Generic, RuntimeHelperId::DynCallMethod, vals);
            cx.set_ty(r, MirType::Generic);
            Ok(Some(r))
        }
        "assert_eq" => {
            if args.len() != 2 {
                return Err(cx.err("assert_eq takes exactly two arguments"));
            }
            let a = lower_expr(hir, cx, &args[0])?;
            let b = lower_expr(hir, cx, &args[1])?;
            let block = cx.current_block;
            let a = match cx.ty_of(a) { Some(MirType::F64) => return Err(cx.err("assert_eq float lane — use assert_approx")), _ => a };
            let b = match cx.ty_of(b) { Some(MirType::F64) => return Err(cx.err("assert_eq float lane — use assert_approx")), _ => b };
            cx.builder.call_native(block, MirType::Unit, RuntimeHelperId::AssertEq, vec![a, b]);
            let n = cx.builder.const_null(block);
            cx.set_ty(n, MirType::Generic);
            Ok(Some(n))
        }
        "assert_approx" => {
            if args.len() != 2 {
                return Err(cx.err("assert_approx takes exactly two arguments"));
            }
            let a = lower_expr(hir, cx, &args[0])?;
            let b = lower_expr(hir, cx, &args[1])?;
            let a = f64_of(cx, a)?;
            let b = f64_of(cx, b)?;
            let block = cx.current_block;
            cx.builder.call_native(block, MirType::Unit, RuntimeHelperId::AssertApprox, vec![a, b]);
            let n = cx.builder.const_null(block);
            cx.set_ty(n, MirType::Generic);
            Ok(Some(n))
        }
        "assert_true" | "assert_false" => {
            if args.len() != 1 {
                return Err(cx.err("assert_true/false take exactly one argument"));
            }
            let v = lower_expr(hir, cx, &args[0])?;
            let block = cx.current_block;
            let h = if callee == "assert_true" { RuntimeHelperId::AssertTrue } else { RuntimeHelperId::AssertFalse };
            cx.builder.call_native(block, MirType::Unit, h, vec![v]);
            let n = cx.builder.const_null(block);
            cx.set_ty(n, MirType::Generic);
            Ok(Some(n))
        }
        "print" => {
            if args.len() != 1 {
                return Err(cx.err("print takes exactly one argument"));
            }
            let v = lower_expr(hir, cx, &args[0])?;
            let block = cx.current_block;
            match cx.ty_of(v) {
                Some(MirType::I64) | Some(MirType::F64) => {
                    cx.builder.call_native(
                        block,
                        MirType::Unit,
                        RuntimeHelperId::Print,
                        vec![v],
                    );
                }
                Some(MirType::Ref(crate::mir::RefKind::String)) => {
                    cx.builder.call_native(
                        block,
                        MirType::Unit,
                        RuntimeHelperId::PrintStr,
                        vec![v],
                    );
                }
                _ => {
                    return Err(cx.err(
                        "print(i64|f64|string) — other print lanes arrive with the ABI growth",
                    ));
                }
            }
            cx.builder.gc_safepoint(block);
            let n = cx.builder.const_null(block);
            cx.set_ty(n, MirType::Generic);
            Ok(Some(n))
        }
        "toNumber" | "parseInt" => {
            if args.len() != 1 {
                return Err(cx.err("toNumber takes exactly one argument"));
            }
            let v = lower_expr(hir, cx, &args[0])?;
            let block = cx.current_block;
            match cx.ty_of(v) {
                Some(MirType::I64) => Ok(Some(v)),
                Some(MirType::F64) => Ok(Some(v)),
                _ => {
                    let r = cx.builder.call_native(
                        block,
                        MirType::I64,
                        RuntimeHelperId::StringToInt,
                        vec![v],
                    );
                    cx.set_ty(r, MirType::I64);
                    Ok(Some(r))
                }
            }
        }
        "typeof" => {
            if args.len() != 1 {
                return Err(cx.err("typeof takes exactly one argument"));
            }
            let block = cx.current_block;
            let lit_tag = match &args[0] {
                HirExpr::IntLit(..) | HirExpr::FloatLit(..) => Some("number"),
                HirExpr::BoolLit(..) => Some("boolean"),
                HirExpr::StringLit(..) => Some("string"),
                HirExpr::ArrayLit { .. } => Some("array"),
                HirExpr::ObjectLit { .. } => Some("object"),
                _ => None,
            };
            let r = if let Some(tag) = lit_tag {
                let s = cx.builder.const_string(block, tag);
                cx.set_ty(s, MirType::Ref(crate::mir::RefKind::String));
                s
            } else {
                let v = lower_expr(hir, cx, &args[0])?;
                let block = cx.current_block;
                let r = cx.builder.call_native(
                    block,
                    MirType::Ref(crate::mir::RefKind::String),
                    RuntimeHelperId::TypeOf,
                    vec![v],
                );
                cx.set_ty(r, MirType::Ref(crate::mir::RefKind::String));
                r
            };
            Ok(Some(r))
        }
        _ => Ok(None),
    }
}

/// Değeri f64'e terfi ettir (i64 → IntToFloat gerekirse)
fn f64_of(cx: &mut FnCx, v: ValueId) -> Result<ValueId, LowerError> {
    match cx.ty_of(v) {
        Some(MirType::F64) => Ok(v),
        Some(MirType::I64) => {
            let block = cx.current_block;
            let f = cx.builder.int_to_float(block, v);
            cx.set_ty(f, MirType::F64);
            Ok(f)
        }
        _ => Err(cx.err("Math two-arg needs float operands")),
    }
}
