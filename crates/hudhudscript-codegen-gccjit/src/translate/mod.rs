//! MIR → gccjit çevirisi (i64-handle lane). Phi blok parametreleri gccjit'te
//! yoktur: her phi için fonksiyon-lokal değişken ayrılır, öncüller atamayı
//! jump'tan önce yapar (C-dili denkimi; gcc SSA'ya yükseltir).

use std::collections::HashMap;

use gccjit::{BinaryOp, Block, ComparisonOp, Function, LValue, RValue, ToRValue};
use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{
    BlockId, CmpOp, FunctionId, MirFunction, MirModule, MirTerminator, MirType, RuntimeHelperId,
    ValueId,
};

use crate::abi::{self, Abi};

mod arith;
mod call_native;
mod exit;
mod insts;

use exit::translate_terminator;
use insts::translate_insts;

/// Bağlam başına TEK Abi (struct/alan kimlikleri gccjit'te nesne-kimliklidir).
pub fn build_abi<'ctx>(gcx: &'ctx gccjit::Context<'ctx>) -> Abi<'ctx> {
    abi::build(gcx)
}

/// Context başına bir kez declare edilen extern helper'lar.
pub(crate) struct ExtFns<'ctx> {
    pub ptr_to_i64: Function<'ctx>,
    pub i64_to_ptr: Function<'ctx>,
    pub print_int: Function<'ctx>,
    pub print_float: Function<'ctx>,
    pub print_str: Function<'ctx>,
    pub select_i64: Function<'ctx>,
    pub string_concat: Function<'ctx>,
    pub string_len: Function<'ctx>,
    pub string_eq: Function<'ctx>,
    pub int_to_string: Function<'ctx>,
    pub float_to_string: Function<'ctx>,
    pub array_new: Function<'ctx>,
    pub array_push: Function<'ctx>,
    pub array_get: Function<'ctx>,
    pub array_set: Function<'ctx>,
    pub array_len: Function<'ctx>,
    pub globals_handle: Function<'ctx>,
    pub string_char_at: Function<'ctx>,
    pub fmod: Function<'ctx>,
    pub global_get: Function<'ctx>,
    pub global_set: Function<'ctx>,
    pub string_to_int: Function<'ctx>,
    pub string_split: Function<'ctx>,
    pub string_index_of: Function<'ctx>,
    pub typeof_fn: Function<'ctx>,
    pub string_cmp: Function<'ctx>,
    pub string_append: Function<'ctx>,
    pub throw_fn: Function<'ctx>,
    pub has_exception: Function<'ctx>,
    pub catch_fn: Function<'ctx>,
    pub array_filled: Function<'ctx>,
    pub array_fill: Function<'ctx>,
    pub f64_from_i64: Function<'ctx>,
    pub i64_from_f64: Function<'ctx>,
    pub array_pop: Function<'ctx>,
    pub array_join: Function<'ctx>,
    pub string_substring: Function<'ctx>,
    pub object_new: Function<'ctx>,
    pub object_set: Function<'ctx>,
    pub object_get: Function<'ctx>,
    pub object_has: Function<'ctx>,
    pub object_len: Function<'ctx>,
    pub date_millis: Function<'ctx>,
    pub math_sin: Function<'ctx>,
    pub math_sqrt: Function<'ctx>,
    pub math_cos: Function<'ctx>,
    pub math_floor: Function<'ctx>,
    pub math_abs: Function<'ctx>,
    pub math_pow: Function<'ctx>,
    pub math_min: Function<'ctx>,
    pub math_max: Function<'ctx>,
    /// BigInt promote helper'ları (cranelift emit_promoted_op denklemi)
    pub num_add: Function<'ctx>,
    pub num_sub: Function<'ctx>,
    pub num_mul: Function<'ctx>,
    pub num_div: Function<'ctx>,
    pub num_rem: Function<'ctx>,
    pub num_cmp: Function<'ctx>,
}

