//! MIR → CLIF translation (uniform native-entry ABI, §6.2/§12.4).
//!
//! Native signature (stage one):
//! `extern "C" fn(argc: u32, args: *const i64, out: *mut JitExit)`
//! — results written through `out` (portable sret-by-pointer).
//!
//! Accepted MIR (everything else is REJECTED with a clear BackendError
//! naming the construct — never approximated):
//! - exactly one block,
//! - `Param { I64 }`, `ConstInt`, checked `Add/Sub/Mul { I64 }`,
//!   zero-and-MIN/-1-guarded `Div/Rem { I64 }`, `Cmp { I64 }`,
//! - `Return(v)` terminator (Bool widens to i64 0/1 at the exit).

use cranelift::prelude::types::{I32, I64};
use cranelift::prelude::{
    AbiParam, FunctionBuilder, FunctionBuilderContext, InstBuilder, MemFlags, Value,
};
use cranelift_module::Module;

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirFunction, MirInst, MirType};

mod abi;
pub(super) mod array_insts;
mod arith;
mod call;
mod helpers;
mod terminators;

use abi::translate_abi;
use arith::translate_arith;
use call::translate_call;
use helpers::{bitcast_i64_to_f64, discriminator, ensure_f64, operand, record, reject};
use terminators::emit_terminator;

// ── ConstString STRING_REGISTRY kayıt stratejisi (v0.9.9 regresyon fix) ──
//
// v0.9.1 typeof düzeltmesi kaydı fonksiyon gövdesine inline yazmıştı: döngü
// içindeki her string sabiti her İTERASYONDA hudhud_register_string
// (TLS+RefCell+HashSet) ödüyordu — object_churn +95%, method_dispatch +97%,
// number_parse +215% regresyonlarının kök nedeni.
//
// JIT: inline çağrı YOK; backend finalize sonrası her string data sembolünü
// host tarafında BİR KEZ kaydeder (register_finalized_strings).
// AOT: inline çağrı KALIR (binary kendi kendine kaydeder; linker shim'i
// gelene dek tek doğru yol — AOT'ta host finalize anı yoktur).

