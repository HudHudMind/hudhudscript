//! Lowering context and shared utilities for typed HIR → MIR lowering.

use std::collections::HashMap;

use hudhudscript_types::HirFunction;

use crate::builder::MirFunctionBuilder;
use crate::mir::{BlockId, MirType, ValueId};
use crate::LowerError;

pub(crate) struct FnCx {
    pub(crate) builder: MirFunctionBuilder,
    pub(crate) name: String,
    pub(crate) value_tys: Vec<Option<MirType>>,
    pub(crate) bindings: HashMap<String, ValueId>,
    pub(crate) current_block: BlockId,
    /// Modül içindeki bilinen fonksiyonlar: isim → (FunctionId, param tipleri, dönüş tipi)
    pub(crate) module_functions: HashMap<String, (crate::mir::FunctionId, Vec<MirType>, MirType)>,
    /// İçteki döngünün (varsa) exit bloğu + phi isimleri — break hedefi
    pub(crate) loop_exit: Option<(BlockId, Vec<String>)>,
    /// Modül-geneli let isimleri ve tipleri; _hudhud_init'te depolanır, başka
    /// fonksiyonlarda ObjectGet/Set ile erişilir (handle = hudhud_globals)
    pub(crate) module_globals: HashMap<String, (u32, MirType)>,
    pub(crate) is_init: bool,
    /// İç döngünün header'ı + phi isimleri — continue hedefi
    pub(crate) loop_header: Option<(BlockId, Vec<String>)>,
    /// Dizi değişkenlerinin eleman tipleri: dizi_adı → eleman_tipi
    pub(crate) array_elem_tys: HashMap<String, MirType>,
    /// 2D dizi değişkenlerinin iç eleman tipleri: dizi_adı → eleman_tipi
    pub(crate) array_inner_elem_tys: HashMap<String, MirType>,
    /// En içteki try bloğunun catch hedefi
    pub(crate) catch_target: Option<BlockId>,
}

impl FnCx {
    pub(crate) fn set_ty(&mut self, v: ValueId, ty: MirType) {
        let idx = v.0 as usize;
        while self.value_tys.len() <= idx {
            self.value_tys.push(None);
        }
        self.value_tys[idx] = Some(ty);
    }

    pub(crate) fn ty_of(&self, v: ValueId) -> Option<MirType> {
        self.value_tys.get(v.0 as usize).copied().flatten()
    }

    pub(crate) fn err(&self, what: &str) -> LowerError {
        LowerError::Unsupported {
            function: self.name.clone(),
            reason: what.to_string(),
        }
    }
}

/// Modül-global depoya yaz: GlobalSet(slot, v)
pub(crate) fn emit_global_store(cx: &mut FnCx, name: &str, v: ValueId) {
    let b = cx.current_block;
    if let Some(&(slot, _)) = cx.module_globals.get(name) {
        let slot_val = cx.builder.const_i64(b, slot as i64);
        cx.builder.call_native(
            b,
            MirType::Generic,
            crate::mir::RuntimeHelperId::GlobalSet,
            vec![slot_val, v],
        );
    } else {
        let h = cx.builder.call_native(
            b,
            MirType::Generic,
            crate::mir::RuntimeHelperId::GlobalsHandle,
            vec![],
        );
        let k = cx.builder.const_string(b, name);
        cx.builder.object_set(b, h, k, v);
    }
}

/// Sayısal koşul → != 0 (Bool); Bool aynen. Ref/Generic reddedilir.
pub(crate) fn truthy(
    hir: &HirFunction,
    cx: &mut FnCx,
    c: ValueId,
) -> Result<ValueId, LowerError> {
    match cx.ty_of(c) {
        Some(MirType::Bool) => Ok(c),
        Some(MirType::I64) => {
            let b = cx.current_block;
            let zero = cx.builder.const_i64(b, 0);
            let v = cx.builder.cmp(b, crate::mir::CmpOp::Ne, MirType::I64, c, zero);
            cx.set_ty(v, MirType::Bool);
            Ok(v)
        }
        Some(MirType::F64) => {
            let b = cx.current_block;
            let zero = cx.builder.const_f64(b, 0.0);
            let v = cx.builder.cmp(b, crate::mir::CmpOp::Ne, MirType::F64, c, zero);
            cx.set_ty(v, MirType::Bool);
            Ok(v)
        }
        _ => Err(LowerError::Unsupported {
            function: hir.name.clone(),
            reason: "condition must be bool or numeric (truthy)".into(),
        }),
    }
}
