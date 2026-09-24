//! MIR instruction çevirisi (LLVM backend alt modülü).
//! Aritmetik/karşılaştırma kolları (BigInt promote'lu) arith.rs'tedir.

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirType, RuntimeHelperId, ValueId};
use inkwell::values::{BasicValue, FunctionValue};

use super::{ext_f64_to_i64_fn, ext_i64_fn, FnCx};

pub(crate) fn translate_insts<'ctx, 'm>(
    cx: &mut FnCx<'ctx, 'm>,
    func: FunctionValue<'ctx>,
    _bid: u32,
    insts: &[hudhudscript_mir::MirInst],
    string_table: &[std::sync::Arc<str>],
) -> Result<(), BackendError> {

    use hudhudscript_mir::MirInst as I;
    let i64t = cx.i64t;
    for inst in insts {
        // arith kolları blok bölebilir; builder merge'ye konumlanır ve
        // kalan instruction'lar otomatik oraya yazılır (terminator dahil —
        // term.rs güncel bloktan phi incoming kaydeder)
        if super::arith::try_translate_arith(cx, func, inst)? {
            continue;
        }
        match inst {
            I::Param { dst, ty, index } => {
                let args_ptr = func.get_nth_param(1).unwrap().into_pointer_value();
                let v = if *ty == MirType::F64 {
                    // F64 parametre uniform ABI'de BİT DESENİ olarak taşınır;
                    // eski kod bitleri ADRES sanıp int_to_ptr + deref yapıyordu
                    // → float parametreli her çağrı SEGFAULT (LLVM #fix:
                    // polynomial/n_body/monte_carlo/simpson/fasta ailesi).
                    let rawp = unsafe { cx.b.build_in_bounds_gep(args_ptr, &[i64t.const_int(*index as u64, false)], "argfp") };
                    let raw = cx.b.build_load(rawp, "arg_raw").into_int_value();
                    cx.b.build_bitcast(raw, cx.f64t, "arg_f")
                } else {
                    let elem = unsafe { cx.b.build_in_bounds_gep(args_ptr, &[i64t.const_int(*index as u64, false)], "argp") };
                    cx.b.build_load(elem, "arg")
                };
                cx.env.insert(dst.0, v);
            }
            I::ConstInt { dst, value, .. } => {
                cx.env.insert(dst.0, i64t.const_int(*value as u64, false).as_basic_value_enum());
            }
            I::ConstBool { dst, value } => {
                cx.env.insert(dst.0, i64t.const_int(*value as u64, false).as_basic_value_enum());
            }
            I::ConstNull { dst } => {
                cx.env.insert(dst.0, i64t.const_zero().as_basic_value_enum());
            }
            I::ConstFloat { dst, bits, .. } => {
                let f = f64::from_bits(*bits as u64);
                cx.env.insert(dst.0, cx.f64t.const_float(f).as_basic_value_enum());
            }
            I::ConstString { dst, index } => {
                let s = string_table.get(*index as usize)
                    .map(|s| s.as_ref().to_string())
                    .unwrap_or_default();
                let gv = cx.b.build_global_string_ptr(&s, &format!("str_{index}"));
                let h = cx.to_handle(gv.as_pointer_value(), "strh");
                cx.env.insert(dst.0, h.as_basic_value_enum());
            }
            I::LogicalAnd { dst, lhs, rhs } => {
                let v = cx.b.build_and(cx.ival(*lhs)?, cx.ival(*rhs)?, "and");
                cx.env.insert(dst.0, v.as_basic_value_enum());
            }
            I::LogicalOr { dst, lhs, rhs } => {
                let v = cx.b.build_or(cx.ival(*lhs)?, cx.ival(*rhs)?, "or");
                cx.env.insert(dst.0, v.as_basic_value_enum());
            }
            I::LogicalNot { dst, src } => {
                let one = i64t.const_int(1, false);
                let v = cx.b.build_int_sub(one, cx.ival(*src)?, "not");
                cx.env.insert(dst.0, v.as_basic_value_enum());
            }
            I::UnaryNeg { dst, ty, src } => {
                if *ty == MirType::F64 {
                    let v = cx.b.build_float_neg(cx.fval(*src)?, "fneg");
                    cx.env.insert(dst.0, v.as_basic_value_enum());
                } else {
                    let zero = i64t.const_zero();
                    let v = cx.b.build_int_sub(zero, cx.ival(*src)?, "neg");
                    cx.env.insert(dst.0, v.as_basic_value_enum());
                }
            }
            I::StringConcat { dst, lhs, rhs } => {
                let f = ext_i64_fn(cx, "hudhud_string_concat", 2);
                let l = cx.ival(*lhs)?;
                let r = cx.ival(*rhs)?;
                let p = cx.b.build_call(f, &[l.into(), r.into()], "concat").try_as_basic_value();
                cx.env.insert(dst.0, p.left().unwrap().as_basic_value_enum());
            }
            I::StringLen { dst, src } => {
                let f = ext_i64_fn(cx, "hudhud_string_len", 1);
                let s = cx.ival(*src)?;
                let v = cx.b.build_call(f, &[s.into()], "slen").try_as_basic_value();
                cx.env.insert(dst.0, v.left().unwrap().as_basic_value_enum());
            }
            I::StringEq { dst, lhs, rhs } => {
                let f = ext_i64_fn(cx, "hudhud_string_eq", 2);
                let l = cx.ival(*lhs)?;
                let r = cx.ival(*rhs)?;
                let v = cx.b.build_call(f, &[l.into(), r.into()], "seq").try_as_basic_value();
                cx.env.insert(dst.0, v.left().unwrap().as_basic_value_enum());
            }
            I::IntToString { dst, src } => {
                let f = ext_i64_fn(cx, "hudhud_int_to_string", 1);
                let v = cx.ival(*src)?;
                let r = cx.b.build_call(f, &[v.into()], "i2s").try_as_basic_value();
                cx.env.insert(dst.0, r.left().unwrap().as_basic_value_enum());
            }
            I::FloatToString { dst, src } => {
                let f = ext_f64_to_i64_fn(cx, "hudhud_float_to_string");
                let v = cx.fval(*src)?;
                let r = cx.b.build_call(f, &[v.into()], "f2s").try_as_basic_value();
                cx.env.insert(dst.0, r.left().unwrap().as_basic_value_enum());
            }
            I::ArrayNew { dst, capacity } => {
                let f = ext_i64_fn(cx, "hudhud_array_new", 1);
                let cap = cx.ival(*capacity)?;
                let r = cx.b.build_call(f, &[cap.into()], "arrnew").try_as_basic_value();
                cx.env.insert(dst.0, r.left().unwrap().as_basic_value_enum());
            }
            I::ArrayPush { arr, value } => {
                let f = cx.ext_void_fn("hudhud_array_push", 2);
                let a = cx.ival(*arr)?;
                // f64 eleman: bit deseni i64 slot'a bitcast (sayısal çevrim
                // DEĞİL — ArraySet/CallStatic deseni; fft/lu/spectral_norm
                // ailesinin yanlış-sonuç köküydü)
                let v = if cx.is_f64(*value) {
                    cx.b
                        .build_bitcast(cx.fval(*value)?, i64t, "apb")
                        .into_int_value()
                } else {
                    cx.ival(*value)?
                };
                cx.b.build_call(f, &[a.into(), v.into()], "");
            }
            I::ArrayGet { dst, arr, index, ty } => {
                let f = ext_i64_fn(cx, "hudhud_array_get", 2);
                let a = cx.ival(*arr)?;
                let i = cx.ival(*index)?;
                let r = cx.b.build_call(f, &[a.into(), i.into()], "arrget").try_as_basic_value()
                    .left().unwrap().into_int_value();
                // f64 eleman: bit deseni f64'e bitcast (sayısal çevrim DEĞİL)
                let v = if *ty == MirType::F64 {
                    cx.b.build_bitcast(r, cx.f64t, "aelemf")
                } else {
                    r.as_basic_value_enum()
                };
                cx.env.insert(dst.0, v);
            }
            I::ArraySet { arr, index, value } => {
                let f = cx.ext_void_fn("hudhud_array_set", 3);
                let a = cx.ival(*arr)?;
                let i = cx.ival(*index)?;
                let v = if cx.is_f64(*value) {
                    cx.b.build_bitcast(cx.fval(*value)?, cx.i64t, "asetb").into_int_value()
                } else {
                    cx.ival(*value)?
                };
                cx.b.build_call(f, &[a.into(), i.into(), v.into()], "");
            }
            I::ArrayPop { dst, arr } => {
                let f = ext_i64_fn(cx, "hudhud_array_pop", 1);
                let a = cx.ival(*arr)?;
                let r = cx.b.build_call(f, &[a.into()], "arrpop").try_as_basic_value();
                cx.env.insert(dst.0, r.left().unwrap());
            }
            I::ArrayLen { dst, arr } => {
                let f = ext_i64_fn(cx, "hudhud_array_len", 1);
                let a = cx.ival(*arr)?;
                let r = cx.b.build_call(f, &[a.into()], "arrlen").try_as_basic_value();
                cx.env.insert(dst.0, r.left().unwrap().as_basic_value_enum());
            }
            I::ObjectNew { dst } => {
                let f = ext_i64_fn(cx, "hudhud_object_new", 0);
                let r = cx.b.build_call(f, &[], "objnew").try_as_basic_value();
                cx.env.insert(dst.0, r.left().unwrap().as_basic_value_enum());
            }
            I::ObjectSet { obj, key, value } => {
                let f = cx.ext_void_fn("hudhud_object_set", 3);
                let o = cx.ival(*obj)?;
                let k = cx.ival(*key)?;
                // f64 değer: bit deseni i64 slot'a bitcast (ArrayPush deseni)
                let v = if cx.is_f64(*value) {
                    cx.b
                        .build_bitcast(cx.fval(*value)?, i64t, "osb")
                        .into_int_value()
                } else {
                    cx.ival(*value)?
                };
                cx.b.build_call(f, &[o.into(), k.into(), v.into()], "");
            }
            I::ObjectGet { dst, obj, key, .. } => {
                let f = ext_i64_fn(cx, "hudhud_object_get", 2);
                let o = cx.ival(*obj)?;
                let k = cx.ival(*key)?;
                let r = cx.b.build_call(f, &[o.into(), k.into()], "objget").try_as_basic_value();
                cx.env.insert(dst.0, r.left().unwrap().as_basic_value_enum());
            }
            I::ObjectHas { dst, obj, key } => {
                let f = ext_i64_fn(cx, "hudhud_object_has", 2);
                let o = cx.ival(*obj)?;
                let k = cx.ival(*key)?;
                let r = cx.b.build_call(f, &[o.into(), k.into()], "objhas").try_as_basic_value();
                cx.env.insert(dst.0, r.left().unwrap().as_basic_value_enum());
            }
            I::ObjectLen { dst, obj } => {
                let f = ext_i64_fn(cx, "hudhud_object_len", 1);
                let o = cx.ival(*obj)?;
                let r = cx.b.build_call(f, &[o.into()], "objlen").try_as_basic_value();
                cx.env.insert(dst.0, r.left().unwrap().as_basic_value_enum());
            }
            I::CallStatic { dst, ty, callee, args, .. } => {
                let callee_fn = cx
                    .module_funcs
                    .get(&callee.0)
                    .copied()
                    .ok_or_else(|| cx.err(&format!("unknown callee index {}", callee.0)))?;
                // Scratch entry'de ayrıldı (döngüde alloca = stack büyütür).
                // 8 arg'a kadar destek — fazlası MIR tarafından reddedilmeli.
                let arr = cx
                    .call_scratch
                    .expect("call scratch allocated at entry")
                    .0;
                for (i, a) in args.iter().enumerate() {
                    // f64 argüman: bit deseni korunur (uniform i64 ABI)
                    let v = if cx.is_f64(*a) {
                        cx.b.build_bitcast(cx.fval(*a)?, cx.i64t, "csa").into_int_value()
                    } else {
                        cx.ival(*a)?
                    };
                    let elem = unsafe { cx.b.build_in_bounds_gep(arr, &[i64t.const_int(i as u64, false)], "argelem") };
                    cx.b.build_store(elem, v);
                }
                let out = cx
                    .call_scratch
                    .expect("call scratch allocated at entry")
                    .1;
                let argc = cx.i32t.const_int(args.len() as u64, false);
                cx.b.build_call(callee_fn, &[argc.into(), arr.into(), out.into()], "callstatic");
                let valp = unsafe { cx.b.build_struct_gep(out, 1, "outval").unwrap() };
                let raw = cx.b.build_load(valp, "retval").into_int_value();
                // f64 dönüş: bit deseni f64'e bitcast (sayısal çevrim DEĞİL)
                let v = if *ty == MirType::F64 {
                    cx.b.build_bitcast(raw, cx.f64t, "csr")
                } else {
                    raw.as_basic_value_enum()
                };
                cx.env.insert(dst.0, v);
            }
            I::CallNative { dst, ty, helper, args, .. } => {
                super::call_native::translate_call_native(cx, *dst, ty, helper, args)?;
            }
            I::StringCharAt { dst, s, index } => {
                // s[i] — char_at(ptr, i64) → ptr handle (LLVM'de HİÇ yoktu;
                // 15 string benchmark'ı AOT-LLVM'de reddediliyordu)
                let f = ext_ptr_i64_fn(cx, "hudhud_string_char_at");
                let sv = cx.to_ptr(cx.ival(*s)?, "scv");
                let i = cx.ival(*index)?;
                let r = cx.b.build_call(f, &[sv.into(), i.into()], "charat").try_as_basic_value();
                let h = cx.to_handle(r.left().unwrap().into_pointer_value(), "ch");
                cx.env.insert(dst.0, h.as_basic_value_enum());
            }
            I::StringSubstring { dst, s, start, end } => {
                let f = ext_substring_fn(cx);
                let sv = cx.to_ptr(cx.ival(*s)?, "ssv");
                let st = cx.ival(*start)?;
                let en = cx.ival(*end)?;
                let r = cx.b.build_call(f, &[sv.into(), st.into(), en.into()], "substr").try_as_basic_value();
                let h = cx.to_handle(r.left().unwrap().into_pointer_value(), "sj");
                cx.env.insert(dst.0, h.as_basic_value_enum());
            }
            I::ArrayJoin { dst, arr, sep } => {
                let f = ext_join_fn(cx);
                let a = cx.to_ptr(cx.ival(*arr)?, "aja");
                let sp = cx.to_ptr(cx.ival(*sep)?, "ajs");
                let r = cx.b.build_call(f, &[a.into(), sp.into()], "join").try_as_basic_value();
                let h = cx.to_handle(r.left().unwrap().into_pointer_value(), "jn");
                cx.env.insert(dst.0, h.as_basic_value_enum());
            }
            I::IntToFloat { dst, src } => {
                let raw = cx.val(*src)?;
                let v = if cx.is_f64(*src) {
                    raw
                } else {
                    cx.b
                        .build_signed_int_to_float(raw.into_int_value(), cx.f64t, "i2f")
                        .as_basic_value_enum()
                };
                cx.env.insert(dst.0, v);
            }
            I::Neg { dst, ty, src } => {
                if *ty == MirType::F64 {
                    let v = cx.b.build_float_neg(cx.fval(*src)?, "fneg");
                    cx.env.insert(dst.0, v.as_basic_value_enum());
                } else {
                    let zero = cx.i64t.const_zero();
                    let v = cx.b.build_int_sub(zero, cx.ival(*src)?, "neg");
                    cx.env.insert(dst.0, v.as_basic_value_enum());
                }
            }
            I::Not { dst, src } => {
                let one = cx.i64t.const_int(1, false);
                let v = cx.b.build_int_sub(one, cx.ival(*src)?, "not");
                cx.env.insert(dst.0, v.as_basic_value_enum());
            }
            I::GcSafepoint => {}
            other => {
                return Err(cx.err(&format!("instruction {other:?} not translated yet")));
            }
        }
    }
    Ok(())
}