thread_local! {
    static INLINE_STRING_REG: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static STRING_DATA_NAMES: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// AOT yolu: inline kaydı açar (emit_object girişinde; çıkışta kapatılır).
pub fn set_inline_string_reg(v: bool) {
    INLINE_STRING_REG.with(|c| c.set(v));
}

pub(crate) fn inline_string_reg() -> bool {
    INLINE_STRING_REG.with(|c| c.get())
}

/// ConstString data sembol adını kayda al (backend finalize sonrası host-side
/// kayıt için). Aynı sembol birden çok kez not edilebilir — tekrarsız kaydedilir.
pub(crate) fn note_string_data(name: &str) {
    STRING_DATA_NAMES.with(|v| v.borrow_mut().push(name.to_string()));
}

/// Not edilen string sembollerini al ve listeyi temizle.
pub fn take_string_data_names() -> Vec<String> {
    STRING_DATA_NAMES.with(|v| std::mem::take(&mut *v.borrow_mut()))
}

/// Finalize edilmiş modülün string data sembollerini STRING_REGISTRY'ye
/// host tarafında kaydeder (modül başına bir kez; çalıştırılan kodda maliyet YOK).
/// get_finalized_data JITModule'e özgü — AOT yolu bu fonksiyonu çağırmaz.
pub fn register_finalized_strings(module: &mut cranelift_jit::JITModule) {
    for name in take_string_data_names() {
        if let Some(cranelift_module::FuncOrDataId::Data(id)) = module.get_name(&name) {
            let (ptr, _size) = module.get_finalized_data(id);
            hudhudscript_native_abi::hudhud_register_string(ptr as *const std::ffi::c_char);
        }
    }
}

pub fn translate<M: Module>(
    module: &mut M,
    func_ctx: &mut FunctionBuilderContext,
    func: &MirFunction,
) -> Result<String, BackendError> {
    translate_with_module(module, func_ctx, func, false, &[])
}

/// Modül bağlamlı çeviri: CallStatic, declare edilmiş FuncId'lere call yapar.
pub fn translate_with_module<M: Module>(
    module: &mut M,
    func_ctx: &mut FunctionBuilderContext,
    func: &MirFunction,
    is_aot: bool,
    module_func_ids: &[cranelift_module::FuncId],
) -> Result<String, BackendError> {
    let ptr = module.isa().pointer_type();
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(I32)); // argc
    sig.params.push(AbiParam::new(ptr)); // args
    sig.params.push(AbiParam::new(ptr)); // out (JitExit*)

    let mut code_ctx = module.make_context();
    code_ctx.func.signature = sig;
    let mut builder = FunctionBuilder::new(&mut code_ctx.func, func_ctx);
    let entry = builder.create_block();
    builder.append_block_params_for_function_params(entry);
    builder.switch_to_block(entry);

    let args_ptr = builder.block_params(entry)[1];
    let out_ptr = builder.block_params(entry)[2];

    // Value environment: MIR ValueId -> (CLIF value, MIR type) — function-scoped
    let mut env: Vec<Option<(Value, MirType)>> = Vec::new();
    let ov_var = builder.declare_var(I32);
    let dz_var = builder.declare_var(I32);
    let zero32 = builder.ins().iconst(I32, 0);
    builder.def_var(ov_var, zero32);
    builder.def_var(dz_var, zero32);

    // MIR BlockId -> CLIF Block mapping — TÜM blokları ÖNCE oluştur
    // (CondBranch'ler ileri bloklara işaret edebilir)
    let mut block_map: std::collections::HashMap<u32, cranelift::prelude::Block> =
        std::collections::HashMap::new();
    block_map.insert(0, entry);
    for blk in &func.blocks {
        if !block_map.contains_key(&blk.id.0) {
            let b = builder.create_block();
            for (ty, _) in &blk.params {
                builder.append_block_param(b, match ty {
                    MirType::I64 => I64,
                    MirType::F64 => cranelift::prelude::types::F64,
                    MirType::Bool => I64, // bool → i8 yerine i64 (uniform exit)
                    MirType::Generic => I64,
                    // ref'ler i64-handle lane'inde opak handle olarak taşınır
                    MirType::Ref(_) => I64,
                    other => {
                        return Err(reject(func, "BlockParam", &format!("type {other} not supported in block params yet")))
                    }
                });
            }
            block_map.insert(blk.id.0, b);
        }
    }

    for block in &func.blocks {
        let clif_blk = block_map[&block.id.0];
        if builder.current_block() != Some(clif_blk) {
            builder.switch_to_block(clif_blk);
        }

    // Blok parametrelerini env'e kaydet (phi değerleri)
    for (i, (ty, vid)) in block.params.iter().enumerate() {
        let clif_val = builder.block_params(clif_blk)[i];
        record(&mut env, *vid, clif_val, *ty);
    }

    for inst in &block.insts {
        match inst {
            MirInst::Param { dst, ty, index }
                if matches!(ty, MirType::I64 | MirType::F64 | MirType::Ref(_) | MirType::Generic) =>
            {
                let raw = builder.ins().load(I64, MemFlags::new(), args_ptr, (*index as i32) * 8);
                if *ty == MirType::F64 {
                    let f = bitcast_i64_to_f64(&mut builder, raw);
                    record(&mut env, *dst, f, MirType::F64);
                } else {
                    // Ref/Generic handle'lar i64 olarak yaşar
                    let et = match ty {
                        MirType::Ref(_) => *ty,
                        _ => MirType::I64,
                    };
                    record(&mut env, *dst, raw, et);
                }
            }
            MirInst::ConstInt { dst, value, .. } => {
                let v = builder.ins().iconst(I64, *value);
                record(&mut env, *dst, v, MirType::I64);
            }
            MirInst::ConstNull { dst } => {
                let v = builder.ins().iconst(I64, 0);
                record(&mut env, *dst, v, MirType::Generic);
            }
            MirInst::ConstBool { dst, value } => {
                // i64 şeridi: bool = i64 0|1
                let v = builder.ins().iconst(I64, *value as i64);
                record(&mut env, *dst, v, MirType::Bool);
            }
            MirInst::ConstFloat { dst, bits, .. } => {
                let raw = builder.ins().iconst(I64, *bits as i64);
                let f = bitcast_i64_to_f64(&mut builder, raw);
                record(&mut env, *dst, f, MirType::F64);
            }
            // ── Mantıksal operatörler (eager; i64 0|1 → and/or/not) ──
            MirInst::LogicalAnd { dst, lhs, rhs } => {
                let (l, _) = operand(&env, *lhs, func)?;
                let (r, _) = operand(&env, *rhs, func)?;
                let b = builder.ins().band(l, r);
                record(&mut env, *dst, b, MirType::Bool);
            }
            MirInst::LogicalOr { dst, lhs, rhs } => {
                let (l, _) = operand(&env, *lhs, func)?;
                let (r, _) = operand(&env, *rhs, func)?;
                let b = builder.ins().bor(l, r);
                record(&mut env, *dst, b, MirType::Bool);
            }
            MirInst::LogicalNot { dst, src } => {
                let (s, _) = operand(&env, *src, func)?;
                let one = builder.ins().iconst(I64, 1);
                let b = builder.ins().isub(one, s);
                record(&mut env, *dst, b, MirType::Bool);
            }
            MirInst::UnaryNeg { dst, ty, src } => {
                let (s, st) = operand(&env, *src, func)?;
                if *ty == MirType::F64 || st == MirType::F64 {
                    let sf = ensure_f64(&mut builder, s, st);
                    let result = builder.ins().fneg(sf);
                    record(&mut env, *dst, result, MirType::F64);
                } else {
                    let zero = builder.ins().iconst(I64, 0);
                    let result = builder.ins().isub(zero, s);
                    record(&mut env, *dst, result, MirType::I64);
                }
            }
            // ── Aritmetik + karşılaştırma (arith; §18 bayrak akümülasyonu) ──
            MirInst::Add { .. } | MirInst::Sub { .. } | MirInst::Mul { .. }
            | MirInst::Div { .. } | MirInst::Rem { .. } | MirInst::Cmp { .. } => {
                translate_arith(inst, &mut builder, &mut env, module, func, ov_var, dz_var)?;
            }
            // ── String/array ABI helper'ları (abi) ──
            inst if abi::handles(inst) => {
                translate_abi(inst, &mut builder, &mut env, module, ptr, func)?;
            }
            // ── Çağrılar (call; uniform ABI + native helper'lar) ──
            MirInst::CallStatic { .. } | MirInst::CallNative { .. } => {
                translate_call(inst, &mut builder, &mut env, module, ptr, func, module_func_ids, out_ptr, ov_var, is_aot)?;
            }
            MirInst::GcSafepoint => {
                // No-op: GC entegrasyonu (§17) gelene kadar güvenli —
                // i64 şeridinde GC kökü yok, toplanacak değer yok.
            }
            other => {
                let name = discriminator(other);
                return Err(reject(func, name, "not lowered yet"));
            }
        }
    }

    // ── Blok terminator'u ──
    emit_terminator(&mut builder, &env, func, block, &block_map, out_ptr, ov_var, dz_var)?;
    } // ── func.blocks döngüsü sonu ──

    // Fonksiyon tamam — finalize et
    builder.seal_all_blocks();
    builder.finalize();

    let symbol = format!("hudhud_{}", func.name);
    if std::env::var("HUDHUD_DUMP_CLIF").is_ok() {
        eprintln!("=== CLIF for {} ===\n{}", func.name, code_ctx.func.display());
    }
    if let Err(e) = cranelift_codegen::verifier::verify_function(&code_ctx.func, module.isa()) {
        if std::env::var("HUDHUD_JIT_TRACE").is_ok() || std::env::var("HUDHUD_DUMP_CLIF").is_ok() {
            eprintln!("=== DETAILED VERIFIER ERRORS for {} ===\n{e}", func.name);
        }
    }
    let id = module
        .declare_function(&symbol, cranelift_module::Linkage::Export, &code_ctx.func.signature)
        .map_err(|e| BackendError::new("DECLARE_FAIL", format!("declare {symbol}: {e}")).in_function(func.name.to_string()))?;
    module.define_function(id, &mut code_ctx).map_err(|e| BackendError::new("DEFINE_FAIL", format!("define {symbol}: {e}")).in_function(func.name.to_string()))?;
    module.clear_context(&mut code_ctx);
    Ok(symbol)
}

/// Modül içi çeviri: CallStatic, declare edilmiş FuncId'lere call yapar.
pub fn translate_in_module<M: Module>(
    module: &mut M,
    func_ctx: &mut FunctionBuilderContext,
    func: &MirFunction,
    module_func_ids: &[cranelift_module::FuncId],
    _mir_module: &hudhudscript_mir::MirModule,
) -> Result<String, BackendError> {
    translate_with_module(module, func_ctx, func, false, module_func_ids)
}
