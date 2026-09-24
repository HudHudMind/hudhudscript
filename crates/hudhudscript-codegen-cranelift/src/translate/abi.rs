//! MIR string/array instruction'ları → native ABI helper çağrıları
//! (hudhud_string_* / hudhud_array_*; §D Runtime ABI).

use cranelift::prelude::types::{F64, I64, Type};
use cranelift::prelude::{AbiParam, FunctionBuilder, InstBuilder};
use cranelift_module::{DataDescription, Linkage, Module};

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirFunction, MirInst, MirType};

use super::helpers::{bitcast_f64_to_i64, bitcast_i64_to_f64, operand, record, reject};

type Env = Vec<Option<(cranelift::prelude::Value, MirType)>>;

/// Bu instruction ABI helper'a mı denk geliyor? (match guard için)
pub(super) fn handles(inst: &MirInst) -> bool {
    matches!(
        inst,
        MirInst::ArrayNew { .. }
            | MirInst::ArrayPush { .. }
            | MirInst::ArrayGet { .. }
            | MirInst::ArraySet { .. }
            | MirInst::ArrayLen { .. }
            | MirInst::ArrayPop { .. }
            | MirInst::ArrayJoin { .. }
            | MirInst::StringSubstring { .. }
            | MirInst::IntToFloat { .. }
            | MirInst::StringCharAt { .. }
            | MirInst::ConstString { .. }
            | MirInst::StringConcat { .. }
            | MirInst::StringLen { .. }
            | MirInst::StringEq { .. }
            | MirInst::IntToString { .. }
            | MirInst::FloatToString { .. }
            | MirInst::ObjectNew { .. }
            | MirInst::ObjectSet { .. }
            | MirInst::ObjectGet { .. }
            | MirInst::ObjectHas { .. }
            | MirInst::ObjectLen { .. }
    )
}

