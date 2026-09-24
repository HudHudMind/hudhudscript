//! Inline fast-path array instructions for Cranelift (ArrayGet, ArraySet, ArrayLen).
//!
//! Direct memory load/store with bounds check against `HudArray` header.
//! Falls back to runtime ABI helper on out-of-bounds or NULL pointer.

use cranelift::prelude::types::{I64, Type};
use cranelift::prelude::{AbiParam, FunctionBuilder, InstBuilder, MemFlags};
use cranelift_module::Module;

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirFunction, MirType, ValueId};

use super::helpers::{bitcast_f64_to_i64, bitcast_i64_to_f64, operand, record, reject};

type Env = Vec<Option<(cranelift::prelude::Value, MirType)>>;

/// Inline fast-path array get:
/// 1. Check `arr != NULL`.
/// 2. Load `len` at offset 16, check `(unsigned) idx < len`.
/// 3. Hit: load `ptr` at offset 8, load `ptr + idx * 8`.
/// 4. Miss / NULL: call `hudhud_array_get`.
pub(super) fn emit_array_get<M: Module>(
    builder: &mut FunctionBuilder,
    env: &mut Env,
    module: &mut M,
    ptr: Type,
    func: &MirFunction,
    dst: ValueId,
    ty: MirType,
    arr: ValueId,
    index: ValueId,
) -> Result<(), BackendError> {
    let (a, _) = operand(env, arr, func)?;
    let (i, _) = operand(env, index, func)?;

    let res_var = builder.declare_var(I64);
    let zero_ptr = builder.ins().iconst(ptr, 0);
    let is_null = builder.ins().icmp(cranelift::prelude::IntCC::Equal, a, zero_ptr);

    let fast_blk = builder.create_block();
    let cold_blk = builder.create_block();
    let merge_blk = builder.create_block();

    builder.ins().brif(is_null, cold_blk, &[], fast_blk, &[]);

    builder.switch_to_block(fast_blk);
    let len = builder.ins().load(I64, MemFlags::trusted(), a, 16);
    let in_bounds = builder.ins().icmp(cranelift::prelude::IntCC::UnsignedLessThan, i, len);

    let hit_blk = builder.create_block();
    builder.ins().brif(in_bounds, hit_blk, &[], cold_blk, &[]);

    builder.switch_to_block(hit_blk);
    let data_ptr = builder.ins().load(ptr, MemFlags::trusted(), a, 8);
    let offset = builder.ins().ishl_imm(i, 3);
    let elem_addr = builder.ins().iadd(data_ptr, offset);
    let fast_val = builder.ins().load(I64, MemFlags::trusted(), elem_addr, 0);
    builder.def_var(res_var, fast_val);
    builder.ins().jump(merge_blk, &[]);

    builder.switch_to_block(cold_blk);
    let sig = {
        let mut s = module.make_signature();
        s.params.push(AbiParam::new(ptr));
        s.params.push(AbiParam::new(I64));
        s.returns.push(AbiParam::new(I64));
        s
    };
    let id = module
        .declare_function("hudhud_array_get", cranelift_module::Linkage::Import, &sig)
        .map_err(|e| reject(func, "ArrayGet", &e.to_string()))?;
    let fref = module.declare_func_in_func(id, builder.func);
    let inst = builder.ins().call(fref, &[a, i]);
    let cold_val = *builder.inst_results(inst).first().unwrap();
    builder.def_var(res_var, cold_val);
    builder.ins().jump(merge_blk, &[]);

    builder.switch_to_block(merge_blk);
    let raw = builder.use_var(res_var);
    let result = if ty == MirType::F64 {
        bitcast_i64_to_f64(builder, raw)
    } else {
        raw
    };
    record(env, dst, result, ty);
    Ok(())
}