pub(crate) fn declare_exts<'ctx>(abi: &Abi<'ctx>) -> ExtFns<'ctx> {
    let d = |name: &str, ps: &[gccjit::Type<'ctx>], r: gccjit::Type<'ctx>| abi::declare_fn(abi, name, ps, r);
    ExtFns {
        ptr_to_i64: d("hudhud_ptr_to_i64", &[abi.void_ptr], abi.ll),
        i64_to_ptr: d("hudhud_i64_to_ptr", &[abi.ll], abi.void_ptr),
        print_int: d("hudhud_print_int", &[abi.ll], abi.i32t),
        print_float: d("hudhud_print_float", &[abi.f64t], abi.i32t),
        print_str: d("hudhud_print_str", &[abi.void_ptr], abi.i32t),
        select_i64: d("hudhud_select_i64", &[abi.ll, abi.ll, abi.ll], abi.ll),
        string_concat: d("hudhud_string_concat", &[abi.void_ptr, abi.void_ptr], abi.void_ptr),
        string_len: d("hudhud_string_len", &[abi.void_ptr], abi.ll),
        string_eq: d("hudhud_string_eq", &[abi.void_ptr, abi.void_ptr], abi.ll),
        int_to_string: d("hudhud_int_to_string", &[abi.ll], abi.void_ptr),
        float_to_string: d("hudhud_float_to_string", &[abi.f64t], abi.void_ptr),
        array_new: d("hudhud_array_new", &[abi.ll], abi.void_ptr),
        array_push: d("hudhud_array_push", &[abi.void_ptr, abi.ll], abi.void_ty),
        array_get: d("hudhud_array_get", &[abi.void_ptr, abi.ll], abi.ll),
        array_set: d("hudhud_array_set", &[abi.void_ptr, abi.ll, abi.ll], abi.void_ty),
        array_len: d("hudhud_array_len", &[abi.void_ptr], abi.ll),
        array_pop: d("hudhud_array_pop", &[abi.void_ptr], abi.ll),
        array_join: d("hudhud_array_join", &[abi.void_ptr, abi.void_ptr], abi.void_ptr),
        string_substring: d("hudhud_string_substring", &[abi.void_ptr, abi.ll, abi.ll], abi.void_ptr),
        object_new: d("hudhud_object_new", &[], abi.void_ptr),
        object_set: d("hudhud_object_set", &[abi.void_ptr, abi.void_ptr, abi.ll], abi.void_ty),
        object_get: d("hudhud_object_get", &[abi.void_ptr, abi.void_ptr], abi.ll),
        object_has: d("hudhud_object_has", &[abi.void_ptr, abi.void_ptr], abi.ll),
        object_len: d("hudhud_object_len", &[abi.void_ptr], abi.ll),
        date_millis: d("hudhud_date_millis", &[], abi.ll),
        math_sin: d("hudhud_math_sin", &[abi.f64t], abi.f64t),
        math_sqrt: d("hudhud_math_sqrt", &[abi.f64t], abi.f64t),
        math_cos: d("hudhud_math_cos", &[abi.f64t], abi.f64t),
        math_floor: d("hudhud_math_floor", &[abi.f64t], abi.f64t),
        math_abs: d("hudhud_math_abs", &[abi.f64t], abi.f64t),
        math_pow: d("hudhud_math_pow", &[abi.f64t, abi.f64t], abi.f64t),
        math_min: d("hudhud_math_min", &[abi.f64t, abi.f64t], abi.f64t),
        math_max: d("hudhud_math_max", &[abi.f64t, abi.f64t], abi.f64t),
        globals_handle: d("hudhud_globals", &[], abi.ll),
        string_char_at: d("hudhud_string_char_at", &[abi.void_ptr, abi.ll], abi.void_ptr),
        fmod: d("fmod", &[abi.f64t, abi.f64t], abi.f64t),
        global_get: d("hudhud_global_get", &[abi.ll], abi.ll),
        global_set: d("hudhud_global_set", &[abi.ll, abi.ll], abi.ll),
        string_to_int: d("hudhud_string_to_int", &[abi.void_ptr], abi.ll),
        string_split: d("hudhud_string_split", &[abi.void_ptr, abi.void_ptr], abi.void_ptr),
        string_index_of: d("hudhud_string_index_of", &[abi.void_ptr, abi.void_ptr], abi.ll),
        typeof_fn: d("hudhud_typeof", &[abi.ll], abi.void_ptr),
        string_cmp: d("hudhud_string_cmp", &[abi.void_ptr, abi.void_ptr], abi.ll),
        string_append: d("hudhud_string_append", &[abi.void_ptr, abi.void_ptr], abi.void_ptr),
        throw_fn: d("hudhud_throw", &[abi.ll], abi.ll),
        has_exception: d("hudhud_has_exception", &[], abi.ll),
        catch_fn: d("hudhud_catch", &[], abi.ll),
        array_filled: d("hudhud_array_filled", &[abi.ll, abi.ll], abi.void_ptr),
        array_fill: d("hudhud_array_fill", &[abi.void_ptr, abi.ll, abi.ll], abi.ll),
        f64_from_i64: d("hudhud_f64_bits_from_i64", &[abi.ll], abi.f64t),
        i64_from_f64: d("hudhud_i64_bits_from_f64", &[abi.f64t], abi.ll),
        num_add: d("hudhud_num_add", &[abi.ll, abi.ll], abi.ll),
        num_sub: d("hudhud_num_sub", &[abi.ll, abi.ll], abi.ll),
        num_mul: d("hudhud_num_mul", &[abi.ll, abi.ll], abi.ll),
        num_div: d("hudhud_num_div", &[abi.ll, abi.ll], abi.ll),
        num_rem: d("hudhud_num_rem", &[abi.ll, abi.ll], abi.ll),
        num_cmp: d("hudhud_num_cmp", &[abi.ll, abi.ll], abi.ll),
    }
}

