//! CallNative (runtime helper) çevirisi (translate alt modülü).
//!
//! gccjit'te çağrı rvalue'u kullanılmazsa ölü kod olarak atılır: yan etkili
//! tüm helper çağrıları `add_eval` ile zorunlu tutulur (Throw, GlobalSet,
//! ArrayFill düşürme regresyonları burada kapatıldı).

use gccjit::{Block, RValue, ToRValue};
use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirType, RuntimeHelperId, ValueId};

use super::FnCx;

pub(crate) fn translate_call_native<'ctx, 'a>(
    cx: &mut FnCx<'ctx, 'a>,
    gb: Block<'ctx>,
    dst: ValueId,
    ty: &MirType,
    helper: &RuntimeHelperId,
    args: &[ValueId],
) -> Result<(), BackendError> {
    let g = cx.abi.gcx;
    match helper {
        RuntimeHelperId::Print => {
            let v = cx.val(args[0])?;
            // f64 → float helper (int helper'a f64 geçirmek tip
            // karışıklığı/bit deseni üretir)
            let call = if cx.float_vals.contains(&args[0].0) {
                g.new_call(None, cx.ext.print_float, &[v])
            } else {
                g.new_call(None, cx.ext.print_int, &[v])
            };
            gb.add_eval(None, call);
            cx.store(gb, dst, cx.ll(0), false);
        }
        RuntimeHelperId::PrintStr => {
            let v = cx.to_ptr(cx.val(args[0])?);
            let call = g.new_call(None, cx.ext.print_str, &[v]);
            gb.add_eval(None, call);
            cx.store(gb, dst, cx.ll(0), false);
        }
        RuntimeHelperId::ThrowOverflow => {
            return Err(cx.err("throw lane arrives with the trampoline step"));
        }
        RuntimeHelperId::GlobalsHandle => {
            let v = g.new_call(None, cx.ext.globals_handle, &[]);
            cx.store(gb, dst, v, false);
        }
        RuntimeHelperId::DateMillis => {
            let v = g.new_call(None, cx.ext.date_millis, &[]);
            cx.store(gb, dst, v, false);
        }
        RuntimeHelperId::GlobalGet => {
            let slot = cx.val(args[0])?;
            let v = g.new_call(None, cx.ext.global_get, &[slot]);
            // F64 global: i64→f64 helper (bit pattern)
            let v = if *ty == MirType::F64 {
                let raw = cx.ext.f64_from_i64;
                g.new_call(None, raw, &[v])
            } else {
                v
            };
            cx.store(gb, dst, v, *ty == MirType::F64);
        }
        RuntimeHelperId::GlobalSet => {
            let slot = cx.val(args[0])?;
            let val = cx.val(args[1])?;
            // F64 değer: i64'ye bitcast
            let val_i = if cx.float_vals.contains(&args[1].0) {
                let cast = cx.ext.i64_from_f64;
                g.new_call(None, cast, &[val])
            } else {
                val
            };
            let call = g.new_call(None, cx.ext.global_set, &[slot, val_i]);
            gb.add_eval(None, call);
            cx.store(gb, dst, cx.ll(0), false);
        }
        RuntimeHelperId::StringToInt => {
            let sv = cx.to_ptr(cx.val(args[0])?);
            let v = g.new_call(None, cx.ext.string_to_int, &[sv]);
            cx.store(gb, dst, v, false);
        }
        RuntimeHelperId::StringSplit => {
            let sv = cx.to_ptr(cx.val(args[0])?);
            let dv = cx.to_ptr(cx.val(args[1])?);
            let r = g.new_call(None, cx.ext.string_split, &[sv, dv]);
            cx.store(gb, dst, cx.to_handle(r), false);
        }
        RuntimeHelperId::StringIndexOf => {
            let sv = cx.to_ptr(cx.val(args[0])?);
            let nv = cx.to_ptr(cx.val(args[1])?);
            let v = g.new_call(None, cx.ext.string_index_of, &[sv, nv]);
            cx.store(gb, dst, v, false);
        }
        RuntimeHelperId::TypeOf => {
            let v = cx.val(args[0])?;
            let r = g.new_call(None, cx.ext.typeof_fn, &[v]);
            cx.store(gb, dst, cx.to_handle(r), false);
        }
        RuntimeHelperId::StringCmp => {
            let a = cx.to_ptr(cx.val(args[0])?);
            let b = cx.to_ptr(cx.val(args[1])?);
            let v = g.new_call(None, cx.ext.string_cmp, &[a, b]);
            cx.store(gb, dst, v, false);
        }
        RuntimeHelperId::StringAppend => {
            let a = cx.to_ptr(cx.val(args[0])?);
            let b = cx.to_ptr(cx.val(args[1])?);
            let r = g.new_call(None, cx.ext.string_append, &[a, b]);
            cx.store(gb, dst, cx.to_handle(r), false);
        }
        RuntimeHelperId::Throw => {
            let v = cx.val(args[0])?;
            let call = g.new_call(None, cx.ext.throw_fn, &[v]);
            gb.add_eval(None, call);
        }
        RuntimeHelperId::HasException => {
            let v = g.new_call(None, cx.ext.has_exception, &[]);
            cx.store(gb, dst, v, false);
        }
        RuntimeHelperId::Catch => {
            let v = g.new_call(None, cx.ext.catch_fn, &[]);
            cx.store(gb, dst, v, false);
        }
        RuntimeHelperId::ArrayFilled => {
            let len = cx.val(args[0])?;
            let val = cx.val(args[1])?;
            let r = g.new_call(None, cx.ext.array_filled, &[len, val]);
            cx.store(gb, dst, cx.to_handle(r), false);
        }
        RuntimeHelperId::ArrayFill => {
            let handle = cx.val(args[0])?;
            let ap = cx.to_ptr(handle);
            let len = cx.val(args[1])?;
            let val = cx.val(args[2])?;
            let call = g.new_call(None, cx.ext.array_fill, &[ap, len, val]);
            gb.add_eval(None, call);
            // cranelift paritesi: dst = dizi handle'ı geri döner
            cx.store(gb, dst, handle, false);
        }
        RuntimeHelperId::MathSin => {
            let v = cx.val(args[0])?;
            let r = g.new_call(None, cx.ext.math_sin, &[v]);
            cx.store(gb, dst, r, true);
        }
        RuntimeHelperId::MathSqrt => {
            let v = cx.val(args[0])?;
            let r = g.new_call(None, cx.ext.math_sqrt, &[v]);
            cx.store(gb, dst, r, true);
        }
        RuntimeHelperId::MathCos => {
            let v = cx.val(args[0])?;
            let r = g.new_call(None, cx.ext.math_cos, &[v]);
            cx.store(gb, dst, r, true);
        }
        RuntimeHelperId::MathFloor => {
            let v = cx.val(args[0])?;
            let r = g.new_call(None, cx.ext.math_floor, &[v]);
            cx.store(gb, dst, r, true);
        }
        RuntimeHelperId::MathAbs => {
            let v = cx.val(args[0])?;
            let r = g.new_call(None, cx.ext.math_abs, &[v]);
            cx.store(gb, dst, r, true);
        }
        RuntimeHelperId::MathPow | RuntimeHelperId::MathMin | RuntimeHelperId::MathMax => {
            // int argümanlar (Math.pow(2, 10) literalleri) sayısal f64 cast ile girer
            let ra = cx.val(args[0])?;
            let rb = cx.val(args[1])?;
            let a = if cx.float_vals.contains(&args[0].0) { ra } else { g.new_cast(None, ra, cx.abi.f64t) };
            let b = if cx.float_vals.contains(&args[1].0) { rb } else { g.new_cast(None, rb, cx.abi.f64t) };
            let f = match helper {
                RuntimeHelperId::MathPow => cx.ext.math_pow,
                RuntimeHelperId::MathMin => cx.ext.math_min,
                _ => cx.ext.math_max,
            };
            let r = g.new_call(None, f, &[a, b]);
            cx.store(gb, dst, r, true);
        }
    }
    let _ = g;
    Ok(())
}
