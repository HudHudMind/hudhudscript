//! MIR aritmetik/karşılaştırma instruction'ları → CLIF.
//! §18: checked i64 taşma/sıfır-bölme bayrakları birikir, çıkışta seçilir.

use cranelift::prelude::types::{I32, I64};
use cranelift::prelude::{FunctionBuilder, InstBuilder, IntCC, Variable};

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{CmpOp, MirFunction, MirInst, MirType};

use super::helpers::{
    emit_checked, ensure_f64, operand, record, reject, require_i64, OverflowKind,
};

type Env = Vec<Option<(cranelift::prelude::Value, MirType)>>;

fn is_pointer_candidate(builder: &mut FunctionBuilder, val: cranelift::prelude::Value) -> cranelift::prelude::Value {
    let tag = builder.ins().ushr_imm(val, 48);
    builder.ins().icmp_imm(IntCC::Equal, tag, 0xB161)
}

fn emit_promoted_op<M: cranelift_module::Module>(
    builder: &mut FunctionBuilder,
    module: &mut M,
    l: cranelift::prelude::Value,
    r: cranelift::prelude::Value,
    fast_val: cranelift::prelude::Value,
    ov: cranelift::prelude::Value,
    helper_name: &str,
    func: &MirFunction,
) -> Result<cranelift::prelude::Value, BackendError> {
    let is_l_ptr = is_pointer_candidate(builder, l);
    let is_r_ptr = is_pointer_candidate(builder, r);
    let ptr_op = builder.ins().bor(is_l_ptr, is_r_ptr);
    let need_slow = builder.ins().bor(ov, ptr_op);

    let fast_block = builder.create_block();
    let slow_block = builder.create_block();
    let merge_block = builder.create_block();
    builder.append_block_param(merge_block, I64);

    builder.ins().brif(need_slow, slow_block, &[], fast_block, &[]);

    use cranelift::prelude::codegen::ir::instructions::BlockArg;
    // Fast block: pure hardware value
    builder.switch_to_block(fast_block);
    let arg_fast = [BlockArg::Value(fast_val)];
    builder.ins().jump(merge_block, &arg_fast);

    // Slow block: call native BigInt helper (hudhud_num_add / sub / mul)
    builder.switch_to_block(slow_block);
    let mut sig = module.make_signature();
    sig.params.push(cranelift::prelude::AbiParam::new(I64));
    sig.params.push(cranelift::prelude::AbiParam::new(I64));
    sig.returns.push(cranelift::prelude::AbiParam::new(I64));
    let fid = module
        .declare_function(helper_name, cranelift_module::Linkage::Import, &sig)
        .map_err(|e| reject(func, helper_name, &format!("declare {helper_name}: {e}")))?;
    let fref = module.declare_func_in_func(fid, builder.func);
    let call = builder.ins().call(fref, &[l, r]);
    let slow_res = builder.inst_results(call)[0];
    // NOT: promote edilen taşma ÇÖZÜLMÜŞTÜR (doğru BigInt değeri) — ov_var'a
    // 1 yazmak sahte §18 OVERFLOW exit üretirdi (F13: main içinde i64::MAX+1
    // → status 1). VM oracle: promote = başarı, exit 0.
    let arg_slow = [BlockArg::Value(slow_res)];
    builder.ins().jump(merge_block, &arg_slow);

    // Merge block
    builder.switch_to_block(merge_block);
    Ok(builder.block_params(merge_block)[0])
}