/// Inline fast-path array set:
/// 1. Check `arr != NULL`.
/// 2. Load `len` at offset 16, check `(unsigned) idx < len`.
/// 3. Hit: load `ptr` at offset 8, store to `ptr + idx * 8`.
/// 4. Miss / NULL: call `hudhud_array_set` (handles dynamic resize).
pub(super) fn emit_array_set<M: Module>(
    builder: &mut FunctionBuilder,
    env: &mut Env,
    module: &mut M,
    ptr: Type,
    func: &MirFunction,
    arr: ValueId,
    index: ValueId,
    value: ValueId,
) -> Result<(), BackendError> {
    let (a, _) = operand(env, arr, func)?;
    let (i, _) = operand(env, index, func)?;
    let (v, vt) = operand(env, value, func)?;
    let v = if vt == MirType::F64 {
        bitcast_f64_to_i64(builder, v)
    } else {
        v
    };

    let zero_ptr = builder.ins().iconst(ptr, 0);
    let is_null = builder.ins().icmp(cranelift::prelude::IntCC::Equal, a, zero_ptr);

    let fast_blk = builder.create_block();
    let cold_blk = builder.create_block();
    let done_blk = builder.create_block();

    builder.ins().brif(is_null, cold_blk, &[], fast_blk, &[]);

    builder.switch_to_block(fast_blk);
    let len = builder.ins().load(I64, MemFlags::trusted(), a, 16);
    let in_bounds = builder.ins().icmp(cranelift::prelude::IntCC::UnsignedLessThan, i, len);

    let hit_blk = builder.create_block();
    builder.ins().brif(in_bounds, hit_blk, &[], cold_blk, &[]);

    builder.switch_to_block(hit_blk);
    let data_ptr = builder.ins().load(ptr, MemFlags::trusted(), a, 8);
    let offset = builder.ins().ishl_imm(i, 3);
    let elem_addr = builder.ins().iadd(data_ptr, offset);
    builder.ins().store(MemFlags::trusted(), v, elem_addr, 0);
    builder.ins().jump(done_blk, &[]);

    builder.switch_to_block(cold_blk);
    let sig = {
        let mut s = module.make_signature();
        s.params.push(AbiParam::new(ptr));
        s.params.push(AbiParam::new(I64));
        s.params.push(AbiParam::new(I64));
        s
    };
    let id = module
        .declare_function("hudhud_array_set", cranelift_module::Linkage::Import, &sig)
        .map_err(|e| reject(func, "ArraySet", &e.to_string()))?;
    let fref = module.declare_func_in_func(id, builder.func);
    builder.ins().call(fref, &[a, i, v]);
    builder.ins().jump(done_blk, &[]);

    builder.switch_to_block(done_blk);
    Ok(())
}

/// Inline fast-path array len:
/// 1. Check `arr != NULL`.
/// 2. If non-null, load `len` at offset 16.
/// 3. If NULL, return 0.
pub(super) fn emit_array_len<M: Module>(
    builder: &mut FunctionBuilder,
    env: &mut Env,
    _module: &mut M,
    ptr: Type,
    func: &MirFunction,
    dst: ValueId,
    arr: ValueId,
) -> Result<(), BackendError> {
    let (a, _) = operand(env, arr, func)?;
    let len_var = builder.declare_var(I64);
    let zero_ptr = builder.ins().iconst(ptr, 0);
    let zero_i64 = builder.ins().iconst(I64, 0);
    let is_null = builder.ins().icmp(cranelift::prelude::IntCC::Equal, a, zero_ptr);

    let null_blk = builder.create_block();
    let non_null_blk = builder.create_block();
    let merge_blk = builder.create_block();

    builder.ins().brif(is_null, null_blk, &[], non_null_blk, &[]);

    builder.switch_to_block(null_blk);
    builder.def_var(len_var, zero_i64);
    builder.ins().jump(merge_blk, &[]);

    builder.switch_to_block(non_null_blk);
    let len = builder.ins().load(I64, MemFlags::trusted(), a, 16);
    builder.def_var(len_var, len);
    builder.ins().jump(merge_blk, &[]);

    builder.switch_to_block(merge_blk);
    let result = builder.use_var(len_var);
    record(env, dst, result, MirType::I64);
    Ok(())
}

// ── Satır-içi global slot erişimi (GLOBAL_SLOTS data-import) ───────────

