//! Builtin runtime helpers for CallNative.

use cranelift::prelude::types::{F64, I32, I64, Type};
use cranelift::prelude::{AbiParam, FunctionBuilder, InstBuilder, Value};
use cranelift_module::{Linkage, Module};

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirFunction, MirType, RefKind, RuntimeHelperId, ValueId};

use super::helpers::{
    bitcast_f64_to_i64, bitcast_i64_to_f64, operand, record, reject,
};

type Env = Vec<Option<(Value, MirType)>>;

pub(super) fn translate_call_native<M: Module>(
    dst: ValueId,
    ty: &MirType,
    helper: &RuntimeHelperId,
    args: &[ValueId],
    builder: &mut FunctionBuilder,
    env: &mut Env,
    module: &mut M,
    ptr: Type,
    func: &MirFunction,
) -> Result<(), BackendError> {
    if super::call_native_math::translate_math_or_assert(
        helper, dst, args, builder, env, module, func,
    )? {
        return Ok(());
    }

    match helper {
        RuntimeHelperId::PrintStr => {
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
                .declare_function("hudhud_print_str", Linkage::Import, &print_sig)
                .map_err(|e| reject(func, "PrintStr", &format!("declare: {e}")))?;
            let print_ref = module.declare_func_in_func(print_id, builder.func);
            let _rc = builder.ins().call(print_ref, &[v]);
            let n = builder.ins().iconst(I64, 0);
            record(env, dst, n, MirType::Generic);
        }
        RuntimeHelperId::Print => {
            if args.len() != 1 {
                return Err(reject(func, "CallNative(Print)", "takes exactly 1 arg"));
            }
            let (v, vt) = operand(env, args[0], func)?;
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
                record(env, dst, zero, MirType::Generic);
                return Ok(());
            }

            let print_sig = {
                let mut sig = module.make_signature();
                sig.params.push(AbiParam::new(I64));
                sig.returns.push(AbiParam::new(I32));
                sig
            };
            let print_id = module
                .declare_function("hudhud_print_int", Linkage::Import, &print_sig)
                .map_err(|e| {
                    BackendError::new("DECLARE_FAIL", format!("declare print: {e}"))
                        .in_function(func.name.to_string())
                })?;
            let print_ref = module.declare_func_in_func(print_id, builder.func);
            let _rc = builder.ins().call(print_ref, &[v]);
            let n = builder.ins().iconst(I64, 0);
            record(env, dst, n, MirType::Generic);
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
            record(env, dst, result, MirType::Generic);
        }
        RuntimeHelperId::GlobalGet => {
            let (slot, _) = operand(env, args[0], func)?;
            let raw = super::array_insts::emit_global_load(
                builder, module, ptr, func, slot,
            )?;
            let result = if *ty == MirType::F64 {
                bitcast_i64_to_f64(builder, raw)
            } else {
                raw
            };
            record(env, dst, result, *ty);
        }
        RuntimeHelperId::GlobalSet => {
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
            record(env, dst, zero, MirType::Generic);
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
            record(env, dst, result, MirType::I64);
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
            record(env, dst, result, MirType::Ref(RefKind::Array));
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
            record(env, dst, result, MirType::I64);
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
            record(env, dst, result, MirType::Ref(hudhudscript_mir::RefKind::String));
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
            record(env, dst, result, MirType::I64);
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
            record(env, dst, result, MirType::Ref(RefKind::String));
        }
        RuntimeHelperId::DynCallMethod => {
            if args.len() > 7 {
                return Err(reject(func, "DynCallMethod", "max 5 dyn args"));
            }
            let sig = {
                let mut sig = module.make_signature();
                for _ in 0..7 {
                    sig.params.push(AbiParam::new(I64));
                }
                sig.returns.push(AbiParam::new(I64));
                sig
            };
            let id = module
                .declare_function("hudhud_dyn_call_method", Linkage::Import, &sig)
                .map_err(|e| reject(func, "DynCallMethod", &format!("declare: {e}")))?;
            let fref = module.declare_func_in_func(id, builder.func);
            let zero = builder.ins().iconst(I64, 0);
            let mut vals: Vec<_> = Vec::with_capacity(7);
            for a in args.iter() {
                vals.push(operand(env, *a, func).unwrap().0);
            }
            while vals.len() < 7 {
                vals.push(zero);
            }
            let call = builder.ins().call(fref, &vals);
            let res = *builder.inst_results(call).first().unwrap();
            record(env, dst, res, MirType::I64);
        }
        RuntimeHelperId::Input | RuntimeHelperId::Confirm => {
            let (name, what) = match helper {
                RuntimeHelperId::Input => ("hudhud_input", "Input"),
                _ => ("hudhud_confirm", "Confirm"),
            };
            let sig = {
                let mut sig = module.make_signature();
                sig.params.push(AbiParam::new(I64));
                sig.returns.push(AbiParam::new(I64));
                sig
            };
            let id = module
                .declare_function(name, Linkage::Import, &sig)
                .map_err(|e| reject(func, what, &format!("declare: {e}")))?;
            let fref = module.declare_func_in_func(id, builder.func);
            let a0 = operand(env, args[0], func).unwrap().0;
            let call = builder.ins().call(fref, &[a0]);
            let res = *builder.inst_results(call).first().unwrap();
            record(env, dst, res, MirType::I64);
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
            record(env, dst, result, MirType::I64);
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
            record(env, dst, result, MirType::Generic);
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
            record(env, dst, result, MirType::Ref(RefKind::Array));
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
            record(env, dst, a, MirType::Ref(RefKind::Array));
        }
        _ => return Err(reject(func, "CallNative", "unsupported runtime helper")),
    }
    Ok(())
}
