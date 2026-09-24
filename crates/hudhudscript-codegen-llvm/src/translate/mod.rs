//! MIR → LLVM IR çevirisi (i64-handle lane). LLVM'de phi/select/inttoptr
//! birer instruction'dır: MIR phi'leri birebir LLVM phi node'larına eşler,
//! öncü terminator'ları incoming değerleri ekler.

use std::collections::HashMap;

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{
    CmpOp, FunctionId, MirFunction, MirModule, MirTerminator, MirType, RuntimeHelperId, ValueId,
};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;
use inkwell::FloatPredicate;

mod arith;
mod call_native;
mod insts;
mod term;

use insts::translate_insts;
use term::translate_terminator;

pub(crate) struct FnCx<'ctx, 'm> {
    pub(crate) ctx: &'ctx Context,
    pub(crate) b: Builder<'ctx>,
    pub(crate) module: &'m Module<'ctx>,
    pub(crate) i64t: inkwell::types::IntType<'ctx>,
    pub(crate) f64t: inkwell::types::FloatType<'ctx>,
    pub(crate) i32t: inkwell::types::IntType<'ctx>,
    pub(crate) i8ptr: inkwell::types::PointerType<'ctx>,
    pub(crate) exit_ty: inkwell::types::StructType<'ctx>,
    pub(crate) env: HashMap<u32, BasicValueEnum<'ctx>>,
    /// CallStatic çağrı alanları — entry'de BİR KEZ ayrılır (döngü içinde
    /// alloca = her iterasyonda stack'i büyütür → ~131k iterasyonda stack
    /// overflow; AOT-LLVM crash ailesinin kökü). (args, out) çifti.
    pub(crate) call_scratch: Option<(
        inkwell::values::PointerValue<'ctx>,
        inkwell::values::PointerValue<'ctx>,
    )>,
    pub(crate) blocks: HashMap<u32, BasicBlock<'ctx>>,
    /// (blok, phi-index) → LLVM phi node
    pub(crate) phis: HashMap<(u32, u32), inkwell::values::PhiValue<'ctx>>,
    /// f64 tipinde yaratılmış phi'ler — incoming/okuma tarafı tipi buna göre seçer
    pub(crate) phi_f64: std::collections::HashSet<(u32, u32)>,
    pub(crate) module_funcs: HashMap<u32, FunctionValue<'ctx>>,
    /// §18 bayrak akümülatörleri (cranelift ov_var denklemi): entry'de alloca+0,
    /// her checked op anında OR, Return'de load. mem2reg SSA'ya yükseltir.
    pub(crate) ov_acc: PointerValue<'ctx>,
    pub(crate) dz_acc: PointerValue<'ctx>,
}

impl<'ctx, 'm> FnCx<'ctx, 'm> {
    pub(crate) fn err(&self, what: &str) -> BackendError {
        BackendError::new("UNSUPPORTED_MIR", format!("llvm: {what}"))
    }

    pub(crate) fn val(&self, v: ValueId) -> Result<BasicValueEnum<'ctx>, BackendError> {
        self.env
            .get(&v.0)
            .copied()
            .ok_or_else(|| self.err(&format!("v{} used before definition", v.0)))
    }

    pub(crate) fn ival(&self, v: ValueId) -> Result<IntValue<'ctx>, BackendError> {
        let bv = self.val(v)?;
        match bv {
            BasicValueEnum::IntValue(i) => Ok(i),
            BasicValueEnum::FloatValue(f) => Ok(self.b.build_float_to_signed_int(f, self.i64t, "f2i")),
            _ => Err(self.err("expected scalar value")),
        }
    }

    pub(crate) fn fval(&self, v: ValueId) -> Result<FloatValue<'ctx>, BackendError> {
        let bv = self.val(v)?;
        match bv {
            BasicValueEnum::FloatValue(f) => Ok(f),
            BasicValueEnum::IntValue(i) => Ok(self.b.build_signed_int_to_float(i, self.f64t, "i2f")),
            _ => Err(self.err("expected float value")),
        }
    }

    pub(crate) fn is_f64(&self, v: ValueId) -> bool {
        matches!(self.env.get(&v.0), Some(BasicValueEnum::FloatValue(_)))
    }

    /// handle → pointer (LLVM'de serbest dönüşüm)
    pub(crate) fn to_ptr(&self, h: IntValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        self.b.build_int_to_ptr(h, self.i8ptr, name)
    }

    pub(crate) fn to_handle(&self, p: PointerValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.b.build_ptr_to_int(p, self.i64t, name)
    }

    /// §18 bayrak OR'lama: i1 bayrağı zext'leyip akümülatör alloca'sına yazar.
    pub(crate) fn or_flag(&self, flag: IntValue<'ctx>, acc: PointerValue<'ctx>) {
        let wide = if flag.get_type().get_bit_width() < 64 {
            self.b.build_int_z_extend(flag, self.i64t, "fl64")
        } else {
            flag
        };
        let cur = self.b.build_load(acc, "accv").into_int_value();
        let merged = self.b.build_or(cur, wide, "accn");
        self.b.build_store(acc, merged);
    }


    /// (i64 × n) → void extern helper (idempotent declare).
    pub(crate) fn ext_void_fn(&self, name: &str, n: usize) -> FunctionValue<'ctx> {
        if let Some(f) = self.module.get_function(name) {
            return f;
        }
        let ps: Vec<BasicMetadataTypeEnum<'ctx>> = vec![self.i64t.into(); n];
        let fty = self.ctx.void_type().fn_type(&ps, false);
        self.module.add_function(name, fty, None)
    }
}