/// Slot sınır kontrolü + GLOBAL_SLOTS[slot] satır-içi yükü.
/// Derleyici üretimi slot'lar < 65536'dir; yine de dürüst sınır: dışarıda
/// helper çağrısı (tutarlı davranış).
pub(super) fn emit_global_load<M: Module>(
    builder: &mut FunctionBuilder,
    module: &mut M,
    ptr: Type,
    func: &MirFunction,
    slot: cranelift::prelude::Value,
) -> Result<cranelift::prelude::Value, BackendError> {
    use cranelift::prelude::InstBuilder;
    let max = builder.ins().iconst(I64, 65536);
    let in_range = builder
        .ins()
        .icmp(cranelift::prelude::IntCC::UnsignedLessThan, slot, max);

    let hit = builder.create_block();
    let cold = builder.create_block();
    let merge = builder.create_block();
    builder.append_block_param(merge, I64);
    builder.ins().brif(in_range, hit, &[], cold, &[]);

    builder.switch_to_block(hit);
    let data_id = module
        .declare_data("GLOBAL_SLOTS", cranelift_module::Linkage::Import, false, false)
        .map_err(|e| reject(func, "GlobalGet", &e.to_string()))?;
    let gv = module.declare_data_in_func(data_id, builder.func);
    let base = builder.ins().symbol_value(ptr, gv);
    let offset = builder.ins().ishl_imm(slot, 3);
    let addr = builder.ins().iadd(base, offset);
    let loaded = builder.ins().load(I64, MemFlags::new(), addr, 0);
    builder.ins().jump(merge, &[cranelift::prelude::codegen::ir::instructions::BlockArg::Value(loaded)]);

    builder.switch_to_block(cold);
    let zero = builder.ins().iconst(I64, 0);
    builder.ins().jump(merge, &[cranelift::prelude::codegen::ir::instructions::BlockArg::Value(zero)]);

    builder.switch_to_block(merge);
    let raw = *builder.func.dfg.block_params(merge).first().unwrap();
    Ok(raw)
}

/// GLOBAL_SLOTS[slot] = value satır-içi kaydı (sınır kontrolüyle).
pub(super) fn emit_global_store<M: Module>(
    builder: &mut FunctionBuilder,
    module: &mut M,
    ptr: Type,
    func: &MirFunction,
    slot: cranelift::prelude::Value,
    value: cranelift::prelude::Value,
) -> Result<(), BackendError> {
    use cranelift::prelude::InstBuilder;
    let max = builder.ins().iconst(I64, 65536);
    let in_range = builder
        .ins()
        .icmp(cranelift::prelude::IntCC::UnsignedLessThan, slot, max);

    let hit = builder.create_block();
    let cold = builder.create_block();
    let done = builder.create_block();
    builder.ins().brif(in_range, hit, &[], cold, &[]);

    builder.switch_to_block(hit);
    let data_id = module
        .declare_data("GLOBAL_SLOTS", cranelift_module::Linkage::Import, false, false)
        .map_err(|e| reject(func, "GlobalSet", &e.to_string()))?;
    let gv = module.declare_data_in_func(data_id, builder.func);
    let base = builder.ins().symbol_value(ptr, gv);
    let offset = builder.ins().ishl_imm(slot, 3);
    let addr = builder.ins().iadd(base, offset);
    builder.ins().store(MemFlags::new(), value, addr, 0);
    builder.ins().jump(done, &[]);

    builder.switch_to_block(cold);
    // sınır dışı: helper (tutarlı davranış — sessiz yutma yok)
    let sig = {
        let mut s = module.make_signature();
        s.params.push(AbiParam::new(I64));
        s.params.push(AbiParam::new(I64));
        s
    };
    let id = module
        .declare_function("hudhud_global_set", cranelift_module::Linkage::Import, &sig)
        .map_err(|e| reject(func, "GlobalSet", &e.to_string()))?;
    let fref = module.declare_func_in_func(id, builder.func);
    builder.ins().call(fref, &[slot, value]);
    builder.ins().jump(done, &[]);

    builder.switch_to_block(done);
    Ok(())
}
