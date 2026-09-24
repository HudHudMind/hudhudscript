//! CallNative (runtime helper) çevirisi (LLVM backend alt modülü).
//! Yan etkili çağrılar void fn olarak declare edilir ve doğrudan build_call
//! ile yayınlanır (LLVM'de kullanılmayan çağrı da ölü kod elenmez —
//! çağrının kendisi yan etkidir).

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirType, RuntimeHelperId, ValueId};
use inkwell::values::{BasicValue, FunctionValue};

use super::{ext_f64_fn, ext_f64_fn2, ext_f64_to_i32_fn, ext_i64_fn, FnCx};

pub(crate) fn translate_call_native<'ctx, 'm>(
    cx: &mut FnCx<'ctx, 'm>,
    dst: ValueId,
    ty: &MirType,
    helper: &RuntimeHelperId,
    args: &[ValueId],
) -> Result<(), BackendError> {
    let i64t = cx.i64t;
    match helper {
        RuntimeHelperId::Print => {
            if cx.is_f64(args[0]) {
                // f64 → float helper (ival sayısal çevirirdi)
                let f = ext_f64_to_i32_fn(cx, "hudhud_print_float");
                let v = cx.fval(args[0])?;
                cx.b.build_call(f, &[v.into()], "");
            } else {
                let f = ext_i64_fn(cx, "hudhud_print_int", 1);
                let v = cx.ival(args[0])?;
                cx.b.build_call(f, &[v.into()], "");
            }
            cx.env.insert(dst.0, i64t.const_zero().as_basic_value_enum());
        }
        RuntimeHelperId::PrintStr => {
            let f = ext_i64_fn(cx, "hudhud_print_str", 1);
            let v = cx.ival(args[0])?;
            cx.b.build_call(f, &[v.into()], "");
            cx.env.insert(dst.0, i64t.const_zero().as_basic_value_enum());
        }
        RuntimeHelperId::ThrowOverflow => {
            return Err(cx.err("throw lane arrives with the trampoline step"));
        }
        RuntimeHelperId::GlobalsHandle => {
            let f = ext_i64_fn(cx, "hudhud_globals", 0);
            let r = cx.b.build_call(f, &[], "gh").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::GlobalGet => {
            let f = ext_i64_fn(cx, "hudhud_global_get", 1);
            let slot = cx.ival(args[0])?;
            let r = cx.b.build_call(f, &[slot.into()], "gg").try_as_basic_value()
                .left().unwrap().into_int_value();
            // F64 global: bit deseni f64'e çöz (sayısal çevrim DEĞİL)
            let v = if *ty == MirType::F64 {
                cx.b.build_bitcast(r, cx.f64t, "ggf")
            } else {
                r.as_basic_value_enum()
            };
            cx.env.insert(dst.0, v);
        }
        RuntimeHelperId::GlobalSet => {
            let f = ext_i64_fn(cx, "hudhud_global_set", 2);
            let slot = cx.ival(args[0])?;
            let val = if cx.is_f64(args[1]) {
                cx.b.build_bitcast(cx.fval(args[1])?, cx.i64t, "gsb").into_int_value()
            } else {
                cx.ival(args[1])?
            };
            cx.b.build_call(f, &[slot.into(), val.into()], "");
            cx.env.insert(dst.0, i64t.const_zero().as_basic_value_enum());
        }
        RuntimeHelperId::StringToInt => {
            let f = ext_i64_fn(cx, "hudhud_string_to_int", 1);
            let v = cx.ival(args[0])?;
            let r = cx.b.build_call(f, &[v.into()], "s2i").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::StringSplit => {
            let f = ext_i64_fn(cx, "hudhud_string_split", 2);
            let s = cx.ival(args[0])?;
            let d = cx.ival(args[1])?;
            let r = cx.b.build_call(f, &[s.into(), d.into()], "split").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::StringIndexOf => {
            let f = ext_i64_fn(cx, "hudhud_string_index_of", 2);
            let s = cx.ival(args[0])?;
            let n = cx.ival(args[1])?;
            let r = cx.b.build_call(f, &[s.into(), n.into()], "idx").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::TypeOf => {
            let f = ext_i64_fn(cx, "hudhud_typeof", 1);
            let v = cx.ival(args[0])?;
            let r = cx.b.build_call(f, &[v.into()], "ty").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::StringCmp | RuntimeHelperId::StringAppend => {
            let name = if helper == &RuntimeHelperId::StringCmp { "hudhud_string_cmp" } else { "hudhud_string_append" };
            let f = ext_i64_fn(cx, name, 2);
            let a = cx.ival(args[0])?;
            let b = cx.ival(args[1])?;
            let r = cx.b.build_call(f, &[a.into(), b.into()], "scall").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::DateMillis => {
            let f = ext_i64_fn(cx, "hudhud_date_millis", 0);
            let r = cx.b.build_call(f, &[], "dms").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::MathPow | RuntimeHelperId::MathMin | RuntimeHelperId::MathMax => {
            let (name, what) = match helper {
                RuntimeHelperId::MathPow => ("hudhud_math_pow", "pow"),
                RuntimeHelperId::MathMin => ("hudhud_math_min", "min"),
                _ => ("hudhud_math_max", "max"),
            };
            let f = ext_f64_fn2(cx, name);
            let a = cx.fval(args[0])?;
            let b = cx.fval(args[1])?;
            let r = cx.b.build_call(f, &[a.into(), b.into()], what).try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::MathFloor | RuntimeHelperId::MathAbs => {
            let (name, what) = match helper {
                RuntimeHelperId::MathFloor => ("hudhud_math_floor", "floor"),
                _ => ("hudhud_math_abs", "abs"),
            };
            let f = ext_f64_fn(cx, name);
            let v = cx.fval(args[0])?;
            let r = cx.b.build_call(f, &[v.into()], what).try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::MathSin | RuntimeHelperId::MathSqrt | RuntimeHelperId::MathCos => {
            let (name, what) = match helper {
                RuntimeHelperId::MathSin => ("hudhud_math_sin", "sin"),
                RuntimeHelperId::MathSqrt => ("hudhud_math_sqrt", "sqrt"),
                _ => ("hudhud_math_cos", "cos"),
            };
            let f = ext_f64_fn(cx, name);
            let v = cx.fval(args[0])?;
            let r = cx.b.build_call(f, &[v.into()], what).try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::Throw => {
            let f = ext_i64_fn(cx, "hudhud_throw", 1);
            let v = cx.ival(args[0])?;
            cx.b.build_call(f, &[v.into()], "throw_call");
        }
        RuntimeHelperId::HasException => {
            let f = ext_i64_fn(cx, "hudhud_has_exception", 0);
            let r = cx.b.build_call(f, &[], "has_ex").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::Catch => {
            let f = ext_i64_fn(cx, "hudhud_catch", 0);
            let r = cx.b.build_call(f, &[], "catch_ex").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::ArrayFilled => {
            let f = ext_i64_fn(cx, "hudhud_array_filled", 2);
            let len = cx.ival(args[0])?;
            let val = cx.ival(args[1])?;
            let r = cx.b.build_call(f, &[len.into(), val.into()], "arr_filled").try_as_basic_value();
            cx.env.insert(dst.0, r.left().unwrap());
        }
        RuntimeHelperId::ArrayFill => {
            // gerçek imza: (arr, length, value) → void; dst = dizi handle
            let f = cx.ext_void_fn("hudhud_array_fill", 3);
            let arr = cx.ival(args[0])?;
            let len = cx.ival(args[1])?;
            let val = if cx.is_f64(args[2]) {
                cx.b.build_bitcast(cx.fval(args[2])?, cx.i64t, "afvb").into_int_value()
            } else {
                cx.ival(args[2])?
            };
            cx.b.build_call(f, &[arr.into(), len.into(), val.into()], "arr_fill");
            cx.env.insert(dst.0, arr.as_basic_value_enum());
        }
    }
    Ok(())
}