pub(crate) fn ext_i64_fn<'ctx, 'm>(cx: &FnCx<'ctx, 'm>, name: &str, n: usize) -> FunctionValue<'ctx> {
    if let Some(f) = cx.module.get_function(name) {
        return f;
    }
    let meta: Vec<BasicMetadataTypeEnum<'ctx>> = vec![cx.i64t.into(); n];
    let fty = cx.i64t.fn_type(&meta, false);
    cx.module.add_function(name, fty, None)
}

/// (f64) → i64 extern helper.
fn ext_f64_to_i64_fn<'ctx, 'm>(cx: &FnCx<'ctx, 'm>, name: &str) -> FunctionValue<'ctx> {
    if let Some(f) = cx.module.get_function(name) {
        return f;
    }
    let fty = cx.i64t.fn_type(&[cx.f64t.into()], false);
    cx.module.add_function(name, fty, None)
}

/// (f64, f64) → f64 extern helper.
pub(crate) fn ext_f64_fn2<'ctx, 'm>(cx: &FnCx<'ctx, 'm>, name: &str) -> FunctionValue<'ctx> {
    if let Some(f) = cx.module.get_function(name) {
        return f;
    }
    let fty = cx.f64t.fn_type(&[cx.f64t.into(), cx.f64t.into()], false);
    cx.module.add_function(name, fty, None)
}

/// (f64) → i32 extern helper (print_float).
pub(crate) fn ext_f64_to_i32_fn<'ctx, 'm>(cx: &FnCx<'ctx, 'm>, name: &str) -> FunctionValue<'ctx> {
    if let Some(f) = cx.module.get_function(name) {
        return f;
    }
    let fty = cx.i32t.fn_type(&[cx.f64t.into()], false);
    cx.module.add_function(name, fty, None)
}

/// (f64) → f64 extern helper.
pub(crate) fn ext_f64_fn<'ctx, 'm>(cx: &FnCx<'ctx, 'm>, name: &str) -> FunctionValue<'ctx> {
    if let Some(f) = cx.module.get_function(name) {
        return f;
    }
    let fty = cx.f64t.fn_type(&[cx.f64t.into()], false);
    cx.module.add_function(name, fty, None)
}