/// ABI helper instruction'larını çevir. `handles` true dönerse çağrılır.
pub(super) fn translate_abi<M: Module>(
    inst: &MirInst,
    builder: &mut FunctionBuilder,
    env: &mut Env,
    module: &mut M,
    ptr: Type,
    func: &MirFunction,
) -> Result<(), BackendError> {
    match inst {
            MirInst::ArrayNew { dst, capacity } => {
                let (cap, _) = operand(env, *capacity, func)?;
                let sig = { let mut s = module.make_signature(); s.params.push(AbiParam::new(I64)); s.returns.push(AbiParam::new(ptr)); s };
                let id = module.declare_function("hudhud_array_new", cranelift_module::Linkage::Import, &sig)
                    .map_err(|e| reject(func, "ArrayNew", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[cap]);
                let result = *builder.inst_results(inst).first().unwrap();
                record(env, *dst, result, MirType::Generic);
            }
            MirInst::ArrayPush { arr, value } => {
                let (a, _) = operand(env, *arr, func)?;
                let (v, vt) = operand(env, *value, func)?;
                let v = if vt == MirType::F64 { bitcast_f64_to_i64(builder, v) } else { v };
                let sig = { let mut s = module.make_signature(); s.params.push(AbiParam::new(ptr)); s.params.push(AbiParam::new(I64)); s };
                let id = module.declare_function("hudhud_array_push", cranelift_module::Linkage::Import, &sig)
                    .map_err(|e| reject(func, "ArrayPush", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                builder.ins().call(fref, &[a, v]);
            }
            MirInst::ArrayGet { dst, ty, arr, index } => {
                super::array_insts::emit_array_get(builder, env, module, ptr, func, *dst, *ty, *arr, *index)?;
            }
            MirInst::ArraySet { arr, index, value } => {
                super::array_insts::emit_array_set(builder, env, module, ptr, func, *arr, *index, *value)?;
            }
            MirInst::ArrayJoin { dst, arr, sep } => {
                let (a, _) = operand(env, *arr, func)?;
                let (s, _) = operand(env, *sep, func)?;
                let sig = { let mut sig = module.make_signature(); sig.params.push(AbiParam::new(ptr)); sig.params.push(AbiParam::new(ptr)); sig.returns.push(AbiParam::new(ptr)); sig };
                let id = module.declare_function("hudhud_array_join", Linkage::Import, &sig)
                    .map_err(|e| reject(func, "ArrayJoin", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[a, s]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "ArrayJoin", "helper returned no value"))?;
                record(env, *dst, result, MirType::Ref(hudhudscript_mir::RefKind::String));
            }
            MirInst::StringSubstring { dst, s, start, end } => {
                let (sv, _) = operand(env, *s, func)?;
                let (st, _) = operand(env, *start, func)?;
                let (en, _) = operand(env, *end, func)?;
                let sig = {
                    let mut sig = module.make_signature();
                    sig.params.push(AbiParam::new(ptr));
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(ptr));
                    sig
                };
                let id = module.declare_function("hudhud_string_substring", Linkage::Import, &sig)
                    .map_err(|e| reject(func, "StringSubstring", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[sv, st, en]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "StringSubstring", "helper returned no value"))?;
                record(env, *dst, result, MirType::Ref(hudhudscript_mir::RefKind::String));
            }
            MirInst::IntToFloat { dst, src } => {
                let (v, vt) = operand(env, *src, func)?;
                let result = if vt == MirType::F64 { v } else { builder.ins().fcvt_from_sint(F64, v) };
                record(env, *dst, result, MirType::F64);
            }
            MirInst::StringCharAt { dst, s, index } => {
                let (sv, _) = operand(env, *s, func)?;
                let (i, _) = operand(env, *index, func)?;
                let sig = { let mut s2 = module.make_signature(); s2.params.push(AbiParam::new(ptr)); s2.params.push(AbiParam::new(I64)); s2.returns.push(AbiParam::new(ptr)); s2 };
                let id = module.declare_function("hudhud_string_char_at", Linkage::Import, &sig)
                    .map_err(|e| reject(func, "StringCharAt", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[sv, i]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "StringCharAt", "helper returned no value"))?;
                record(env, *dst, result, MirType::Ref(hudhudscript_mir::RefKind::String));
            }
            MirInst::ArrayPop { dst, arr } => {
                let (a, _) = operand(env, *arr, func)?;
                let sig = { let mut s = module.make_signature(); s.params.push(AbiParam::new(ptr)); s.returns.push(AbiParam::new(I64)); s };
                let id = module.declare_function("hudhud_array_pop", Linkage::Import, &sig)
                    .map_err(|e| reject(func, "ArrayPop", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[a]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "ArrayPop", "helper returned no value"))?;
                record(env, *dst, result, MirType::I64);
            }
            MirInst::ArrayLen { dst, arr } => {
                super::array_insts::emit_array_len(builder, env, module, ptr, func, *dst, *arr)?;
            }
            MirInst::ConstString { dst, index } => {
                // String sabiti modül DATA'sı olarak gömülür: JIT'te JIT
                // belleğine, AOT'te object dosyasının .data/.rodata bölümüne.
                // C ABI helper'lar NUL-sonlandırmalı bayt dizisi bekler.
                let s = func.string_table.get(*index as usize)
                    .map(|s| s.as_ref().to_string())
                    .unwrap_or_default();
                let data_name = format!("hudhud_str_{}_{}", func.name, index);
                let data_id = module
                    .declare_data(&data_name, Linkage::Export, false, false)
                    .map_err(|e| reject(func, "ConstString", &format!("declare data: {e}")))?;
                let mut bytes = s.into_bytes();
                bytes.push(0);
                let mut dd = DataDescription::new();
                dd.define(bytes.into_boxed_slice());
                module
                    .define_data(data_id, &dd)
                    .map_err(|e| reject(func, "ConstString", &format!("define data: {e}")))?;
                let gv = module.declare_data_in_func(data_id, builder.func);
                let v = builder.ins().global_value(I64, gv);
                // STRING_REGISTRY kaydı (typeof bu pointer'ı tanısın):
                // JIT — host tarafında, modül başına BİR KEZ (v0.9.9: inline
                // çağrı döngü içinde milyonlarca HashSet insert ödetiyordu);
                // AOT — inline (binary kendi kaydeder; host finalize anı yok).
                super::note_string_data(&data_name);
                if super::inline_string_reg() {
                    let reg_sig = {
                        let mut rs = module.make_signature();
                        rs.params.push(AbiParam::new(ptr));
                        rs
                    };
                    let reg_id = module
                        .declare_function("hudhud_register_string", Linkage::Import, &reg_sig)
                        .map_err(|e| reject(func, "ConstString", &format!("declare register: {e}")))?;
                    let reg_ref = module.declare_func_in_func(reg_id, builder.func);
                    builder.ins().call(reg_ref, &[v]);
                }
                record(env, *dst, v, MirType::Ref(hudhudscript_mir::RefKind::String));
            }
            MirInst::StringConcat { dst, lhs, rhs } => {
                let (l, _) = operand(env, *lhs, func)?;
                let (r, _) = operand(env, *rhs, func)?;
                let sig = {
                    let mut s = module.make_signature();
                    s.params.push(AbiParam::new(ptr));
                    s.params.push(AbiParam::new(ptr));
                    s.returns.push(AbiParam::new(ptr));
                    s
                };
                let id = module.declare_function("hudhud_string_concat", cranelift_module::Linkage::Import, &sig)
                    .map_err(|e| reject(func, "StringConcat", &format!("declare: {e}")))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[l, r]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "StringConcat", "helper returned no value"))?;
                record(env, *dst, result, MirType::Generic);
            }
            MirInst::StringLen { dst, src } => {
                let (s, _) = operand(env, *src, func)?;
                let sig = {
                    let mut s = module.make_signature();
                    s.params.push(AbiParam::new(ptr));
                    s.returns.push(AbiParam::new(I64));
                    s
                };
                let id = module.declare_function("hudhud_string_len", cranelift_module::Linkage::Import, &sig)
                    .map_err(|e| reject(func, "StringLen", &format!("declare: {e}")))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[s]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "StringLen", "helper returned no value"))?;
                record(env, *dst, result, MirType::I64);
            }
            MirInst::StringEq { dst, lhs, rhs } => {
                let (l, _) = operand(env, *lhs, func)?;
                let (r, _) = operand(env, *rhs, func)?;
                let sig = {
                    let mut s = module.make_signature();
                    s.params.push(AbiParam::new(ptr));
                    s.params.push(AbiParam::new(ptr));
                    s.returns.push(AbiParam::new(I64));
                    s
                };
                let id = module.declare_function("hudhud_string_eq", cranelift_module::Linkage::Import, &sig)
                    .map_err(|e| reject(func, "StringEq", &format!("declare: {e}")))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[l, r]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "StringEq", "helper returned no value"))?;
                record(env, *dst, result, MirType::I64);
            }
            MirInst::IntToString { dst, src } => {
                let (v, _) = operand(env, *src, func)?;
                let sig = {
                    let mut s = module.make_signature();
                    s.params.push(AbiParam::new(I64));
                    s.returns.push(AbiParam::new(ptr));
                    s
                };
                let id = module.declare_function("hudhud_int_to_string", cranelift_module::Linkage::Import, &sig)
                    .map_err(|e| reject(func, "IntToString", &format!("declare: {e}")))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[v]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "IntToString", "helper returned no value"))?;
                record(env, *dst, result, MirType::Generic);
            }
            MirInst::FloatToString { dst, src } => {
                let (v, _) = operand(env, *src, func)?;
                let sig = {
                    let mut s = module.make_signature();
                    s.params.push(AbiParam::new(F64));
                    s.returns.push(AbiParam::new(ptr));
                    s
                };
                let id = module.declare_function("hudhud_float_to_string", cranelift_module::Linkage::Import, &sig)
                    .map_err(|e| reject(func, "FloatToString", &format!("declare: {e}")))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[v]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "FloatToString", "helper returned no value"))?;
                record(env, *dst, result, MirType::Generic);
            }
            MirInst::ObjectNew { dst } => {
                let sig = { let mut s = module.make_signature(); s.returns.push(AbiParam::new(ptr)); s };
                let id = module.declare_function("hudhud_object_new", Linkage::Import, &sig)
                    .map_err(|e| reject(func, "ObjectNew", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "ObjectNew", "helper returned no value"))?;
                record(env, *dst, result, MirType::Generic);
            }
            MirInst::ObjectSet { obj, key, value } => {
                let (o, _) = operand(env, *obj, func)?;
                let (k, _) = operand(env, *key, func)?;
                let (v, vt) = operand(env, *value, func)?;
                let v = if vt == MirType::F64 { bitcast_f64_to_i64(builder, v) } else { v };
                let sig = { let mut s = module.make_signature(); s.params.push(AbiParam::new(ptr)); s.params.push(AbiParam::new(ptr)); s.params.push(AbiParam::new(I64)); s };
                let id = module.declare_function("hudhud_object_set", Linkage::Import, &sig)
                    .map_err(|e| reject(func, "ObjectSet", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                builder.ins().call(fref, &[o, k, v]);
            }
            MirInst::ObjectGet { dst, ty, obj, key } => {
                let (o, _) = operand(env, *obj, func)?;
                let (k, _) = operand(env, *key, func)?;
                let sig = { let mut s = module.make_signature(); s.params.push(AbiParam::new(ptr)); s.params.push(AbiParam::new(ptr)); s.returns.push(AbiParam::new(I64)); s };
                let id = module.declare_function("hudhud_object_get", Linkage::Import, &sig)
                    .map_err(|e| reject(func, "ObjectGet", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[o, k]);
                let raw = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "ObjectGet", "helper returned no value"))?;
                let result = if *ty == MirType::F64 { bitcast_i64_to_f64(builder, raw) } else { raw };
                record(env, *dst, result, *ty);
            }
            MirInst::ObjectHas { dst, obj, key } => {
                let (o, _) = operand(env, *obj, func)?;
                let (k, _) = operand(env, *key, func)?;
                let sig = { let mut s = module.make_signature(); s.params.push(AbiParam::new(ptr)); s.params.push(AbiParam::new(ptr)); s.returns.push(AbiParam::new(I64)); s };
                let id = module.declare_function("hudhud_object_has", Linkage::Import, &sig)
                    .map_err(|e| reject(func, "ObjectHas", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[o, k]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "ObjectHas", "helper returned no value"))?;
                record(env, *dst, result, MirType::Bool);
            }
            MirInst::ObjectLen { dst, obj } => {
                let (o, _) = operand(env, *obj, func)?;
                let sig = { let mut s = module.make_signature(); s.params.push(AbiParam::new(ptr)); s.returns.push(AbiParam::new(I64)); s };
                let id = module.declare_function("hudhud_object_len", Linkage::Import, &sig)
                    .map_err(|e| reject(func, "ObjectLen", &e.to_string()))?;
                let fref = module.declare_func_in_func(id, builder.func);
                let inst = builder.ins().call(fref, &[o]);
                let result = *builder.inst_results(inst).first()
                    .ok_or_else(|| reject(func, "ObjectLen", "helper returned no value"))?;
                record(env, *dst, result, MirType::I64);
            }
        _ => unreachable!("handles() önceden filtreledi"),
    }
    Ok(())
}