pub(crate) struct FnCx<'ctx, 'a> {
    pub(crate) abi: &'a Abi<'ctx>,
    pub(crate) ext: &'a ExtFns<'ctx>,
    pub(crate) func: Function<'ctx>,
    pub(crate) env: HashMap<u32, RValue<'ctx>>,
    pub(crate) blocks: HashMap<u32, Block<'ctx>>,
    /// (blok-id, param-index) → phi lokal değişkeni
    pub(crate) phi_locals: HashMap<(u32, u32), LValue<'ctx>>,
    /// f64 tipinde phi lokalleri — incoming/okuma tarafı tipi buna göre seçer
    pub(crate) phi_f64: std::collections::HashSet<(u32, u32)>,
    pub(crate) module_funcs: HashMap<u32, Function<'ctx>>,
    pub(crate) float_vals: std::collections::HashSet<u32>,
    /// §18 bayrak akümülatörleri (cranelift ov_var/dz_var denklemi): entry'de
    /// 0'a init edilir, her checked op anında OR'lanır, Return'de okunur.
    pub(crate) ov_acc: LValue<'ctx>,
    pub(crate) dz_acc: LValue<'ctx>,
}

impl<'ctx, 'a> FnCx<'ctx, 'a> {
    pub(crate) fn ll(&self, v: i64) -> RValue<'ctx> {
        self.abi.gcx.new_rvalue_from_long(self.abi.ll, v)
    }

    pub(crate) fn val(&self, v: ValueId) -> Result<RValue<'ctx>, BackendError> {
        self.env
            .get(&v.0)
            .copied()
            .ok_or_else(|| self.err(&format!("v{} used before definition", v.0)))
    }

    pub(crate) fn err(&self, what: &str) -> BackendError {
        BackendError::new("UNSUPPORTED_MIR", format!("gccjit: {what}"))
    }

    /// Sonucu bir kez materialize eder: gccjit rvalue'ları saftır — çağrı
    /// içeren ifade her kullanımda yeniden değerlenir. Değer tanım noktasında
    /// local'e yazılır (MIR SSA disiplininin C-dili denkimi).
    pub(crate) fn store(
        &mut self,
        gb: Block<'ctx>,
        dst: ValueId,
        v: RValue<'ctx>,
        is_f64: bool,
    ) {
        let ty = if is_f64 { self.abi.f64t } else { self.abi.ll };
        let local = self.func.new_local(None, ty, format!("v{}", dst.0));
        gb.add_assignment(None, local, v);
        if is_f64 {
            self.float_vals.insert(dst.0);
        } else {
            self.float_vals.remove(&dst.0);
        }
        self.env.insert(dst.0, local.to_rvalue());
    }

    pub(crate) fn as_f64(&self, v: RValue<'ctx>, from: MirType) -> RValue<'ctx> {
        if from == MirType::F64 {
            v
        } else {
            self.abi.gcx.new_cast(None, v, self.abi.f64t)
        }
    }

    /// handle (i64) → helper çağrısıyla pointer'a
    pub(crate) fn to_ptr(&self, v: RValue<'ctx>) -> RValue<'ctx> {
        self.abi.gcx.new_call(None, self.ext.i64_to_ptr, &[v])
    }

    /// pointer → handle (i64)
    pub(crate) fn to_handle(&self, p: RValue<'ctx>) -> RValue<'ctx> {
        self.abi.gcx.new_call(None, self.ext.ptr_to_i64, &[p])
    }

    /// §18 bayrak OR'lama: akümülatör lokaline anında yazar (blok bağımsız).
    pub(crate) fn or_flag(&self, gb: Block<'ctx>, flag: RValue<'ctx>, overflow: bool) {
        let g = self.abi.gcx;
        let acc = if overflow { self.ov_acc } else { self.dz_acc };
        let cur = acc.to_rvalue();
        let newv = g.new_binary_op(None, BinaryOp::BitwiseOr, self.abi.ll, cur, flag);
        gb.add_assignment(None, acc, newv);
    }

    /// §18 bölme-sıfır bayrağı (dz_acc)
    pub(crate) fn div_zero_flags_or(&self, gb: Block<'ctx>, flag: RValue<'ctx>) {
        self.or_flag(gb, flag, false);
    }
}

/// Modül fonksiyonlarını uniform ABI ile declare eder (bir kez, bağlam başına).
pub(crate) fn declare_module_functions<'ctx>(
    abi: &Abi<'ctx>,
    module: &MirModule,
) -> Result<HashMap<u32, Function<'ctx>>, BackendError> {
    let gcx = abi.gcx;
    let mut module_funcs: HashMap<u32, Function<'ctx>> = HashMap::new();
    for (i, mf) in module.functions.iter().enumerate() {
        let symbol = format!("hudhud_{}", mf.name);
        let ps = abi::entry_params(&abi);
        let gfn = gcx.new_function(None, gccjit::FunctionType::Exported, abi.void_ty, &ps, &symbol, false);
        module_funcs.insert(i as u32, gfn);
    }
    Ok(module_funcs)
}