fn emit_promoted_div<M: cranelift_module::Module>(
    builder: &mut FunctionBuilder,
    module: &mut M,
    l: cranelift::prelude::Value,
    r: cranelift::prelude::Value,
    fast_val: cranelift::prelude::Value,
    extra_slow: cranelift::prelude::Value,
    helper_name: &str,
    func: &MirFunction,
) -> Result<cranelift::prelude::Value, BackendError> {
    use cranelift::prelude::codegen::ir::instructions::BlockArg;
    let is_l_ptr = is_pointer_candidate(builder, l);
    let is_r_ptr = is_pointer_candidate(builder, r);
    // F13/VM paritesi (v0.9.21): MIN/-1 de promote edilir — extra_slow,
    // çağıranın hesapladığı min_neg1 bayrağı (BigInt tag'i ile aynı yola).
    let need_slow = {
        let ptr_op = builder.ins().bor(is_l_ptr, is_r_ptr);
        builder.ins().bor(ptr_op, extra_slow)
    };

    let fast_block = builder.create_block();
    let slow_block = builder.create_block();
    let merge_block = builder.create_block();
    builder.append_block_param(merge_block, I64);

    builder.ins().brif(need_slow, slow_block, &[], fast_block, &[]);

    builder.switch_to_block(fast_block);
    let arg_fast = [BlockArg::Value(fast_val)];
    builder.ins().jump(merge_block, &arg_fast);

    builder.switch_to_block(slow_block);
    let mut sig = module.make_signature();
    sig.params.push(cranelift::prelude::AbiParam::new(I64));
    sig.params.push(cranelift::prelude::AbiParam::new(I64));
    sig.returns.push(cranelift::prelude::AbiParam::new(I64));
    let fid = module
        .declare_function(helper_name, cranelift_module::Linkage::Import, &sig)
        .map_err(|e| reject(func, helper_name, &format!("declare {helper_name}: {e}")))?;
    let fref = module.declare_func_in_func(fid, builder.func);
    let call = builder.ins().call(fref, &[l, r]);
    let slow_res = builder.inst_results(call)[0];
    let arg_slow = [BlockArg::Value(slow_res)];
    builder.ins().jump(merge_block, &arg_slow);

    builder.switch_to_block(merge_block);
    Ok(builder.block_params(merge_block)[0])
}

fn emit_promoted_cmp<M: cranelift_module::Module>(
    builder: &mut FunctionBuilder,
    module: &mut M,
    l: cranelift::prelude::Value,
    r: cranelift::prelude::Value,
    fast_res: cranelift::prelude::Value,
    op: CmpOp,
    func: &MirFunction,
) -> Result<cranelift::prelude::Value, BackendError> {
    use cranelift::prelude::codegen::ir::instructions::BlockArg;
    let is_l_ptr = is_pointer_candidate(builder, l);
    let is_r_ptr = is_pointer_candidate(builder, r);
    let need_slow = builder.ins().bor(is_l_ptr, is_r_ptr);

    let fast_block = builder.create_block();
    let slow_block = builder.create_block();
    let merge_block = builder.create_block();
    builder.append_block_param(merge_block, I64);

    builder.ins().brif(need_slow, slow_block, &[], fast_block, &[]);

    builder.switch_to_block(fast_block);
    let arg_fast = [BlockArg::Value(fast_res)];
    builder.ins().jump(merge_block, &arg_fast);

    builder.switch_to_block(slow_block);
    let mut sig = module.make_signature();
    sig.params.push(cranelift::prelude::AbiParam::new(I64));
    sig.params.push(cranelift::prelude::AbiParam::new(I64));
    sig.returns.push(cranelift::prelude::AbiParam::new(I64));
    let fid = module
        .declare_function("hudhud_num_cmp", cranelift_module::Linkage::Import, &sig)
        .map_err(|e| reject(func, "hudhud_num_cmp", &format!("declare hudhud_num_cmp: {e}")))?;
    let fref = module.declare_func_in_func(fid, builder.func);
    let call = builder.ins().call(fref, &[l, r]);
    let cmp_res = builder.inst_results(call)[0];
    let zero64 = builder.ins().iconst(I64, 0);
    let cc = match op {
        CmpOp::Eq => IntCC::Equal,
        CmpOp::Ne => IntCC::NotEqual,
        CmpOp::Lt => IntCC::SignedLessThan,
        CmpOp::Le => IntCC::SignedLessThanOrEqual,
        CmpOp::Gt => IntCC::SignedGreaterThan,
        CmpOp::Ge => IntCC::SignedGreaterThanOrEqual,
    };
    let b = builder.ins().icmp(cc, cmp_res, zero64);
    let slow_res = builder.ins().uextend(I64, b);
    let arg_slow = [BlockArg::Value(slow_res)];
    builder.ins().jump(merge_block, &arg_slow);

    builder.switch_to_block(merge_block);
    Ok(builder.block_params(merge_block)[0])
}

