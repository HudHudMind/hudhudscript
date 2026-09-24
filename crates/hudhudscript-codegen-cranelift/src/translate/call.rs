//! Çağrı instruction'ları: CallStatic (uniform ABI, stack slot argümanları)
//! ve CallNative (print helper'ları).

use cranelift::prelude::types::{F64, I32, I64, Type};
use cranelift::prelude::{AbiParam, FunctionBuilder, InstBuilder, MemFlags, Value, Variable};
use cranelift_module::{Linkage, Module};

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirFunction, MirInst, MirType, RefKind};
use hudhudscript_native_abi::JIT_EXIT_OVERFLOW;

use super::helpers::{
    bitcast_f64_to_i64, bitcast_i64_to_f64, operand, record, reject, OFF_STATUS, OFF_VALUE,
};

type Env = Vec<Option<(Value, MirType)>>;

pub(super) fn translate_call<M: Module>(
    inst: &MirInst,
    builder: &mut FunctionBuilder,
    env: &mut Env,
    module: &mut M,
    ptr: Type,
    func: &MirFunction,
    module_func_ids: &[cranelift_module::FuncId],
    out_ptr: Value,
    ov_var: Variable,
    is_aot: bool,
) -> Result<(), BackendError> {
    match inst {
            MirInst::CallStatic { dst, ty, callee, args, .. } => {
                // Callee'nin uniform ABI'sine çağrı:
                // (argc: i32, args: *i64, out: *JitExit) → void
                let fid = module_func_ids.get(callee.0 as usize).copied()
                    .ok_or_else(|| reject(func, "CallStatic", &format!("unknown callee index {}", callee.0)))?;

                // Stack slot'lar: args dizisi + JitExit çıktısı
                let args_bytes = (args.len() * 8) as u32;
                let ss_args = builder.create_sized_stack_slot(
                    cranelift::prelude::codegen::ir::StackSlotData::new(
                        cranelift::prelude::codegen::ir::StackSlotKind::ExplicitSlot,
                        args_bytes.max(8),
                        3,
                    ),
                );
                let ss_out = builder.create_sized_stack_slot(
                    cranelift::prelude::codegen::ir::StackSlotData::new(
                        cranelift::prelude::codegen::ir::StackSlotKind::ExplicitSlot,
                        16,
                        3,
                    ),
                );

                // Args'ı stack slot'a yaz
                for (i, a) in args.iter().enumerate() {
                    let (v, vt) = operand(env, *a, func)?;
                    let v = if vt == MirType::F64 { bitcast_f64_to_i64(builder, v) } else { v };
                    builder.ins().stack_store(v, ss_args, (i * 8) as i32);
                }

                // Stack adreslerini al
                let args_addr = builder.ins().stack_addr(ptr, ss_args, 0);
                let out_addr = builder.ins().stack_addr(ptr, ss_out, 0);
                let argc = builder.ins().iconst(I32, args.len() as i64);

                // Callee'yi çağır
                let callee_ref = module.declare_func_in_func(fid, builder.func);
                let _call = builder.ins().call(callee_ref, &[argc, args_addr, out_addr]);

                // JitExit.status'u oku (offset 0) ve ov_var'a akümüle et
                let callee_status = builder.ins().stack_load(I32, ss_out, OFF_STATUS);
                let cur_ov = builder.use_var(ov_var);
                let new_ov = builder.ins().bor(cur_ov, callee_status);
                builder.def_var(ov_var, new_ov);

                // JitExit.value'yı oku (offset 8)
                let raw = builder.ins().stack_load(I64, ss_out, OFF_VALUE);
                let result = if *ty == MirType::F64 { bitcast_i64_to_f64(builder, raw) } else { raw };
                record(env, *dst, result, *ty);
            }
            MirInst::CallNative { dst, ty, helper, args } => {
                use hudhudscript_mir::RuntimeHelperId;
                match helper {
                    RuntimeHelperId::PrintStr => {

                        // hudhud_print_str(*const c_char) → i32; tek argüman (string ptr)
                        if args.len() != 1 {
                            return Err(reject(func, "CallNative(PrintStr)", "takes exactly 1 arg"));
                        }
                        let (v, _) = operand(env, args[0], func)?;
                        let print_sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(ptr));
                            sig.returns.push(AbiParam::new(I32));
                            sig
                        };
                        let print_id = module
                            .declare_function("hudhud_print_str", cranelift_module::Linkage::Import, &print_sig)
                            .map_err(|e| reject(func, "PrintStr", &format!("declare: {e}")))?;
                        let print_ref = module.declare_func_in_func(print_id, builder.func);
                        let _rc = builder.ins().call(print_ref, &[v]);
                        let n = builder.ins().iconst(I64, 0);
                        record(env, *dst, n, MirType::Generic);
                    }
                    RuntimeHelperId::Print => {

                        // hudhud_print_int(i64) | hudhud_print_float(f64) → i32
                        if args.len() != 1 {
                            return Err(reject(func, "CallNative(Print)", "takes exactly 1 arg"));
                        }
                        let (v, vt) = operand(env, args[0], func)?;

                        // f64 değer int helper'a verilemez (CLIF verifier reddi →
                        // sahte VM fallback); tipe göre doğru helper'ı seç.
                        if vt == MirType::F64 {
                            let fl_sig = {
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(F64));
                                sig.returns.push(AbiParam::new(I32));
                                sig
                            };
                            let fl_id = module
                                .declare_function("hudhud_print_float", Linkage::Import, &fl_sig)
                                .map_err(|e| reject(func, "Print", &format!("declare float: {e}")))?;
                            let fl_ref = module.declare_func_in_func(fl_id, builder.func);
                            builder.ins().call(fl_ref, &[v]);
                            let zero = builder.ins().iconst(I64, 0);
                            record(env, *dst, zero, MirType::Generic);
                            return Ok(());
                        }

                        // External fonksiyon declare et (bir kez)
                        let print_sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(I64));
                            sig.returns.push(AbiParam::new(I32));
                            sig
                        };
                        let print_id = module
                            .declare_function(
                                "hudhud_print_int",
                                cranelift_module::Linkage::Import,
                                &print_sig,
                            )
                            .map_err(|e| {
                                BackendError::new("DECLARE_FAIL", format!("declare print: {e}"))
                                    .in_function(func.name.to_string())
                            })?;
                        let print_ref = module.declare_func_in_func(print_id, builder.func);
                        let _rc = builder.ins().call(print_ref, &[v]);

                        // print değersiz: null döner
                        let n = builder.ins().iconst(I64, 0);
                        record(env, *dst, n, MirType::Generic);
                    }
                    RuntimeHelperId::ThrowOverflow => {
                        return Err(reject(func, "CallNative(ThrowOverflow)", "throw lane arrives with the trampoline step"));
                    }
                    RuntimeHelperId::GlobalsHandle => {
                        let sig = { let mut sig = module.make_signature(); sig.returns.push(AbiParam::new(I64)); sig };
                        let id = module
                            .declare_function("hudhud_globals", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "GlobalsHandle", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let inst = builder.ins().call(fref, &[]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "GlobalsHandle", "helper returned no value"))?;
                        record(env, *dst, result, MirType::Generic);
                    }
                    RuntimeHelperId::GlobalGet => {
                        // SATIR İÇİ: GLOBAL_SLOTS data-import'tan doğrudan
                        // yük (hudhud_global_get çağrısı ~9M/koşu — GoL'de
                        // her a[i] öncesi). Sınır dışı slot: yardımcı çağrısı.
                        let (slot, _) = operand(env, args[0], func)?;
                        let raw = super::array_insts::emit_global_load(
                            builder, module, ptr, func, slot,
                        )?;
                        let result = if *ty == MirType::F64 {
                            bitcast_i64_to_f64(builder, raw)
                        } else {
                            raw
                        };
                        record(env, *dst, result, *ty);
                    }
                    RuntimeHelperId::GlobalSet => {
                        // SATIR İÇİ store (GlobalGet ile aynı data-import)
                        let (slot, _) = operand(env, args[0], func)?;
                        let (val, vt) = operand(env, args[1], func)?;
                        let v = if vt == MirType::F64 {
                            bitcast_f64_to_i64(builder, val)
                        } else {
                            val
                        };
                        super::array_insts::emit_global_store(
                            builder, module, ptr, func, slot, v,
                        )?;
                        let zero = builder.ins().iconst(I64, 0);
                        record(env, *dst, zero, MirType::Generic);
                    }
                    RuntimeHelperId::DateMillis => {
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.returns.push(AbiParam::new(I64));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_date_millis", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "DateMillis", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let inst = builder.ins().call(fref, &[]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "DateMillis", "helper returned no value"))?;
                        record(env, *dst, result, MirType::I64);
                    }
                    RuntimeHelperId::StringToInt => {
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(ptr));
                            sig.returns.push(AbiParam::new(I64));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_string_to_int", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "StringToInt", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let (s, _) = operand(env, args[0], func)?;
                        let inst = builder.ins().call(fref, &[s]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "StringToInt", "helper returned no value"))?;
                        record(env, *dst, result, MirType::I64);
                    }
                    RuntimeHelperId::StringSplit => {
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(ptr));
                            sig.params.push(AbiParam::new(ptr));
                            sig.returns.push(AbiParam::new(ptr));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_string_split", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "StringSplit", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let (s, _) = operand(env, args[0], func)?;
                        let (delim, _) = operand(env, args[1], func)?;
                        let inst = builder.ins().call(fref, &[s, delim]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "StringSplit", "helper returned no value"))?;
                        record(env, *dst, result, MirType::Ref(RefKind::Array));
                    }
                    RuntimeHelperId::StringIndexOf => {
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(ptr));
                            sig.params.push(AbiParam::new(ptr));
                            sig.returns.push(AbiParam::new(I64));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_string_index_of", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "StringIndexOf", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let (s, _) = operand(env, args[0], func)?;
                        let (needle, _) = operand(env, args[1], func)?;
                        let inst = builder.ins().call(fref, &[s, needle]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "StringIndexOf", "helper returned no value"))?;
                        record(env, *dst, result, MirType::I64);
                    }
                    RuntimeHelperId::TypeOf => {
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(I64));
                            sig.returns.push(AbiParam::new(ptr));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_typeof", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "TypeOf", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let (v, _) = operand(env, args[0], func)?;
                        let inst = builder.ins().call(fref, &[v]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "TypeOf", "helper returned no value"))?;
                        record(env, *dst, result, MirType::Ref(hudhudscript_mir::RefKind::String));
                    }
                    RuntimeHelperId::StringCmp => {
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(ptr));
                            sig.params.push(AbiParam::new(ptr));
                            sig.returns.push(AbiParam::new(I64));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_string_cmp", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "StringCmp", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let (a, _) = operand(env, args[0], func)?;
                        let (b, _) = operand(env, args[1], func)?;
                        let inst = builder.ins().call(fref, &[a, b]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "StringCmp", "helper returned no value"))?;
                        record(env, *dst, result, MirType::I64);
                    }
                    RuntimeHelperId::StringAppend => {
                        let (s, _) = operand(env, args[0], func)?;
                        let (suffix, _) = operand(env, args[1], func)?;
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(ptr));
                            sig.params.push(AbiParam::new(ptr));
                            sig.returns.push(AbiParam::new(ptr));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_string_append", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "StringAppend", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let inst = builder.ins().call(fref, &[s, suffix]);
                        let result = *builder.inst_results(inst).first().unwrap();
                        record(env, *dst, result, MirType::Ref(RefKind::String));
                    }
                    RuntimeHelperId::MathSin | RuntimeHelperId::MathSqrt | RuntimeHelperId::MathCos
                    | RuntimeHelperId::MathFloor | RuntimeHelperId::MathAbs
                    | RuntimeHelperId::MathPow | RuntimeHelperId::MathMin | RuntimeHelperId::MathMax => {
                        #[allow(unreachable_patterns)]
                        let (name, what, two_arg) = match helper {
                            RuntimeHelperId::MathSin => ("hudhud_math_sin", "MathSin", false),
                            RuntimeHelperId::MathSqrt => ("hudhud_math_sqrt", "MathSqrt", false),
                            RuntimeHelperId::MathCos => ("hudhud_math_cos", "MathCos", false),
                            RuntimeHelperId::MathFloor => ("hudhud_math_floor", "MathFloor", false),
                            RuntimeHelperId::MathAbs => ("hudhud_math_abs", "MathAbs", false),
                            RuntimeHelperId::MathPow => ("hudhud_math_pow", "MathPow", true),
                            RuntimeHelperId::MathMin => ("hudhud_math_min", "MathMin", true),
                            RuntimeHelperId::MathMax => ("hudhud_math_max", "MathMax", true),
                            _ => unreachable!("math arm guarded by outer match"),
                        };
                        let expected_args = if two_arg { 2 } else { 1 };
                        if args.len() != expected_args {
                            return Err(reject(func, what, if two_arg { "takes exactly 2 args" } else { "takes exactly 1 arg" }));
                        }
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(F64));
                            if two_arg {
                                sig.params.push(AbiParam::new(F64));
                            }
                            sig.returns.push(AbiParam::new(F64));
                            sig
                        };
                        let id = module
                            .declare_function(name, Linkage::Import, &sig)
                            .map_err(|e| reject(func, what, &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let inst = if two_arg {
                            let (a, _) = operand(env, args[0], func)?;
                            let (b, _) = operand(env, args[1], func)?;
                            builder.ins().call(fref, &[a.into(), b.into()])
                        } else {
                            let (v, _) = operand(env, args[0], func)?;
                            builder.ins().call(fref, &[v])
                        };
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, what, "helper returned no value"))?;
                        record(env, *dst, result, MirType::F64);
                    }
                    RuntimeHelperId::Throw => {
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(I64));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_throw", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "Throw", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let (v, _) = operand(env, args[0], func)?;
                        builder.ins().call(fref, &[v]);
                    }
                    RuntimeHelperId::HasException => {
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.returns.push(AbiParam::new(I64));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_has_exception", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "HasException", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let inst = builder.ins().call(fref, &[]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "HasException", "helper returned no value"))?;
                        record(env, *dst, result, MirType::I64);
                    }
                    RuntimeHelperId::Catch => {
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.returns.push(AbiParam::new(I64));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_catch", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "Catch", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let inst = builder.ins().call(fref, &[]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "Catch", "helper returned no value"))?;
                        record(env, *dst, result, MirType::Generic);
                    }
                    RuntimeHelperId::ArrayFilled => {
                        let (len, _) = operand(env, args[0], func)?;
                        let (val, vt) = operand(env, args[1], func)?;
                        let val = if vt == MirType::F64 { bitcast_f64_to_i64(builder, val) } else { val };
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(I64));
                            sig.params.push(AbiParam::new(I64));
                            sig.returns.push(AbiParam::new(ptr));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_array_filled", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "ArrayFilled", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        let inst = builder.ins().call(fref, &[len, val]);
                        let result = *builder.inst_results(inst).first()
                            .ok_or_else(|| reject(func, "ArrayFilled", "helper returned no value"))?;
                        record(env, *dst, result, MirType::Ref(RefKind::Array));
                    }
                    RuntimeHelperId::ArrayFill => {
                        let (a, _) = operand(env, args[0], func)?;
                        let (len, _) = operand(env, args[1], func)?;
                        let (val, vt) = operand(env, args[2], func)?;
                        let val = if vt == MirType::F64 { bitcast_f64_to_i64(builder, val) } else { val };
                        let sig = {
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(ptr));
                            sig.params.push(AbiParam::new(I64));
                            sig.params.push(AbiParam::new(I64));
                            sig
                        };
                        let id = module
                            .declare_function("hudhud_array_fill", Linkage::Import, &sig)
                            .map_err(|e| reject(func, "ArrayFill", &format!("declare: {e}")))?;
                        let fref = module.declare_func_in_func(id, builder.func);
                        builder.ins().call(fref, &[a, len, val]);
                        record(env, *dst, a, MirType::Ref(RefKind::Array));
                    }
                }
            }
        _ => unreachable!("yalnız CallStatic/CallNative gelmeli"),
    }
    Ok(())
}