/// Bir MIR fonksiyonunu gccjit Function'ına çevirir; sembol adını döner.
pub fn translate_function<'ctx>(
    abi: &Abi<'ctx>,
    module: &MirModule,
    f: &MirFunction,
    idx: usize,
    module_funcs: &HashMap<u32, Function<'ctx>>,
    ext: &ExtFns<'ctx>,
) -> Result<String, BackendError> {
    let gcx = abi.gcx;

    let func = module_funcs
        .get(&(idx as u32))
        .copied()
        .ok_or_else(|| BackendError::new("INTERNAL", "function index missing"))?;

    let mut cx = FnCx {
        abi,
        ext,
        func,
        env: HashMap::new(),
        blocks: HashMap::new(),
        phi_locals: HashMap::new(),
        phi_f64: std::collections::HashSet::new(),
        module_funcs: module_funcs.clone(),
        float_vals: std::collections::HashSet::new(),
        ov_acc: func.new_local(None, abi.ll, "ov_acc"),
        dz_acc: func.new_local(None, abi.ll, "dz_acc"),
    };

    // Giriş: args işaretçisinden Param'ları yükle
    let args_param = func.get_param(1).to_rvalue();

    // 2) Blokları oluştur; phi parametreleri TİPLİ lokal değişken yap
    //    (f64 param → f64 local — i64 zorunlu phi double atamada tip hatası
    //    üretip tüm derlemeyi düşürüyordu: mandelbrot sınıfı VM fallback'leri)
    let g = gcx;
    let reachable = reachable_blocks(f);
    for blk in &f.blocks {
        if !reachable.contains(&blk.id.0) {
            // gccjit erişilemez blokları reddeder (binary_trees: "unreachable
            // block") — MIR'da return-sonrası ölü bloklar çevrilmez
            continue;
        }
        let gb = func.new_block(format!("b{}", blk.id.0));
        cx.blocks.insert(blk.id.0, gb);
        for (i, (ty, _)) in blk.params.iter().enumerate() {
            let gcc_ty = if *ty == MirType::F64 { abi.f64t } else { abi.ll };
            let local = func.new_local(None, gcc_ty, format!("phi_{}_{}", blk.id.0, i));
            if *ty == MirType::F64 {
                cx.phi_f64.insert((blk.id.0, i as u32));
            }
            cx.phi_locals.insert((blk.id.0, i as u32), local);
        }
    }

    // 3) Blokları doldur (yalnız erişilebilirler)
    for blk in &f.blocks {
        if !reachable.contains(&blk.id.0) {
            continue;
        }
        let gb = cx.blocks[&blk.id.0];
        if blk.id == f.entry {
            // §18 akümülatör init'i entry'yi domine eder — her yol tanımlı değer okur
            gb.add_assignment(None, cx.ov_acc, cx.ll(0));
            gb.add_assignment(None, cx.dz_acc, cx.ll(0));
        }
        // phi değerleri = lokal değişkenlerin o anki değeri; f64 phi'ler
        // float_vals'a da girer (Return bitcast / print / CallStatic arg
        // dönüşümleri tip bilgisini buradan okur)
        for (i, (ty, vid)) in blk.params.iter().enumerate() {
            let local = cx.phi_locals[&(blk.id.0, i as u32)].to_rvalue();
            cx.env.insert(vid.0, local);
            if *ty == MirType::F64 {
                cx.float_vals.insert(vid.0);
            }
        }
        // Promote bölmeleri gb'yi merge bloğuna kaydırabilir — terminator
        // HER ZAMAN dönen güncel bloğa yazılmalı (eski blok terminated).
        let gb_now = translate_insts(
            &mut cx,
            gb,
            blk.id.0,
            &blk.insts,
            &args_param,
            &f.string_table,
        )?;
        translate_terminator(&mut cx, gb_now, blk)?;    }

    Ok(format!("hudhud_{}", f.name))
}

/// Entry'den terminator kenarları üzerinden erişilebilen blok kümesi
/// (ölü bloklar gccjit'e verilmez).
fn reachable_blocks(f: &MirFunction) -> std::collections::HashSet<u32> {
    let mut seen = std::collections::HashSet::new();
    let mut work = vec![f.entry.0];
    while let Some(id) = work.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(blk) = f.blocks.iter().find(|b| b.id.0 == id) else { continue };
        match &blk.terminator {
            Some(MirTerminator::Branch { target, .. }) => work.push(target.0),
            Some(MirTerminator::CondBranch { then_block, else_block, .. }) => {
                work.push(then_block.0);
                work.push(else_block.0);
            }
            _ => {}
        }
    }
    seen
}