pub(super) fn translate_arith<M: cranelift_module::Module>(
    inst: &MirInst,
    builder: &mut FunctionBuilder,
    env: &mut Env,
    module: &mut M,
    func: &MirFunction,
    _ov_var: Variable,
    dz_var: Variable,
) -> Result<(), BackendError> {
    match inst {
            MirInst::Add { dst, lhs, rhs, .. } => {
                let (l, lt) = operand(env, *lhs, func)?;
                let (r, rt) = operand(env, *rhs, func)?;
                if lt == MirType::F64 || rt == MirType::F64 {
                    let lf = ensure_f64(builder, l, lt);
                    let rf = ensure_f64(builder, r, rt);
                    let result = builder.ins().fadd(lf, rf);
                    record(env, *dst, result, MirType::F64);
                } else {
                    require_i64(lt, rt, "Add", func)?;
                    let (sum, ov) = emit_checked(builder, l, r, OverflowKind::Add);
                    let res = emit_promoted_op(builder, module, l, r, sum, ov, "hudhud_num_add", func)?;
                    record(env, *dst, res, MirType::I64);
                }
            }
            MirInst::Sub { dst, lhs, rhs, .. } => {
                let (l, lt) = operand(env, *lhs, func)?;
                let (r, rt) = operand(env, *rhs, func)?;
                if lt == MirType::F64 || rt == MirType::F64 {
                    let lf = ensure_f64(builder, l, lt);
                    let rf = ensure_f64(builder, r, rt);
                    let result = builder.ins().fsub(lf, rf);
                    record(env, *dst, result, MirType::F64);
                } else {
                    require_i64(lt, rt, "Sub", func)?;
                    let (diff, ov) = emit_checked(builder, l, r, OverflowKind::Sub);
                    let res = emit_promoted_op(builder, module, l, r, diff, ov, "hudhud_num_sub", func)?;
                    record(env, *dst, res, MirType::I64);
                }
            }
            MirInst::Mul { dst, lhs, rhs, .. } => {
                let (l, lt) = operand(env, *lhs, func)?;
                let (r, rt) = operand(env, *rhs, func)?;
                if lt == MirType::F64 || rt == MirType::F64 {
                    let lf = ensure_f64(builder, l, lt);
                    let rf = ensure_f64(builder, r, rt);
                    let result = builder.ins().fmul(lf, rf);
                    record(env, *dst, result, MirType::F64);
                } else {
                    require_i64(lt, rt, "Mul", func)?;
                    let (prod, ov) = emit_checked(builder, l, r, OverflowKind::Mul);
                    let res = emit_promoted_op(builder, module, l, r, prod, ov, "hudhud_num_mul", func)?;
                    record(env, *dst, res, MirType::I64);
                }
            }
            MirInst::Div { dst, lhs, rhs, .. } | MirInst::Rem { dst, lhs, rhs, .. } => {
                let is_div = matches!(inst, MirInst::Div { .. });
                let what = if is_div { "Div" } else { "Rem" };
                let (l, lt) = operand(env, *lhs, func)?;
                let (r, rt) = operand(env, *rhs, func)?;

                // F64 bölme: IEEE 754 (inf/nan doğal; SIGFPE YOK)
                if is_div && (lt == MirType::F64 || rt == MirType::F64) {
                    let lf = ensure_f64(builder, l, lt);
                    let rf = ensure_f64(builder, r, rt);
                    let result = builder.ins().fdiv(lf, rf);
                    record(env, *dst, result, MirType::F64);
                    return Ok(());
                }
                if !is_div && (lt == MirType::F64 || rt == MirType::F64) {
                    // IEEE remainder: fmod (VM oracle ile aynı semantik)
                    let lf = ensure_f64(builder, l, lt);
                    let rf = ensure_f64(builder, r, rt);
                    // Cranelift'te fmod opcode yok — libc fmod extern çağrısı
                    use cranelift::prelude::types::F64 as F64T;
                    use cranelift::prelude::AbiParam;
                    use cranelift_module::{Linkage, Module};
                    let sig = {
                        let mut sig = module.make_signature();
                        sig.params.push(AbiParam::new(F64T));
                        sig.params.push(AbiParam::new(F64T));
                        sig.returns.push(AbiParam::new(F64T));
                        sig
                    };
                    let id = module
                        .declare_function("fmod", Linkage::Import, &sig)
                        .map_err(|e| reject(func, "Rem", &format!("declare fmod: {e}")))?;
                    let fref = module.declare_func_in_func(id, builder.func);
                    let inst = builder.ins().call(fref, &[lf.into(), rf.into()]);
                    let result = *builder.inst_results(inst).first()
                        .ok_or_else(|| reject(func, "Rem", "fmod returned no value"))?;
                    record(env, *dst, result, MirType::F64);
                    return Ok(());
                }

                require_i64(lt, rt, what, func)?;

                // §18 korumaları (dallanmasız): sıfıra bölme ve i64::MIN/-1
                // bayrak olarak birikir; çıkışta seçilir.
                let zero = builder.ins().iconst(I64, 0);
                let neg1 = builder.ins().iconst(I64, -1);
                let min = builder.ins().iconst(I64, i64::MIN as i64);

                let is_zero = builder.ins().icmp(IntCC::Equal, r, zero);
                let dz_i32 = builder.ins().uextend(I32, is_zero);
                let cur_dz = builder.use_var(dz_var);
                let new_dz = builder.ins().bor(cur_dz, dz_i32);
                builder.def_var(dz_var, new_dz);

                let is_min = builder.ins().icmp(IntCC::Equal, l, min);
                let is_neg1 = builder.ins().icmp(IntCC::Equal, r, neg1);
                let min_neg1 = builder.ins().band(is_min, is_neg1);
                // F13/VM paritesi: MIN/-1 promote edilir (num_div → BigInt
                // 2^63) — §18 OVERFLOW bayrağına YAZILMAZ, çözülmüştür.

                // SIGFPE koruması (dallanmasız): bölen sıfır veya MIN/-1
                // ise select ile 1'e değiştir — sonuç çöp olur ama status
                // zaten DivZero raporlar; asla sinyal üretmez.
                let one_c = builder.ins().iconst(I64, 1);
                let safe_from_dz = builder.ins().select(is_zero, one_c, r);
                let safe_r = builder.ins().select(min_neg1, one_c, safe_from_dz);
                let fast_val = if is_div {
                    builder.ins().sdiv(l, safe_r)
                } else {
                    builder.ins().srem(l, safe_r)
                };
                let helper = if is_div { "hudhud_num_div" } else { "hudhud_num_rem" };
                let result =
                    emit_promoted_div(builder, module, l, r, fast_val, min_neg1, helper, func)?;
                record(env, *dst, result, MirType::I64);
            }
            MirInst::Cmp { dst, op, lhs, rhs, .. } => {
                let (l, lt) = operand(env, *lhs, func)?;
                let (r, rt) = operand(env, *rhs, func)?;
                if lt == MirType::F64 || rt == MirType::F64 {
                    let lf = ensure_f64(builder, l, lt);
                    let rf = ensure_f64(builder, r, rt);
                    let fcc = match op {
                        CmpOp::Eq => cranelift::prelude::codegen::ir::condcodes::FloatCC::Equal,
                        CmpOp::Ne => cranelift::prelude::codegen::ir::condcodes::FloatCC::NotEqual,
                        CmpOp::Lt => cranelift::prelude::codegen::ir::condcodes::FloatCC::LessThan,
                        CmpOp::Le => cranelift::prelude::codegen::ir::condcodes::FloatCC::LessThanOrEqual,
                        CmpOp::Gt => cranelift::prelude::codegen::ir::condcodes::FloatCC::GreaterThan,
                        CmpOp::Ge => cranelift::prelude::codegen::ir::condcodes::FloatCC::GreaterThanOrEqual,
                    };
                    let b = builder.ins().fcmp(fcc, lf, rf);
                    record(env, *dst, b, MirType::Bool);
                } else {
                    let is_i64_like = |t: MirType| matches!(t, MirType::I64 | MirType::Bool | MirType::Generic | MirType::Ref(_));
                    if matches!(op, CmpOp::Eq | CmpOp::Ne) {
                        if !is_i64_like(lt) || !is_i64_like(rt) {
                            return Err(reject(func, "Cmp", &format!("cannot compare {lt} and {rt}")));
                        }
                    } else {
                        require_i64(lt, rt, "Cmp", func)?;
                    }
                    let cc = match op {
                        CmpOp::Eq => IntCC::Equal,
                        CmpOp::Ne => IntCC::NotEqual,
                        CmpOp::Lt => IntCC::SignedLessThan,
                        CmpOp::Le => IntCC::SignedLessThanOrEqual,
                        CmpOp::Gt => IntCC::SignedGreaterThan,
                        CmpOp::Ge => IntCC::SignedGreaterThanOrEqual,
                    };
                    let fast_b = builder.ins().icmp(cc, l, r);
                    let fast_res = builder.ins().uextend(cranelift::prelude::types::I64, fast_b);
                    let result = emit_promoted_cmp(builder, module, l, r, fast_res, *op, func)?;
                    record(env, *dst, result, MirType::Bool);
                }
            }
        _ => unreachable!("yalnız Add/Sub/Mul/Div/Rem/Cmp gelmeli"),
    }
    Ok(())
}