/// (ptr, i64) → ptr extern helper (string_char_at biçimi).
fn ext_ptr_i64_fn<'ctx, 'm>(cx: &FnCx<'ctx, 'm>, name: &str) -> FunctionValue<'ctx> {
    if let Some(f) = cx.module.get_function(name) {
        return f;
    }
    let fty = cx.i8ptr.fn_type(&[cx.i8ptr.into(), cx.i64t.into()], false);
    cx.module.add_function(name, fty, None)
}

/// (ptr, i64, i64) → ptr extern helper (string_substring biçimi).
fn ext_substring_fn<'ctx, 'm>(cx: &FnCx<'ctx, 'm>) -> FunctionValue<'ctx> {
    let name = "hudhud_string_substring";
    if let Some(f) = cx.module.get_function(name) {
        return f;
    }
    let fty = cx.i8ptr.fn_type(&[cx.i8ptr.into(), cx.i64t.into(), cx.i64t.into()], false);
    cx.module.add_function(name, fty, None)
}

/// (ptr, ptr) → ptr extern helper (array_join biçimi).
fn ext_join_fn<'ctx, 'm>(cx: &FnCx<'ctx, 'm>) -> FunctionValue<'ctx> {
    let name = "hudhud_array_join";
    if let Some(f) = cx.module.get_function(name) {
        return f;
    }
    let fty = cx.i8ptr.fn_type(&[cx.i8ptr.into(), cx.i8ptr.into()], false);
    cx.module.add_function(name, fty, None)
}