pub fn translate_function<'ctx, 'm>(
    ctx: &'ctx Context,
    module: &'m Module<'ctx>,
    mir: &MirModule,
    f: &MirFunction,
    idx: usize,
) -> Result<String, BackendError> {
    let i64t = ctx.i64_type();
    let f64t = ctx.f64_type();
    let i32t = ctx.i32_type();
    let i8ptr = ctx.i8_type().ptr_type(inkwell::AddressSpace::default());
    let exit_ty = ctx.struct_type(&[i32t.into(), i64t.into()], false);

    // 1) uniform ABI: (i32 argc, i64* args, JitExit* out) → void
    let entry_ty = ctx.void_type().fn_type(
        &[i32t.into(), i64t.ptr_type(inkwell::AddressSpace::default()).into(), exit_ty.ptr_type(inkwell::AddressSpace::default()).into()],
        false,
    );
    let mut module_funcs: HashMap<u32, FunctionValue<'_>> = HashMap::new();
    for (i, mf) in mir.functions.iter().enumerate() {
        let symbol = format!("hudhud_{}", mf.name);
        // Mevcut sembol (erken deklarasyon dahil) HER ZAMAN yeniden kullanılır:
        // add_function ad çakışmasında tanımı "hudhud_x.1"e kaydırır ve EE
        // lookup'u boş deklarasyona takılırdı (hudhud_main.1 regresyonu).
        let fv = if let Some(existing) = module.get_function(&symbol) {
            existing
        } else {
            module.add_function(&symbol, entry_ty, None)
        };
        module_funcs.insert(i as u32, fv);
    }
    let func = module_funcs
        .get(&(idx as u32))
        .copied()
        .ok_or_else(|| BackendError::new("INTERNAL", "function index missing"))?;

    let b = ctx.create_builder();

    // 2) bloklar + phi node'ları
    let entry_bb = ctx.append_basic_block(func, "b0");
    let mut blocks: HashMap<u32, BasicBlock> = HashMap::new();
    blocks.insert(f.blocks[0].id.0, entry_bb);
    for blk in f.blocks.iter().skip(1) {
        let name = format!("b{}", blk.id.0);
        let bb = ctx.append_basic_block(func, name.as_str());
        blocks.insert(blk.id.0, bb);
    }
    // §18 akümülatörleri entry'de alloca+init — her yol tanımlı değer okur
    b.position_at_end(entry_bb);
    let ov_acc = b.build_alloca(i64t, "ov_acc");
    let dz_acc = b.build_alloca(i64t, "dz_acc");
    b.build_store(ov_acc, i64t.const_zero());
    b.build_store(dz_acc, i64t.const_zero());

    let mut cx = FnCx {
        ctx,
        b,
        module,
        i64t,
        f64t,
        i32t,
        i8ptr,
        exit_ty,
        env: HashMap::new(),
        call_scratch: None,
        blocks,
        phis: HashMap::new(),
        phi_f64: std::collections::HashSet::new(),
        module_funcs,
        ov_acc,
        dz_acc,
    };

    for blk in &f.blocks {
        let bb = cx.blocks[&blk.id.0];
        cx.b.position_at_end(bb);
        for (i, (ty, vid)) in blk.params.iter().enumerate() {
            let pname = format!("phi_{}_{}", blk.id.0, i);
            // Tipli phi: f64 parametre f64 phi taşınır — bitcast gidiş-dönüşü
            // (ve okumada sayısal sitofp çöpü) ortadan kalkar
            if *ty == MirType::F64 {
                let phi = cx.b.build_phi(f64t, pname.as_str());
                cx.phi_f64.insert((blk.id.0, i as u32));
                cx.phis.insert((blk.id.0, i as u32), phi);
                cx.env.insert(vid.0, phi.as_basic_value());
            } else {
                let phi = cx.b.build_phi(i64t, pname.as_str());
                cx.phis.insert((blk.id.0, i as u32), phi);
                cx.env.insert(vid.0, phi.as_basic_value());
            }
        }
    }

    // CallStatic scratch: entry'de BİR kez (8 arg + JitExit) — döngü içinde
    // alloca her iterasyonda stack'i büyütür (~131k iterasyonda stack
    // overflow; AOT-LLVM crash ailesinin ikinci kökü).
    {
        let entry_bb = cx.blocks[&f.blocks[0].id.0];
        cx.b.position_at_end(entry_bb);
        let count = cx.i64t.const_int(8, false);
        let args_buf = unsafe { cx.b.build_array_alloca(cx.i64t, count, "call_args_entry") };
        let out_buf = cx.b.build_alloca(cx.exit_ty, "call_out_entry");
        cx.call_scratch = Some((args_buf, out_buf));
    }

    // 3) blokları doldur (phi incoming'ler terminator'larda eklenir)
    for blk in &f.blocks {
        let bb = cx.blocks[&blk.id.0];
        cx.b.position_at_end(bb);
        translate_insts(&mut cx, func, blk.id.0, &blk.insts, &f.string_table)?;
        translate_terminator(&mut cx, func, blk)?;
    }

    Ok(format!("hudhud_{}", f.name))
}


