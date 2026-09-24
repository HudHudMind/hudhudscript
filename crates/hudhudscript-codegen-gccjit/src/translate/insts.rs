//! MIR instruction'larının gccjit'e çevirisi (translate alt modülü).
//! CallNative (runtime helper) kolları call_native.rs'tedir;
//! aritmetik/karşılaştırma kolları (BigInt promote'lu) arith.rs'tedir.

use gccjit::{BinaryOp, Block, RValue, ToRValue};
use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirType, ValueId};

use super::FnCx;

/// Blok doldurur ve GÜNCEL bloğu döndürür: arith promote kolları bloğu
/// bölünce kalan instruction'lar ile terminator merge bloğuna yazılmalıdır.
pub(crate) fn translate_insts<'ctx, 'a>(
    cx: &mut FnCx<'ctx, 'a>,
    gb: Block<'ctx>,
    _bid: u32,
    insts: &[hudhudscript_mir::MirInst],
    args_param: &RValue<'ctx>,
    string_table: &[std::sync::Arc<str>],
) -> Result<Block<'ctx>, BackendError> {
    let g = cx.abi.gcx;
    let mut gb = gb;
    use hudhudscript_mir::MirInst as I;
    for inst in insts {
        if super::arith::try_translate_arith(cx, &mut gb, inst)? {
            continue;
        }
        match inst {
            I::Param { dst, ty, index } => {
                let elem = g.new_array_access(None, *args_param, g.new_rvalue_from_long(cx.abi.ll, *index as i64));
                if *ty == MirType::F64 {
                    // f64 parametre uniform ABI'de bit deseni olarak taşınır
                    let raw = elem.to_rvalue();
                    let v = g.new_call(None, cx.ext.f64_from_i64, &[raw]);
                    cx.store(gb, *dst, v, true);
                } else {
                    cx.store(gb, *dst, elem.to_rvalue(), false);
                }
            }
            I::ConstInt { dst, value, .. } => {
                cx.store(gb, *dst, cx.ll(*value), false);
            }
            I::ConstBool { dst, value } => {
                cx.store(gb, *dst, cx.ll(*value as i64), false);
            }
            I::ConstNull { dst } => {
                cx.store(gb, *dst, cx.ll(0), false);
            }
            I::ConstFloat { dst, bits, .. } => {
                let f = f64::from_bits(*bits as u64);
                cx.store(gb, *dst, g.new_rvalue_from_double(cx.abi.f64t, f), true);
            }
            I::ConstString { dst, index } => {
                let s = string_table.get(*index as usize)
                    .map(|s| s.as_ref().to_string())
                    .unwrap_or_default();
                let lit = g.new_string_literal(s);
                cx.store(gb, *dst, cx.to_handle(lit), false);
            }
            I::LogicalAnd { dst, lhs, rhs } => {
                let l = cx.val(*lhs)?;
                let r = cx.val(*rhs)?;
                let v = g.new_binary_op(None, BinaryOp::BitwiseAnd, cx.abi.ll, l, r);
                cx.store(gb, *dst, v, false);
            }
            I::LogicalOr { dst, lhs, rhs } => {
                let l = cx.val(*lhs)?;
                let r = cx.val(*rhs)?;
                let v = g.new_binary_op(None, BinaryOp::BitwiseOr, cx.abi.ll, l, r);
                cx.store(gb, *dst, v, false);
            }
            I::LogicalNot { dst, src } => {
                let s = cx.val(*src)?;
                let one = cx.ll(1);
                let v = g.new_binary_op(None, BinaryOp::Minus, cx.abi.ll, one, s);
                cx.store(gb, *dst, v, false);
            }
            I::UnaryNeg { dst, ty, src } => {
                let s = cx.val(*src)?;
                if *ty == MirType::F64 {
                    let v = g.new_unary_op(None, gccjit::UnaryOp::Minus, cx.abi.f64t, s);
                    cx.store(gb, *dst, v, true);
                } else {
                    let zero = cx.ll(0);
                    let v = g.new_binary_op(None, BinaryOp::Minus, cx.abi.ll, zero, s);
                    cx.store(gb, *dst, v, false);
                }
            }
            I::StringConcat { dst, lhs, rhs } => {
                let l = cx.to_ptr(cx.val(*lhs)?);
                let r = cx.to_ptr(cx.val(*rhs)?);
                let p = g.new_call(None, cx.ext.string_concat, &[l, r]);
                cx.store(gb, *dst, cx.to_handle(p), false);
            }
            I::StringLen { dst, src } => {
                let s = cx.to_ptr(cx.val(*src)?);
                let v = g.new_call(None, cx.ext.string_len, &[s]);
                cx.store(gb, *dst, v, false);
            }
            I::StringEq { dst, lhs, rhs } => {
                let l = cx.to_ptr(cx.val(*lhs)?);
                let r = cx.to_ptr(cx.val(*rhs)?);
                let v = g.new_call(None, cx.ext.string_eq, &[l, r]);
                cx.store(gb, *dst, v, false);
            }
            I::IntToString { dst, src } => {
                let v = cx.val(*src)?;
                let p = g.new_call(None, cx.ext.int_to_string, &[v]);
                cx.store(gb, *dst, cx.to_handle(p), false);
            }
            I::FloatToString { dst, src } => {
                let v = cx.val(*src)?;
                let p = g.new_call(None, cx.ext.float_to_string, &[v]);
                cx.store(gb, *dst, cx.to_handle(p), false);
            }
            I::ArrayNew { dst, capacity } => {
                let cap = cx.val(*capacity)?;
                let p = g.new_call(None, cx.ext.array_new, &[cap]);
                cx.store(gb, *dst, cx.to_handle(p), false);
            }
            I::ArrayPush { arr, value } => {
                let a = cx.to_ptr(cx.val(*arr)?);
                let raw = cx.val(*value)?;
                let v = if cx.float_vals.contains(&value.0) {
                    g.new_call(None, cx.ext.i64_from_f64, &[raw])
                } else {
                    raw
                };
                let call = g.new_call(None, cx.ext.array_push, &[a, v]);
                gb.add_eval(None, call);
            }
            I::ArrayGet { dst, arr, index, ty } => {
                let a = cx.to_ptr(cx.val(*arr)?);
                let i = cx.val(*index)?;
                let raw = g.new_call(None, cx.ext.array_get, &[a, i]);
                // f64 eleman: bit deseni f64'e çöz (sayısal cast DEĞİL —
                // LLVM "aelemf" denklemi; ham cast GCC ICE üretir)
                let v = if *ty == MirType::F64 {
                    g.new_call(None, cx.ext.f64_from_i64, &[raw])
                } else {
                    raw
                };
                cx.store(gb, *dst, v, *ty == MirType::F64);
            }
            I::ArraySet { arr, index, value } => {
                let a = cx.to_ptr(cx.val(*arr)?);
                let i = cx.val(*index)?;
                let raw = cx.val(*value)?;
                let v = if cx.float_vals.contains(&value.0) {
                    g.new_call(None, cx.ext.i64_from_f64, &[raw])
                } else {
                    raw
                };
                let call = g.new_call(None, cx.ext.array_set, &[a, i, v]);
                gb.add_eval(None, call);
            }
            I::StringCharAt { dst, s, index } => {
                let sv = cx.to_ptr(cx.val(*s)?);
                let i = cx.val(*index)?;
                let p = g.new_call(None, cx.ext.string_char_at, &[sv, i]);
                cx.store(gb, *dst, cx.to_handle(p), false);
            }
            I::ArrayLen { dst, arr } => {
                let a = cx.to_ptr(cx.val(*arr)?);
                let v = g.new_call(None, cx.ext.array_len, &[a]);
                cx.store(gb, *dst, v, false);
            }
            I::ObjectNew { dst } => {
                let p = g.new_call(None, cx.ext.object_new, &[]);
                cx.store(gb, *dst, cx.to_handle(p), false);
            }
            I::ObjectSet { obj, key, value } => {
                let o = cx.to_ptr(cx.val(*obj)?);
                let k = cx.to_ptr(cx.val(*key)?);
                let raw = cx.val(*value)?;
                // f64 değer: bit deseni i64 slot'a (sayısal cast DEĞİL —
                // ArrayPush/ArraySet deseni; ham double long param'a tip hatası)
                let v = if cx.float_vals.contains(&value.0) {
                    g.new_call(None, cx.ext.i64_from_f64, &[raw])
                } else {
                    raw
                };
                let call = g.new_call(None, cx.ext.object_set, &[o, k, v]);
                gb.add_eval(None, call);
            }
            I::ObjectGet { dst, obj, key, .. } => {
                let o = cx.to_ptr(cx.val(*obj)?);
                let k = cx.to_ptr(cx.val(*key)?);
                let v = g.new_call(None, cx.ext.object_get, &[o, k]);
                cx.store(gb, *dst, v, false);
            }
            I::ObjectHas { dst, obj, key } => {
                let o = cx.to_ptr(cx.val(*obj)?);
                let k = cx.to_ptr(cx.val(*key)?);
                let v = g.new_call(None, cx.ext.object_has, &[o, k]);
                cx.store(gb, *dst, v, false);
            }
            I::ObjectLen { dst, obj } => {
                let o = cx.to_ptr(cx.val(*obj)?);
                let v = g.new_call(None, cx.ext.object_len, &[o]);
                cx.store(gb, *dst, v, false);
            }
            I::CallStatic { dst, ty, callee, args, .. } => {
                let callee_fn = cx
                    .module_funcs
                    .get(&callee.0)
                    .copied()
                    .ok_or_else(|| cx.err(&format!("unknown callee index {}", callee.0)))?;
                // args buffer: local array + JitExit local
                let arr_ty = g.new_array_type(None, cx.abi.ll, args.len().max(1) as i32);
                let arr = cx.func.new_local(None, arr_ty, "call_args");
                for (i, a) in args.iter().enumerate() {
                    let raw = cx.val(*a)?;
                    // f64 argüman: bit deseni i64 slot'a (sayısal cast DEĞİL)
                    let v = if cx.float_vals.contains(&a.0) {
                        g.new_call(None, cx.ext.i64_from_f64, &[raw])
                    } else {
                        raw
                    };
                    let elem = g.new_array_access(None, arr.to_rvalue(), g.new_rvalue_from_long(cx.abi.ll, i as i64));
                    gb.add_assignment(None, elem, v);
                }
                let out_local = cx.func.new_local(None, cx.abi.jit_exit, "call_out");
                // eleman-0 adresi: long* (dizinin adresi long[N]* olurdu)
                let elem0 = g.new_array_access(None, arr.to_rvalue(), g.new_rvalue_from_long(cx.abi.ll, 0));
                let arr_addr = elem0.get_address(None);
                let out_addr = out_local.get_address(None);
                let argc = g.new_rvalue_from_long(cx.abi.u32t, args.len() as i64);
                let call = g.new_call(None, callee_fn, &[argc, arr_addr, out_addr]);
                gb.add_eval(None, call);
                let val_field = out_local.access_field(None, cx.abi.f_value);
                let raw = val_field.to_rvalue();
                // F64 dönen callee: bit pattern'i f64'ye çöz (cranelift bitcast_i64_to_f64)
                if *ty == MirType::F64 {
                    let v = g.new_call(None, cx.ext.f64_from_i64, &[raw]);
                    cx.store(gb, *dst, v, true);
                } else {
                    cx.store(gb, *dst, raw, false);
                }
            }
            I::CallNative { dst, ty, helper, args, .. } => {
                super::call_native::translate_call_native(cx, gb, *dst, ty, helper, args)?;
            }
            I::ArrayPop { dst, arr } => {
                let a = cx.to_ptr(cx.val(*arr)?);
                let v = g.new_call(None, cx.ext.array_pop, &[a]);
                cx.store(gb, *dst, v, false);
            }
            I::ArrayJoin { dst, arr, sep } => {
                let a = cx.to_ptr(cx.val(*arr)?);
                let sp = cx.to_ptr(cx.val(*sep)?);
                let r = g.new_call(None, cx.ext.array_join, &[a, sp]);
                cx.store(gb, *dst, cx.to_handle(r), false);
            }
            I::StringSubstring { dst, s, start, end } => {
                let sv = cx.to_ptr(cx.val(*s)?);
                let st = cx.val(*start)?;
                let en = cx.val(*end)?;
                let r = g.new_call(None, cx.ext.string_substring, &[sv, st, en]);
                cx.store(gb, *dst, cx.to_handle(r), false);
            }
            I::IntToFloat { dst, src } => {
                let raw = cx.val(*src)?;
                let v = if cx.float_vals.contains(&src.0) {
                    raw
                } else {
                    g.new_cast(None, raw, cx.abi.f64t)
                };
                cx.store(gb, *dst, v, true);
            }
            I::Neg { dst, ty, src } => {
                let sv = cx.val(*src)?;
                if *ty == MirType::F64 {
                    let v = g.new_unary_op(None, gccjit::UnaryOp::Minus, cx.abi.f64t, sv);
                    cx.store(gb, *dst, v, true);
                } else {
                    let zero = cx.ll(0);
                    let v = g.new_binary_op(None, BinaryOp::Minus, cx.abi.ll, zero, sv);
                    cx.store(gb, *dst, v, false);
                }
            }
            I::Not { dst, src } => {
                let sv = cx.val(*src)?;
                let one = cx.ll(1);
                let v = g.new_binary_op(None, BinaryOp::Minus, cx.abi.ll, one, sv);
                cx.store(gb, *dst, v, false);
            }
            I::GcSafepoint => {}
            other => {
                return Err(cx.err(&format!("instruction {:?} not translated yet", std::mem::discriminant(other))));
            }
        }
    }
    Ok(gb)
}
